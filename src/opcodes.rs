use crate::cpu::CPU;
use rand::Rng;

impl CPU {

    fn get_nnn(opcode: u16) -> u16 {
        opcode & 0x0FFF // bitmask bottom 12 bits
    }

    fn get_xy(opcode: u16) -> (usize, usize) {
        let x = (opcode & 0x0F00) >> 8; // bitmask bits 8-12 from the bottom and shift right
        let y = (opcode & 0x00F0) >> 4; // bitmask bits 4-8 from the bottom and shift right

        (x as usize, y as usize)
    }

    fn get_xkk(opcode: u16) -> (usize, u8) {
        let x = (opcode & 0x0F00) >> 8; // bitmask bits 8-12 from the bottom
        let kk = (opcode & 0x00FF) as u8; // bitmask bottom 8 bits

        (x as usize, kk)
    }

    fn get_x(opcode: u16) -> usize {
        let x = (opcode & 0x0F00) >> 8; // bitmask bits 8-12 from the bottom

        x as usize
    }

    // clear display
    pub fn op_00e0(&mut self) {
        self.display = [[false ; 64]; 32];
    }

    // return from subroutine
    pub fn op_00ee(&mut self) {
        self.sp -= 1; // decrement sp to go to subroutine address
        self.pc = self.stack[self.sp as usize]; // set pc to subroutine return address
    }

    // jump to nnn
    pub fn op_1nnn(&mut self, opcode: u16) {
        let address = CPU::get_nnn(opcode);
        self.pc = address;
    }

    // call subroutine at nnn
    pub fn op_2nnn(&mut self, opcode: u16) {
        let subroutine_address = CPU::get_nnn(opcode);

        self.stack[self.sp as usize] = self.pc;
        self.sp += 1; // increment sp
        self.pc = subroutine_address;
    }

    // skips next instruction if register x = kk
    pub fn op_3xkk(&mut self, opcode: u16) {
        let (x, kk) = CPU::get_xkk(opcode);

        if self.v[x] == kk { // increments by 2 if x = kk
            self.pc += 2;
        }
    }

    // skips next instruction if register Vx != kk
    pub fn op_4xkk(&mut self, opcode: u16) {
        let (x, kk) = CPU::get_xkk(opcode);

        if self.v[x] != kk { // increments by 2 if Vx != kk
            self.pc += 2;
        }
    }

    // skips next instruction if Vx = Vy
    pub fn op_5xy0(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);

