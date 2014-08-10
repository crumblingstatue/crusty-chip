use super::Chip8;
use std::slice::bytes::copy_memory;

// 0nnn - SYS addr
// Jump to a machine code routine at nnn.
//
// This instruction is only used on the old computers on which Chip-8 was
// originally implemented. It is ignored by modern interpreters.
pub fn jump_to_sys_routine(slf: &mut Chip8, addr: uint) {
    // Do nothing
}

// 00E0 - CLS
// Clear the display.
pub fn clear_display(slf: &mut Chip8) {
    for px in slf.display.mut_iter() {
        *px = 0;
    }
}

// 00EE - RET
// Return from a subroutine.
//
// The interpreter sets the program counter to the address at the top of
// the stack, then subtracts 1 from the stack pointer.
pub fn ret_from_subroutine(slf: &mut Chip8) {
    slf.pc = slf.stack[slf.sp as uint];
    slf.sp -= 1;
}

// 1nnn - JP addr
// Jump to location nnn.
//
// The interpreter sets the program counter to nnn.
pub fn jump_addr(slf: &mut Chip8, addr: u16) {
    slf.pc = addr;
}

// 2nnn - CALL addr
// Call subroutine at nnn.
//
// The interpreter increments the stack pointer, then puts the current PC
// on the top of the stack. The PC is then set to nnn.
pub fn call_subroutine(slf: &mut Chip8, addr: uint) {
    slf.sp += 1;
    slf.stack[slf.sp as uint] = slf.pc;
    slf.pc = addr as u16;
}

// 3xkk - SE Vx, byte
// Skip next instruction if Vx = kk.
//
// The interpreter compares register Vx to kk, and if they are equal,
// increments the program counter by 2.
pub fn skip_next_vx_eq(slf: &mut Chip8, x: uint, to: u8) {
    if slf.v[x] == to {
        slf.pc += 2;
    }
}

// 4xkk - SNE Vx, byte
// Skip next instruction if Vx != kk.
//
// The interpreter compares register Vx to kk, and if they are not equal,
// increments the program counter by 2.
pub fn skip_next_vx_ne(slf: &mut Chip8, x: uint, to: u8) {
    if slf.v[x] != to {
        slf.pc += 2;
    }
}

// 5xy0 - SE Vx, Vy
// Skip next instruction if Vx = Vy.
//
// The interpreter compares register Vx to register Vy, and if they are
// equal, increments the program counter by 2.
pub fn skip_next_vx_eq_vy(slf: &mut Chip8, x: uint, y: uint) {
    if slf.v[x] == slf.v[y] {
        slf.pc += 2;
    }
}

// 6xkk - LD Vx, byte
// Set Vx = kk.
//
// The interpreter puts the value kk into register Vx.
pub fn set_vx_byte(slf: &mut Chip8, x: uint, byte: u8) {
    slf.v[x] = byte;
}

// 7xkk - ADD Vx, byte
// Set Vx = Vx + kk.
//
// Adds the value kk to the value of register Vx, then stores the
// result in Vx.
pub fn add_vx_byte(slf: &mut Chip8, x: uint, byte: u8) {
    slf.v[x] += byte;
}

// 8xy0 - LD Vx, Vy
// Set Vx = Vy.
//
// Stores the value of register Vy in register Vx.
pub fn set_vx_to_vy(slf: &mut Chip8, x: uint, y: uint) {
    slf.v[x] = slf.v[y];
}

pub fn set_i(slf: &mut Chip8, to: u16) {
    slf.i = to;
}

pub fn set_vx_rand_and(slf: &mut Chip8, x: uint, to: u8) {
    use std::rand::{task_rng, Rng};
    let mut rgen = task_rng();
    slf.v[x] = rgen.gen::<u8>() & to;
}

pub fn display_sprite(slf: &mut Chip8, vx: uint, vy: uint, n: uint) {
    use super::{ DISPLAY_WIDTH, DISPLAY_HEIGHT };
    let x_off = slf.v[vx] as uint;
    let y_off = slf.v[vy] as uint * DISPLAY_WIDTH;
    let offset = x_off + y_off;

    for mut y in range(0u, n) {
        if y >= DISPLAY_HEIGHT {
            y = y - DISPLAY_HEIGHT;
        }
        let b = slf.ram[slf.i as uint + y];
        for mut x in range(0u, 8) {
            if x >= DISPLAY_WIDTH {
                x = x - DISPLAY_WIDTH;
            }
            let idx = offset + (y * DISPLAY_WIDTH) + x;
            if idx < DISPLAY_WIDTH * DISPLAY_HEIGHT {
                slf.display[idx] ^= b & (0b10000000 >> x);
            } else {
                println!("Warning: Out of bounds VRAM write: {}", idx);
            }
        }
    }

    (slf.draw_callback)(slf.display);
}

pub fn add_vx_vy(slf: &mut Chip8, x: uint, y: uint) {
    let result = (slf.v[x] + slf.v[y]) as u16;
    slf.v[0xF] = (result > 255) as u8;
    slf.v[x] = result as u8;
}

