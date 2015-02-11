use super::VirtualMachine;

// 0nnn - SYS addr
// Jump to a machine code routine at nnn.
//
// This instruction is only used on the old computers on which Chip-8 was
// originally implemented. It is ignored by modern interpreters.
pub fn jump_to_sys_routine(_vm: &mut VirtualMachine, _addr: usize) {
    // Do nothing
}

// 00E0 - CLS
// Clear the display.
pub fn clear_display(vm: &mut VirtualMachine) {
    for px in vm.display.iter_mut() {
        *px = 0;
    }
}

// 00EE - RET
// Return from a subroutine.
//
// The interpreter sets the program counter to the address at the top of
// the stack, then subtracts 1 from the stack pointer.
pub fn ret_from_subroutine(vm: &mut VirtualMachine) {
    vm.pc = vm.stack[vm.sp as usize];
    vm.sp -= 1;
}

// 1nnn - JP addr
// Jump to location nnn.
//
// The interpreter sets the program counter to nnn.
pub fn jump_addr(vm: &mut VirtualMachine, addr: u16) {
    vm.pc = addr;
}

// 2nnn - CALL addr
// Call subroutine at nnn.
//
// The interpreter increments the stack pointer, then puts the current PC
// on the top of the stack. The PC is then set to nnn.
pub fn call_subroutine(vm: &mut VirtualMachine, addr: usize) {
    vm.sp += 1;
    vm.stack[vm.sp as usize] = vm.pc;
    vm.pc = addr as u16;
}

// 3xkk - SE Vx, byte
// Skip next instruction if Vx = kk.
//
// The interpreter compares register Vx to kk, and if they are equal,
// increments the program counter by 2.
pub fn skip_next_vx_eq(vm: &mut VirtualMachine, x: usize, to: u8) {
    if vm.v[x] == to {
        vm.pc += 2;
    }
}

// 4xkk - SNE Vx, byte
// Skip next instruction if Vx != kk.
//
// The interpreter compares register Vx to kk, and if they are not equal,
// increments the program counter by 2.
pub fn skip_next_vx_ne(vm: &mut VirtualMachine, x: usize, to: u8) {
    if vm.v[x] != to {
        vm.pc += 2;
    }
}

// 5xy0 - SE Vx, Vy
// Skip next instruction if Vx = Vy.
//
// The interpreter compares register Vx to register Vy, and if they are
// equal, increments the program counter by 2.
pub fn skip_next_vx_eq_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    if vm.v[x] == vm.v[y] {
        vm.pc += 2;
    }
}

// 6xkk - LD Vx, byte
// Set Vx = kk.
//
// The interpreter puts the value kk into register Vx.
pub fn set_vx_byte(vm: &mut VirtualMachine, x: usize, byte: u8) {
    vm.v[x] = byte;
}

// 7xkk - ADD Vx, byte
// Set Vx = Vx + kk.
//
// Adds the value kk to the value of register Vx, then stores the
// result in Vx.
pub fn add_vx_byte(vm: &mut VirtualMachine, x: usize, byte: u8) {
    vm.v[x] += byte;
}

// 8xy0 - LD Vx, Vy
// Set Vx = Vy.
//
// Stores the value of register Vy in register Vx.
pub fn set_vx_to_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[x] = vm.v[y];
}

// 8xy1 - OR Vx, Vy
// Set Vx = Vx OR Vy.
//
// Performs a bitwise OR on the values of Vx and Vy, then stores the result
// in Vx. A bitwise OR compares the corrseponding bits from two values,
// and if either bit is 1, then the same bit in the result is also 1.
// Otherwise, it is 0.
pub fn set_vx_to_vx_or_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[x] |= vm.v[y];
}

// 8xy2 - AND Vx, Vy
// Set Vx = Vx AND Vy.
//
// Performs a bitwise AND on the values of Vx and Vy, then stores the
// result in Vx. A bitwise AND compares the corrseponding bits from two
// values, and if both bits are 1, then the same bit in the result is also
// 1. Otherwise, it is 0.
pub fn set_vx_to_vx_and_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[x] &= vm.v[y];
}

// 8xy3 - XOR Vx, Vy
// Set Vx = Vx XOR Vy.
//
// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores
// the result in Vx. An exclusive OR compares the corrseponding bits from
// two values, and if the bits are not both the same, then the
// corresponding bit in the result is set to 1. Otherwise, it is 0.
pub fn set_vx_to_vx_xor_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[x] ^= vm.v[y];
}

