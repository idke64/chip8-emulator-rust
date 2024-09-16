#[allow(dead_code)]

use std::fs::File;
use std::io::Read;

pub struct CPU {
    pub memory: [u8 ; 4096], // 4096 bytes of memory from 0x000 to 0xFFF
    pub v: [u8 ; 16], // 16 general 8-bit registers Vx - V0 to VF
    pub delay: u8, // special 8-bit register
    pub sound: u8, // special 8-bit register
    pub pc: u16, // program counter should be set at 0x200 by default
    pub sp: u8, // stack pointer
    pub stack: [u16 ; 16], // stack
    pub keyboard: [bool ; 16], // 1-F keyboard, false = key up, true = key down
    pub display: [[bool ; 64]; 32], // 64x32 monochrome display
    pub i: u16, // 16-bit index register
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = CPU {
            memory: [0; 4096],
            v: [0; 16],
            display: [[false ; 64]; 32],
            delay: 0,
            sound: 0,
            pc: 512, //0x200
            sp: 0,
            stack: [0; 16],
            keyboard: [false; 16],
            i: 0, 

        };
        cpu.load_fontset();
        return cpu;
    }

    // loads fontset into memory
    fn load_fontset(&mut self) {
        // sprites for hex digits 0-F: 0,1,2,3,4,5,6,7,8,9,A,B,C,D,E,F
        // 5x8 bit per digit
        let fontset: [[u8; 5]; 16] = [
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
        
        // insert fontset in memory starting from 0x50
        for i in 0..fontset.len() {
            for j in 0..fontset[i].len() {
                self.memory[0x50 + (5 * i) + j] = fontset[i][j];
            }
        }

    }
    
    // loads instructions from file into memory starting at 0x200
    pub fn load_instructions(&mut self, filename: &str) {
        let mut file = File::open(filename).expect("there was an issue opening the file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("failed to read");

        let start_address = 0x200;
        for (i, &byte) in buffer.iter().enumerate() {
            self.memory[start_address + i] = byte;
        }
    }

    // fetches 2-byte opcodes at pc
    pub fn fetch_opcode(&mut self) -> u16 {
        let high = self.memory[self.pc as usize] as u16;
        let low = self.memory[(self.pc + 1) as usize] as u16;

        // combines high and low bytes into a 2 byte
        let opcode = (high << 8) | low;

        self.pc += 2;
        opcode
    }

    // matches opcode to function
    fn execute_opcode(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.op_00e0(), // Clear the display
                0x00EE => self.op_00ee(), // Return from subroutine
                _ => println!("Unknown opcode: {:#04X}", opcode),
            },
            0x1000 => self.op_1nnn(opcode),
            0x2000 => self.op_2nnn(opcode),
            0x3000 => self.op_3xkk(opcode),
            0x4000 => self.op_4xkk(opcode),
            0x5000 => self.op_5xy0(opcode),
            0x6000 => self.op_6xkk(opcode),
            0x7000 => self.op_7xkk(opcode),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.op_8xy0(opcode),
                0x0001 => self.op_8xy1(opcode),
                0x0002 => self.op_8xy2(opcode),
                0x0003 => self.op_8xy3(opcode),
                0x0004 => self.op_8xy4(opcode),
                0x0005 => self.op_8xy5(opcode),
                0x0006 => self.op_8xy6(opcode),
                0x0007 => self.op_8xy7(opcode),
                0x000E => self.op_8xye(opcode),
                _ => println!("Unknown opcode: {:#04X}", opcode),
            },
            0x9000 => self.op_9xy0(opcode),
            0xA000 => self.op_annn(opcode),
            0xB000 => self.op_bnnn(opcode),
            0xC000 => self.op_cxkk(opcode),
            0xD000 => self.op_dxyn(opcode),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(opcode),
                0x00A1 => self.op_exa1(opcode),
                _ => println!("Unknown opcode: {:#04X}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(opcode),
                0x000A => self.op_fx0a(opcode),
                0x0015 => self.op_fx15(opcode),
                0x0018 => self.op_fx18(opcode),
                0x001E => self.op_fx1e(opcode),
                0x0029 => self.op_fx29(opcode),
                0x0033 => self.op_fx33(opcode),
                0x0055 => self.op_fx55(opcode),
                0x0065 => self.op_fx65(opcode),
                _ => println!("Unknown opcode: {:#04X}", opcode),
            },
            _ => println!("Unknown opcode: {:#04X}", opcode),
        }
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch_opcode();

        self.execute_opcode(opcode);

        if self.sp as usize >= self.stack.len() {
            panic!("Stack overflow");
        }

        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.sound > 0 {
            if self.sound == 1 {
                // play sound
            }
            self.sound -= 1;
        }
    }
}