use crusty_chip::{decode, VirtualMachine, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use getopts::Options;
use sfml::graphics::{RenderTarget, RenderWindow, Sprite, Texture, Transformable};
use sfml::system::Clock;
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use std::fs::File;
use std::io::Read;

fn sfml_key_to_ch8(code: Key) -> Option<u8> {
    Some(match code {
        Key::NUM1 => 1,
        Key::NUM2 => 2,
        Key::NUM3 => 3,
        Key::NUM4 => 0xC,
        Key::Q => 4,
        Key::W => 5,
        Key::E => 6,
        Key::R => 0xD,
        Key::A => 7,
        Key::S => 8,
        Key::D => 9,
        Key::F => 0xE,
        Key::Z => 0xA,
        Key::X => 0,
        Key::C => 0xB,
        Key::V => 0xF,
        _ => return None,
    })
}

fn usage(progname: &str, opts: &Options) -> String {
    let brief = format!("{} rom_file", progname);
    format!("Usage: {}", opts.usage(&brief))
}

fn run() -> i32 {
    let mut args = std::env::args();
    let progname = args.next().expect("Missing program name?");
    let mut opts = Options::new();
    opts.optflag("", "pause", "Start in a paused state");

    let matches = match opts.parse(args) {
        Ok(matches) => matches,
        Err(e) => {
            eprintln!("{}\n\n{}", e, usage(&progname, &opts));
            return 1;
        }
    };

    let mut paused = matches.opt_present("pause");

    let filename = match matches.free.get(0) {
        Some(filename) => filename,
        None => {
            eprintln!("Required filename as first positional argument.\n");
            eprintln!("{}", usage(&progname, &opts));
            return 1;
        }
    };

    let mut clock = Clock::start();

    let mut file = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open \"{}\": {}", filename, e);
            return 1;
        }
    };

    if file.metadata().unwrap().len() > crusty_chip::MEM_SIZE as u64 {
        eprintln!(
            "File \"{}\" is too big to be a proper CHIP-8 ROM.",
            filename
        );
        return 1;
    }

    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .unwrap_or_else(|e| panic!("Failed to read rom: {}", e));

    let scale = 10;

    let mut ch8 = VirtualMachine::new();
    match ch8.load_rom(&data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading rom: \"{}\". Aborting.", e);
            return 1;
        }
    }

    let ctx = ContextSettings::default();
    let mut win = RenderWindow::new(
        VideoMode::new(
            DISPLAY_WIDTH as u32 * scale,
            DISPLAY_HEIGHT as u32 * scale,
            32,
        ),
        "CrustyChip",
        Style::CLOSE,
        &ctx,
    );
    win.set_vertical_sync_enabled(true);

    let mut tex = Texture::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32).unwrap();
    let mut saved_states = [None; 10];
    let mut printed_info = false;
    let mut cycles_made: u64 = 0;

    loop {
        let mut advance = false;
        while let Some(event) = win.poll_event() {
            match event {
                Event::Closed => return 0,
                Event::KeyPressed {
                    code, ctrl, shift, ..
                } => {
                    if code == Key::P {
                        paused = !paused;
                    } else if code == Key::R && ctrl {
                        ch8 = VirtualMachine::new();
                        ch8.load_rom(&data).expect("ROM data too big? It changed?");
                    } else if code == Key::PERIOD {
                        advance = true;
                    } else if let Some(key) = sfml_key_to_ch8(code) {
                        ch8.press_key(key);
                    }
                    macro_rules! state_key (
                        ($s: expr, $k: ident) => (
                            if code == Key::$k {
                                if shift {
                                    saved_states[$s] = Some(ch8);
                                    eprintln!("Saved state {}.", $s);
                                } else if let Some(state) = saved_states[$s] {
                                    ch8 = state;
                                    eprintln!("Loaded state {}.", $s);
                                }
                            }
                        )
                    );
                    state_key!(0, F1);
                    state_key!(1, F2);
                    state_key!(2, F3);
                    state_key!(3, F4);
                    state_key!(4, F5);
                    state_key!(5, F6);
                    state_key!(6, F7);
                    state_key!(7, F8);
                    state_key!(8, F9);
                    state_key!(9, F10);
                }
                Event::KeyReleased { code, .. } => {
                    if let Some(key) = sfml_key_to_ch8(code) {
                        ch8.release_key(key);
                    }
                }
                _ => {}
            }
        }
        let mut cycles = 0;
        while !(ch8.display_updated() || ch8.waiting_for_key()) {
            do_emulation_cycle(
                &mut clock,
                &mut ch8,
                paused,
                &mut printed_info,
                &mut cycles_made,
                advance,
            );
            cycles += 1;
            // Take a little break and render if the machine takes too long
            // to respond
            if cycles > 50_000 {
                break;
            }
        }
        render_screen(&mut win, &mut tex, &ch8, scale as f32);
        ch8.clear_du_flag();
    }
}

fn do_emulation_cycle(
    clock: &mut Clock,
    ch8: &mut VirtualMachine,
    paused: bool,
    printed_info: &mut bool,
    cycles_made: &mut u64,
    advance: bool,
) {
    if clock.elapsed_time().as_seconds() >= 1.0 / 60.0 {
        ch8.decrement_timers();
        clock.restart();
    }

    if paused && !*printed_info {
        let raw_ins = ch8.get_ins();
        println!(
            "Cycle {}, pc @ {:#x}, ins: {:#x?} raw: {:#x}",
            cycles_made,
            ch8.pc(),
            decode(raw_ins),
            raw_ins
        );
        *printed_info = true;
    }

    if !paused || advance {
        ch8.do_cycle();
        *cycles_made += 1;
        *printed_info = false;
    }
}

fn render_screen(win: &mut RenderWindow, tex: &mut Texture, ch8: &VirtualMachine, scale: f32) {
    let mut pixels = [255u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4];

    for (i, b) in ch8.display().iter().enumerate() {
        let idx = i * 4;
        if *b == 0u8 {
            for p in pixels[idx..idx + 3].iter_mut() {
                *p = 0;
            }
        }
    }

    unsafe {
        tex.update_from_pixels(&pixels, DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, 0, 0);
    }
    let mut sprite = Sprite::with_texture(&tex);
    sprite.set_scale((scale, scale));
    win.draw(&sprite);
    win.display();
}

fn main() {
    std::process::exit(run());
}
