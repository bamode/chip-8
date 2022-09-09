#![allow(unused)]
fn main() {
    // nibble extraction testing
    // test with 1111_1110_1100_1000
    // want to discard 1111
    // nnn or addr = 1110_1100_1000
    // n or nibble = 1000
    // x           = 1110
    // y           = 1100
    // kk or byte  = 1100_1000
    let t = 0b1111_1110_1100_1000;
    println!("test: {:b}", t);
    println!("opcode: {:b}", t >> 12);
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

    pub fn execute_instruction(&mut self, inst: u16) {
        let op = inst >> 12;
        let nnn = inst & 0b0000_1111_1111_1111;
        let n = inst & 0b0000_0000_0000_1111;
        let x = (inst & 0b0000_1111_0000_0000) >> 8;
        let y = (inst & 0b0000_0000_1111_0000) >> 4;       
        let kk = inst & 0b0000_0000_1111_1111;
        let raw_op = RawOpcode::new(op, x, y, n, kk, nnn);
        let opcode = Opcode::from(&raw_op);
        match opcode {
            Opcode::Clear => { self.clear() },
            Opcode::Jump => { self.jump(nnn) },
            Opcode::SetVX => { },
            Opcode::AddVX => { },
            Opcode::SetI => { },
            Opcode::Draw => { },
        }
    }

    fn clear(&mut self) {
        // I plan to use crossterm for this
        unimplemented!();
    }

    fn jump(&mut self, nnn: u16) {
        self.pc == nnn;
    }
}

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

enum Opcode {
    Clear, // 00E0
    Jump, // 1NNN
    SetVX, // 6XNN
    AddVX, // 7XNN
    SetI, // ANNN
    Draw, // DXYN
}

impl std::convert::From<&RawOpcode> for Opcode {
    fn from(raw_op: &RawOpcode) -> Opcode {
        // we should be able to implement this as a series of matches
        match raw_op.op {
            0 => { 
                if raw_op.x == 0 && raw_op.y == 0xE && raw_op.n == 0 {
                    return Opcode::Clear 
                } else {
                    panic!()
                }
            },
            1 => { return Opcode::Jump },
            6 => { return Opcode::SetVX },
            7 => { return Opcode::AddVX },
            0xA => { return Opcode::SetI },
            0xD => { return Opcode::Draw },
            _ => { unimplemented!() },
        }
    }
}