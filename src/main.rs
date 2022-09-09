#![allow(unused)]
use std::io::{stdout, Write};
use bitvec::prelude::*;
use crossterm::{cursor, QueueableCommand, execute, Result, style::{self, Stylize}, terminal, terminal::size};

fn main() {
    let mut chip8 = Chip8::new();
    let file = std::fs::read("IBM Logo.ch8").unwrap();
    let file = file.as_slice();
    for (i, byte) in file.iter().enumerate() {
        chip8.cpu.mem[i + 0x200] = *byte;
    }
    chip8.run().unwrap();
}

#[derive(Debug)]
struct Chip8 {
    cpu: Cpu,
    clock: Clock,
    term: Terminal,
}

impl Chip8 {
    pub fn new() -> Self {
        let cpu = Cpu::new();
        let clock = Clock;
        let term = Terminal;
        Chip8 { cpu, clock, term }
    }
    pub fn run(&mut self) -> std::result::Result<(), TerminalError> {
        self.term.clear_screen();
        loop {
            let next_inst = self.cpu.fetch_next();
            let msg = self.cpu.execute_instruction(next_inst);
            match msg {
                Chip8Message::None => { },
                Chip8Message::ClearScreen => { self.term.clear_screen()? },
                Chip8Message::DrawScreen => { self.term.draw_screen(&self.cpu.disp)? },
            }
            self.clock.tick();
        }
    }
}

enum Chip8Message {
    None,
    ClearScreen,
    DrawScreen,
}

const CLOCK_RATE: f64 = 700. ; // Hz, 700 instructions per second

#[derive(Debug)]
struct Clock;

impl Clock {
    fn tick(&self) {
        std::thread::sleep(std::time::Duration::from_millis((1. / CLOCK_RATE * 100.).round() as u64));
    }
}

#[derive(Debug)]
struct Terminal;

impl Terminal {
    const MIN_WIDTH: u16 = 64;
    const MIN_HEIGHT: u16 = 32;

    fn check_bounds(&self, w: u16, h: u16) -> std::result::Result<(), TerminalError> {
        if w < Self::MIN_WIDTH || h < Self::MIN_HEIGHT {
            return Err(TerminalError::ErrorKind("terminal is too small to display screen".to_string()))
        }
        Ok(())
    }
}

trait TerminalBackend { 
    type Error;
    fn clear_screen(&mut self) -> std::result::Result<(), Self::Error>;
    fn draw_screen(&mut self, display: &[[u8; 32]; 64]) -> std::result::Result<(), Self::Error>;
}

#[derive(Clone, Debug)]
enum TerminalError { 
    ErrorKind(String),
}

impl std::fmt::Display for TerminalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TerminalError::ErrorKind(s) => writeln!(f, "error: {}", s)?,
        }
        Ok(())
    }
}

impl std::error::Error for TerminalError { }

impl From<std::io::Error> for TerminalError {
    fn from(err: std::io::Error) -> TerminalError {
        dbg!(err);
        TerminalError::ErrorKind("received an io error".to_string())
    }
}

impl TerminalBackend for Terminal {
    type Error = TerminalError;
    fn clear_screen(&mut self) -> std::result::Result<(), Self::Error> { 
        let (width, height) = size()?;
        self.check_bounds(width, height)?;
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        Ok(()) 
    }

    fn draw_screen(&mut self, disp: &[[u8; 32]; 64]) -> std::result::Result<(), Self::Error> { 
        let (width, height) = size()?;
        self.check_bounds(width, height)?;
        let mut stdout = stdout();
        for (i, row) in disp.iter().enumerate() {
            for (j, pix) in row.iter().enumerate() {
                stdout.queue(cursor::MoveTo(i as u16, j as u16))?;
                if *pix == 0 {
                    stdout.queue(style::PrintStyledContent("█".black()))?;
                } else if *pix == 1 {
                    stdout.queue(style::PrintStyledContent("█".white()))?;
                } else {
                    return Err(TerminalError::ErrorKind("display pixel set to value other than 0 or 1".to_string()))
                }
            }
        }
        stdout.flush();
        Ok(()) 
    }
}

type Memory = [u8; 4096];
type Display = [[u8; 32]; 64];
type I = u16;
type Stack = [u16; 16];
type DelayTimer = u8;
type SoundTimer = u8;
type Register = [u8; 16];
type ProgramCounter = u16;

/// All CHIP-8 programs start the program counter here.
const START: u16 = 0x200;

#[derive(Debug)]
struct Cpu {
    mem: Memory,
    disp: Display,
    index: I,
    stack: Stack,
    dt: DelayTimer,
    st: SoundTimer,
    reg: Register,
    pc: ProgramCounter,
}

