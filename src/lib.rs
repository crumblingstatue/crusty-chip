//! CHIP-8 interpreter backend.
//! You can develop your own frontends for it.
//!
//! The reference frontend is
//! [crusty-chip-sfml](https://github.com/crumblingstatue/crusty-chip-sfml).
//!

#![feature(inclusive_range_syntax)]

#![warn(missing_docs)]

use std::num::Wrapping;
use std::ops::{Deref, DerefMut};
use std::{error, fmt};

mod ops;

extern crate bit_utils;

/// 4 bit value extracted from an instruction.
pub type Nibble = u8;
/// 8 bit value extracted from an instruction.
pub type Byte = u8;
/// 12 bit value extracted from an instruction.
pub type Semiword = u16;

#[allow(missing_docs)]
#[derive(Debug)]
/// A CHIP-8 instruction.
pub enum Instruction {
    ClearDisplay,
    Return,
    JumpToSysRoutine {
        addr: Semiword,
    },
    JumpToAddress {
        addr: Semiword,
    },
    CallSubroutine {
        addr: Semiword,
    },
    SkipNextVxEq {
        x: Nibble,
        cmp_with: Byte,
    },
    SkipNextVxNe {
        x: Nibble,
        cmp_with: Byte,
    },
    SkipNextVxEqVy {
        x: Nibble,
        y: Nibble,
    },
    SetVxByte {
        x: Nibble,
        to: Byte,
    },
    AddVxByte {
        x: Nibble,
        rhs: Byte,
    },
    SetVxToVy {
        x: Nibble,
        y: Nibble,
    },
    SetVxToVxOrVy {
        x: Nibble,
        y: Nibble,
    },
    SetVxToVxAndVy {
        x: Nibble,
        y: Nibble,
    },
    SetVxToVxXorVy {
        x: Nibble,
        y: Nibble,
    },
    AddVxVy {
        x: Nibble,
        y: Nibble,
    },
    SubVxVy {
        x: Nibble,
        y: Nibble,
    },
    SetVxToVyShr1 {
        x: Nibble,
        y: Nibble,
    },
    SubnVxVy {
        x: Nibble,
        y: Nibble,
    },
    SetVxToVyShl1 {
        x: Nibble,
        y: Nibble,
    },
    SkipNextVxNeVy {
        x: Nibble,
        y: Nibble,
    },
    SetI {
        to: Semiword,
    },
    SetVxRandAnd {
        x: Nibble,
        and: Byte,
    },
    DisplaySprite {
        x: Nibble,
        y: Nibble,
        n: Nibble,
    },
    SkipNextKeyVxNotPressed {
        x: Nibble,
    },
    SkipNextKeyVxPressed {
        x: Nibble,
    },
    SetVxToDelayTimer {
        x: Nibble,
    },
    WaitForKeypressStoreInVx {
        x: Nibble,
    },
    SetDelayTimer {
        x: Nibble,
    },
    SetSoundTimer {
        x: Nibble,
    },
    AddVxToI {
        x: Nibble,
    },
    SetIToLocOfDigitVx {
        x: Nibble,
    },
    StoreBcdOfVxToI {
        x: Nibble,
    },
    CopyV0ThroughVxToMem {
        x: Nibble,
    },
    ReadV0ThroughVxFromMem {
        x: Nibble,
    },
    Unknown,
}

