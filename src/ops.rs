use super::VirtualMachine;
use std::num::Wrapping;

pub fn jump_to_sys_routine(_vm: &mut VirtualMachine, _addr: usize) {
    // Do nothing
}

pub fn clear_display(vm: &mut VirtualMachine) {
    for px in vm.display.iter_mut() {
        *px = 0;
    }
}

pub fn ret_from_subroutine(vm: &mut VirtualMachine) {
    vm.pc = vm.stack[vm.sp as usize];
    vm.sp -= 1;
}

pub fn jump_addr(vm: &mut VirtualMachine, addr: u16) {
    vm.pc = addr;
}

pub fn call_subroutine(vm: &mut VirtualMachine, addr: usize) {
    vm.sp += 1;
    vm.stack[vm.sp as usize] = vm.pc;
    vm.pc = addr as u16;
}

pub fn skip_next_vx_eq(vm: &mut VirtualMachine, x: usize, to: u8) {
    if vm.v[x].0 == to {
        vm.pc += 2;
    }
}

pub fn skip_next_vx_ne(vm: &mut VirtualMachine, x: usize, to: u8) {
    if vm.v[x].0 != to {
        vm.pc += 2;
    }
}

pub fn skip_next_vx_eq_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    if vm.v[x] == vm.v[y] {
        vm.pc += 2;
    }
}

pub fn set_vx_byte(vm: &mut VirtualMachine, x: usize, byte: u8) {
    vm.v[x].0 = byte;
}

pub fn add_vx_byte(vm: &mut VirtualMachine, x: usize, byte: u8) {
    vm.v[x] = vm.v[x] + Wrapping(byte);
}

pub fn set_vx_to_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[x] = vm.v[y];
}

pub fn set_vx_to_vx_or_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[x] = vm.v[x] | vm.v[y];
}

pub fn set_vx_to_vx_and_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[x] = vm.v[x] & vm.v[y];
}

pub fn set_vx_to_vx_xor_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[x] = vm.v[x] ^ vm.v[y];
}

pub fn add_vx_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[0xF].0 = if vm.v[x].0 as u16 + vm.v[y].0 as u16 > 255 {
        1
    } else {
        0
    };
    vm.v[x] = vm.v[x] + vm.v[y];
}

pub fn sub_vx_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[0xF].0 = if vm.v[x] > vm.v[y] { 1 } else { 0 };
    vm.v[x] = vm.v[x] - vm.v[y];
}

pub fn subn_vx_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[0xF].0 = if vm.v[y] > vm.v[x] { 1 } else { 0 };
    vm.v[x] = vm.v[y] - vm.v[x];
}

pub fn set_vx_to_vy_shr_1(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[0xF].0 = nth_bit(vm.v[y].0, 7);
    vm.v[x] = vm.v[y] >> 1;
}

pub fn set_vx_to_vy_shl_1(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[0xF].0 = nth_bit(vm.v[y].0, 0);
    vm.v[x] = vm.v[y] << 1;
}

fn nth_bit(byte: u8, pos: usize) -> u8 {
    use bit_utils::BitInformation;
    if byte.has_x_bit(7 - pos) { 1 } else { 0 }
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

pub fn skip_next_vx_ne_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    if vm.v[x] != vm.v[y] {
        vm.pc += 2;
    }
}

pub fn set_i(vm: &mut VirtualMachine, to: u16) {
    vm.i = to;
}

extern crate rand;

pub fn set_vx_rand_and(vm: &mut VirtualMachine, x: usize, to: u8) {
    use self::rand::Rng;
    let mut rgen = rand::thread_rng();
    vm.v[x].0 = rgen.gen::<u8>() & to;
}

pub fn display_sprite(vm: &mut VirtualMachine, vx: usize, vy: usize, n: usize) {
    use super::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

    vm.v[0xF].0 = 0;

    for y in 0..n {
        let b = vm.ram[vm.i as usize + y];
        for x in 0..8 {
            let xx = x + vm.v[vx].0 as usize;
            let yy = y + vm.v[vy].0 as usize;

            if xx < DISPLAY_WIDTH && yy < DISPLAY_HEIGHT {
                let idx = yy * DISPLAY_WIDTH + xx;
                if b & (0b10000000 >> x) != 0 {
                    if vm.display[idx] == 1 {
                        vm.v[0xF].0 = 1;
                    }
                    vm.display[idx] ^= 1;
                }
            }
        }
    }

    vm.display_updated = true;
}

pub fn skip_next_key_vx_not_pressed(vm: &mut VirtualMachine, x: usize) {
    if !vm.keys[vm.v[x].0 as usize] {
        vm.pc += 2;
    }
}

pub fn skip_next_key_vx_pressed(vm: &mut VirtualMachine, x: usize) {
    if vm.keys[vm.v[x].0 as usize] {
        vm.pc += 2;
    }
}

pub fn set_vx_to_delay_timer(vm: &mut VirtualMachine, x: usize) {
    vm.v[x].0 = vm.delay_timer;
}

pub fn wait_for_keypress_store_in_vx(vm: &mut VirtualMachine, x: usize) {
    vm.keypress_wait.wait = true;
    vm.keypress_wait.vx = x;
}

pub fn set_delay_timer(vm: &mut VirtualMachine, x: usize) {
    vm.delay_timer = vm.v[x].0;
}

pub fn set_sound_timer(vm: &mut VirtualMachine, x: usize) {
    vm.sound_timer = vm.v[x].0;
}

pub fn add_vx_to_i(vm: &mut VirtualMachine, x: usize) {
    vm.i += vm.v[x].0 as u16;
}

pub fn set_i_to_loc_of_digit_vx(vm: &mut VirtualMachine, x: usize) {
    vm.i = (vm.v[x] * Wrapping(5)).0 as u16;
}

pub fn store_bcd_of_vx_to_i(vm: &mut VirtualMachine, x: usize) {
    let num = vm.v[x].0; // TODO: Should probably be wrapping
    let h = num / 100;
    let t = (num - h * 100) / 10;
    let o = num - h * 100 - t * 10;
    vm.ram[vm.i as usize] = h;
    vm.ram[vm.i as usize + 1] = t;
    vm.ram[vm.i as usize + 2] = o;
}

pub fn copy_v0_through_vx_to_mem(vm: &mut VirtualMachine, x: usize) {
    for pos in 0...x {
        vm.ram[(vm.i + pos as u16) as usize] = vm.v[pos as usize].0;
    }
    vm.i += (x + 1) as u16;
}

pub fn read_v0_through_vx_from_mem(vm: &mut VirtualMachine, x: usize) {
    for pos in 0...x {
        vm.v[pos as usize].0 = vm.ram[(vm.i + pos as u16) as usize];
    }
    vm.i += (x + 1) as u16;
}

#[test]
fn test_strore_bcd_of_vx_to_i() {
    let mut vm = VirtualMachine::new();
    vm.v[0].0 = 146;
    vm.i = 0;
    store_bcd_of_vx_to_i(&mut vm, 0);
    assert!(vm.ram[0] == 1);
    assert!(vm.ram[1] == 4);
    assert!(vm.ram[2] == 6);
}
