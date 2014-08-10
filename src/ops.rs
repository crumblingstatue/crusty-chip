use super::Chip8;
use std::slice::bytes::copy_memory;

// 0nnn - SYS addr
// Jump to a machine code routine at nnn.
//
// This instruction is only used on the old computers on which Chip-8 was
// originally implemented. It is ignored by modern interpreters.
pub fn jump_to_sys_routine(ch8: &mut Chip8, addr: uint) {
    // Do nothing
}

// 00E0 - CLS
// Clear the display.
pub fn clear_display(ch8: &mut Chip8) {
    for px in ch8.display.mut_iter() {
        *px = 0;
    }
}

// 00EE - RET
// Return from a subroutine.
//
// The interpreter sets the program counter to the address at the top of
// the stack, then subtracts 1 from the stack pointer.
pub fn ret_from_subroutine(ch8: &mut Chip8) {
    ch8.pc = ch8.stack[ch8.sp as uint];
    ch8.sp -= 1;
}

// 1nnn - JP addr
// Jump to location nnn.
//
// The interpreter sets the program counter to nnn.
pub fn jump_addr(ch8: &mut Chip8, addr: u16) {
    ch8.pc = addr;
}

// 2nnn - CALL addr
// Call subroutine at nnn.
//
// The interpreter increments the stack pointer, then puts the current PC
// on the top of the stack. The PC is then set to nnn.
pub fn call_subroutine(ch8: &mut Chip8, addr: uint) {
    ch8.sp += 1;
    ch8.stack[ch8.sp as uint] = ch8.pc;
    ch8.pc = addr as u16;
}

// 3xkk - SE Vx, byte
// Skip next instruction if Vx = kk.
//
// The interpreter compares register Vx to kk, and if they are equal,
// increments the program counter by 2.
pub fn skip_next_vx_eq(ch8: &mut Chip8, x: uint, to: u8) {
    if ch8.v[x] == to {
        ch8.pc += 2;
    }
}

// 4xkk - SNE Vx, byte
// Skip next instruction if Vx != kk.
//
// The interpreter compares register Vx to kk, and if they are not equal,
// increments the program counter by 2.
pub fn skip_next_vx_ne(ch8: &mut Chip8, x: uint, to: u8) {
    if ch8.v[x] != to {
        ch8.pc += 2;
    }
}

// 5xy0 - SE Vx, Vy
// Skip next instruction if Vx = Vy.
//
// The interpreter compares register Vx to register Vy, and if they are
// equal, increments the program counter by 2.
pub fn skip_next_vx_eq_vy(ch8: &mut Chip8, x: uint, y: uint) {
    if ch8.v[x] == ch8.v[y] {
        ch8.pc += 2;
    }
}

// 6xkk - LD Vx, byte
// Set Vx = kk.
//
// The interpreter puts the value kk into register Vx.
pub fn set_vx_byte(ch8: &mut Chip8, x: uint, byte: u8) {
    ch8.v[x] = byte;
}

// 7xkk - ADD Vx, byte
// Set Vx = Vx + kk.
//
// Adds the value kk to the value of register Vx, then stores the
// result in Vx.
pub fn add_vx_byte(ch8: &mut Chip8, x: uint, byte: u8) {
    ch8.v[x] += byte;
}

// 8xy0 - LD Vx, Vy
// Set Vx = Vy.
//
// Stores the value of register Vy in register Vx.
pub fn set_vx_to_vy(ch8: &mut Chip8, x: uint, y: uint) {
    ch8.v[x] = ch8.v[y];
}

// 8xy2 - AND Vx, Vy
// Set Vx = Vx AND Vy.
//
// Performs a bitwise AND on the values of Vx and Vy, then stores the
// result in Vx. A bitwise AND compares the corrseponding bits from two
// values, and if both bits are 1, then the same bit in the result is also
// 1. Otherwise, it is 0.
pub fn set_vx_to_vx_and_vy(ch8: &mut Chip8, x: uint, y: uint) {
    ch8.v[x] &= ch8.v[y];
}

// 8xy3 - XOR Vx, Vy
// Set Vx = Vx XOR Vy.
//
// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores
// the result in Vx. An exclusive OR compares the corrseponding bits from
// two values, and if the bits are not both the same, then the
// corresponding bit in the result is set to 1. Otherwise, it is 0.
pub fn set_vx_to_vx_xor_vy(ch8: &mut Chip8, x: uint, y: uint) {
    ch8.v[x] ^= ch8.v[y];
}

