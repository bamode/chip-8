use crate::chip::Chip8Message;
use crate::opcode::*;
use bitvec::prelude::*;

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
pub struct Cpu {
    pub mem: Memory,
    pub disp: Display,
    index: I,
    stack: Stack,
    pub dt: DelayTimer,
    pub st: SoundTimer,
    reg: Register,
    pc: ProgramCounter,
}

pub const FONT_SET: [u8; 80] = [
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
            mem,
            disp,
            index,
            stack,
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
        let raw_op = RawOpcode::new(op, x, y, n, kk);
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
                break;
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
        let r = rand::random::<u8>();
        self.reg[x as usize] = r & (nn as u8);
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
        let minuend = self.reg[x as usize];
        let subtrahend = self.reg[y as usize];
        if minuend < subtrahend {
            self.reg[0xF] = 0;
            self.reg[x as usize] = (256 - (subtrahend - minuend) as u16) as u8;
        } else {
            self.reg[0xF] = 1;
            self.reg[x as usize] = minuend - subtrahend;
        }
    }

    fn sub_vx_from_vy(&mut self, x: u16, y: u16) {
        let minuend = self.reg[y as usize];
        let subtrahend = self.reg[x as usize];
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
        let flag = flag >> 7;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_jump() {
        let mut cpu = Cpu::new();
        cpu.execute_instruction(0x1234);
        assert_eq!(cpu.pc, 0x234);
    }

    #[test]
    fn test_return_sub() {
        let mut cpu = Cpu::new();
        cpu.stack[0] = 0x222;
        cpu.execute_instruction(0x00EE);
        assert_eq!(cpu.pc, 0x222);
    }

    #[test]
    fn test_goto_sub() {
        let mut cpu = Cpu::new();
        cpu.pc = 1;
        cpu.execute_instruction(0x2123);
        assert_eq!(cpu.pc, 0x123);
        assert_eq!(cpu.stack[0], 1);
    }

    #[test]
    fn test_skip_equal() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 0x01;
        cpu.execute_instruction(0x3001);
        assert_eq!(cpu.pc, 0x202);
    }
    
    #[test]
    fn test_skip_not_equal() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 0x02;
        cpu.execute_instruction(0x4001);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skip_vx_equal_vy() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 1;
        cpu.reg[1] = 1;
        cpu.execute_instruction(0x5010);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_set_vx() {
        let mut cpu = Cpu::new();
        cpu.execute_instruction(0x6012);
        assert_eq!(cpu.reg[0], 0x12);
    }

    #[test]
    fn test_add_without_carry() {
        let mut cpu = Cpu::new();
        cpu.reg[1] = 0x1;
        cpu.execute_instruction(0x7112);
        assert_eq!(cpu.reg[1], 0x13);
        assert_eq!(cpu.reg[0xF], 0x0);
    }

    #[test]
    fn test_set_vx_to_vy() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 1;
        cpu.reg[1] = 2;
        cpu.execute_instruction(0x8010);
        assert_eq!(cpu.reg[0], 2);
    }

    #[test]
    fn test_binary_or() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 0b010;
        cpu.reg[1] = 0b110;
        cpu.execute_instruction(0x8011);
        assert_eq!(cpu.reg[0], 0b110);
    }

    #[test]
    fn test_binary_and() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 0b011;
        cpu.reg[1] = 0b110;
        cpu.execute_instruction(0x8012);
        assert_eq!(cpu.reg[0], 0b010);
    }
    
    #[test]
    fn test_binary_xor() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 0b011;
        cpu.reg[1] = 0b111;
        cpu.execute_instruction(0x8013);
        assert_eq!(cpu.reg[0], 0b100);
    }

    #[test]
    fn test_add_with_carry() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 128;
        cpu.reg[1] = 128;
        cpu.execute_instruction(0x8014);
        assert_eq!(cpu.reg[0], 0);
        assert_eq!(cpu.reg[0xF], 1);

        let mut cpu = Cpu::new();
        cpu.reg[0] = 2;
        cpu.reg[1] = 3;
        cpu.execute_instruction(0x8014);
        assert_eq!(cpu.reg[0], 5);
        assert_eq!(cpu.reg[0xF], 0);
    }

    #[test]
    fn test_sub_vy_from_vx() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 4;
        cpu.reg[1] = 2;
        cpu.execute_instruction(0x8015);
        assert_eq!(cpu.reg[0], 2);
        assert_eq!(cpu.reg[0xF], 1);

        let mut cpu = Cpu::new();
        cpu.reg[0] = 2;
        cpu.reg[1] = 4;
        cpu.execute_instruction(0x8015);
        assert_eq!(cpu.reg[0], 254);
        assert_eq!(cpu.reg[0xF], 0);
    }

    #[test]
    fn test_shift_right() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 0xFF;
        cpu.execute_instruction(0x8016);
        assert_eq!(cpu.reg[0], 0x7F);
        assert_eq!(cpu.reg[0xF], 1);
    }

    #[test]
    fn test_shift_left() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 0xFF;
        cpu.execute_instruction(0x801E);
        assert_eq!(cpu.reg[0], 0xFE);
        assert_eq!(cpu.reg[0xF], 1);
    }

    #[test]
    fn test_skip_vx_not_equal_vy() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 1;
        cpu.execute_instruction(0x9010);
        assert_eq!(cpu.pc, 0x202);
        cpu.reg[1] = 1;
        cpu.execute_instruction(0x9010);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_set_index() {
        let mut cpu = Cpu::new();
        cpu.execute_instruction(0xA123);
        assert_eq!(cpu.index, 0x123);
    }

    #[test]
    fn test_jump_with_offset() {
        let mut cpu = Cpu::new();
        cpu.reg[0] = 1;
        cpu.execute_instruction(0xB123);
        assert_eq!(cpu.pc, 0x124)
    }

    #[test]
    fn test_draw() {
        let mut cpu = Cpu::new();
        cpu.mem[0] = 1;
    }
}
