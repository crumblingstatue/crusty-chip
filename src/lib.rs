//! CHIP-8 interpreter backend.
//! You can develop your own frontends for it.
//!
//! The reference frontend is
//! [crusty-chip-sfml](https://github.com/crumblingstatue/crusty-chip/tree/master/sfml).
//!

#![warn(missing_docs, trivial_casts, trivial_numeric_casts)]

use std::{fmt::Write, num::Wrapping};

mod ops;

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
    JumpToSysRoutine { addr: Semiword },
    JumpToAddress { addr: Semiword },
    CallSubroutine { addr: Semiword },
    SkipNextVxEq { x: Nibble, cmp_with: Byte },
    SkipNextVxNe { x: Nibble, cmp_with: Byte },
    SkipNextVxEqVy { x: Nibble, y: Nibble },
    SetVxByte { x: Nibble, to: Byte },
    AddVxByte { x: Nibble, rhs: Byte },
    SetVxToVy { x: Nibble, y: Nibble },
    SetVxToVxOrVy { x: Nibble, y: Nibble },
    SetVxToVxAndVy { x: Nibble, y: Nibble },
    SetVxToVxXorVy { x: Nibble, y: Nibble },
    AddVxVy { x: Nibble, y: Nibble },
    SubVxVy { x: Nibble, y: Nibble },
    SetVxToVyShr1 { x: Nibble, y: Nibble },
    SubnVxVy { x: Nibble, y: Nibble },
    SetVxToVyShl1 { x: Nibble, y: Nibble },
    SkipNextVxNeVy { x: Nibble, y: Nibble },
    SetI { to: Semiword },
    SetVxRandAnd { x: Nibble, and: Byte },
    DisplaySprite { x: Nibble, y: Nibble, n: Nibble },
    SkipNextKeyVxNotPressed { x: Nibble },
    SkipNextKeyVxPressed { x: Nibble },
    SetVxToDelayTimer { x: Nibble },
    WaitForKeypressStoreInVx { x: Nibble },
    SetDelayTimer { x: Nibble },
    SetSoundTimer { x: Nibble },
    AddVxToI { x: Nibble },
    SetIToLocOfDigitVx { x: Nibble },
    StoreBcdOfVxToI { x: Nibble },
    CopyV0ThroughVxToMem { x: Nibble },
    ReadV0ThroughVxFromMem { x: Nibble },
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
        0x0 => match nnn {
            0x0E0 => ClearDisplay,
            0x0EE => Return,
            _ => JumpToSysRoutine { addr: nnn },
        },
        0x1 => JumpToAddress { addr: nnn },
        0x2 => CallSubroutine { addr: nnn },
        0x3 => SkipNextVxEq { x, cmp_with: kk },
        0x4 => SkipNextVxNe { x, cmp_with: kk },
        0x5 => match n {
            0x0 => SkipNextVxEqVy { x, y },
            _ => Unknown,
        },
        0x6 => SetVxByte { x, to: kk },
        0x7 => AddVxByte { x, rhs: kk },
        0x8 => match n {
            0x0 => SetVxToVy { x, y },
            0x1 => SetVxToVxOrVy { x, y },
            0x2 => SetVxToVxAndVy { x, y },
            0x3 => SetVxToVxXorVy { x, y },
            0x4 => AddVxVy { x, y },
            0x5 => SubVxVy { x, y },
            0x6 => SetVxToVyShr1 { x, y },
            0x7 => SubnVxVy { x, y },
            0xE => SetVxToVyShl1 { x, y },
            _ => Unknown,
        },
        0x9 => match n {
            0x0 => SkipNextVxNeVy { x, y },
            _ => Unknown,
        },
        0xA => SetI { to: nnn },
        0xC => SetVxRandAnd { x, and: kk },
        0xD => DisplaySprite { x, y, n },
        0xE => match kk {
            0xA1 => SkipNextKeyVxNotPressed { x },
            0x9E => SkipNextKeyVxPressed { x },
            _ => Unknown,
        },
        0xF => match kk {
            0x07 => SetVxToDelayTimer { x },
            0x0A => WaitForKeypressStoreInVx { x },
            0x15 => SetDelayTimer { x },
            0x18 => SetSoundTimer { x },
            0x1E => AddVxToI { x },
            0x29 => SetIToLocOfDigitVx { x },
            0x33 => StoreBcdOfVxToI { x },
            0x55 => CopyV0ThroughVxToMem { x },
            0x65 => ReadV0ThroughVxFromMem { x },
            _ => Unknown,
        },
        _ => Unknown,
    }
}

const START_ADDR: u16 = 0x200;
/// The memory size of the Chip-8 virtual machine.
/// It doesn't make sense to feed it data something larger than this, so you can use this
/// to .e.g. reject files that are larger than this when loading the ROM.
pub const MEM_SIZE: usize = 4096;
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
    vx: usize,
}

/// A CHIP-8 virtual machine.
#[derive(Clone)]
pub struct VirtualMachine {
    ram: [u8; MEM_SIZE],
    v: [Wrapping<u8>; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: Wrapping<u8>,
    stack: [u16; 16],
    display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    display_updated: bool,
    keys: [bool; 16],
    keypress_wait: KeypressWait,
    halt: bool,
    /// Message log
    pub log: String,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        VirtualMachine::new()
    }
}