// 8xy4 - ADD Vx, Vy
// Set Vx = Vx + Vy, set VF = carry.
//
// The values of Vx and Vy are added together. If the result is greater than
// 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits
// of the result are kept, and stored in Vx.
pub fn add_vx_vy(ch8: &mut Chip8, x: uint, y: uint) {
    let result = (ch8.v[x] + ch8.v[y]) as u16;
    ch8.v[0xF] = (result > 255) as u8;
    ch8.v[x] = result as u8;
}

// 8xy5 - SUB Vx, Vy
// Set Vx = Vx - Vy, set VF = NOT borrow.
//
// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx,
// and the results stored in Vx.
pub fn sub_vx_vy(ch8: &mut Chip8, x: uint, y: uint) {
    ch8.v[0xF] = (ch8.v[x] > ch8.v[y]) as u8;
    ch8.v[x] -= ch8.v[y];
}

// 8xy6 - SHR Vx {, Vy}
// Set Vx = Vx SHR 1.
//
// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
// Then Vx is divided by 2.
pub fn set_vx_to_vx_shr_1(ch8: &mut Chip8, x: uint) {
    ch8.v[0xF] = if check_bit(ch8.v[x], 0) {1} else {0};
    ch8.v[x] /= 2;
}

// 8xyE - SHL Vx {, Vy}
// Set Vx = Vx SHL 1.
//
// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise
// to 0. Then Vx is multiplied by 2.
pub fn set_vx_to_vx_shl_1(ch8: &mut Chip8, x: uint) {
    ch8.v[0xF] = if check_bit(ch8.v[x], 7) {1} else {0};
    ch8.v[x] *= 2;
}

fn check_bit(byte: u8, pos: uint) -> bool {
    byte & (1 << pos) != 0
}