// 8xy4 - ADD Vx, Vy
// Set Vx = Vx + Vy, set VF = carry.
//
// The values of Vx and Vy are added together. If the result is greater than
// 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits
// of the result are kept, and stored in Vx.
pub fn add_vx_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    let result = (vm.v[x] + vm.v[y]) as u16;
    vm.v[0xF] = if result > 255 {1} else {0};
    vm.v[x] = result as u8;
}

// 8xy5 - SUB Vx, Vy
// Set Vx = Vx - Vy, set VF = NOT borrow.
//
// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx,
// and the results stored in Vx.
pub fn sub_vx_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    vm.v[0xF] = if vm.v[x] > vm.v[y] {1} else {0};
    vm.v[x] -= vm.v[y];
}

// 8xy6 - SHR Vx {, Vy}
// Set Vx = Vx SHR 1.
//
// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
// Then Vx is divided by 2.
pub fn set_vx_to_vx_shr_1(vm: &mut VirtualMachine, x: usize) {
    vm.v[0xF] = if check_bit(vm.v[x], 0) {1} else {0};
    vm.v[x] /= 2;
}

// 8xyE - SHL Vx {, Vy}
// Set Vx = Vx SHL 1.
//
// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise
// to 0. Then Vx is multiplied by 2.
pub fn set_vx_to_vx_shl_1(vm: &mut VirtualMachine, x: usize) {
    vm.v[0xF] = if check_bit(vm.v[x], 7) {1} else {0};
    vm.v[x] *= 2;
}

fn check_bit(byte: u8, pos: usize) -> bool {
    byte & (1 << pos) != 0
}

#[test]
fn test_check_bit() {
    assert!(check_bit(0b10000000, 7));
    for i in 0..7 {
        assert!(!check_bit(0b10000000, i));
    }
    assert!(check_bit(0b01000000, 6));
    assert!(check_bit(0b00100000, 5));
    assert!(check_bit(0b00010000, 4));
    assert!(check_bit(0b00001000, 3));
    assert!(check_bit(0b00000100, 2));
    assert!(check_bit(0b00000010, 1));
    assert!(check_bit(0b00000001, 0));
}

// 9xy0 - SNE Vx, Vy
// Skip next instruction if Vx != Vy.
//
// The values of Vx and Vy are compared, and if they are not equal,
// the program counter is increased by 2.
pub fn skip_next_vx_ne_vy(vm: &mut VirtualMachine, x: usize, y: usize) {
    if vm.v[x] != vm.v[y] {
        vm.pc += 2;
    }
}

// Annn - LD I, addr
// Set I = nnn.
//
// The value of register I is set to nnn.
pub fn set_i(vm: &mut VirtualMachine, to: u16) {
    vm.i = to;
}

extern crate rand;

// Cxkk - RND Vx, byte
// Set Vx = random byte AND kk.
//
// The interpreter generates a random number from 0 to 255, which is then ANDed
// with the value kk. The results are stored in Vx.
// See instruction 8xy2 for more information on AND.
pub fn set_vx_rand_and(vm: &mut VirtualMachine, x: usize, to: u8) {
    use self::rand::Rng;
    let mut rgen = rand::thread_rng();
    vm.v[x] = rgen.gen::<u8>() & to;
}

// Dxyn - DRW Vx, Vy, nibble
// Display n-byte sprite starting at memory location I at (Vx, Vy),
// set VF = collision.
//
// The interpreter reads n bytes from memory, starting at the address stored in
// I. These bytes are then displayed as sprites on screen at coordinates
// (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any
// pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite
// is positioned so part of it is outside the coordinates of the display, it
// wraps around to the opposite side of the screen. See instruction 8xy3 for
// more information on XOR, and section 2.4, Display, for more information on
// the Chip-8 screen and sprites.
pub fn display_sprite(vm: &mut VirtualMachine, vx: usize, vy: usize, n: usize) {
    use super::{ DISPLAY_WIDTH, DISPLAY_HEIGHT };

    vm.v[0xF] = 0;

    for mut y in (0us..n) {
        let b = vm.ram[vm.i as usize + y];
        for mut x in (0us.. 8) {
            let xx = x + (vm.v[vx] as usize % DISPLAY_WIDTH);
            let yy = y + (vm.v[vy] as usize % DISPLAY_HEIGHT);

            if xx < DISPLAY_WIDTH && yy < DISPLAY_HEIGHT {
                let idx = yy * DISPLAY_WIDTH + xx;
                if b & (0b10000000 >> x) != 0 {
                    if vm.display[idx] == 1 {
                        vm.v[0xF] = 1;
                    }
                    vm.display[idx] ^= 1;
                }
            }
        }
    }

    (vm.draw_callback)(&vm.display);
}

