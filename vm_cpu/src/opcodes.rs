use std::fmt::Display;

use crate::registers::Register;

#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    PushRegReg,
    PushRegVal,
    PopReg,
    AddRegReg,
    AddRegNum,
    Jump,
    Call,
    Halt,
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    PushRegReg(Register, Register),
    PushRegVal(Register, usize),
    PopReg(Register),
    AddRegReg(Register, Register),
    AddRegNum(Register, usize),
    Call,
    Jump(usize),
    Halt,
}

impl From<Instruction> for OpCode {
    fn from(value: Instruction) -> Self {
        match value {
            Instruction::PushRegReg(_, _) => Self::PushRegReg,
            Instruction::PushRegVal(_, _) => Self::PushRegVal,
            Instruction::PopReg(_) => Self::PopReg,
            Instruction::AddRegReg(_, _) => Self::AddRegReg,
            Instruction::AddRegNum(_, _) => Self::AddRegNum,
            Instruction::Call => Self::Call,
            Instruction::Jump(_) => Self::Jump,
            Instruction::Halt => Self::Halt,
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::PushRegReg(register, register1) => {
                f.write_str(&format!("Pushing({register1} into {register})"))
            }
            Instruction::PushRegVal(register, val) => {
                f.write_str(&format!("Pushing({val} into {register})"))
            }
            Instruction::PopReg(register) => f.write_str(&format!("Popping {register}")),
            Instruction::AddRegReg(register, register1) => {
                f.write_str(&format!("Adding({register} and {register1})"))
            }
            Instruction::AddRegNum(register, val) => {
                f.write_str(&format!("Adding({val} to {register})"))
            }
            Instruction::Call => f.write_str("Call"),
            Instruction::Jump(addr) => f.write_str(&format!("Jumping to address {addr:#02x}")),
            Instruction::Halt => f.write_str("Halt"),
        }
    }
}

impl From<OpCode> for u16 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::PushRegReg => 0,
            OpCode::PushRegVal => 1,
            OpCode::PopReg => 2,
            OpCode::AddRegReg => 3,
            OpCode::AddRegNum => 4,
            OpCode::Call => 5,
            OpCode::Jump => 6,
            OpCode::Halt => 7,
        }
    }
}

impl TryFrom<u16> for OpCode {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::PushRegReg),
            1 => Ok(OpCode::PushRegVal),
            2 => Ok(OpCode::PopReg),
            3 => Ok(OpCode::AddRegReg),
            4 => Ok(OpCode::AddRegNum),
            5 => Ok(OpCode::Call),
            6 => Ok(OpCode::Jump),
            7 => Ok(OpCode::Halt),
            _ => {
                println!("could not convert {value} into opcode");
                Err(())
            }
        }
    }
}
