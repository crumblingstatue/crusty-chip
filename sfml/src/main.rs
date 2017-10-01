extern crate crusty_chip;
extern crate getopts;
extern crate sfml;

use getopts::Options;

fn usage(progname: &str, opts: &Options) -> String {
    let brief = format!("{} rom_file", progname);
    format!("Usage: {}", opts.usage(&brief))
}

fn run() -> i32 {
    use crusty_chip::{decode, VirtualMachine, DISPLAY_HEIGHT, DISPLAY_WIDTH};
    use sfml::graphics::{Color, RenderTarget, RenderWindow, Sprite, Texture, Transformable};
    use sfml::system::Clock;
    use sfml::window::{ContextSettings, Event, Style, VideoMode};
    use std::fs::File;
    use std::io::Read;

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
    // TODO: Dunno if this always reads the whole file
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

    let mut tex = Texture::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32).unwrap();

    win.clear(&Color::BLACK);
    win.display();
    let mut saved_states = [None; 10];
    let mut printed_info = false;
    let mut cycles_made: u64 = 0;

    loop {
        let mut advance = false;
        let mut redisplay = false;
        while let Some(event) = win.poll_event() {
            use sfml::window::Key;
            fn key_mapping(code: Key) -> Option<usize> {
                match code {
                    Key::Num1 => Some(1),
                    Key::Num2 => Some(2),
                    Key::Num3 => Some(3),
                    Key::Num4 => Some(0xC),
                    Key::Q => Some(4),
                    Key::W => Some(5),
                    Key::E => Some(6),
                    Key::R => Some(0xD),
                    Key::A => Some(7),
                    Key::S => Some(8),
                    Key::D => Some(9),
                    Key::F => Some(0xE),
                    Key::Z => Some(0xA),
                    Key::X => Some(0),
                    Key::C => Some(0xB),
                    Key::V => Some(0xF),
                    _ => None,
                }
            }
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
                    } else if code == Key::Period {
                        advance = true;
                    } else if let Some(key) = key_mapping(code) {
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
                Event::KeyReleased { code, .. } => if let Some(key) = key_mapping(code) {
                    ch8.release_key(key);
                },
                Event::GainedFocus => redisplay = true,
                _ => {}
            }
        }

        if redisplay {
            win.display();
        }

        if clock.elapsed_time().as_seconds() >= 1.0 / 60.0 {
            ch8.decrement_timers();
            clock.restart();
        }

        if paused {
            std::thread::sleep(std::time::Duration::from_millis(25));
        }

        if paused && !printed_info {
            let raw_ins = ch8.get_ins();
            println!(
                "Cycle {}, pc @ {:x}, ins: {:?} ({:x})",
                cycles_made,
                ch8.pc(),
                decode(raw_ins),
                raw_ins
            );
            printed_info = true;
        }

        if !paused || advance {
            ch8.do_cycle();
            cycles_made += 1;
            printed_info = false;
        }

        if ch8.display_updated() {
            let mut pixels = [255u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 4];

            for (i, b) in ch8.display().iter().enumerate() {
                let idx = i * 4;
                if *b == 0u8 {
                    for p in pixels[idx..idx + 3].iter_mut() {
                        *p = 0;
                    }
                }
            }

            tex.update_from_pixels(&pixels, DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, 0, 0);
            let mut sprite = Sprite::with_texture(&tex);
            sprite.set_scale((scale as f32, scale as f32));
            win.draw(&sprite);
            win.display();
        }
    }
}

fn main() {
    std::process::exit(run());
}
