#![allow(unused)]
use bitvec::prelude::*;
use crossterm::{
    cursor, execute,
    style::{self, Stylize},
    terminal,
    terminal::size,
    QueueableCommand, Result,
};
use std::io::{stdout, Write};

fn main() -> std::result::Result<(), TerminalError> {
    let input = clap::builder::Command::new("chippers")
        .args(&[clap::arg!(<FILE> "chip-8 rom file")])
        .get_matches();
    terminal::enable_raw_mode().unwrap();
    let mut chip8 = Chip8::new();
    load_font_set(&mut chip8);
    let file = std::fs::read(&input.get_one::<String>("FILE").unwrap()).unwrap();
    let file = file.as_slice();
    for (i, byte) in file.iter().enumerate() {
        chip8.cpu.mem[i + 0x200] = *byte;
    }
    chip8.run()?;
    Ok(())
}

const FONT_SET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn load_font_set(c8: &mut Chip8) {
    for (i, byte) in FONT_SET.iter().enumerate() {
        c8.cpu.mem[i + 0x50] = *byte;
    }
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
                Chip8Message::None => {}
                Chip8Message::ClearScreen => self.term.clear_screen()?,
                Chip8Message::DrawScreen => self.term.draw_screen(&self.cpu.disp)?,
            }
            if self.cpu.dt > 0 {
                self.cpu.dt -= 1;
            }
            if self.cpu.st > 0 {
                self.cpu.st -= 1;
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

const CLOCK_RATE: f64 = 100.; // Hz, 700 instructions per second

#[derive(Debug)]
struct Clock;

impl Clock {
    fn tick(&self) {
        std::thread::sleep(std::time::Duration::from_millis(
            (1. / CLOCK_RATE * 1000.).round() as u64,
        ));
    }
}

#[derive(Debug)]
struct Terminal;

impl Terminal {
    const MIN_WIDTH: u16 = 64;
    const MIN_HEIGHT: u16 = 32;

    fn check_bounds(&self, w: u16, h: u16) -> std::result::Result<(), TerminalError> {
        if w < Self::MIN_WIDTH || h < Self::MIN_HEIGHT {
            return Err(TerminalError::ErrorKind(format!(
                "terminal is too small to display screen: {}x{}",
                w, h
            )));
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

impl std::error::Error for TerminalError {}

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
                    return Err(TerminalError::ErrorKind(
                        "display pixel set to value other than 0 or 1".to_string(),
                    ));
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
type StackPointer = usize;
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
    sp: StackPointer,
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
        let sp = 0;
        let dt = 0;
        let st = 0;
        let reg = [0u8; 16];
        let pc = START;

        Self {
            mem,
            disp,
            index,
            stack,
            sp,
            dt,
            st,
            reg,
            pc,
        }
    }

    pub fn fetch_next(&mut self) -> u16 {
        let next_inst = ((self.mem[self.pc as usize] as u16) << 8) as u16
            + (self.mem[self.pc as usize + 1]) as u16;
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
            Opcode::None => Chip8Message::None,
            Opcode::Error => {
                println!("cpu status: {:?}", self);
                panic!()
            }
            Opcode::Clear => Chip8Message::ClearScreen,
            Opcode::Jump => {
                self.jump(nnn);
                Chip8Message::None
            }
            Opcode::ReturnSub => {
                self.return_sub();
                Chip8Message::None
            }
            Opcode::GotoSub => {
                self.goto_sub(nnn);
                Chip8Message::None
            }
            Opcode::SkipEqual => {
                self.skip_equal(x, kk);
                Chip8Message::None
            }
            Opcode::SkipNotEqual => {
                self.skip_not_equal(x, kk);
                Chip8Message::None
            }
            Opcode::SkipVXEqualVY => {
                self.skip_vx_equal_vy(x, y);
                Chip8Message::None
            }
            Opcode::SkipVXNotEqualVY => {
                self.skip_vx_not_equal_vy(x, y);
                Chip8Message::None
            }
            Opcode::SkipIfKey => {
                self.skip_if_key(x);
                Chip8Message::None
            }
            Opcode::SkipIfNotKey => {
                self.skip_if_not_key(x);
                Chip8Message::None
            }
            Opcode::GetKey => {
                self.get_key(x);
                Chip8Message::None
            }
            Opcode::SetVX => {
                self.set_vx(x, kk);
                Chip8Message::None
            }
            Opcode::AddVX => {
                self.add_vx(x, kk);
                Chip8Message::None
            }
            Opcode::SetI => {
                self.set_i(nnn);
                Chip8Message::None
            }
            Opcode::AddI => {
                self.add_i(x);
                Chip8Message::None
            }
            Opcode::JumpWithOffset => {
                self.jump_with_offset(nnn);
                Chip8Message::None
            }
            Opcode::Random => {
                self.random(x, kk);
                Chip8Message::None
            }
            Opcode::FontCharacter => {
                self.font_character(x);
                Chip8Message::None
            }
            Opcode::Draw => {
                self.draw(x, y, n);
                Chip8Message::DrawScreen
            }
            Opcode::SetVXToVY => {
                self.set_vx_to_vy(x, y);
                Chip8Message::None
            }
            Opcode::BinaryOr => {
                self.binary_or(x, y);
                Chip8Message::None
            }
            Opcode::BinaryAnd => {
                self.binary_and(x, y);
                Chip8Message::None
            }
            Opcode::BinaryXor => {
                self.binary_xor(x, y);
                Chip8Message::None
            }
            Opcode::AddVYToVX => {
                self.add_vy_to_vx(x, y);
                Chip8Message::None
            }
            Opcode::SubVYFromVX => {
                self.sub_vy_from_vx(x, y);
                Chip8Message::None
            }
            Opcode::SubVXFromVY => {
                self.sub_vx_from_vy(x, y);
                Chip8Message::None
            }
            Opcode::ShiftRight => {
                self.shift_right(x, y);
                Chip8Message::None
            }
            Opcode::ShiftLeft => {
                self.shift_left(x, y);
                Chip8Message::None
            }
            Opcode::BinaryCodedDecimalConversion => {
                self.binary_coded_decimal_conversion(x);
                Chip8Message::None
            }
            Opcode::SetVXToDT => {
                self.set_vx_to_dt(x);
                Chip8Message::None
            }
            Opcode::SetDTToVX => {
                self.set_dt_to_vx(x);
                Chip8Message::None
            }
            Opcode::SetSTToVX => {
                self.set_st_to_vx(x);
                Chip8Message::None
            }
            Opcode::SaveRegisterToMemory => {
                self.save_register_to_memory(x);
                Chip8Message::None
            }
            Opcode::LoadRegisterFromMemory => {
                self.load_register_from_memory(x);
                Chip8Message::None
            }
            _ => unimplemented!(),
        }
    }

    fn jump(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn return_sub(&mut self) {
        let mut stop = 0;
        for (i, sr) in self.stack.iter().enumerate().rev() {
            if *sr != 0 {
                self.pc = *sr;
                stop = i;
            }
        }
        self.stack[stop] = 0;
    }

    fn goto_sub(&mut self, nnn: u16) {
        let mut stop = 0;
        for (i, sr) in self.stack.iter().enumerate() {
            if *sr == 0 {
                stop = i;
            }
        }
        self.stack[stop] = self.pc;
        self.pc = nnn;
    }

    fn skip_equal(&mut self, x: u16, nn: u16) {
        if self.reg[x as usize] == nn as u8 {
            self.pc += 2;
        }
    }

    fn skip_not_equal(&mut self, x: u16, nn: u16) {
        if self.reg[x as usize] != nn as u8 {
            self.pc += 2;
        }
    }

    fn skip_vx_equal_vy(&mut self, x: u16, y: u16) {
        if self.reg[x as usize] == self.reg[y as usize] {
            self.pc += 2;
        }
    }

    fn skip_vx_not_equal_vy(&mut self, x: u16, y: u16) {
        if self.reg[x as usize] != self.reg[y as usize] {
            self.pc += 2;
        }
    }

    fn skip_if_key(&mut self, x: u16) {
        let key = self.reg[x as usize];
        let keystate = crossterm::event::poll(std::time::Duration::from_secs(0)).unwrap();
        if keystate {
            let keypress = match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(crossterm::event::KeyEvent { code, .. }) => code,
                _ => crossterm::event::KeyCode::Null,
            };
            let keypress: Option<u8> = match keypress {
                crossterm::event::KeyCode::Char(c) => match c {
                    '1' => Some(1),
                    '2' => Some(2),
                    '3' => Some(3),
                    '4' => Some(0xC),
                    'q' => Some(4),
                    'w' => Some(5),
                    'e' => Some(6),
                    'r' => Some(0xD),
                    'a' => Some(7),
                    's' => Some(8),
                    'd' => Some(9),
                    'f' => Some(0xF),
                    'z' => Some(0xA),
                    'x' => Some(0),
                    'c' => Some(0xB),
                    'v' => Some(0xF),
                    _ => None,
                },
                crossterm::event::KeyCode::Null | _ => None,
            };
            if let Some(k) = keypress {
                if k == key {
                    self.pc += 2;
                }
            }
        }
    }

    fn skip_if_not_key(&mut self, x: u16) {
        let key = self.reg[x as usize];
        let keystate = crossterm::event::poll(std::time::Duration::from_secs(0)).unwrap();
        if keystate {
            let keypress = match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(crossterm::event::KeyEvent { code, .. }) => code,
                _ => crossterm::event::KeyCode::Null,
            };
            let keypress: Option<u8> = match keypress {
                crossterm::event::KeyCode::Char(c) => match c {
                    '1' => Some(1),
                    '2' => Some(2),
                    '3' => Some(3),
                    '4' => Some(0xC),
                    'q' => Some(4),
                    'w' => Some(5),
                    'e' => Some(6),
                    'r' => Some(0xD),
                    'a' => Some(7),
                    's' => Some(8),
                    'd' => Some(9),
                    'f' => Some(0xF),
                    'z' => Some(0xA),
                    'x' => Some(0),
                    'c' => Some(0xB),
                    'v' => Some(0xF),
                    _ => None,
                },
                crossterm::event::KeyCode::Null | _ => None,
            };
            if let Some(k) = keypress {
                if k != key {
                    self.pc += 2;
                    return;
                }
            } else {
                self.pc += 2;
                return;
            }
        } else {
            self.pc += 2;
            return;
        }
    }

    fn get_key(&mut self, x: u16) {
        let ret = crossterm::event::poll(std::time::Duration::from_secs(0)).unwrap();
        if ret {
            let keypress = match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(crossterm::event::KeyEvent { code, .. }) => code,
                _ => crossterm::event::KeyCode::Null,
            };
            let keypress: Option<u8> = match keypress {
                crossterm::event::KeyCode::Char(c) => match c {
                    '1' => Some(1),
                    '2' => Some(2),
                    '3' => Some(3),
                    '4' => Some(0xC),
                    'q' => Some(4),
                    'w' => Some(5),
                    'e' => Some(6),
                    'r' => Some(0xD),
                    'a' => Some(7),
                    's' => Some(8),
                    'd' => Some(9),
                    'f' => Some(0xF),
                    'z' => Some(0xA),
                    'x' => Some(0),
                    'c' => Some(0xB),
                    'v' => Some(0xF),
                    _ => None,
                },
                crossterm::event::KeyCode::Null | _ => None,
            };
            if let Some(k) = keypress {
                self.reg[x as usize] = k;
            } else {
                self.pc -= 2;
                return;
            }
        } else {
            self.pc -= 2;
            return;
        }
    }

    fn set_vx(&mut self, x: u16, nn: u16) {
        self.reg[x as usize] = nn as u8;
    }

    fn add_vx(&mut self, x: u16, nn: u16) {
        let res = self.reg[x as usize] as u16 + nn;
        let res = res % 256;
        self.reg[x as usize] = res as u8;
    }

    fn set_i(&mut self, nnn: u16) {
        self.index = nnn;
    }

    fn add_i(&mut self, x: u16) {
        self.index += self.reg[x as usize] as u16;
        if self.index >= 0x1000 {
            self.reg[0xF] = 1;
        }
    }

    fn jump_with_offset(&mut self, nnn: u16) {
        self.pc = nnn + self.reg[0] as u16;
    }

    fn random(&mut self, x: u16, nn: u16) {
        use rand::prelude::*;

        let r = rand::random::<u8>();
        self.reg[x as usize] = r & nn as u8;
    }

    fn font_character(&mut self, x: u16) {
        let c = self.reg[x as usize];
        self.index = c as u16 * 5 + 0x50;
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
                    self.reg[0xF as usize] = 0;
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

    fn set_vx_to_vy(&mut self, x: u16, y: u16) {
        self.reg[x as usize] = self.reg[y as usize];
    }

    fn binary_or(&mut self, x: u16, y: u16) {
        self.reg[x as usize] = self.reg[x as usize] | self.reg[y as usize];
    }

    fn binary_and(&mut self, x: u16, y: u16) {
        self.reg[x as usize] = self.reg[x as usize] & self.reg[y as usize];
    }

    fn binary_xor(&mut self, x: u16, y: u16) {
        self.reg[x as usize] = self.reg[x as usize] ^ self.reg[y as usize];
    }

    fn add_vy_to_vx(&mut self, x: u16, y: u16) {
        let mut temp_res = self.reg[x as usize] as u16 + self.reg[y as usize] as u16;
        if temp_res > 255 {
            self.reg[0xF] = 1;
            temp_res = temp_res % 256;
        } else {
            self.reg[0xF] = 0;
        }
        self.reg[x as usize] = temp_res as u8;
    }

    fn sub_vy_from_vx(&mut self, x: u16, y: u16) {
        let mut minuend = self.reg[x as usize];
        let mut subtrahend = self.reg[y as usize];
        if minuend < subtrahend {
            self.reg[0xF] = 0;
            self.reg[x as usize] = (256 - (subtrahend - minuend) as u16) as u8;
        } else {
            self.reg[0xF] = 1;
            self.reg[x as usize] = minuend - subtrahend;
        }
    }

    fn sub_vx_from_vy(&mut self, x: u16, y: u16) {
        let mut minuend = self.reg[y as usize];
        let mut subtrahend = self.reg[x as usize];
        if minuend < subtrahend {
            self.reg[0xF] = 0;
            self.reg[x as usize] = (256 - (subtrahend - minuend) as u16) as u8;
        } else {
            self.reg[0xF] = 1;
            self.reg[x as usize] = minuend - subtrahend;
        }
    }

    fn shift_right(&mut self, x: u16, _y: u16) {
        let flag = self.reg[x as usize] & 0b0000_0001;
        self.reg[x as usize] = self.reg[x as usize] >> 1;
        self.reg[0xF] = flag;
    }

    fn shift_left(&mut self, x: u16, _y: u16) {
        let flag = self.reg[x as usize] & 0b1000_0000;
        self.reg[x as usize] = self.reg[x as usize] << 1;
        self.reg[0xF] = flag;
    }

    fn binary_coded_decimal_conversion(&mut self, x: u16) {
        let n = self.reg[x as usize];
        let hundreds = n / 100;
        let tens = n / 10;
        let ones = n % 10;
        self.mem[self.index as usize] = hundreds;
        self.mem[self.index as usize + 1] = tens;
        self.mem[self.index as usize + 2] = ones;
    }

    fn set_vx_to_dt(&mut self, x: u16) {
        self.reg[x as usize] = self.dt;
    }

    fn set_dt_to_vx(&mut self, x: u16) {
        self.dt = self.reg[x as usize];
    }

    fn set_st_to_vx(&mut self, x: u16) {
        self.st = self.reg[x as usize];
    }

    fn save_register_to_memory(&mut self, x: u16) {
        for i in 0..=x {
            self.mem[self.index as usize + i as usize] = self.reg[i as usize];
        }
    }

    fn load_register_from_memory(&mut self, x: u16) {
        for i in 0..=x {
            self.reg[i as usize] = self.mem[self.index as usize + i as usize];
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
        RawOpcode {
            op,
            x,
            y,
            n,
            kk,
            nnn,
        }
    }
}

impl std::fmt::Display for RawOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:x}{:x}{:x}{:x}", self.op, self.x, self.y, self.n)?;
        Ok(())
    }
}

