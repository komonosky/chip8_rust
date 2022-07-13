use rand::random;

pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;
const FONSTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
const RAM_SIZE: usize = 4096;
const START_ADDRESS: u16 = 0x200;

pub struct CPU {
    pc: u16,    // program counter
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
    v: [u8; 16],    // V registers
    i: u16,         // I register
    // stack: Vec<u16>,
    stack: [u16; 16],
    sp: u16,    // stack pointer
    keyboard: [bool; 16],
    delay_timer: u8,
    sound_timer: u8,
}

impl CPU {
    pub fn new() -> Self {
        let mut new_cpu = Self {
            pc: START_ADDRESS,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            v: [0; 16],
            i: 0,
            // stack: vec![0; 16],
            stack: [0; 16],
            sp: 0,
            keyboard: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        };

        new_cpu.ram[..80].copy_from_slice(&FONSTSET);   // load fontset
        new_cpu
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn get_keys(&mut self, idx: usize, pressed: bool) {
        assert!(idx < 16, "Keyboard index must be under 16 - backend error");
        self.keyboard[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDRESS as usize;
        let end = start + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
        self.v = [0; 16];
        self.i = 0;
        self.stack = [0; 16];
        self.sp = 0;
        self.keyboard = [false; 16];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.ram[..80].copy_from_slice(&FONSTSET);
    }

    // Push and Pop methods for the stack
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        assert!(self.stack.len() > 0, "Cannot pop() from empty stack!");
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn step(&mut self) {
        // Fetch
        let opcode = self.fetch();
        // decode and execute
        self.execute(opcode);
    }

    fn fetch(&mut self) -> u16 {
        let opcode = ((self.ram[self.pc as usize] as u16) << 8) | (self.ram[(self.pc + 1) as usize] as u16);
        self.pc += 2;
        opcode
    }

    fn execute(&mut self, opcode: u16) {
        let nibble = (opcode & 0xF000) >> 12;
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let n = opcode & 0x000F;
        let nn = opcode & 0x00FF;
        let nnn = opcode & 0x0FFF;

        match (nibble, x, y, n) {
            (0, 0, 0, 0) => return, // nop
            (0, 0, 0xE, 0) => {     
                // clear screen
                self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
            },
            (0, 0, 0xE, 0xE) => {
                // Return from subroutine
                self.pc = self.pop();
            },
            (1, _, _, _) => {
                // Jump
                self.pc = nnn;
            },
            (2, _, _, _) => {
                // Call subroutine
                self.push(self.pc);
                self.pc = nnn;
            },
            (3, _, _, _) => {
                // Skip
                if self.v[x as usize] == nn as u8 {
                    self.pc += 2;
                }
            },
            (4, _, _, _) => {
                // Skip
                if self.v[x as usize] != nn as u8 {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => {
                // Skip
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => {
                // Set
                self.v[x as usize] = nn as u8;
            },
            (7, _, _, _) => {
                // Add
                // Rust will panic if this overflows, so need to use wrapping_add 
                self.v[x as usize] = self.v[x as usize].wrapping_add(nn as u8);
            },
            // Logical and Arithmetic instructions
            (8, _, _, 0) => {
                // Set
                self.v[x as usize] = self.v[y as usize];
            },
            (8, _, _, 1) => {
                // Binary OR
                self.v[x as usize] |= self.v[y as usize];
            },
            (8, _, _, 2) => {
                // Binary AND
                self.v[x as usize] &= self.v[y as usize];
            },
            (8, _, _, 3) => {
                // Logical XOR
                self.v[x as usize] ^= self.v[y as usize]; 
            },
            (8, _, _, 4) => {
                // Add
                // Add v[y] to v[x]. If it overflows, v[0xF] is set to 1
                let (new_vx, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                let new_vf = if carry { 1 } else { 0 };
                self.v[x as usize] = new_vx;
                self.v[0xF] = new_vf;
            },
            (8, _, _, 5) => {
                // Subtract
                // Subtract v[y] from v[x]. If it underflows, v[0xF] is set to 0
                let (new_vx, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v[x as usize] = new_vx;
                self.v[0xF] = new_vf;
            },
            (8, _, _, 6) => {
                // Shift
                // Right shift v[x], store the dropped bit in v[0xF]
                let least_sig_bit = self.v[x as usize] & 1;     // get dropped bit
                self.v[x as usize] >>= 1;                       // shift v[x]
                self.v[0xF] = least_sig_bit;
            },
            (8, _, _, 7) => {
                // Subtract
                // Subtract v[x] from v[y]. If it underflows, v[0xF] is set to 0
                let (new_vy, borrow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v[y as usize] = new_vy;
                self.v[0xF] = new_vf;
            },
            (8, _, _, 0xE) => {
                // Shift
                // Left shift v[x], store the dropped bit in v[0xF]
                let most_sig_bit = (self.v[x as usize] >> 7) & 1;
                self.v[x as usize] <<= 1;
                self.v[0xF] = most_sig_bit;
            },
            (9, _, _, 0) => {
                // Skip
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            },
            (0xA, _, _, _) => {
                // Set Index
                self.i = nnn;
            },
            (0xB, _, _, _) => {
                // Jump with offset
                self.pc = nnn + (self.v[0] as u16);
            },
            (0xC, _, _, _) => {
                // Random
                // Generate a random number, binary AND with nn, place result into v[x]
                let rng: u8 = random();
                self.v[x as usize] = rng & (nn as u8);
            },
            (0xD, _, _, _) => {
                // Draw

                // Get x and y coordinates
                let x_coord = self.v[x as usize] as u16;
                let y_coord = self.v[y as usize] as u16;
                self.v[0xF] = 0;
                let mut flipped_pixels = false;

                // Iterate over each row of the sprite
                for j in 0..n {     // y-axis
                    let pixels = self.ram[(self.i + j as u16) as usize];
                    // Iterate through each column in the row
                    for k in 0..8 {     // x-axis
                        // Get current pixels bit, flip if 1
                        if (pixels & (0b10000000 >> k)) != 0 {
                            // Wrap sprite around screen
                            let a = (x_coord + k) as usize % SCREEN_WIDTH;
                            let b = (y_coord + j) as usize % SCREEN_HEIGHT;

                            // Get pixel index
                            let idx = a + SCREEN_WIDTH * b;
                            // Check if pixel is about to be flipped and set
                            flipped_pixels |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

               self.v[0xF] = if flipped_pixels { 1 } else { 0 };
            },
            (0xE, _, 9, 0xE) => {
                // Skip if key in v[x] is pressed
                if self.keyboard[self.v[x as usize] as usize] {
                    self.pc += 2;
                }
            },
            (0xE, _, 0xA, 1) => {
                // Skip if key in v[x] is not pressed
                if !self.keyboard[self.v[x as usize] as usize] {
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => {
                // Set v[x] to the current value of the delay timer
                self.v[x as usize] = self.delay_timer;
            },
            (0xF, _, 1, 5) => {
                // Set the delay timer to the value in v[x]
                self.delay_timer = self.v[x as usize];
            },
            (0xF, _, 1, 8) => {
                // Set the sound timer to the value in v[x]
                self.sound_timer = self.v[x as usize];
            },
            (0xF, _, 1, 0xE) => {
                // Add to Index
                // Add v[x] t index register
                self.i = self.i.wrapping_add(self.v[x as usize] as u16);
            },
            (0xF, _, 0, 0xA) => {
                // Get key
                // Stop execution and wait for key input
                let mut pressed = false;
                for i in 0..self.keyboard.len() {
                    if self.keyboard[i] {
                        self.v[x as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc += 2;
                }
            },
            (0xF, _, 2, 9) => {
                // Font character
                // set I to hex address stored in v[x]
                self.i = (self.v[x as usize] as u16) * 5    // each font sprite takes up 5 bytes, so multiply by 5 to get RAM address
            },
            (0xF, _, 3, 3) => {
                // Binary-coded decimal conversion
                self.ram[self.i as usize] = self.v[x as usize] / 100;
                self.ram[(self.i as usize) + 1] = self.v[x as usize] % 100 / 10;
                self.ram[(self.i as usize) + 2] = self.v[x as usize] % 10;
            },
            (0xF, _, 5, 5) => {
                // Store registers to memory
                for idx in 0..=(x as usize) {
                    self.ram[(self.i as usize) + idx] = self.v[idx];
                }
            },
            (0xF, _, 6, 5) => {
                for idx in 0..=(x as usize) {
                    self.v[idx] = self.ram[(self.i as usize) + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", opcode),
        }
    }

    pub fn timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // play beep - not used in this implementation (yet)
            }
            self.sound_timer -= 1;
        }   
    }
}