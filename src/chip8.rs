use std::io::{self, Write};
use fastrand;
use macroquad::prelude::*;

pub enum Ch8Errs {
    UnknownOpcode(u16)
}

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u16,
    delay_timer: u8,
    sound_timer: u8,
    screen: [[bool; 64]; 32],
    keypad: [bool; 16],
    waiting_for_key: Option<u8>,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            screen: [[false; 64]; 32],
            keypad: [false; 16],
            waiting_for_key: None
        };
        chip8.load_font();
        request_new_screen_size(1024.0, 512.0);

        chip8
    }
    
    fn load_font(&mut self) {
        let font: [u8; 80] = [
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
        
        self.memory[0..80].copy_from_slice(&font);
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        let start_address = 0x200;
        for (i, byte) in rom.iter().enumerate() {
            self.memory[start_address + i] = *byte;
        }
        self.pc = 0x200;
    }

    #[inline(always)]
    fn get_bits(&self, value: u16, start: u16, end: u16) -> u16 {
        let mask = (1 << (end - start + 1)) - 1;
        (value >> start) & mask
    }

    pub fn print_regs(&self) {
        print!("\r");
        for v in &self.v {
            print!("{v:<3} ");
        }
        io::stdout().flush().unwrap();
    }

    pub fn print_heads(&self) {
        for (i, _) in self.v.iter().enumerate() {
            print!("v{i:X}  ");
        }
        println!()
    }


    pub fn cycle(&mut self) -> Result<(), Ch8Errs> {
        let opcode = ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[(self.pc + 1) as usize] as u16);
        self.pc += 2;

        let mut x = self.get_bits(opcode, 8, 11) as usize;
        let mut y = self.get_bits(opcode, 4, 7) as usize;
        let n = (opcode & 0xF) as u8;
        let nn = (opcode & 0xFF) as u8;
        let nnn = opcode & 0x0FFF;
        match opcode >> 12 {
            0x0 => {
                match opcode {
                    0x00E0 => self.screen = [[false; 64]; 32],
                    0x00EE => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize]
                    }
                    _ => return Err(Ch8Errs::UnknownOpcode(opcode))
                }
            },
            0x1 => {
                self.pc = nnn
            },
            0x2 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },
            0x3 => {
                if self.v[x] == nn {
                    self.pc += 2
                }
            },
            0x4 => {
                if self.v[x] != nn {
                    self.pc += 2
                }
            },
            0x5 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2
                }
            },
            0x6 => {   // LD Vx, nn
                self.v[x] = nn;
            },
            0x7 => {
                self.v[x] = self.v[x].wrapping_add(nn)
            },
            0x8 => {
                match n {
                    0x0 => self.v[x] = self.v[y],
                    0x1 => self.v[x] = self.v[x] | self.v[y],
                    0x2 => self.v[x] = self.v[x] & self.v[y],
                    0x3 => self.v[x] = self.v[x] ^ self.v[y],
                    0x4 => {
                        let ret: u16 = (self.v[x] + self.v[y]) as u16;
                        self.v[0xF] = if ret > 0xFF { 1 } else { 0 };
                        self.v[x] = ret as u8 // & 0xFF
                    },
                    0x5 => {
                        self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
                        self.v[x] = self.v[x] - self.v[y];
                    },
                    0x6 => {
                        self.v[0xF] = self.v[y] & 1;
                        self.v[x] = self.v[y] >> 1;
                    },
                    0x7 => {
                        self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
                        self.v[x] = self.v[y] - self.v[x];
                    },
                    0xE => {
                        self.v[0xF] = self.v[y] >> 7;
                        self.v[x] = self.v[y] << 1;
                    },
                    _ => return Err(Ch8Errs::UnknownOpcode(opcode))
                }
            },
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2
                }
            },
            0xA => {},
            0xB => {
                self.pc = nnn + self.v[0x0] as u16;
            },
            0xC => {
                self.v[x] = fastrand::u8(0..255)
            },
            0xD => {},
            0xE => {
                match nn {
                    0x9E => {
                        if self.keypad[self.v[x] as usize] {
                            self.pc += 2
                        }
                    }
                    0xA1 => {
                        if !self.keypad[self.v[x] as usize] {
                            self.pc += 2
                        }
                    }
                    _ => return Err(Ch8Errs::UnknownOpcode(opcode))
                }
            },
            0xF => {
                match nn {
                    0x07 => self.v[x] = self.delay_timer,
                    0x0A => {
                        self.waiting_for_key = Some(x as u8);
                        self.pc -= 2;
                    },
                    0x15 => self.delay_timer = self.v[x],
                    0x18 => self.sound_timer = self.v[x],
                    0x1E => self.i = self.i + self.v[x] as u16,
                    0x29 => self.i = self.v[x] as u16 * 5,
                    0x33 => {
                            let val = self.v[x];
                            self.memory[self.i as usize] = val / 100;
                            self.memory[self.i as usize + 1] = (val / 10) % 10;
                            self.memory[self.i as usize + 2] = val % 10;
                    },
                    0x55 => {
                        let reg_i = self.i;
                        for i in 0..=x {
                            self.memory[reg_i as usize + i] = self.v[i]
                        } 
                    },
                    0x65 => {
                        let reg_i = self.i;
                        for i in 0..=x {
                            self.v[i] = self.memory[reg_i as usize + i]
                        } 
                    }
                    _ => return Err(Ch8Errs::UnknownOpcode(opcode))
                }
            },

            _ => unreachable!()
        }

        Ok(())
    }

    pub fn handle_waiting_key(&mut self) {
        if let Some(reg) = self.waiting_for_key {
            if let Some(key) = self.get_pressed_ch8_key() {
                self.v[reg as usize] = key;
                self.waiting_for_key = None;
                self.pc += 2;
            }
        }
    }

    fn get_pressed_ch8_key(&self) -> Option<u8> {
        if is_key_pressed(KeyCode::Key1) { return Some(0x1); }
        if is_key_pressed(KeyCode::Key2) { return Some(0x2); }
        if is_key_pressed(KeyCode::Key3) { return Some(0x3); }
        if is_key_pressed(KeyCode::Key4) { return Some(0xC); }
        
        if is_key_pressed(KeyCode::Q) { return Some(0x4); }
        if is_key_pressed(KeyCode::W) { return Some(0x5); }
        if is_key_pressed(KeyCode::E) { return Some(0x6); }
        if is_key_pressed(KeyCode::R) { return Some(0xD); }
        
        if is_key_pressed(KeyCode::A) { return Some(0x7); }
        if is_key_pressed(KeyCode::S) { return Some(0x8); }
        if is_key_pressed(KeyCode::D) { return Some(0x9); }
        if is_key_pressed(KeyCode::F) { return Some(0xE); }
        
        if is_key_pressed(KeyCode::Z) { return Some(0xA); }
        if is_key_pressed(KeyCode::X) { return Some(0x0); }
        if is_key_pressed(KeyCode::C) { return Some(0xB); }
        if is_key_pressed(KeyCode::V) { return Some(0xF); }
        
        None
    }

    pub fn update_keypad_state(&mut self) {
        self.keypad[0x0] = is_key_down(KeyCode::Key0);
        self.keypad[0x1] = is_key_down(KeyCode::Key1);
        self.keypad[0x2] = is_key_down(KeyCode::Key2);
        self.keypad[0x3] = is_key_down(KeyCode::Key3);
        self.keypad[0x4] = is_key_down(KeyCode::Key4);
        self.keypad[0x5] = is_key_down(KeyCode::Key5);
        self.keypad[0x6] = is_key_down(KeyCode::Key6);
        self.keypad[0x7] = is_key_down(KeyCode::Key7);
        self.keypad[0x8] = is_key_down(KeyCode::Key8);
        self.keypad[0x9] = is_key_down(KeyCode::Key9);
        self.keypad[0xA] = is_key_down(KeyCode::A);
        self.keypad[0xB] = is_key_down(KeyCode::B);
        self.keypad[0xC] = is_key_down(KeyCode::C);
        self.keypad[0xD] = is_key_down(KeyCode::D);
        self.keypad[0xE] = is_key_down(KeyCode::E);
        self.keypad[0xF] = is_key_down(KeyCode::F);
    }
}