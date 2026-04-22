use std::{env, fs};

mod chip8;
use chip8::Chip8;
use macroquad::prelude::*;

#[macroquad::main("chip8")]
async fn main() {
    let argv: Vec<String> = env::args().collect();
    let ch8_file = argv.get(1);
    let show_regs = argv.contains(&"--show-regs".to_string());

    if ch8_file.is_none() {
        eprintln!("Usage: {} <chip-8 rom path>", &argv[0]);
        return;
    }

    let result = fs::read(ch8_file.unwrap());
    match result {
        Ok(rom) => {
            let mut chip8 = Chip8::new();
            chip8.load_rom(rom);
            if show_regs {
                chip8.print_heads()
            }
            loop {
                chip8.update_keypad_state();
                chip8.cycle();
                if show_regs {
                    chip8.print_regs();
                }
                next_frame().await;
            }
        },
        Err(e) => eprintln!("Failed to read file: {e}")
    }
}