//! CHIP-8 emulator backend.
//! You can develop your own frontends for it.
//!
//! The reference frontend is
//! [crusty-chip-sfml](https://github.com/crumblingstatue/crusty-chip-sfml).
//!
//! The CHIP-8 technical documentation in the comments is copied from
//! http://devernay.free.fr/hacks/chip8/C8TECH10.HTM,
//! Copyright (c) Thomas P. Greene.

#![experimental]

use std::slice::bytes::copy_memory;

mod ops;

static START_ADDR: u16 = 0x200;
static MEM_SIZE: uint = 4096;
pub static DISPLAY_WIDTH: uint = 64;
pub static DISPLAY_HEIGHT: uint = 32;

static FONTSET: [u8, .. 5 * 0x10] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// Draw callback.
///
/// Takes a slice of u8, where each u8 is a pixel.
///
/// A **non-zero** pixel is visible, a **zero** pixel is not visible.
///
/// The coordinate system is from top-left to bottom-right.
///
/// Example:
///
/// ```rust
///     use crusty_chip::{ DISPLAY_WIDTH, VirtualMachine };
///     // Dump the pixels to stdout
///     fn dump_pixels(pixels: &[u8]) {
///         for (i, px) in pixels.iter().enumerate() {
///             match *px {
///                 0 => print!(" "),
///                 _ => print!("#")
///             }
///             if i % DISPLAY_WIDTH == 0 {
///                 print!("\n");
///             }
///         }
///     }
///     let mut ch8 = VirtualMachine::new(dump_pixels);
///     // ...
/// ```
pub type DrawCallback<'a> = |pixels: &[u8]|: 'a;

struct KeypressWait {
    wait: bool,
    vx: uint
}

/// CHIP-8 virtual machine
pub struct VirtualMachine<'a> {
    ram: [u8, .. MEM_SIZE],
    v: [u8, .. 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: [u16, .. 16],
    display: [u8, .. DISPLAY_WIDTH * DISPLAY_HEIGHT],
    draw_callback: DrawCallback<'a>,
    keys: [bool, .. 16],
    keypress_wait: KeypressWait
}

impl <'a> VirtualMachine <'a> {

