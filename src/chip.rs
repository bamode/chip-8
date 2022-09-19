use crate::cpu::*;
use crate::terminal::*;

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Chip8 {
    pub cpu: Cpu,
    clock: Clock,
    timer: Instant,
    term: Terminal,
}

impl Chip8 {
    pub fn new() -> Self {
        let cpu = Cpu::new();
        let clock = Clock;
        let timer = Instant::now();
        let term = Terminal;
        Chip8 {
            cpu,
            clock,
            timer,
            term,
        }
    }
    pub fn run(&mut self) -> std::result::Result<(), TerminalError> {
        self.term.clear_screen()?;
        loop {
            let next_inst = self.cpu.fetch_next();
            let msg = self.cpu.execute_instruction(next_inst);
            match msg {
                Chip8Message::None => {}
                Chip8Message::ClearScreen => self.term.clear_screen()?,
                Chip8Message::DrawScreen => self.term.draw_screen(&self.cpu.disp)?,
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
}

pub enum Chip8Message {
    None,
    ClearScreen,
    DrawScreen,
}

pub const CLOCK_RATE: f64 = 100.; // Hz, 700 instructions per second

#[derive(Debug)]
pub struct Clock;

impl Clock {
    pub fn tick(&self) {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