        if self.v[x] == self.v[y] { // increments by 2 if Vx == Vy
            self.pc += 2;
        }
    }

    // sets Vx = kk
    pub fn op_6xkk(&mut self, opcode: u16) {
        let (x, kk) = CPU::get_xkk(opcode);

        self.v[x] = kk;
    }

    // increments Vx by kk
    pub fn op_7xkk(&mut self, opcode: u16) {
        let (x, kk) = CPU::get_xkk(opcode);

        self.v[x] = self.v[x].wrapping_add(kk);
    }

    // sets Vx = Vy
    pub fn op_8xy0(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);

        self.v[x] = self.v[y];
    }

    // sets Vx = (Vx OR Vy) 
    pub fn op_8xy1(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);

        self.v[x] = (self.v[x] | self.v[y]);
    }

    // sets Vx = (Vx AND Vy)
    pub fn op_8xy2(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);

        self.v[x] = (self.v[x] & self.v[y]);
    }

    // sets Vx = (Vx XOR Vy)
    pub fn op_8xy3(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);

        self.v[x] = (self.v[x] ^ self.v[y]);
    }

    // sets Vx = Vx + Vy, set VF = carry ()
    pub fn op_8xy4(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);
        
        let (result, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xF] = if carry {1} else {0};
        self.v[x] = result;
    }

    // sets Vx = Vx - Vy, set VF = NOT borrow
    pub fn op_8xy5(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);
        
        self.v[0xF] = if self.v[x] > self.v[y] {1} else {0};

        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
    }

    // set Vx = Vy SHR 1
    pub fn op_8xy6(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);

        self.v[0xF] = self.v[x] & 0x1;

        self.v[x] >>= 1;
    }

    pub fn op_8xy7(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);
        
        self.v[0xF] = if self.v[y] > self.v[x] {1} else {0};

        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
    }

    // set Vx = Vy SHL 1
    pub fn op_8xye(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);

        self.v[0xF] = (self.v[x] >> 7) & 0x1;

        self.v[x] <<= 1;
    }

    pub fn op_9xy0(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);

        if self.v[x] != self.v[y] { // increments by 2 if Vx != Vy
            self.pc += 2;
        }
    }

    // set register I to nnn
    pub fn op_annn(&mut self, opcode: u16) {
        let address = CPU::get_nnn(opcode);

        self.i = address;
    }

    // jump to location nnn + V0
    pub fn op_bnnn(&mut self, opcode: u16) {
        let address = CPU::get_nnn(opcode);

        self.pc = address + self.v[0] as u16;
    }

    pub fn op_cxkk(&mut self, opcode: u16) {
        let (x, kk) = CPU::get_xkk(opcode);

        let mut rng = rand::thread_rng();
        let rand: u8 = rng.gen(); // generates random 8-bit number 0-255

        self.v[x] = kk & rand;

    }

    pub fn op_dxyn(&mut self, opcode: u16) {
        let (x, y) = CPU::get_xy(opcode);
        let n: u8 = (opcode & 0x000F) as u8; // bitmask bottom 4-bits and caste into u8 gets height

        self.v[0xF] = 0; // set collision flag to 0 (no collision yet)

        //read n bytes from memory starting at I
        for byte in 0..n {
            // get sprite from memory
            let sprite = self.memory[self.i as usize + byte as usize];
            // get current y based on index in height; wrapped if it exceeds height of 32
            let curr_y = (self.v[y] as usize + byte as usize) % 32;
            // loop through each bit in the sprite byte
            for bit in 0..8 {
                let curr_x = (self.v[x] as usize + bit as usize) % 64;

                let pixel = (sprite >> (7 - bit)) & 0x01;

                // flip if pixel == 1
                if pixel == 1 {
                    // collision aka if both sprite and screen == true

                    if self.display[curr_y][curr_x] {
                        self.v[0xF] = 1;
                    }
                    self.display[curr_y][curr_x] ^= true;

                }

            }
        }


    }
    
    // skips next instruction if key with value of Vx is pressed
    pub fn op_ex9e(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        // true = key down
        if self.keyboard[self.v[x] as usize] {
            self.pc += 2;
        }
    }

    // skips next instruction if key with value of Vx is not being pressed
    pub fn op_exa1(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        // false = key up
        if !self.keyboard[self.v[x] as usize] {
            self.pc += 2;
        }
    }

    // set vx to delay timer
    pub fn op_fx07(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        self.v[x] = self.delay;
    }

    // wait for key to be pressed then set the value of key in Vx
    pub fn op_fx0a(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        // check if a key is pressed and then set Vx to that key
        for (key, &pressed) in self.keyboard.iter().enumerate() {
            if pressed {
                self.v[x] = key as u8;
                return;
            }
        }

        // else, decrement pc by 2 to repeat instruction
        self.pc -= 2;
    }

    // set delay timer = Vx
    pub fn op_fx15(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        self.delay = self.v[x];
    }

    // set sound timer = Vx
    pub fn op_fx18(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        self.sound = self.v[x];
    }

    // set I = I + Vx
    pub fn op_fx1e(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);
        
        self.i += self.v[x] as u16;
    }

    // set register I = location of sprite for digit Vx
    pub fn op_fx29(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        self.i = (0x50 + (self.v[x] * 5)) as u16;
    }

    // set i, i+1, i+2 equal to Vx's 100s, 10s, and 1s digit respectively
    pub fn op_fx33(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        let hundreds = self.v[x] / 100;
        let tens = (self.v[x] % 100) / 10;
        let ones = self.v[x] % 10;

        self.memory[self.i as usize] = hundreds;
        self.memory[(self.i + 1) as usize] = tens;
        self.memory[(self.i + 2) as usize] = ones;
    }

    // store registers V0 through Vx in memory starting at I
    pub fn op_fx55(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        for i in 0..=x {
            self.memory[self.i as usize + i] = self.v[i]; 
        }
    }

    // read registers V0 thorugh Vx in memory starting at I
    pub fn op_fx65(&mut self, opcode: u16) {
        let x = CPU::get_x(opcode);

        for i in 0..=x {
            self.v[i] = self.memory[self.i as usize + i]; 
        }
    }


}