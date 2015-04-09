//! CHIP-8 emulator backend.
//! You can develop your own frontends for it.
//!
//! The reference frontend is
//! [crusty-chip-sfml](https://github.com/crumblingstatue/crusty-chip-sfml).
//!
//! The CHIP-8 technical documentation in the comments is copied from
//! http://devernay.free.fr/hacks/chip8/C8TECH10.HTM,
//! Copyright (c) Thomas P. Greene.

#![feature(core)]
#![warn(missing_docs)]

use std::slice::bytes::copy_memory;
use std::num::Wrapping;
use std::ops::{Deref, DerefMut};

mod ops;

const START_ADDR: u16 = 0x200;
const MEM_SIZE: usize = 4096;
/// The width of the Chip8's display in pixels.
pub const DISPLAY_WIDTH: usize = 64;
/// The height of the Chip8's display in pixels.
pub const DISPLAY_HEIGHT: usize = 32;

static FONTSET: [u8; 5 * 0x10] = [
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

#[derive(Clone, Copy)]
struct KeypressWait {
    wait: bool,
    vx: usize
}

macro_rules! array_wrap (
    ($name:ident, $typ:ty) => (
        #[derive(Copy)]
        struct $name($typ);
        impl Clone for $name {
            fn clone(&self) -> $name { *self }
        }
        impl Deref for $name {
            type Target = $typ;
            fn deref(&self) -> &$typ { &self.0 }
        }
        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut $typ { &mut self.0 }
        }
    )
);

array_wrap!(DisplayArray, [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT]);
array_wrap!(MemArray, [u8; MEM_SIZE]);

/// A CHIP-8 virtual machine.
#[derive(Clone, Copy)]
pub struct VirtualMachine {
    ram: MemArray,
    v: [Wrapping<u8>; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    display: DisplayArray,
    display_updated: bool,
    keys: [bool; 16],
    keypress_wait: KeypressWait
}

impl VirtualMachine {

    /// Constructs a new VirtualMachine.
    pub fn new() -> VirtualMachine {
        let mut ch8 = VirtualMachine {
            ram: MemArray([0; MEM_SIZE]),
            v: [Wrapping(0); 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: START_ADDR,
            sp: 0,
            stack: [0; 16],
            display: DisplayArray([0; DISPLAY_WIDTH * DISPLAY_HEIGHT]),
            display_updated: false,
            keys: [false; 16],
            keypress_wait: KeypressWait {
                wait: false,
                vx: 0
            }
        };
        copy_memory(&FONTSET, &mut ch8.ram[0usize..5 * 0x10]);
        ch8
    }

    /// Loads a ROM into the VirtualMachine.
    ///
    /// ## Arguments ##
    /// * rom - ROM to load
    pub fn load_rom(&mut self, rom: &[u8]) {
        let len = self.ram.len();
        copy_memory(rom, &mut self.ram[START_ADDR as usize .. len]);
    }

    /// Does an emulation cycle.
    pub fn do_cycle(&mut self) {
        self.display_updated = false;
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
            0x2 => ops::call_subroutine(self, nnn as usize),
            0x3 => ops::skip_next_vx_eq(self, x as usize, kk),
            0x4 => ops::skip_next_vx_ne(self, x as usize, kk),
            0x5 => match n {
                0x0 => ops::skip_next_vx_eq_vy(self, x as usize, y as usize),
                _ => panic!("Unknown 0x5XXX instruction: {:x}", ins)
            },
            0x6 => ops::set_vx_byte(self, x as usize, kk),
            0x7 => ops::add_vx_byte(self, x as usize, kk),
            0x8 => {
                let (x, y) = (x as usize, y as usize);
                match n {
                    0x0 => ops::set_vx_to_vy(self, x, y),
                    0x1 => ops::set_vx_to_vx_or_vy(self, x, y),
                    0x2 => ops::set_vx_to_vx_and_vy(self, x, y),
                    0x3 => ops::set_vx_to_vx_xor_vy(self, x, y),
                    0x4 => ops::add_vx_vy(self, x, y),
                    0x5 => ops::sub_vx_vy(self, x, y),
                    0x6 => ops::set_vx_to_vx_shr_1(self, x),
                    0xE => ops::set_vx_to_vx_shl_1(self, x),
                    _ => panic!("Unknown 0x8XXX instruction: {:x}", ins)
                }
            },
            0x9 => match n {
                0x0 => ops::skip_next_vx_ne_vy(self, x as usize, y as usize),
                _ => panic!("Unknown 0x9XXX instruction: {:x}", ins)
            },
            0xA => ops::set_i(self, nnn),
            0xC => ops::set_vx_rand_and(self, x as usize, kk),
            0xD => ops::display_sprite(self, x as usize, y as usize, n as usize),
            0xE => match kk {
                0xA1 => ops::skip_next_key_vx_not_pressed(self, x as usize),
                0x9E => ops::skip_next_key_vx_pressed(self, x as usize),
                _ => panic!("Unknown 0xEXXX instruction: {:x}", ins)
            },
            0xF => {
                match kk {
                    0x07 => ops::set_vx_to_delay_timer(self, x as usize),
                    0x0A => ops::wait_for_keypress_store_in_vx(self, x as usize),
                    0x15 => ops::set_delay_timer(self, x as usize),
                    0x18 => ops::set_sound_timer(self, x as usize),
                    0x1E => ops::add_vx_to_i(self, x as usize),
                    0x29 => ops::set_i_to_loc_of_digit_vx(self, x as usize),
                    0x33 => ops::store_bcd_of_vx_to_i(self, x as usize),
                    0x55 => ops::copy_v0_through_vx_to_mem(self, x as usize),
                    0x65 => ops::read_v0_through_vx_from_mem(self, x as usize),
                    _ => panic!("Unknown 0xFXXX instruction: {:x}", ins)
                }
            },
            _ => panic!("Unknown instruction: {:04x}", ins)
        }
    }

    fn fetch_ins(&mut self) -> u16 {
        let b1 = self.ram[self.pc as usize];
        let b2 = self.ram[(self.pc + 1) as usize];
        self.pc += 2;
        (b1 as u16) << 8 | b2 as u16
    }

    /// Presses a key on the hexadecimal keypad.
    ///
    /// `key` should be in the range `0..15`.
    pub fn press_key(&mut self, key: usize) {
        assert!(key <= 15);
        self.keys[key] = true;
        if self.keypress_wait.wait {
            self.v[self.keypress_wait.vx].0 = key as u8;
            self.keypress_wait.wait = false;
        }
    }

    /// Releases a key on the hexadecimal keypad.
    ///
    /// `key` should be in the range `0..15`.
    pub fn release_key(&mut self, key: usize) {
        assert!(key <= 15);
        self.keys[key] = false;
    }

    /// Decrements the sound and delay timers.
    ///
    /// They should be decremented at a rate of 60 Hz.
    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    /// Returns whether the display has been updated.
    pub fn display_updated(&self) -> bool { self.display_updated }
    /// Returns the contents of the display.
    pub fn display(&self) -> &[u8; DISPLAY_WIDTH * DISPLAY_HEIGHT] { &*self.display }
}