enum Opcode {
    Clear,                        // 00E0
    Jump,                         // 1NNN
    ReturnSub,                    // 00EE
    GotoSub,                      // 2NNN
    SkipEqual,                    // 3XNN
    SkipNotEqual,                 // 4XNN
    SkipVXEqualVY,                // 5XY0
    SkipVXNotEqualVY,             // 9XY0
    SkipIfKey,                    // EX9E
    SkipIfNotKey,                 // EXA1
    GetKey,                       // FX0A
    SetVX,                        // 6XNN
    AddVX,                        // 7XNN, does not effect carry flag
    SetI,                         // ANNN
    AddI,                         // FX1E
    JumpWithOffset, // BNNN, should add configuration for BXNN behavior for CHIP-48 programs
    Random,         // CXNN, generate random, AND with NN, put in VX
    Draw,           // DXYN
    FontCharacter,  // FX29
    SetVXToVY,      // 8XY0
    BinaryOr,       // 8XY1
    BinaryAnd,      // 8XY2
    BinaryXor,      // 8XY3
    AddVYToVX,      // 8XY4, does effect carry flag
    SubVYFromVX,    // 8XY5, put result in VX
    SubVXFromVY,    // 8XY7, put result in VX
    ShiftRight,     // 8XY6, ignore VY in modern implementation
    ShiftLeft,      // 8XYE, ignore VY in modern implementation,
    BinaryCodedDecimalConversion, // FX33
    SetVXToDT,      // FX07
    SetDTToVX,      // FX15
    SetSTToVX,      // FX18
    SaveRegisterToMemory, // FX55
    LoadRegisterFromMemory, // FX65
    None,           // other
    Error,          // error
}