/// Decode a raw instruction into an Instruction structure.
pub fn decode(ins: u16) -> Instruction {
    use self::Instruction::*;

    // A 12-bit value, the lowest 12 bits of the instruction
    let nnn: Semiword = ins & 0x0FFF;
    // A 4-bit value, the lowest 4 bits of the instruction
    let n: Nibble = (ins & 0x000F) as Nibble;
    // A 4-bit value, the lower 4 bits of the high byte of the instruction
    let x: Nibble = ((ins & 0x0F00) >> 8) as Nibble;
    // A 4-bit value, the upper 4 bits of the low byte of the instruction
    let y: Nibble = ((ins & 0x00F0) >> 4) as Nibble;
    // kk or byte - An 8-bit value, the lowest 8 bits of the instruction
    let kk: Byte = (ins & 0x00FF) as Nibble;

    match (ins & 0xF000) >> 12 {
        0x0 => {
            match nnn {
                0x0E0 => ClearDisplay,
                0x0EE => Return,
                _ => JumpToSysRoutine { addr: nnn },
            }
        }
        0x1 => JumpToAddress { addr: nnn },
        0x2 => CallSubroutine { addr: nnn },
        0x3 => {
            SkipNextVxEq {
                x: x,
                cmp_with: kk,
            }
        }
        0x4 => {
            SkipNextVxNe {
                x: x,
                cmp_with: kk,
            }
        }
        0x5 => {
            match n {
                0x0 => SkipNextVxEqVy { x: x, y: y },
                _ => Unknown,
            }
        }
        0x6 => SetVxByte { x: x, to: kk },
        0x7 => AddVxByte { x: x, rhs: kk },
        0x8 => {
            match n {
                0x0 => SetVxToVy { x: x, y: y },
                0x1 => SetVxToVxOrVy { x: x, y: y },
                0x2 => SetVxToVxAndVy { x: x, y: y },
                0x3 => SetVxToVxXorVy { x: x, y: y },
                0x4 => AddVxVy { x: x, y: y },
                0x5 => SubVxVy { x: x, y: y },
                0x6 => SetVxToVyShr1 { x: x, y: y },
                0x7 => SubnVxVy { x: x, y: y },
                0xE => SetVxToVyShl1 { x: x, y: y },
                _ => Unknown,
            }
        }
        0x9 => {
            match n {
                0x0 => SkipNextVxNeVy { x: x, y: y },
                _ => Unknown,
            }
        }
        0xA => SetI { to: nnn },
        0xC => SetVxRandAnd { x: x, and: kk },
        0xD => DisplaySprite { x: x, y: y, n: n },
        0xE => {
            match kk {
                0xA1 => SkipNextKeyVxNotPressed { x: x },
                0x9E => SkipNextKeyVxPressed { x: x },
                _ => Unknown,
            }
        }
        0xF => {
            match kk {
                0x07 => SetVxToDelayTimer { x: x },
                0x0A => WaitForKeypressStoreInVx { x: x },
                0x15 => SetDelayTimer { x: x },
                0x18 => SetSoundTimer { x: x },
                0x1E => AddVxToI { x: x },
                0x29 => SetIToLocOfDigitVx { x: x },
                0x33 => StoreBcdOfVxToI { x: x },
                0x55 => CopyV0ThroughVxToMem { x: x },
                0x65 => ReadV0ThroughVxFromMem { x: x },
                _ => Unknown,
            }
        }
        _ => Unknown,
    }
}

const START_ADDR: u16 = 0x200;
const MEM_SIZE: usize = 4096;
/// The width of the Chip8's display in pixels.
pub const DISPLAY_WIDTH: usize = 64;
/// The height of the Chip8's display in pixels.
pub const DISPLAY_HEIGHT: usize = 32;

static FONTSET: [u8; 5 * 0x10] = [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70,
                                  0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
                                  0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
                                  0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
                                  0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0,
                                  0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
                                  0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
                                  0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80];

#[derive(Clone, Copy)]
struct KeypressWait {
    wait: bool,
    vx: usize,
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
    keypress_wait: KeypressWait,
}

/// Error that can happen when loading a rom.
#[derive(Debug)]
pub enum RomLoadError {
    /// Rom is too big
    TooBig(usize),
}

const MAX_ROM_LEN: usize = MEM_SIZE - START_ADDR as usize;

impl fmt::Display for RomLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RomLoadError::TooBig(size) => {
                write!(f,
                       "Rom size ({}) is too big. The maximum valid rom size is {}.",
                       size,
                       MAX_ROM_LEN)
            }
        }
    }
}

