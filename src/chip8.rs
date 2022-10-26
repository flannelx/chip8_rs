#![allow(dead_code, non_snake_case)]

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

enum PC {
    Next,
    Skip,
    Jump(usize),
}

impl PC {
    pub fn skip_if(skip: bool) -> PC {
        if skip {
            return PC::Skip;
        }
        PC::Next
    }
}
const CHIP8_SCREEN_WIDTH: usize = 64;
const CHIP8_SCREEN_HEIGHT: usize = 32;
const CHIP8_RAM: usize = 4096;

pub struct Chip8 {
    ram: [u8; CHIP8_RAM],
    opcode: u8,
    v: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    screen: [[u8; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT],
    delay_timer: u8,
    sound_timer: u8,
    stack: [usize; 16],
    key: [u8; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut ram = [0; 4096];
        CHIP8_FONTSET
            .iter()
            .enumerate()
            .for_each(|(i, font)| ram[i] = *font);

        Self {
            pc: 0x200,
            opcode: 0,
            i: 0,
            sp: 0,
            ram,
            v: [0; 16],
            screen: [[0; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            key: [0; 16],
        }
    }

    pub fn run(&mut self) {
        let opcode: u16 = self.get_opcode();
        let (i, x, y, n, kk, nnn) = Self::inst_decode(&opcode);

        let pc = match (i, x, y, n) {
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
            (0x08, _, _, 0x0e) => self.inst_8x0e(x),
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
            _ => PC::Next,
        };

        match pc {
            PC::Next => self.pc += 2,
            PC::Skip => self.pc += 4,
            PC::Jump(addr) => self.pc = addr,
        }
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
    fn inst_00e0(&mut self) -> PC {
        for r in 0..CHIP8_SCREEN_HEIGHT {
            for c in 0..CHIP8_SCREEN_WIDTH {
                self.screen[r][c] = 0;
            }
        }

        PC::Next
    }

    // 00ee - RET
    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack point
    fn inst_00ee(&mut self) -> PC {
        self.sp -= 1;
        PC::Jump(self.stack[self.sp])
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    fn inst_1nnn(&mut self, nnn: usize) -> PC {
        PC::Jump(nnn)
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn inst_2nnn(&mut self, nnn: usize) -> PC {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        PC::Jump(nnn)
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn inst_3xkk(&mut self, x: u8, kk: u8) -> PC {
        if x == kk {
            return PC::Skip;
        }
        PC::Next
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn inst_4xkk(&mut self, x: u8, kk: u8) -> PC {
        if x != kk {
            return PC::Skip;
        }
        PC::Next
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn inst_5xy0(&mut self, x: u8, y: u8) -> PC {
        PC::skip_if(x == y)
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    fn inst_6xkk(&mut self, x: u8, kk: u8) -> PC {
        self.v[x as usize] = kk;
        PC::Next
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn inst_7xkk(&mut self, x: u8, kk: u8) -> PC {
        self.v[x as usize] += kk;
        PC::Next
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn inst_8xy0(&mut self, x: u8, y: u8) -> PC {
        todo!()
    }

    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    // A bitwise OR compares the corrseponding bits from two values,
    // and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn inst_8xy1(&mut self, x: u8, y: u8) -> PC {
        todo!()
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    // A bitwise AND compares the corrseponding bits from two values,
    // and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn inst_8xy2(&mut self, x: u8, y: u8) -> PC {
        todo!()
    }

    fn inst_8xy3(&mut self, x: u8, y: u8) -> PC {
        todo!()
    }

    fn inst_8xy4(&mut self, x: u8, y: u8) -> PC {
        todo!()
    }

    fn inst_8xy5(&mut self, x: u8, y: u8) -> PC {
        todo!()
    }

    fn inst_8x06(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_8xy7(&mut self, x: u8, y: u8) -> PC {
        todo!()
    }

    fn inst_8x0e(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_9xy0(&mut self, x: u8, y: u8) -> PC {
        todo!()
    }

    fn inst_annn(&mut self, nnn: usize) -> PC {
        todo!()
    }

    fn inst_bnnn(&mut self, nnn: usize) -> PC {
        todo!()
    }

    fn inst_cxkk(&mut self, x: u8, kk: u8) -> PC {
        todo!()
    }

    fn inst_dxyn(&mut self, x: u8, y: u8, n: u8) -> PC {
        todo!()
    }

    fn inst_ex9e(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_exa1(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx07(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx0a(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx15(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx18(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx1e(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx29(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx33(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx55(&mut self, x: u8) -> PC {
        todo!()
    }

    fn inst_fx65(&mut self, x: u8) -> PC {
        todo!()
    }
}
