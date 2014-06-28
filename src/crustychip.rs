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
        self.pc += 2;

        match ins & 0xF000 {
            0x1000 => self.jump_addr(ins & 0x0FFF),
            0x3000 => self.skip_next_vx_eq(((ins & 0x0F00) >> 8) as uint, (ins & 0x00FF) as u8),
            0x6000 => self.set_vx_byte(((ins & 0x0F00) >> 8) as uint, (ins & 0x00FF) as u8),
            0x7000 => self.add_vx_byte(((ins & 0x0F00) >> 8) as uint, (ins & 0x00FF) as u8),
            0xA000 => self.set_i(ins & 0x0FFF),
            0xC000 => self.set_vx_rand_and(((ins & 0x0F00) >> 8) as uint, (ins & 0x00FF) as u8),
            0xD000 => self.display_sprite(((ins & 0x0F00) >> 8) as uint, ((ins & 0x00F0) >> 4) as uint, ((ins & 0x000F)) as uint),
            _ => fail!("Unkown instruction: {:x}", ins)
        }
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
    }

    fn add_vx_byte(&mut self, x: uint, byte: u8) {
        self.v[x] += byte;
    }

    fn set_vx_byte(&mut self, x: uint, byte: u8) {
        self.v[x] = byte;
    }

    fn jump_addr(&mut self, addr: u16) {
        self.pc = addr;
    }
}
