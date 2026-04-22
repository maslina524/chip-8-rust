use std::io::{self, Write};

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


    pub fn cycle(&mut self) {
        let opcode = ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[(self.pc + 1) as usize] as u16);
        self.pc += 2;

        let x = self.get_bits(opcode, 8, 11);
        let y = self.get_bits(opcode, 4, 7);
        let n = (opcode & 0xF) as u8;
        let nn = (opcode & 0xFF) as u8;
        let nnn = opcode & 0x0FFF;
        match opcode >> 12 {
            0x0 => {},
            0x1 => {
                self.pc = nnn
            },
            0x2 => {},
            0x3 => {},
            0x4 => {},
            0x5 => {},
            0x6 => {   // LD Vx, nn
                self.v[x as usize] = nn;
            },
            0x7 => {},
            0x8 => {},
            0x9 => {},
            0xA => {},
            0xB => {},
            0xC => {},
            0xD => {},
            0xE => {},
            0xF => {},

            _ => unreachable!()
        }
    }
}