pub fn sub_vx_vy(slf: &mut Chip8, x: uint, y: uint) {
    slf.v[0xF] = (slf.v[x] > slf.v[y]) as u8;
    slf.v[x] -= slf.v[y];
}

pub fn add_vx_to_i(slf: &mut Chip8, x: uint) {
    slf.i += x as u16;
}

pub fn copy_v0_through_vx_to_mem(slf: &mut Chip8, x: uint) {
    if x == 0 {
        return;
    }
    copy_memory(slf.ram.mut_slice(slf.i as uint, slf.i as uint + x),
                slf.v.slice(0, x));
}

pub fn read_v0_through_vx_from_mem(slf: &mut Chip8, x: uint) {
    if x == 0 {
        return;
    }
    copy_memory(slf.v.mut_slice(0, x),
                slf.ram.slice(slf.i as uint, slf.i as uint + x));
}

pub fn skip_next_vx_ne_vy(slf: &mut Chip8, x: uint, y: uint) {
    if slf.v[x] != slf.v[y] {
        slf.pc += 2;
    }
}

// Fx33 - LD B, Vx
// Store BCD representation of Vx in memory locations I, I+1, and I+2.
//
// The interpreter takes the decimal value of Vx, and places the hundreds
// digit in memory at location in I, the tens digit at location I+1,
// and the ones digit at location I+2.
pub fn store_bcd_of_vx_to_i(slf: &mut Chip8, x: uint) {
    let num = slf.v[x];
    let h = num / 100;
    let t = (num - h * 100) / 10;
    let o = (num - h * 100 - t * 10);
    slf.ram[slf.i as uint] = h;
    slf.ram[slf.i as uint + 1] = t;
    slf.ram[slf.i as uint + 2] = o;
}

// Fx29 - LD F, Vx
// Set I = location of sprite for digit Vx.
//
// The value of I is set to the location for the hexadecimal sprite
// corresponding to the value of Vx. See section 2.4, Display, for more
// information on the Chip-8 hexadecimal font.
//
// For crusty-chip, the fontset is stored at 0x000
pub fn set_i_to_loc_of_digit_vx(slf: &mut Chip8, x: uint) {
    slf.i = (slf.v[x] * 5) as u16;
}

// Fx0A - LD Vx, K
// Wait for a key press, store the value of the key in Vx.
//
// All execution stops until a key is pressed, then the value of that key
// is stored in Vx.
pub fn wait_for_keypress_store_in_vx(slf: &mut Chip8, x: uint) {
    slf.keypress_wait.wait = true;
    slf.keypress_wait.vx = x;
}

// Fx15 - LD DT, Vx
// Set delay timer = Vx.
//
// DT is set equal to the value of Vx.
pub fn set_delay_timer(slf: &mut Chip8, x: u8) {
    slf.delay_timer = x;
}

// Fx18 - LD ST, Vx
// Set sound timer = Vx.
//
// ST is set equal to the value of Vx.
pub fn set_sound_timer(slf: &mut Chip8, x: u8) {
    slf.sound_timer = x;
}

// Fx07 - LD Vx, DT
// Set Vx = delay timer value.
//
// The value of DT is placed into Vx.
pub fn set_vx_to_delay_timer(slf: &mut Chip8, x: uint) {
    slf.v[x] = slf.delay_timer;
}

// 8xy3 - XOR Vx, Vy
// Set Vx = Vx XOR Vy.
//
// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores
// the result in Vx. An exclusive OR compares the corrseponding bits from
// two values, and if the bits are not both the same, then the
// corresponding bit in the result is set to 1. Otherwise, it is 0.
pub fn set_vx_to_vx_xor_vy(slf: &mut Chip8, x: uint, y: uint) {
    slf.v[x] ^= slf.v[y];
}

// 8xyE - SHL Vx {, Vy}
// Set Vx = Vx SHL 1.
//
// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise
// to 0. Then Vx is multiplied by 2.
pub fn set_vx_to_vx_shl_1(slf: &mut Chip8, x: uint) {
    // TODO: Is this just a left shift by 1?
    slf.v[x] <<= 1;
}

// 8xy2 - AND Vx, Vy
// Set Vx = Vx AND Vy.
//
// Performs a bitwise AND on the values of Vx and Vy, then stores the
// result in Vx. A bitwise AND compares the corrseponding bits from two
// values, and if both bits are 1, then the same bit in the result is also
// 1. Otherwise, it is 0.
pub fn set_vx_to_vx_and_vy(slf: &mut Chip8, x: uint, y: uint) {
    slf.v[x] &= slf.v[y];
}

// ExA1 - SKNP Vx
// Skip next instruction if key with the value of Vx is not pressed.
//
// Checks the keyboard, and if the key corresponding to the value of
// Vx is currently in the up position, PC is increased by 2.
pub fn skip_next_key_vx_not_pressed(slf: &mut Chip8, x: uint) {
    if !slf.keys[slf.v[x] as uint] {
        slf.pc += 2;
    }
}