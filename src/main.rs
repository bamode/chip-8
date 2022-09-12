#![allow(unused)]

use chippers::chip::*;
use chippers::terminal::*;
use crossterm::terminal;
use std::io::{stdout, Write};

fn main() -> std::result::Result<(), TerminalError> {
    let input = clap::builder::Command::new("chippers")
        .args(&[clap::arg!(<FILE> "chip-8 rom file")])
        .get_matches();
    terminal::enable_raw_mode().unwrap();
    let mut chip8 = Chip8::new();
    chip8.load_font_set();
    let file = std::fs::read(&input.get_one::<String>("FILE").unwrap()).unwrap();
    let file = file.as_slice();
    for (i, byte) in file.iter().enumerate() {
        chip8.cpu.mem[i + 0x200] = *byte;
    }
    chip8.run()?;
    Ok(())
}