impl Cpu {
    pub fn new() -> Self {
        let mem = [0u8; 4096];
        let disp = [[0u8; 32]; 64];
        let index = 0;
        let stack = [0u16; 16];
        let dt = 0;
        let st = 0;
        let reg = [0u8; 16];
        let pc = START;

        Self {
            mem, disp, index, stack, dt, st, reg, pc
        }
    }

    pub fn fetch_next(&mut self) -> u16 {
        let next_inst = ((self.mem[self.pc as usize] as u16) << 8) as u16 + (self.mem[self.pc as usize + 1]) as u16;
        self.pc += 2;
        next_inst
    }

    pub fn execute_instruction(&mut self, inst: u16) -> Chip8Message {
        let op = inst >> 12;
        let nnn = inst & 0b0000_1111_1111_1111;
        let n = inst & 0b0000_0000_0000_1111;
        let x = (inst & 0b0000_1111_0000_0000) >> 8;
        let y = (inst & 0b0000_0000_1111_0000) >> 4;       
        let kk = inst & 0b0000_0000_1111_1111;
        let raw_op = RawOpcode::new(op, x, y, n, kk, nnn);
        let opcode = Opcode::from(&raw_op);
        match opcode {
            Opcode::None => { Chip8Message::None },
            Opcode::Error => { println!("cpu status: {:?}", self); panic!() }
            Opcode::Clear => { Chip8Message::ClearScreen },
            Opcode::Jump => { self.jump(nnn); Chip8Message::None },
            Opcode::SetVX => { self.set_vx(x, kk); Chip8Message::None },
            Opcode::AddVX => { self.add_vx(x, kk); Chip8Message::None },
            Opcode::SetI => { self.set_i(nnn); Chip8Message::None },
            Opcode::Draw => { self.draw(x, y, n); Chip8Message::DrawScreen },
        }
    }

    fn jump(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn set_vx(&mut self, x: u16, nn: u16) {
        self.reg[x as usize] = nn as u8;
    }

    fn add_vx(&mut self, x: u16, nn: u16) {
        self.reg[x as usize] += nn as u8;
    }

    fn set_i(&mut self, nnn: u16) {
        self.index = nnn;
    }

    fn draw(&mut self, x: u16, y: u16, n: u16) {
        let mut x_coord = self.reg[x as usize] % 64;
        let start_x_coord = x_coord;
        let mut y_coord = self.reg[y as usize] % 32;
        self.reg[0xF as usize] = 0;
        for i in 0..n {
            x_coord = start_x_coord;
            let sprite_data = self.mem[self.index as usize + i as usize];
            for b in sprite_data.view_bits::<Msb0>().iter().by_val() {
                if b && self.disp[x_coord as usize][y_coord as usize] == 1 {
                    self.disp[x_coord as usize][y_coord as usize] = 0;
                    self.reg[0xF as usize] = 1;
                } else if b && self.disp[x_coord as usize][y_coord as usize] == 0 {
                    self.disp[x_coord as usize][y_coord as usize] = 1;
                }
                if x_coord == 63 {
                    break;
                }
                x_coord += 1;
            }
            y_coord += 1;
            if y_coord == 31 {
                break;
            }
        }
    }
}

#[derive(Debug)]
struct RawOpcode {
    op: u16,
    x: u16,
    y: u16,
    n: u16,
    kk: u16,
    nnn: u16,
}

impl RawOpcode {
    fn new(op: u16, x: u16, y: u16, n: u16, kk: u16, nnn: u16) -> Self {
        RawOpcode { op, x, y, n, kk, nnn }
    }
}

impl std::fmt::Display for RawOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:x}{:x}{:x}{:x}", self.op, self.x, self.y, self.n)?;
        Ok(())
    }
}

enum Opcode {
    Clear, // 00E0
    Jump, // 1NNN
    SetVX, // 6XNN
    AddVX, // 7XNN
    SetI, // ANNN
    Draw, // DXYN
    None, // other
    Error, // error
}

impl std::convert::From<&RawOpcode> for Opcode {
    fn from(raw_op: &RawOpcode) -> Opcode {
        // we should be able to implement this as a series of matches
        match raw_op.op {
            0 => { 
                if raw_op.x == 0 && raw_op.y == 0xE && raw_op.n == 0 {
                    return Opcode::Clear 
                } else {
                    return Opcode::None
                }
            },
            1 => { return Opcode::Jump },
            6 => { return Opcode::SetVX },
            7 => { return Opcode::AddVX },
            0xA => { return Opcode::SetI },
            0xD => { return Opcode::Draw },
            _ => { println!("\nencountered unknown opcode: {}", raw_op); return Opcode::Error },
        }
    }
}