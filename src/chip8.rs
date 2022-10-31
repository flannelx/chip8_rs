use std::path::Path;
use rand::Rng;

pub const CHIP8_SCREEN_WIDTH: usize = 64;
pub const CHIP8_SCREEN_HEIGHT: usize = 32;
pub const CHIP8_RAM: usize = 4096;
pub const CHIP8_START_ADDR: usize = 0x200;
pub const CHIP8_FONTSET: [u8; 80] = [
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

pub struct Chip8 {
    pub ram: [u8; CHIP8_RAM],
    v: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    pub screen: [[u8; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT],
    delay_timer: u8,
    sound_timer: u8,
    stack: [usize; 16],
    keypad: [bool; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut ram = [0; 4096];
        CHIP8_FONTSET
            .iter()
            .enumerate()
            .for_each(|(i, font)| ram[i + 0x50] = *font);

        Self {
            pc: CHIP8_START_ADDR,
            i: 0,
            sp: 0,
            ram,
            v: [0; 16],
            screen: [[0; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            keypad: [false; 16],
        }
    }

    pub fn load_rom(&mut self, path: impl AsRef<Path>) {
        use std::fs::File;
        use std::io::Read;
        let mut f = File::open(path).expect("file not found");
        f.read(&mut self.ram[0x200..]).unwrap();
    }

    pub fn cycle(&mut self, keypad: [bool; 16]) {
        self.keypad = keypad;
        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1
        }
        self.exec();
    }

    pub fn exec(&mut self) {
        let opcode: u16 = self.get_opcode();
        self.pc += 2;
        let (i, x, y, n, kk, nnn) = Self::inst_decode(&opcode);
        //println!("{:#x} {:#x} {:#x} {:#x} {:#x}", i, x, y, n, nnn);

        match (i, x, y, n) {
            (0x00, 0x00, 0x0e, 0x00) => self.inst_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.inst_00ee(),
            (0x01, _, _, _) => self.inst_1nnn(nnn),
            (0x02, _, _, _) => self.inst_2nnn(nnn),
            (0x03, _, _, _) => self.inst_3xkk(x, kk),
            (0x04, _, _, _) => self.inst_4xkk(x, kk),
            (0x05, _, _, 0x00) => self.inst_5xy0(x, y),
            (0x06, _, _, _) => self.inst_6xkk(x, kk),
            (0x07, _, _, _) => self.inst_7xkk(x, kk),
            (0x08, _, _, 0x00) => self.inst_8xy0(x, y),
            (0x08, _, _, 0x01) => self.inst_8xy1(x, y),
            (0x08, _, _, 0x02) => self.inst_8xy2(x, y),
            (0x08, _, _, 0x03) => self.inst_8xy3(x, y),
            (0x08, _, _, 0x04) => self.inst_8xy4(x, y),
            (0x08, _, _, 0x05) => self.inst_8xy5(x, y),
            (0x08, _, _, 0x06) => self.inst_8x06(x),
            (0x08, _, _, 0x07) => self.inst_8xy7(x, y),
            (0x08, _, _, 0x0e) => self.inst_8xye(x),
            (0x09, _, _, 0x00) => self.inst_9xy0(x, y),
            (0x0a, _, _, _) => self.inst_annn(nnn),
            (0x0b, _, _, _) => self.inst_bnnn(nnn),
            (0x0c, _, _, _) => self.inst_cxkk(x, kk),
            (0x0d, _, _, _) => self.inst_dxyn(x, y, n),
            (0x0e, _, 0x09, 0x0e) => self.inst_ex9e(x),
            (0x0e, _, 0x0a, 0x01) => self.inst_exa1(x),
            (0x0f, _, 0x00, 0x07) => self.inst_fx07(x),
            (0x0f, _, 0x00, 0x0a) => self.inst_fx0a(x),
            (0x0f, _, 0x01, 0x05) => self.inst_fx15(x),
            (0x0f, _, 0x01, 0x08) => self.inst_fx18(x),
            (0x0f, _, 0x01, 0x0e) => self.inst_fx1e(x),
            (0x0f, _, 0x02, 0x09) => self.inst_fx29(x),
            (0x0f, _, 0x03, 0x03) => self.inst_fx33(x),
            (0x0f, _, 0x05, 0x05) => self.inst_fx55(x),
            (0x0f, _, 0x06, 0x05) => self.inst_fx65(x),
            _ => (),
        };
    }

    // Instruction (ie. 0x0000)
    //
    // _     - first nibble
    // x     - second nibble - A 4-bit value, the lower 4 bits of the high byte of the instruction
    // y     - third nibble - A 4-bit value, the upper 4 bits of the low byte of the instruction
    // n     - fourth nibble - A 4-bit value, the lowest 4 bits of the instruction
    // kk/nn - An 8-bit value, the lowest 8 bits of the instruction
    // nnn   - The second, third and fourth nibbles. A 12-bit immediate memory address.
    pub fn inst_decode(opcode: &u16) -> (u8, u8, u8, u8, u8, usize) {
        (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
            (opcode & 0x00FF) as u8,
            (opcode & 0x0FFF) as usize,
        )
    }

    //  << 8  10101111________  -   8 bit
    //        ________10101111  -   8 bit
    //        1010111110101111  -  16 bit
    pub fn get_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | self.ram[self.pc + 1] as u16
    }

    // 00e0 - CLS
    // Clear screen
    fn inst_00e0(&mut self) {
        for r in 0..CHIP8_SCREEN_HEIGHT {
            for c in 0..CHIP8_SCREEN_WIDTH {
                self.screen[r][c] = 0;
            }
        }
    }

    // 00ee - RET
    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack point
    fn inst_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    fn inst_1nnn(&mut self, nnn: usize) {
        self.pc = nnn;
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn inst_2nnn(&mut self, nnn: usize) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn inst_3xkk(&mut self, x: u8, kk: u8) {
        if self.v[x as usize] == kk {
            self.pc += 2
        }
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn inst_4xkk(&mut self, x: u8, kk: u8) {
        if self.v[x as usize] != kk {
            self.pc += 2;
        }
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn inst_5xy0(&mut self, x: u8, y: u8) {
        if self.v[x as usize] == self.v[y as usize] {
            self.pc += 2;
        }
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    fn inst_6xkk(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = kk;
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn inst_7xkk(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn inst_8xy0(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];
    }

    // Set Vx = Vx XOR Vy.
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    // An exclusive OR compares the corrseponding bits from two values,
    // and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    fn inst_8xy1(&mut self, x: u8, y: u8) {
        self.v[x as usize] |= self.v[y as usize];
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    // A bitwise AND compares the corrseponding bits from two values,
    // and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn inst_8xy2(&mut self, x: u8, y: u8) {
        self.v[x as usize] &= self.v[y as usize];
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    // An exclusive OR compares the corrseponding bits from two values,
    // and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    fn inst_8xy3(&mut self, x: u8, y: u8) {
        self.v[x as usize] ^= self.v[y as usize];
    }

    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    // Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn inst_8xy4(&mut self, x: u8, y: u8) {
        let sum: u16 = self.v[x as usize] as u16 + self.v[y as usize] as u16;
        self.v[0xF] = if sum > 255 { 1 } else { 0 };
        self.v[x as usize] = (sum & 0xFF) as u8;
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    fn inst_8xy5(&mut self, x: u8, y: u8) {
        self.v[0xF] = if self.v[x as usize] > self.v[y as usize] {
            1
        } else {
            0
        };
        self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);
    }

    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn inst_8x06(&mut self, x: u8) {
        self.v[0xF] = self.v[x as usize] & 1;
        self.v[x as usize] >>= 1;
    }

    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    fn inst_8xy7(&mut self, x: u8, y: u8) {
        self.v[0xF] = if self.v[y as usize] > self.v[x as usize] {
            1
        } else {
            0
        };
        self.v[x as usize] = self.v[y as usize].wrapping_sub(self.v[x as usize]);
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn inst_8xye(&mut self, x: u8) {
        self.v[0xF] = (self.v[x as usize] & 0x80) >> 7;
        self.v[x as usize] <<= 1;
    }

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    fn inst_9xy0(&mut self, x: u8, y: u8) {
        if self.v[x as usize] != self.v[y as usize] {
            self.pc += 2;
        }
    }

    // Annn - LD I, addr
    // Set I = nnn.
    // The value of register I is set to nnn.
    fn inst_annn(&mut self, nnn: usize) {
        self.i = nnn;
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    // The program counter is set to nnn plus the value of V0.
    fn inst_bnnn(&mut self, nnn: usize) {
        self.pc = (self.v[0] as usize) + nnn
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    // The results are stored in Vx. See instruction 8xy2 for more information on AND.
    fn inst_cxkk(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = rand::thread_rng().gen::<u8>() & kk;
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address stored in I.
    // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    // If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen.
    // See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    fn inst_dxyn(&mut self, x: u8, y: u8, n: u8) {
        self.v[0xF] = 0;
        for byte in 0..n as usize {
            let y = (self.v[y as usize] as usize + byte) % CHIP8_SCREEN_HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x as usize] as usize + bit) % CHIP8_SCREEN_WIDTH;
                let color = (self.ram[self.i + byte] >> (7 - bit)) & 1;
                self.v[0xF] |= color & self.screen[y][x];
                self.screen[y][x] ^= color;
            }
        }
    }

    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    fn inst_ex9e(&mut self, x: u8) {
        if self.keypad[self.v[x as usize] as usize] {
            self.pc += 2
        }
    }

    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    fn inst_exa1(&mut self, x: u8) {
        if !self.keypad[self.v[x as usize] as usize] {
            self.pc += 2;
        }
    }

    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    // The value of DT is placed into Vx.
    fn inst_fx07(&mut self, x: u8) {
        self.v[x as usize] = self.delay_timer;
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn inst_fx0a(&mut self, x: u8) {
        for i in 0..self.keypad.len() {
            if self.keypad[i] {
                self.keypad[i] = false;
                self.v[x as usize] = i as u8;
                return;
            }
        }
        self.pc -= 2;
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    // DT is set equal to the value of Vx.
    fn inst_fx15(&mut self, x: u8) {
        self.delay_timer = self.v[x as usize];
    }

    // Fx18 - LD ST, Vx
    // Set sound timer = Vx.
    // ST is set equal to the value of
    fn inst_fx18(&mut self, x: u8) {
        self.sound_timer = self.v[x as usize];
    }

    // Fx1E - ADD I, Vx
    // Set I = I + Vx.
    // The values of I and Vx are added, and the results are stored in I.
    fn inst_fx1e(&mut self, x: u8) {
        self.i += self.v[x as usize] as usize;
    }

    // Fx29 - LD F, Vx
    // Set I = location of sprite for digit Vx.
    // The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
    // See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    fn inst_fx29(&mut self, x: u8) {
        self.i = (self.v[x as usize] as usize) * 5 + 0x50;
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I,
    // the tens digit at location I+1, and the ones digit at location I+2.
    fn inst_fx33(&mut self, x: u8) {
        self.ram[self.i] = self.v[x as usize] / 100;
        self.ram[self.i + 1] = self.v[x as usize] % 100 / 10;
        self.ram[self.i + 2] = self.v[x as usize] % 10;
    }

    // Fx55 - LD [I], Vx
    // Store registers V0 through Vx in memory starting at location I.
    // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    fn inst_fx55(&mut self, x: u8) {
        for i in 0..=x as usize {
            self.ram[self.i + i] = self.v[i];
        }
    }

    // Fx65 - LD Vx, [I]
    // Read registers V0 through Vx from memory starting at location I.
    // The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn inst_fx65(&mut self, x: u8) {
        for i in 0..=x as usize {
            self.v[i] = self.ram[self.i + i];
        }
    }
}