#[test]
fn test_check_bit() {
    assert!(check_bit(0b10000000, 7));
    for i in range(0u, 7) {
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
pub fn skip_next_vx_ne_vy(ch8: &mut Chip8, x: uint, y: uint) {
    if ch8.v[x] != ch8.v[y] {
        ch8.pc += 2;
    }
}

// Annn - LD I, addr
// Set I = nnn.
//
// The value of register I is set to nnn.
pub fn set_i(ch8: &mut Chip8, to: u16) {
    ch8.i = to;
}

// Cxkk - RND Vx, byte
// Set Vx = random byte AND kk.
//
// The interpreter generates a random number from 0 to 255, which is then ANDed
// with the value kk. The results are stored in Vx.
// See instruction 8xy2 for more information on AND.
pub fn set_vx_rand_and(ch8: &mut Chip8, x: uint, to: u8) {
    use std::rand::{task_rng, Rng};
    let mut rgen = task_rng();
    ch8.v[x] = rgen.gen::<u8>() & to;
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
pub fn display_sprite(ch8: &mut Chip8, vx: uint, vy: uint, n: uint) {
    // TODO: Implement setting VF if collision occurs
    use super::{ DISPLAY_WIDTH, DISPLAY_HEIGHT };
    let x_off = ch8.v[vx] as uint;
    let y_off = ch8.v[vy] as uint * DISPLAY_WIDTH;
    let offset = x_off + y_off;

    for mut y in range(0u, n) {
        if y >= DISPLAY_HEIGHT {
            y = y - DISPLAY_HEIGHT;
        }
        let b = ch8.ram[ch8.i as uint + y];
        for mut x in range(0u, 8) {
            if x >= DISPLAY_WIDTH {
                x = x - DISPLAY_WIDTH;
            }
            let idx = offset + (y * DISPLAY_WIDTH) + x;
            if idx < DISPLAY_WIDTH * DISPLAY_HEIGHT {
                ch8.display[idx] ^= b & (0b10000000 >> x);
            } else {
                println!("Warning: Out of bounds VRAM write: {}", idx);
            }
        }
    }

    (ch8.draw_callback)(ch8.display);
}

// ExA1 - SKNP Vx
// Skip next instruction if key with the value of Vx is not pressed.
//
// Checks the keyboard, and if the key corresponding to the value of
// Vx is currently in the up position, PC is increased by 2.
pub fn skip_next_key_vx_not_pressed(ch8: &mut Chip8, x: uint) {
    if !ch8.keys[ch8.v[x] as uint] {
        ch8.pc += 2;
    }
}

// Ex9E - SKP Vx
// Skip next instruction if key with the value of Vx is pressed.
//
// Checks the keyboard, and if the key corresponding to the value of Vx is
// currently in the down position, PC is increased by 2.
pub fn skip_next_key_vx_pressed(ch8: &mut Chip8, x: uint) {
    if ch8.keys[ch8.v[x] as uint] {
        ch8.pc += 2;
    }
}

// Fx07 - LD Vx, DT
// Set Vx = delay timer value.
//
// The value of DT is placed into Vx.
pub fn set_vx_to_delay_timer(ch8: &mut Chip8, x: uint) {
    ch8.v[x] = ch8.delay_timer;
}

// Fx0A - LD Vx, K
// Wait for a key press, store the value of the key in Vx.
//
// All execution stops until a key is pressed, then the value of that key
// is stored in Vx.
pub fn wait_for_keypress_store_in_vx(ch8: &mut Chip8, x: uint) {
    ch8.keypress_wait.wait = true;
    ch8.keypress_wait.vx = x;
}

// Fx15 - LD DT, Vx
// Set delay timer = Vx.
//
// DT is set equal to the value of Vx.
pub fn set_delay_timer(ch8: &mut Chip8, x: u8) {
    ch8.delay_timer = x;
}

// Fx18 - LD ST, Vx
// Set sound timer = Vx.
//
// ST is set equal to the value of Vx.
pub fn set_sound_timer(ch8: &mut Chip8, x: u8) {
    ch8.sound_timer = x;
}

// Fx1E - ADD I, Vx
// Set I = I + Vx.
//
// The values of I and Vx are added, and the results are stored in I.
pub fn add_vx_to_i(ch8: &mut Chip8, x: uint) {
    ch8.i += ch8.v[x] as u16;
}

// Fx29 - LD F, Vx
// Set I = location of sprite for digit Vx.
//
// The value of I is set to the location for the hexadecimal sprite
// corresponding to the value of Vx. See section 2.4, Display, for more
// information on the Chip-8 hexadecimal font.
//
// For crusty-chip, the fontset is stored at 0x000
pub fn set_i_to_loc_of_digit_vx(ch8: &mut Chip8, x: uint) {
    ch8.i = (ch8.v[x] * 5) as u16;
}

// Fx33 - LD B, Vx
// Store BCD representation of Vx in memory locations I, I+1, and I+2.
//
// The interpreter takes the decimal value of Vx, and places the hundreds
// digit in memory at location in I, the tens digit at location I+1,
// and the ones digit at location I+2.
pub fn store_bcd_of_vx_to_i(ch8: &mut Chip8, x: uint) {
    let num = ch8.v[x];
    let h = num / 100;
    let t = (num - h * 100) / 10;
    let o = (num - h * 100 - t * 10);
    ch8.ram[ch8.i as uint] = h;
    ch8.ram[ch8.i as uint + 1] = t;
    ch8.ram[ch8.i as uint + 2] = o;
}

// Fx55 - LD [I], Vx
// Store registers V0 through Vx in memory starting at location I.
//
// The interpreter copies the values of registers V0 through Vx into memory,
// starting at the address in I.
pub fn copy_v0_through_vx_to_mem(ch8: &mut Chip8, x: uint) {
    if x == 0 {
        return;
    }
    copy_memory(ch8.ram.mut_slice(ch8.i as uint, ch8.i as uint + x),
                ch8.v.slice(0, x));
}

// Fx65 - LD Vx, [I]
// Read registers V0 through Vx from memory starting at location I.
//
// The interpreter reads values from memory starting at location I into
// registers V0 through Vx.
pub fn read_v0_through_vx_from_mem(ch8: &mut Chip8, x: uint) {
    if x == 0 {
        return;
    }
    copy_memory(ch8.v.mut_slice(0, x),
                ch8.ram.slice(ch8.i as uint, ch8.i as uint + x));
}

#[test]
fn test_strore_bcd_of_vx_to_i() {
    let mut ch8 = Chip8::new(|_| {});
    ch8.v[0] = 146;
    ch8.i = 0;
    store_bcd_of_vx_to_i(&mut ch8, 0);
    assert!(ch8.ram[0] == 1);
    assert!(ch8.ram[1] == 4);
    assert!(ch8.ram[2] == 6);
}