impl error::Error for RomLoadError {
    fn description(&self) -> &'static str {
        "rom load error"
    }
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
                vx: 0,
            },
        };
        ch8.ram[0usize..5 * 0x10].copy_from_slice(&FONTSET);
        ch8
    }

    /// Loads a ROM into the VirtualMachine.
    ///
    /// ## Arguments ##
    /// * rom - ROM to load
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), RomLoadError> {
        let len = rom.len();
        if len > MAX_ROM_LEN {
            return Err(RomLoadError::TooBig(len));
        }
        self.ram[START_ADDR as usize..START_ADDR as usize + len].copy_from_slice(rom);
        Ok(())
    }

    /// Does an interpretation cycle.
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
        use Instruction::*;
        use ops::*;
        match decode(ins) {
            ClearDisplay => clear_display(self),
            Return => ret_from_subroutine(self),
            JumpToSysRoutine { addr } => jump_to_sys_routine(self, addr as usize),
            JumpToAddress { addr } => jump_addr(self, addr),
            CallSubroutine { addr } => call_subroutine(self, addr as usize),
            SkipNextVxEq { x, cmp_with } => skip_next_vx_eq(self, x as usize, cmp_with),
            SkipNextVxNe { x, cmp_with } => skip_next_vx_ne(self, x as usize, cmp_with),
            SkipNextVxEqVy { x, y } => skip_next_vx_eq_vy(self, x as usize, y as usize),
            SetVxByte { x, to } => set_vx_byte(self, x as usize, to),
            AddVxByte { x, rhs } => add_vx_byte(self, x as usize, rhs),
            SetVxToVy { x, y } => set_vx_to_vy(self, x as usize, y as usize),
            SetVxToVxOrVy { x, y } => set_vx_to_vx_or_vy(self, x as usize, y as usize),
            SetVxToVxAndVy { x, y } => set_vx_to_vx_and_vy(self, x as usize, y as usize),
            SetVxToVxXorVy { x, y } => set_vx_to_vx_xor_vy(self, x as usize, y as usize),
            AddVxVy { x, y } => add_vx_vy(self, x as usize, y as usize),
            SubVxVy { x, y } => sub_vx_vy(self, x as usize, y as usize),
            SetVxToVyShr1 { x, y } => set_vx_to_vy_shr_1(self, x as usize, y as usize),
            SubnVxVy { x, y } => subn_vx_vy(self, x as usize, y as usize),
            SetVxToVyShl1 { x, y } => set_vx_to_vy_shl_1(self, x as usize, y as usize),
            SkipNextVxNeVy { x, y } => skip_next_vx_ne_vy(self, x as usize, y as usize),
            SetI { to } => set_i(self, to),
            SetVxRandAnd { x, and } => set_vx_rand_and(self, x as usize, and),
            DisplaySprite { x, y, n } => display_sprite(self, x as usize, y as usize, n as usize),
            SkipNextKeyVxNotPressed { x } => skip_next_key_vx_not_pressed(self, x as usize),
            SkipNextKeyVxPressed { x } => skip_next_key_vx_pressed(self, x as usize),
            SetVxToDelayTimer { x } => set_vx_to_delay_timer(self, x as usize),
            WaitForKeypressStoreInVx { x } => wait_for_keypress_store_in_vx(self, x as usize),
            SetDelayTimer { x } => set_delay_timer(self, x as usize),
            SetSoundTimer { x } => set_sound_timer(self, x as usize),
            AddVxToI { x } => add_vx_to_i(self, x as usize),
            SetIToLocOfDigitVx { x } => set_i_to_loc_of_digit_vx(self, x as usize),
            StoreBcdOfVxToI { x } => store_bcd_of_vx_to_i(self, x as usize),
            CopyV0ThroughVxToMem { x } => copy_v0_through_vx_to_mem(self, x as usize),
            ReadV0ThroughVxFromMem { x } => read_v0_through_vx_from_mem(self, x as usize),
            Unknown => panic!("Unknown instruction: {}", ins),
        }
    }

    /// Gets the instruction that the program counter is pointing to.
    pub fn get_ins(&self) -> u16 {
        let b1 = self.ram[self.pc as usize];
        let b2 = self.ram[(self.pc + 1) as usize];
        (b1 as u16) << 8 | b2 as u16
    }

    /// Returns the value of the program counter.
    pub fn pc(&self) -> u16 {
        self.pc
    }

    fn fetch_ins(&mut self) -> u16 {
        let ins = self.get_ins();
        self.pc += 2;
        ins
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
    pub fn display_updated(&self) -> bool {
        self.display_updated
    }
    /// Returns the contents of the display.
    pub fn display(&self) -> &[u8; DISPLAY_WIDTH * DISPLAY_HEIGHT] {
        &*self.display
    }
}
