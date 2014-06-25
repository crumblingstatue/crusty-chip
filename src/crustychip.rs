static START_ADDR: u16 = 0x200;
static MEM_SIZE: uint = 4096;
pub static DISPLAY_WIDTH: uint = 64;
pub static DISPLAY_HEIGHT: uint = 32;

static fontset: [[u8, .. 5], .. 0x10] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

pub struct Chip8 {
    ram: [u8, .. MEM_SIZE],
    v: [u8, .. 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: [u16, .. 16],
    display: [u8, .. DISPLAY_WIDTH * DISPLAY_HEIGHT]
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            ram: [0u8, .. MEM_SIZE],
            v: [0u8, .. 16],
            i: 0u16,
            delay_timer: 0u8,
            sound_timer: 0u8,
            pc: START_ADDR,
            sp: 0,
            stack: [0u16, .. 16],
            display: [0u8, .. DISPLAY_WIDTH * DISPLAY_HEIGHT]
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        use std::slice::bytes::copy_memory;
        let len = self.ram.len();
        copy_memory(self.ram.mut_slice(START_ADDR as uint, len), rom);
    }

    pub fn get_display(&self) -> [u8, .. DISPLAY_WIDTH * DISPLAY_HEIGHT] {
        self.display
    }

    pub fn do_cycle(&mut self) {
        let ins = self.get_ins();

        match ins & 0xF000 {
            0x1000 => self.jump_addr(ins & 0x0FFF),
            0x3000 => self.skip_next_vx_eq(((ins & 0x0F00) << 8) as uint, (ins & 0x00FF) as u8),
            0x7000 => self.add_vx_byte(((ins & 0x0F00) << 8) as uint, (ins & 0x00FF) as u8),
            0xA000 => self.set_i(ins & 0x0FFF),
            0xC000 => self.set_vx_rand_and(((ins & 0x0F00) << 8) as uint, (ins & 0x00FF) as u8),
            0xD000 => self.display_sprite(((ins & 0x0F00) << 8) as uint, ((ins & 0x00F0) << 8) as uint, ((ins & 0x000F) << 8) as uint),
            _ => fail!("Unkown instruction: {:x}", ins)
        }

        self.pc += 2;
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

    fn skip_next_vx_eq(&mut self, x: uint, to: u8) {
        if self.v[x] == to {
            self.pc += 2;
        }
    }

    fn display_sprite(&mut self, x: uint, y: uint, n: uint) {
        let bytes = self.ram.slice(self.i as uint, self.i as uint + n);
        let start = y * DISPLAY_WIDTH + x;
        let mut i = 0;

        for b in bytes.iter() {
            let before = self.display[start + i];
            self.display[start + i] ^= *b;

            if before != 0 && self.display[start + i] == 0 {
                self.v[0xf] = 1;
            } else {
                self.v[0xf] = 0;
            }

            i += 1;
        }
    }

    fn add_vx_byte(&mut self, x: uint, byte: u8) {
        self.v[x] = byte;
    }

    fn jump_addr(&mut self, addr: u16) {
        self.pc = addr;
    }
}
