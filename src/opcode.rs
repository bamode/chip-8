#[derive(Debug)]
pub struct RawOpcode {
    op: u16,
    x: u16,
    y: u16,
    n: u16,
    kk: u16,
}

impl RawOpcode {
    pub fn new(op: u16, x: u16, y: u16, n: u16, kk: u16) -> Self {
        RawOpcode {
            op,
            x,
            y,
            n,
            kk,
        }
    }
}

impl std::fmt::Display for RawOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:x}{:x}{:x}{:x}", self.op, self.x, self.y, self.n)?;
        Ok(())
    }
}

pub enum Opcode {
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