impl VirtualMachine {
    /// Constructs a new VirtualMachine.
    pub fn new() -> VirtualMachine {
        let mut ch8 = VirtualMachine {
            ram: [0; MEM_SIZE],
            v: [Wrapping(0); 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: START_ADDR,
            sp: Wrapping(0),
            stack: [0; 16],
            display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            display_updated: false,
            keys: [false; 16],
            keypress_wait: KeypressWait { wait: false, vx: 0 },
            halt: false,
            log: String::new(),
        };
        ch8.ram[0usize..5 * 0x10].copy_from_slice(&FONTSET);
        ch8
    }

    /// Loads a ROM into the VirtualMachine.
    ///
    /// ## Arguments ##
    /// * rom - ROM to load
    pub fn load_rom(&mut self, rom: &[u8]) {
        const MAX_ROM_LEN: usize = MEM_SIZE - START_ADDR as usize;
        let len = std::cmp::min(rom.len(), MAX_ROM_LEN);
        self.ram[START_ADDR as usize..START_ADDR as usize + len].copy_from_slice(&rom[..len]);
    }

    /// Does an interpretation cycle.
    pub fn do_cycle(&mut self) {
        if !self.halt {
            let ins = self.fetch_ins();
            self.dispatch(ins);
        }
    }

    // Decode instruction and execute it
    fn dispatch(&mut self, ins: u16) {
        use Instruction::*;
        match decode(ins) {
            ClearDisplay => self.clear_display(),
            Return => self.ret_from_subroutine(),
            JumpToSysRoutine { addr } => self.jump_to_sys_routine(addr as usize),
            JumpToAddress { addr } => self.jump_addr(addr),
            CallSubroutine { addr } => self.call_subroutine(addr),
            SkipNextVxEq { x, cmp_with } => self.skip_next_vx_eq(x as usize, cmp_with),
            SkipNextVxNe { x, cmp_with } => self.skip_next_vx_ne(x as usize, cmp_with),
            SkipNextVxEqVy { x, y } => self.skip_next_vx_eq_vy(x as usize, y as usize),
            SetVxByte { x, to } => self.set_vx_byte(x as usize, to),
            AddVxByte { x, rhs } => self.add_vx_byte(x as usize, rhs),
            SetVxToVy { x, y } => self.set_vx_to_vy(x as usize, y as usize),
            SetVxToVxOrVy { x, y } => self.set_vx_to_vx_or_vy(x as usize, y as usize),
            SetVxToVxAndVy { x, y } => self.set_vx_to_vx_and_vy(x as usize, y as usize),
            SetVxToVxXorVy { x, y } => self.set_vx_to_vx_xor_vy(x as usize, y as usize),
            AddVxVy { x, y } => self.add_vx_vy(x as usize, y as usize),
            SubVxVy { x, y } => self.sub_vx_vy(x as usize, y as usize),
            SetVxToVyShr1 { x, y } => self.set_vx_to_vy_shr_1(x as usize, y as usize),
            SubnVxVy { x, y } => self.subn_vx_vy(x as usize, y as usize),
            SetVxToVyShl1 { x, y } => self.set_vx_to_vy_shl_1(x as usize, y as usize),
            SkipNextVxNeVy { x, y } => self.skip_next_vx_ne_vy(x as usize, y as usize),
            SetI { to } => self.set_i(to),
            SetVxRandAnd { x, and } => self.set_vx_rand_and(x as usize, and),
            DisplaySprite { x, y, n } => self.display_sprite(x as usize, y as usize, n as usize),
            SkipNextKeyVxNotPressed { x } => self.skip_next_key_vx_not_pressed(x as usize),
            SkipNextKeyVxPressed { x } => self.skip_next_key_vx_pressed(x as usize),
            SetVxToDelayTimer { x } => self.set_vx_to_delay_timer(x as usize),
            WaitForKeypressStoreInVx { x } => self.wait_for_keypress_store_in_vx(x as usize),
            SetDelayTimer { x } => self.set_delay_timer(x as usize),
            SetSoundTimer { x } => self.set_sound_timer(x as usize),
            AddVxToI { x } => self.add_vx_to_i(x as usize),
            SetIToLocOfDigitVx { x } => self.set_i_to_loc_of_digit_vx(x as usize),
            StoreBcdOfVxToI { x } => self.store_bcd_of_vx_to_i(x as usize),
            CopyV0ThroughVxToMem { x } => self.copy_v0_through_vx_to_mem(u16::from(x)),
            ReadV0ThroughVxFromMem { x } => self.read_v0_through_vx_from_mem(u16::from(x)),
            Unknown => writeln!(self.log, "Unknown instruction: {:X}", ins).unwrap(),
        }
    }

    /// Gets the instruction that the program counter is pointing to.
    pub fn get_ins(&mut self) -> u16 {
        let b1 = self.ram.get(self.pc as usize).cloned().unwrap_or_else(|| {
            writeln!(self.log, "Out of bounds when getting instruction. Halted.").unwrap();
            self.halt = true;
            0
        });
        let b2 = self
            .ram
            .get((self.pc + 1) as usize)
            .cloned()
            .unwrap_or_else(|| {
                writeln!(self.log, "Out of bounds when getting instruction. Halted.").unwrap();
                self.halt = true;
                0
            });
        u16::from(b1) << 8 | u16::from(b2)
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
    pub fn press_key(&mut self, key: u8) {
        assert!(key <= 15);
        self.keys[usize::from(key)] = true;
        if self.keypress_wait.wait {
            self.v[self.keypress_wait.vx].0 = key;
            self.keypress_wait.wait = false;
        }
    }

    /// Releases a key on the hexadecimal keypad.
    ///
    /// `key` should be in the range `0..15`.
    pub fn release_key(&mut self, key: u8) {
        assert!(key <= 15);
        self.keys[usize::from(key)] = false;
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
        &self.display
    }
    /// Whether the VM is waiting for a key
    pub fn waiting_for_key(&self) -> bool {
        self.keypress_wait.wait
    }
    /// Clear the display updated flag. Use this after you rendered the display.
    pub fn clear_du_flag(&mut self) {
        self.display_updated = false;
    }
}