    /// Constructs a new `VirtualMachine`.
    ///
    /// ## Arguments ##
    /// * draw_callback - Callback used when drawing
    pub fn new(draw_callback: DrawCallback<'a>) -> VirtualMachine<'a> {
        let mut ch8 = VirtualMachine {
            ram: [0, .. MEM_SIZE],
            v: [0, .. 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: START_ADDR,
            sp: 0,
            stack: [0, .. 16],
            display: [0, .. DISPLAY_WIDTH * DISPLAY_HEIGHT],
            draw_callback: draw_callback,
            keys: [false, .. 16],
            keypress_wait: KeypressWait {
                wait: false,
                vx: 0
            }
        };
        copy_memory(ch8.ram.slice_mut(0u, 5 * 0x10), FONTSET);
        ch8
    }

    /// Load a ROM
    ///
    /// ## Arguments ##
    /// * rom - ROM to load
    pub fn load_rom(&mut self, rom: &[u8]) {
        let len = self.ram.len();
        copy_memory(self.ram.slice_mut(START_ADDR as uint, len), rom);
    }

    /// Do an emulation cycle.
    pub fn do_cycle(&mut self) {
        if self.keypress_wait.wait {
            return;
        }

        let ins = self.fetch_ins();
        self.dispatch(ins);
    }

    // Decode instruction and execute it
    fn dispatch(&mut self, ins: u16) {
        // A 12-bit value, the lowest 12 bits of the instruction
        let nnn = ins & 0x0FFF;
        // A 4-bit value, the lowest 4 bits of the instruction
        let n = ins & 0x000F;
        // A 4-bit value, the lower 4 bits of the high byte of the instruction
        let x = (ins & 0x0F00) >> 8;
        // A 4-bit value, the upper 4 bits of the low byte of the instruction
        let y = (ins & 0x00F0) >> 4;
        // kk or byte - An 8-bit value, the lowest 8 bits of the instruction
        let kk = (ins & 0x00FF) as u8;

        match (ins & 0xF000) >> 12 {
            0x0 => match nnn {
                0x0E0 => ops::clear_display(self, ),
                0x0EE => ops::ret_from_subroutine(self, ),
                _ => ops::jump_to_sys_routine(self, 0)
            },
            0x1 => ops::jump_addr(self, nnn),
            0x2 => ops::call_subroutine(self, nnn as uint),
            0x3 => ops::skip_next_vx_eq(self, x as uint, kk),
            0x4 => ops::skip_next_vx_ne(self, x as uint, kk),
            0x5 => match n {
                0x0 => ops::skip_next_vx_eq_vy(self, x as uint, y as uint),
                _ => fail!("Unknown 0x5XXX instruction: {:x}", ins)
            },
            0x6 => ops::set_vx_byte(self, x as uint, kk),
            0x7 => ops::add_vx_byte(self, x as uint, kk),
            0x8 => {
                let (x, y) = (x as uint, y as uint);
                match n {
                    0x0 => ops::set_vx_to_vy(self, x, y),
                    0x1 => ops::set_vx_to_vx_or_vy(self, x, y),
                    0x2 => ops::set_vx_to_vx_and_vy(self, x, y),
                    0x3 => ops::set_vx_to_vx_xor_vy(self, x, y),
                    0x4 => ops::add_vx_vy(self, x, y),
                    0x5 => ops::sub_vx_vy(self, x, y),
                    0x6 => ops::set_vx_to_vx_shr_1(self, x),
                    0xE => ops::set_vx_to_vx_shl_1(self, x),
                    _ => fail!("Unknown 0x8XXX instruction: {:x}", ins)
                }
            },
            0x9 => match n {
                0x0 => ops::skip_next_vx_ne_vy(self, x as uint, y as uint),
                _ => fail!("Unknown 0x9XXX instruction: {:x}", ins)
            },
            0xA => ops::set_i(self, nnn),
            0xC => ops::set_vx_rand_and(self, x as uint, kk),
            0xD => ops::display_sprite(self, x as uint, y as uint, n as uint),
            0xE => match kk {
                0xA1 => ops::skip_next_key_vx_not_pressed(self, x as uint),
                0x9E => ops::skip_next_key_vx_pressed(self, x as uint),
                _ => fail!("Unknown 0xEXXX instruction: {:x}", ins)
            },
            0xF => {
                match kk {
                    0x07 => ops::set_vx_to_delay_timer(self, x as uint),
                    0x0A => ops::wait_for_keypress_store_in_vx(self, x as uint),
                    0x15 => ops::set_delay_timer(self, x as uint),
                    0x18 => ops::set_sound_timer(self, x as uint),
                    0x1E => ops::add_vx_to_i(self, x as uint),
                    0x29 => ops::set_i_to_loc_of_digit_vx(self, x as uint),
                    0x33 => ops::store_bcd_of_vx_to_i(self, x as uint),
                    0x55 => ops::copy_v0_through_vx_to_mem(self, x as uint),
                    0x65 => ops::read_v0_through_vx_from_mem(self, x as uint),
                    _ => fail!("Unknown 0xFXXX instruction: {:x}", ins)
                }
            },
            _ => fail!("Unknown instruction: {:04x}", ins)
        }
    }

    fn fetch_ins(&mut self) -> u16 {
        let b1 = self.ram[self.pc as uint];
        let b2 = self.ram[(self.pc + 1) as uint];
        self.pc += 2;
        b1 as u16 << 8 | b2 as u16
    }

    /// Press a key on the hexadecimal keypad
    ///
    /// `key` should be in the range `0..15`
    pub fn press_key(&mut self, key: uint) {
        assert!(key <= 15);
        self.keys[key] = true;
        if self.keypress_wait.wait {
            self.v[self.keypress_wait.vx] = key as u8;
            self.keypress_wait.wait = false;
        }
    }

    /// Release a key on the hexadecimal keypad
    ///
    /// `key` should be in the range `0..15`
    pub fn release_key(&mut self, key: uint) {
        assert!(key <= 15);
        self.keys[key] = false;
    }

    /// Decrement the sound and delay timers
    ///
    /// They should be decremented at a rate of 60 Hz
    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
