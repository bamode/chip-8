use chippers::chip::*;
use std::sync::mpsc::{channel, SendError};

fn main() -> std::result::Result<(), SendError<Chip8Message>> {
    let input = clap::builder::Command::new("chippers")
        .args(&[clap::arg!(<FILE> "chip-8 rom file")])
        .get_matches();
    
    let (_tx, rx) = channel();
    let (mut chip8, _chip_rx) = Chip8::new(rx);
    chip8.load_font_set();
    
    let file = std::fs::read(&input.get_one::<String>("FILE").unwrap()).unwrap();
    let file = file.as_slice();
    for (i, byte) in file.iter().enumerate() {
        chip8.cpu.mem[i + 0x200] = *byte;
    }

    chip8.run()?;
    Ok(())
}