// ExA1 - SKNP Vx
// Skip next instruction if key with the value of Vx is not pressed.
//
// Checks the keyboard, and if the key corresponding to the value of
// Vx is currently in the up position, PC is increased by 2.
pub fn skip_next_key_vx_not_pressed(vm: &mut VirtualMachine, x: usize) {
    if !vm.keys[vm.v[x] as usize] {
        vm.pc += 2;
    }
}

// Ex9E - SKP Vx
// Skip next instruction if key with the value of Vx is pressed.
//
// Checks the keyboard, and if the key corresponding to the value of Vx is
// currently in the down position, PC is increased by 2.
pub fn skip_next_key_vx_pressed(vm: &mut VirtualMachine, x: usize) {
    if vm.keys[vm.v[x] as usize] {
        vm.pc += 2;
    }
}

// Fx07 - LD Vx, DT
// Set Vx = delay timer value.
//
// The value of DT is placed into Vx.
pub fn set_vx_to_delay_timer(vm: &mut VirtualMachine, x: usize) {
    vm.v[x] = vm.delay_timer;
}

// Fx0A - LD Vx, K
// Wait for a key press, store the value of the key in Vx.
//
// All execution stops until a key is pressed, then the value of that key
// is stored in Vx.
pub fn wait_for_keypress_store_in_vx(vm: &mut VirtualMachine, x: usize) {
    vm.keypress_wait.wait = true;
    vm.keypress_wait.vx = x;
}

// Fx15 - LD DT, Vx
// Set delay timer = Vx.
//
// DT is set equal to the value of Vx.
pub fn set_delay_timer(vm: &mut VirtualMachine, x: usize) {
    vm.delay_timer = vm.v[x];
}

// Fx18 - LD ST, Vx
// Set sound timer = Vx.
//
// ST is set equal to the value of Vx.
pub fn set_sound_timer(vm: &mut VirtualMachine, x: usize) {
    vm.sound_timer = vm.v[x];
}

// Fx1E - ADD I, Vx
// Set I = I + Vx.
//
// The values of I and Vx are added, and the results are stored in I.
pub fn add_vx_to_i(vm: &mut VirtualMachine, x: usize) {
    vm.i += vm.v[x] as u16;
}

// Fx29 - LD F, Vx
// Set I = location of sprite for digit Vx.
//
// The value of I is set to the location for the hexadecimal sprite
// corresponding to the value of Vx. See section 2.4, Display, for more
// information on the Chip-8 hexadecimal font.
//
// For crusty-chip, the fontset is stored at 0x000
pub fn set_i_to_loc_of_digit_vx(vm: &mut VirtualMachine, x: usize) {
    vm.i = (vm.v[x] * 5) as u16;
}

// Fx33 - LD B, Vx
// Store BCD representation of Vx in memory locations I, I+1, and I+2.
//
// The interpreter takes the decimal value of Vx, and places the hundreds
// digit in memory at location in I, the tens digit at location I+1,
// and the ones digit at location I+2.
pub fn store_bcd_of_vx_to_i(vm: &mut VirtualMachine, x: usize) {
    let num = vm.v[x];
    let h = num / 100;
    let t = (num - h * 100) / 10;
    let o = num - h * 100 - t * 10;
    vm.ram[vm.i as usize] = h;
    vm.ram[vm.i as usize + 1] = t;
    vm.ram[vm.i as usize + 2] = o;
}

// Fx55 - LD [I], Vx
// Store registers V0 through Vx in memory starting at location I.
//
// The interpreter copies the values of registers V0 through Vx into memory,
// starting at the address in I.
pub fn copy_v0_through_vx_to_mem(vm: &mut VirtualMachine, x: usize) {
    for i in (0..x + 1) {
        vm.ram[(vm.i + i as u16) as usize] = vm.v[i];
    }
}

// Fx65 - LD Vx, [I]
// Read registers V0 through Vx from memory starting at location I.
//
// The interpreter reads values from memory starting at location I into
// registers V0 through Vx.
pub fn read_v0_through_vx_from_mem(vm: &mut VirtualMachine, x: usize) {
    for i in (0..x + 1) {
        vm.v[i] = vm.ram[(vm.i + i as u16) as usize];
    }
}

#[test]
fn test_strore_bcd_of_vx_to_i() {
    let mut closure = |_: &_| {};
    let mut vm = VirtualMachine::new(&mut closure);
    vm.v[0] = 146;
    vm.i = 0;
    store_bcd_of_vx_to_i(&mut vm, 0);
    assert!(vm.ram[0] == 1);
    assert!(vm.ram[1] == 4);
    assert!(vm.ram[2] == 6);
}