impl std::convert::From<&RawOpcode> for Opcode {
    fn from(raw_op: &RawOpcode) -> Opcode {
        // we should be able to implement this as a series of matches
        match raw_op.op {
            0 => {
                if raw_op.x == 0 && raw_op.y == 0xE && raw_op.n == 0 {
                    return Opcode::Clear;
                } else if raw_op.x == 0 && raw_op.y == 0xE && raw_op.n == 0xE {
                    return Opcode::ReturnSub;
                } else {
                    return Opcode::None;
                }
            }
            1 => return Opcode::Jump,
            2 => return Opcode::GotoSub,
            3 => return Opcode::SkipEqual,
            4 => return Opcode::SkipNotEqual,
            5 => return Opcode::SkipVXEqualVY,
            6 => return Opcode::SetVX,
            7 => return Opcode::AddVX,
            8 => match raw_op.n {
                0 => return Opcode::SetVXToVY,
                1 => return Opcode::BinaryOr,
                2 => return Opcode::BinaryAnd,
                3 => return Opcode::BinaryXor,
                4 => return Opcode::AddVYToVX,
                5 => return Opcode::SubVXFromVY,
                6 => return Opcode::ShiftRight,
                7 => return Opcode::SubVYFromVX,
                0xE => return Opcode::ShiftLeft,
                _ => unreachable!(),
            },
            9 => return Opcode::SkipVXNotEqualVY,
            0xA => return Opcode::SetI,
            0xB => return Opcode::JumpWithOffset,
            0xC => return Opcode::Random,
            0xD => return Opcode::Draw,
            0xE => match raw_op.kk {
                0x9E => return Opcode::SkipIfKey,
                0xA1 => return Opcode::SkipIfNotKey,
                _ => unreachable!(),
            },
            0xF => match raw_op.kk {
                7 => return Opcode::SetVXToDT,
                0x15 => return Opcode::SetDTToVX,
                0x18 => return Opcode::SetSTToVX,
                0x1E => return Opcode::AddI,
                0x0A => return Opcode::GetKey,
                0x29 => return Opcode::FontCharacter,
                0x33 => return Opcode::BinaryCodedDecimalConversion,
                0x55 => return Opcode::SaveRegisterToMemory,
                0x65 => return Opcode::LoadRegisterFromMemory,
                _ => unreachable!(),
            },
            _ => {
                println!("\nencountered unknown opcode: {}", raw_op);
                return Opcode::Error;
            }
        }
    }
}
