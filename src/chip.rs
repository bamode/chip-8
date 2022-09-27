use crate::cpu::*;

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Chip8 {
    pub cpu: Cpu,
    clock: Clock,
    timer: Instant,
    tx: Sender<Chip8Message>,
}

#[derive(Clone, Debug)]
pub enum KeyCode {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    Null, // No key pressed
    Quit, // Escape or Ctrl+C would be good
}

impl Chip8 {
    pub fn new(rx: Receiver<KeyCode>) -> (Chip8, Receiver<Chip8Message>) {
        let cpu = Cpu::new(rx);
        let clock = Clock;
        let timer = Instant::now();
        let (tx, chip_rx) = channel();
        (
            Chip8 {
                cpu,
                clock,
                timer,
                tx,
            },
            chip_rx,
        )
    }

    pub fn run(&mut self) -> std::result::Result<(), SendError<Chip8Message>> {
        self.tx.send(Chip8Message::ClearScreen).unwrap();
        loop {
            let next_inst = self.cpu.fetch_next();
            let msg = self.cpu.execute_instruction(next_inst);
            match msg {
                Chip8Message::None => {}
                Chip8Message::ClearScreen => self.tx.send(Chip8Message::ClearScreen)?,
                Chip8Message::DrawScreen(d) => self.tx.send(Chip8Message::DrawScreen(d))?,
            }
            let now = Instant::now();
            if now - self.timer > Duration::from_secs_f64(1. / 60.) {
                self.timer = now;
                if self.cpu.dt > 0 {
                    self.cpu.dt -= 1;
                }
                if self.cpu.st > 0 {
                    self.cpu.st -= 1;
                }
            }

            self.clock.tick();
        }
    }
    pub fn load_font_set(&mut self) {
        for (i, byte) in FONT_SET.iter().enumerate() {
            self.cpu.mem[i + 0x50] = *byte;
        }
    }

    pub fn load_rom(&mut self, path: PathBuf) {
        let file = std::fs::read(path).unwrap();
        let file = file.as_slice();
        for (i, byte) in file.iter().enumerate() {
            self.cpu.mem[i + 0x200] = *byte;
        }
    }
}

#[derive(Debug)]
pub enum Chip8Message {
    None,
    ClearScreen,
    DrawScreen([[u8; 32]; 64]),
}

pub const CLOCK_RATE: f64 = 100.; // Hz, 700 instructions per second

#[derive(Debug)]
pub struct Clock;

impl Clock {
    pub fn tick(&self) {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
