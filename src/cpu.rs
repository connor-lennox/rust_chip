use super::display;
use super::keypad;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use rand::prelude::*;

const FONT: [u8; 80] = [
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

const FONT_HEAD: usize = 0x050;

pub struct Cpu {
    pub memory: [u8; 4096],
    pub display: display::Display,
    pub keypad: keypad::Keypad,
    pub pc: u16,
    pub index: usize,
    pub stack: [u16; 32],
    pub stack_pointer: usize,
    pub delay: u8,
    pub sound: u8,
    pub v: [u8; 16],
    pub draw_flag: bool,
}

#[derive(Debug)]
struct Opcode(u8, u8, u8, u8);

impl Cpu {
    pub fn new() -> Cpu {
        let mut c = Cpu {
            memory: [0; 4096],
            display: display::Display::new(),
            keypad: keypad::Keypad::new(),
            pc: 0x200,
            index: 0,
            stack: [0; 32],
            stack_pointer: 0,
            delay: 0,
            sound: 0,
            v: [0; 16],
            draw_flag: false,
        };
        c.init_font();
        c
    }

    fn init_font(&mut self) {
        for i in 0..80 {
            self.memory[FONT_HEAD + i] = FONT[i];
        }
    }

    pub fn read_program(&mut self, path: &Path) -> io::Result<()> {
        let mut f = File::open(path)?;
        let mut buf = [0; 3584];
        f.read(&mut buf)?;
        // Copy read buffer into memory at correct location
        for i in 0..3584 {
            self.memory[i + 0x200] = buf[i];
        }
        Ok(())
    }

    pub fn do_cycle(&mut self) {
        // Fetch opcode and progress program counter
        let op = self.get_current_opcode();
        self.pc += 2;
        self.process_opcode(&op)
    }

    fn get_current_opcode(&self) -> Opcode {
        Opcode(
            (self.memory[self.pc as usize] & 0xF0) >> 4, 
            self.memory[self.pc as usize] & 0x0F, 
            (self.memory[self.pc as usize +1] & 0xF0) >> 4,
            self.memory[self.pc as usize +1] & 0x0F 
        )
    }

    fn process_opcode(&mut self, opcode: &Opcode) {
        let x: usize = opcode.1 as usize;
        let y: usize = opcode.2 as usize;
        let n: u8 = opcode.3;
        let nn: u8 = (opcode.2 << 4) | opcode.3;
        let nnn: u16 = ((opcode.1 as u16) << 8) | ((opcode.2 as u16) << 4) | opcode.3 as u16;

        match opcode {
            // Clear screen
            Opcode(0x0, 0x0, 0xE, 0x0) => {
                self.display.clear();
                self.draw_flag = true;
            }
            // Return
            Opcode(0x0, 0x0, 0xE, 0xE) => {
                self.stack_pointer -= 1;
                self.pc = self.stack[self.stack_pointer];
            }
            // Jump
            Opcode(0x1, _, _, _) => self.pc = nnn,
            // Call subroutine
            Opcode(0x2, _, _, _) => {
                self.stack[self.stack_pointer] = self.pc;
                self.stack_pointer += 1;
                self.pc = nnn;
            }
            // Skip if equal
            Opcode(0x3, _, _, _) => self.pc += if self.v[x] == nn {2} else {0},
            // Skip if not equal
            Opcode(0x4, _, _, _) => self.pc += if self.v[x] != nn {2} else {0},
            // Skip if matching
            Opcode(0x5, _, _, _) => self.pc += if self.v[x] == self.v[y] {2} else {0},
            // Set VX
            Opcode(0x6, _, _, _) => self.v[x] = nn,
            // Add to VX
            Opcode(0x7, _, _, _) => {
                let (sum, _) = self.v[x].overflowing_add(nn);
                self.v[x] = sum;
            }
            // Set VX to VY
            Opcode(0x8, _, _, 0x0) => self.v[x] = self.v[y],
            // Logical OR
            Opcode(0x8, _, _, 0x1) => self.v[x] |= self.v[y],
            // Logical AND
            Opcode(0x8, _, _, 0x2) => self.v[x] &= self.v[y],
            // Logical XOR
            Opcode(0x8, _, _, 0x3) => self.v[x] ^= self.v[y],
            // Add
            Opcode(0x8, _, _, 0x4) => {
                let (sum, overflow): (u8, bool) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = sum;
                self.v[0xF] = if overflow {1} else {0};
            }
            // Subtract VY from VX
            Opcode(0x8, _, _, 0x5) => {
                let (diff, underflow): (u8, bool) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = diff;
                self.v[0xF] = if underflow {0} else {1};
            }
            // Right shift VX
            Opcode(0x8, _, _, 0x6) => {
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            }
            // Subtract VY from VX (placed in VX)
            Opcode(0x8, _, _, 0x7) => {
                let (diff, underflow): (u8, bool) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = diff;
                self.v[0xF] = if underflow {0} else {1};
            }
            // Left shift VX
            Opcode(0x8, _, _, 0xE) => {
                self.v[0xF] = (self.v[x] & 0x8) >> 3;
                self.v[x] <<= 1;
            }
            // Skip if not matching
            Opcode(0x9, _, _, _) => self.pc += if self.v[x] != self.v[y] {2} else {0},
            // Set I immediate
            Opcode(0xA, _, _, _) => self.index = nnn as usize,
            // Jump V0 + immediate
            Opcode(0xB, _, _, _) => self.pc = self.v[0] as u16 + nnn,
            // Generate random number
            // Binary AND random number with nn
            Opcode(0xC, _, _, _) => self.v[x] = random::<u8>() & nn,
            // Display sprite
            Opcode(0xD, _, _, _) => {
                self.v[0xF] = if self.display.draw(self.v[x] as usize, self.v[y] as usize, &self.memory[self.index..self.index+n as usize]) {1} else {0};
                self.draw_flag = true;
            },
            // Skip if key pressed
            Opcode(0xE, _, 0x9, 0xE) => self.pc += if self.keypad.state[self.v[x] as usize] {2} else {0},
            // Skip if key not pressed
            Opcode(0xE, _, 0xA, 0x1) => self.pc += if !self.keypad.state[self.v[x] as usize] {2} else {0},
            // Read delay timer
            Opcode(0xF, _, 0x0, 0x7) => self.v[x] = self.delay,
            // Write delay timer
            Opcode(0xF, _, 0x1, 0x5) => self.delay = self.v[x],
            // Write sound timer
            Opcode(0xF, _, 0x1, 0x8) => self.sound = self.v[x],
            // Add VX to index register
            Opcode(0xF, _, 0x1, 0xE) => {
                self.index += self.v[x] as usize;
                // Optional behaviour: VF set on memory bound overflow
                self.v[0xF] = if self.index >= 0x1000 {1} else {0};
            },
            // Get next key ("blocks", then set VX to key)
            Opcode(0xF, _, 0x0, 0xA) => {
                if self.keypad.just_released {
                    self.v[x] = self.keypad.last_released as u8;
                } else {
                    // "Block" by looping back to this opcode until input received
                    self.pc -= 2;
                }
            }
            // Set I to position of font character in VX (using trailing nibble)
            Opcode(0xF, _, 0x2, 0x9) => {
                let char: usize = self.v[x] as usize & 0x0F;
                self.index = FONT_HEAD + (char * 5);
            },
            // Binary-coded decimal conversion
            Opcode(0xF, _, 0x3, 0x3) => {
                let val = self.v[x];
                self.memory[self.index] = (val / 100) % 10;
                self.memory[self.index + 1] = (val / 10) % 10;
                self.memory[self.index + 2] = val % 10;
            },
            // Store memory
            Opcode(0xF, _, 0x5, 0x5) => {
                for i in 0..=x as usize {
                    self.memory[self.index + i] = self.v[i];
                }
            },
            // Recall memory
            Opcode(0xF, _, 0x6, 0x5) => {
                for i in 0..=x as usize {
                    self.v[i] = self.memory[self.index + i];
                }
            }
            _ => ()
        }
    }
}