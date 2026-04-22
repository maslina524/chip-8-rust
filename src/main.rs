use std::{env, fs};

mod chip8;
use chip8::Chip8;

fn main() {
    let argv: Vec<String> = env::args().collect();
    let ch8_file = argv.get(1);

    if ch8_file.is_none() {
        eprintln!("Usage: {} <chip-8 rom path>", &argv[0]);
        return;
    }

    let result = fs::read(ch8_file.unwrap());
    match result {
        Ok(rom) => run(rom),
        Err(e) => eprintln!("Failed to read file: {e}")
    }
}

fn run(rom: Vec<u8>) {
    println!("{:?}", rom);
    let mut chip8 = Chip8::new();
    chip8.load_rom(rom);
    loop {
        chip8.cycle();
    }
}