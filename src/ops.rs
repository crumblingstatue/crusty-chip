use {
    super::VirtualMachine,
    std::{fmt::Write, num::Wrapping},
};

impl VirtualMachine {
    pub(super) fn jump_to_sys_routine(&mut self, _addr: usize) {
        // Do nothing
    }

    pub(super) fn clear_display(&mut self) {
        for px in self.display.iter_mut() {
            *px = 0;
        }
    }

    pub(super) fn ret_from_subroutine(&mut self) {
        self.pc = self.stack[self.sp.0 as usize];
        self.sp -= 1;
    }

    pub(super) fn jump_addr(&mut self, addr: u16) {
        self.pc = addr;
    }

    pub(super) fn call_subroutine(&mut self, addr: u16) {
        self.sp += 1;
        match self.stack.get_mut(self.sp.0 as usize) {
            Some(mem) => *mem = self.pc,
            None => {
                writeln!(self.log, "Stack out of bounds. Ignoring write.").unwrap();
            }
        };
        self.pc = addr;
    }

    pub(super) fn skip_next_vx_eq(&mut self, x: usize, to: u8) {
        if self.v[x].0 == to {
            self.pc += 2;
        }
    }

    pub(super) fn skip_next_vx_ne(&mut self, x: usize, to: u8) {
        if self.v[x].0 != to {
            self.pc += 2;
        }
    }

    pub(super) fn skip_next_vx_eq_vy(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    pub(super) fn set_vx_byte(&mut self, x: usize, byte: u8) {
        self.v[x].0 = byte;
    }

    pub(super) fn add_vx_byte(&mut self, x: usize, byte: u8) {
        self.v[x] += Wrapping(byte);
    }

    pub(super) fn set_vx_to_vy(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    pub(super) fn set_vx_to_vx_or_vy(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }

    pub(super) fn set_vx_to_vx_and_vy(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }

    pub(super) fn set_vx_to_vx_xor_vy(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
    }

    pub(super) fn add_vx_vy(&mut self, x: usize, y: usize) {
        let cond = u16::from(self.v[x].0) + u16::from(self.v[y].0) > 255;
        self.v[0xF].0 = cond.into();
        self.v[x] += self.v[y];
    }

    pub(super) fn sub_vx_vy(&mut self, x: usize, y: usize) {
        let cond = self.v[x] > self.v[y];
        self.v[0xF].0 = cond.into();
        self.v[x] -= self.v[y];
    }

    pub(super) fn subn_vx_vy(&mut self, x: usize, y: usize) {
        let cond = self.v[y] > self.v[x];
        self.v[0xF].0 = cond.into();
        self.v[x] = self.v[y] - self.v[x];
    }

    pub(super) fn set_vx_to_vy_shr_1(&mut self, x: usize, y: usize) {
        self.v[0xF].0 = nth_bit(self.v[y].0, 7);
        self.v[x] = self.v[y] >> 1;
    }

    pub(super) fn set_vx_to_vy_shl_1(&mut self, x: usize, y: usize) {
        self.v[0xF].0 = nth_bit(self.v[y].0, 0);
        self.v[x] = self.v[y] << 1;
    }
    pub(super) fn skip_next_vx_ne_vy(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    pub(super) fn set_i(&mut self, to: u16) {
        self.i = to;
    }

    pub(super) fn set_vx_rand_and(&mut self, x: usize, to: u8) {
        use rand::Rng;
        let mut rgen = rand::thread_rng();
        self.v[x].0 = rgen.gen::<u8>() & to;
    }

    pub(super) fn display_sprite(&mut self, vx: usize, vy: usize, n: usize) {
        use super::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

        self.v[0xF].0 = 0;

        for y in 0..n {
            let b = self.ram[self.i as usize + y];
            for x in 0..8 {
                let xx = x + self.v[vx].0 as usize;
                let yy = y + self.v[vy].0 as usize;

                if xx < DISPLAY_WIDTH && yy < DISPLAY_HEIGHT {
                    let idx = yy * DISPLAY_WIDTH + xx;
                    if b & (0b1000_0000 >> x) != 0 {
                        if self.display[idx] == 1 {
                            self.v[0xF].0 = 1;
                        }
                        self.display[idx] ^= 1;
                    }
                }
            }
        }

        self.display_updated = true;
    }

    pub(super) fn skip_next_key_vx_not_pressed(&mut self, x: usize) {
        if !self.keys[self.v[x].0 as usize] {
            self.pc += 2;
        }
    }

    pub(super) fn skip_next_key_vx_pressed(&mut self, x: usize) {
        if self.keys[self.v[x].0 as usize] {
            self.pc += 2;
        }
    }

    pub(super) fn set_vx_to_delay_timer(&mut self, x: usize) {
        self.v[x].0 = self.delay_timer;
    }

    pub(super) fn wait_for_keypress_store_in_vx(&mut self, x: usize) {
        self.keypress_wait.wait = true;
        self.keypress_wait.vx = x;
    }

    pub(super) fn set_delay_timer(&mut self, x: usize) {
        self.delay_timer = self.v[x].0;
    }

    pub(super) fn set_sound_timer(&mut self, x: usize) {
        self.sound_timer = self.v[x].0;
    }

    pub(super) fn add_vx_to_i(&mut self, x: usize) {
        self.i += u16::from(self.v[x].0);
    }

    pub(super) fn set_i_to_loc_of_digit_vx(&mut self, x: usize) {
        self.i = u16::from((self.v[x] * Wrapping(5)).0);
    }

    pub(super) fn store_bcd_of_vx_to_i(&mut self, x: usize) {
        let num = self.v[x].0; // TODO: Should probably be wrapping
        let h = num / 100;
        let t = (num - h * 100) / 10;
        let o = num - h * 100 - t * 10;
        self.ram[self.i as usize] = h;
        self.ram[self.i as usize + 1] = t;
        self.ram[self.i as usize + 2] = o;
    }

    pub(super) fn copy_v0_through_vx_to_mem(&mut self, x: u16) {
        for pos in 0..=x {
            self.ram[(self.i + pos) as usize] = self.v[pos as usize].0;
        }
        self.i += x + 1;
    }

    pub(super) fn read_v0_through_vx_from_mem(&mut self, x: u16) {
        for pos in 0..=x {
            self.v[pos as usize].0 = self.ram[(self.i + pos) as usize];
        }
        self.i += x + 1;
    }
}

fn nth_bit(byte: u8, pos: usize) -> u8 {
    use bit_utils::BitInformation;
    byte.has_x_bit(7 - pos).into()
}

#[test]
fn test_nth_bit() {
    assert_eq!(nth_bit(0b10000000, 0), 1);
    for i in 1..8 {
        assert_eq!(nth_bit(0b10000000, i), 0);
    }
    assert_eq!(nth_bit(0b01000000, 1), 1);
    assert_eq!(nth_bit(0b00100000, 2), 1);
    assert_eq!(nth_bit(0b00010000, 3), 1);
    assert_eq!(nth_bit(0b00001000, 4), 1);
    assert_eq!(nth_bit(0b00000100, 5), 1);
    assert_eq!(nth_bit(0b00000010, 6), 1);
    assert_eq!(nth_bit(0b00000001, 7), 1);
}

#[test]
fn test_strore_bcd_of_vx_to_i() {
    let mut vm = VirtualMachine::new();
    vm.v[0].0 = 146;
    vm.i = 0;
    vm.store_bcd_of_vx_to_i(0);
    assert!(vm.ram[0] == 1);
    assert!(vm.ram[1] == 4);
    assert!(vm.ram[2] == 6);
}
