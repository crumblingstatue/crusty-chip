//! CHIP-8 emulator backend.
//! You can develop your own frontends for it.
//!
//! The reference frontend is
//! [crusty-chip-sfml](https://github.com/crumblingstatue/crusty-chip-sfml).
//!
//! The CHIP-8 technical documentation in the comments is copied from
//! http://devernay.free.fr/hacks/chip8/C8TECH10.HTM,
//! Copyright (c) Thomas P. Greene.

use std::slice::bytes::copy_memory;

static START_ADDR: u16 = 0x200;
static MEM_SIZE: uint = 4096;
pub static DISPLAY_WIDTH: uint = 64;
pub static DISPLAY_HEIGHT: uint = 32;

static fontset: [u8, .. 5 * 0x10] = [
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
///     use crustychip::{ DISPLAY_WIDTH, Chip8 };
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
///     let mut ch8 = Chip8::new(dump_pixels);
///     // ...
/// ```
pub type DrawCallback<'a> = |pixels: &[u8]|: 'a;

struct KeypressWait {
    wait: bool,
    vx: uint
}

/// CHIP-8 virtual machine
pub struct Chip8<'a> {
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

impl <'a> Chip8 <'a> {

    /// Create a new Chip8 VM
    ///
    /// ## Arguments ##
    /// * draw_callback - Callback used when drawing
    pub fn new(draw_callback: DrawCallback<'a>) -> Chip8<'a> {
        let mut ch8 = Chip8 {
            ram: [0u8, .. MEM_SIZE],
            v: [0u8, .. 16],
            i: 0u16,
            delay_timer: 0u8,
            sound_timer: 0u8,
            pc: START_ADDR,
            sp: 0,
            stack: [0u16, .. 16],
            display: [0u8, .. DISPLAY_WIDTH * DISPLAY_HEIGHT],
            draw_callback: draw_callback,
            keys: [false, .. 16],
            keypress_wait: KeypressWait {
                wait: false,
                vx: 0
            }
        };
        copy_memory(ch8.ram.mut_slice(0u, 5 * 0x10), fontset);
        ch8
    }

    /// Load a ROM
    ///
    /// ## Arguments ##
    /// * rom - ROM to load
    pub fn load_rom(&mut self, rom: &[u8]) {
        let len = self.ram.len();
        copy_memory(self.ram.mut_slice(START_ADDR as uint, len), rom);
    }

    /// Do an emulation cycle.
    pub fn do_cycle(&mut self) {
        if self.keypress_wait.wait {
            return;
        }

        let ins = self.get_ins();
        self.pc += 2;
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
                0x0E0 => self.clear_display(),
                0x0EE => self.ret_from_subroutine(),
                _ => self.jump_to_sys_routine(0)
            },
            0x1 => self.jump_addr(nnn),
            0x2 => self.call_subroutine(nnn as uint),
            0x3 => self.skip_next_vx_eq(x as uint, kk),
            0x4 => self.skip_next_vx_ne(x as uint, kk),
            0x5 => match n {
                0x0 => self.skip_next_vx_eq_vy(x as uint, y as uint),
                _ => fail!("Unknown 0x5XXX instruction: {:x}", ins)
            },
            0x6 => self.set_vx_byte(x as uint, kk),
            0x7 => self.add_vx_byte(x as uint, kk),
            0x8 => {
                let (x, y) = (x as uint, y as uint);
                match n {
                    0x0 => self.set_vx_to_vy(x, y),
                    0x2 => self.set_vx_to_vx_and_vy(x, y),
                    0x3 => self.set_vx_to_vx_xor_vy(x, y),
                    0x4 => self.add_vx_vy(x, y),
                    0x5 => self.sub_vx_vy(x, y),
                    0xE => self.set_vx_to_vx_shl_1(x),
                    _ => fail!("Unknown 0x8XXX instruction: {:x}", ins)
                }
            },
            0x9 => match n {
                0x0 => self.skip_next_vx_ne_vy(x as uint, y as uint),
                _ => fail!("Unknown 0x9XXX instruction: {:x}", ins)
            },
            0xA => self.set_i(nnn),
            0xC => self.set_vx_rand_and(x as uint, kk),
            0xD => self.display_sprite(x as uint, y as uint, n as uint),
            0xE => match kk {
                0xA1 => self.skip_next_key_vx_not_pressed(x as uint),
                _ => fail!("Unknown 0xEXXX instruction: {:x}", ins)
            },
            0xF => {
                match kk {
                    0x0A => self.wait_for_keypress_store_in_vx(x as uint),
                    0x07 => self.set_vx_to_delay_timer(x as uint),
                    0x15 => self.set_delay_timer(x as u8),
                    0x18 => self.set_sound_timer(x as u8),
                    0x1E => self.add_vx_to_i(x as uint),
                    0x29 => self.set_i_to_loc_of_digit_vx(x as uint),
                    0x33 => self.store_bcd_of_vx_to_i(x as uint),
                    0x55 => self.copy_v0_through_vx_to_mem(x as uint),
                    0x65 => self.read_v0_through_vx_from_mem(x as uint),
                    _ => fail!("Unknown 0xFXXX instruction: {:x}", ins)
                }
            },
            _ => fail!("Unknown instruction: {:04x}", ins)
        }
    }

    // 0nnn - SYS addr
    // Jump to a machine code routine at nnn.
    //
    // This instruction is only used on the old computers on which Chip-8 was
    // originally implemented. It is ignored by modern interpreters.
    fn jump_to_sys_routine(&mut self, addr: uint) {
        // Do nothing
    }

    // 00E0 - CLS
    // Clear the display.
    fn clear_display(&mut self) {
        for px in self.display.mut_iter() {
            *px = 0;
        }
    }

    // 00EE - RET
    // Return from a subroutine.
    //
    // The interpreter sets the program counter to the address at the top of
    // the stack, then subtracts 1 from the stack pointer.
    fn ret_from_subroutine(&mut self) {
        self.pc = self.stack[self.sp as uint];
        self.sp -= 1;
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    //
    // The interpreter sets the program counter to nnn.
    fn jump_addr(&mut self, addr: u16) {
        self.pc = addr;
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    //
    // The interpreter increments the stack pointer, then puts the current PC
    // on the top of the stack. The PC is then set to nnn.
    fn call_subroutine(&mut self, addr: uint) {
        self.sp += 1;
        self.stack[self.sp as uint] = self.pc;
        self.pc = addr as u16;
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    //
    // The interpreter compares register Vx to kk, and if they are equal,
    // increments the program counter by 2.
    fn skip_next_vx_eq(&mut self, x: uint, to: u8) {
        if self.v[x] == to {
            self.pc += 2;
        }
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    //
    // The interpreter compares register Vx to kk, and if they are not equal,
    // increments the program counter by 2.
    fn skip_next_vx_ne(&mut self, x: uint, to: u8) {
        if self.v[x] != to {
            self.pc += 2;
        }
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    //
    // The interpreter compares register Vx to register Vy, and if they are
    // equal, increments the program counter by 2.
    fn skip_next_vx_eq_vy(&mut self, x: uint, y: uint) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    //
    // The interpreter puts the value kk into register Vx.
    fn set_vx_byte(&mut self, x: uint, byte: u8) {
        self.v[x] = byte;
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    //
    // Adds the value kk to the value of register Vx, then stores the
    // result in Vx.
    fn add_vx_byte(&mut self, x: uint, byte: u8) {
        self.v[x] += byte;
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    //
    // Stores the value of register Vy in register Vx.
    fn set_vx_to_vy(&mut self, x: uint, y: uint) {
        self.v[x] = self.v[y];
    }

    fn get_ins(&self) -> u16 {
        let b1 = self.ram[self.pc as uint];
        let b2 = self.ram[(self.pc + 1) as uint];
        b1 as u16 << 8 | b2 as u16
    }

    fn set_i(&mut self, to: u16) {
        self.i = to;
    }

    fn set_vx_rand_and(&mut self, x: uint, to: u8) {
        use std::rand::{task_rng, Rng};
        let mut rgen = task_rng();
        self.v[x] = rgen.gen::<u8>() & to;
    }

    fn display_sprite(&mut self, vx: uint, vy: uint, n: uint) {
        let x_off = self.v[vx] as uint;
        let y_off = self.v[vy] as uint * DISPLAY_WIDTH;
        let offset = x_off + y_off;

        for mut y in range(0u, n) {
            if y >= DISPLAY_HEIGHT {
                y = y - DISPLAY_HEIGHT;
            }
            let b = self.ram[self.i as uint + y];
            for mut x in range(0u, 8) {
                if x >= DISPLAY_WIDTH {
                    x = x - DISPLAY_WIDTH;
                }
                let idx = offset + (y * DISPLAY_WIDTH) + x;
                if idx < DISPLAY_WIDTH * DISPLAY_HEIGHT {
                    self.display[idx] ^= b & (0b10000000 >> x);
                } else {
                    println!("Warning: Out of bounds VRAM write: {}", idx);
                }
            }
        }

        (self.draw_callback)(self.display);
    }

    fn add_vx_vy(&mut self, x: uint, y: uint) {
        let result = (self.v[x] + self.v[y]) as u16;
        self.v[0xF] = (result > 255) as u8;
        self.v[x] = result as u8;
    }

    fn sub_vx_vy(&mut self, x: uint, y: uint) {
        self.v[0xF] = (self.v[x] > self.v[y]) as u8;
        self.v[x] -= self.v[y];
    }

    fn add_vx_to_i(&mut self, x: uint) {
        self.i += x as u16;
    }

    fn copy_v0_through_vx_to_mem(&mut self, x: uint) {
        if x == 0 {
            return;
        }
        copy_memory(self.ram.mut_slice(self.i as uint, self.i as uint + x),
                    self.v.slice(0, x));
    }

    fn read_v0_through_vx_from_mem(&mut self, x: uint) {
        if x == 0 {
            return;
        }
        copy_memory(self.v.mut_slice(0, x),
                    self.ram.slice(self.i as uint, self.i as uint + x));
    }

    fn skip_next_vx_ne_vy(&mut self, x: uint, y: uint) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    //
    // The interpreter takes the decimal value of Vx, and places the hundreds
    // digit in memory at location in I, the tens digit at location I+1,
    // and the ones digit at location I+2.
    fn store_bcd_of_vx_to_i(&mut self, x: uint) {
        let num = self.v[x];
        let h = num / 100;
        let t = (num - h * 100) / 10;
        let o = (num - h * 100 - t * 10);
        self.ram[self.i as uint] = h;
        self.ram[self.i as uint + 1] = t;
        self.ram[self.i as uint + 2] = o;
    }

    // Fx29 - LD F, Vx
    // Set I = location of sprite for digit Vx.
    //
    // The value of I is set to the location for the hexadecimal sprite
    // corresponding to the value of Vx. See section 2.4, Display, for more
    // information on the Chip-8 hexadecimal font.
    //
    // For crusty-chip, the fontset is stored at 0x000
    fn set_i_to_loc_of_digit_vx(&mut self, x: uint) {
        self.i = (self.v[x] * 5) as u16;
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    //
    // All execution stops until a key is pressed, then the value of that key
    // is stored in Vx.
    fn wait_for_keypress_store_in_vx(&mut self, x: uint) {
        self.keypress_wait.wait = true;
        self.keypress_wait.vx = x;
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    //
    // DT is set equal to the value of Vx.
    fn set_delay_timer(&mut self, x: u8) {
        self.delay_timer = x;
    }

    // Fx18 - LD ST, Vx
    // Set sound timer = Vx.
    //
    // ST is set equal to the value of Vx.
    fn set_sound_timer(&mut self, x: u8) {
        self.sound_timer = x;
    }

    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    //
    // The value of DT is placed into Vx.
    fn set_vx_to_delay_timer(&mut self, x: uint) {
        self.v[x] = self.delay_timer;
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    //
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores
    // the result in Vx. An exclusive OR compares the corrseponding bits from
    // two values, and if the bits are not both the same, then the
    // corresponding bit in the result is set to 1. Otherwise, it is 0.
    fn set_vx_to_vx_xor_vy(&mut self, x: uint, y: uint) {
        self.v[x] ^= self.v[y];
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    //
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise
    // to 0. Then Vx is multiplied by 2.
    fn set_vx_to_vx_shl_1(&mut self, x: uint) {
        // TODO: Is this just a left shift by 1?
        self.v[x] <<= 1;
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    //
    // Performs a bitwise AND on the values of Vx and Vy, then stores the
    // result in Vx. A bitwise AND compares the corrseponding bits from two
    // values, and if both bits are 1, then the same bit in the result is also
    // 1. Otherwise, it is 0.
    fn set_vx_to_vx_and_vy(&mut self, x: uint, y: uint) {
        self.v[x] &= self.v[y];
    }

    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    //
    // Checks the keyboard, and if the key corresponding to the value of
    // Vx is currently in the up position, PC is increased by 2.
    fn skip_next_key_vx_not_pressed(&mut self, x: uint) {
        if !self.keys[self.v[x] as uint] {
            self.pc += 2;
        }
    }

    /// Press a key on the hexadecimal keypad
    ///
    /// `key` should be in the range `0..15`
    pub fn press_key(&mut self, key: uint) {
        assert!(key >= 0 && key <= 15);
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
        assert!(key >= 0 && key <= 15);
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

#[test]
fn test_strore_bcd_of_vx_to_i() {
    let mut ch8 = Chip8::new(|_| {});
    ch8.v[0] = 146;
    ch8.i = 0;
    ch8.store_bcd_of_vx_to_i(0);
    assert!(ch8.ram[0] == 1);
    assert!(ch8.ram[1] == 4);
    assert!(ch8.ram[2] == 6);
}
