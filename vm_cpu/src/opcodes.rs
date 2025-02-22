use std::fmt::Display;

use crate::registers::Register;

macro_rules! op_codes {
    ($($variant:ident = $value:expr, $amount:ident = $arg_amount:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy)]
        #[repr(u8)]
        #[rustfmt::skip]
        pub enum OpCode {
            $($variant = $value),*
        }

        impl OpCode {
            pub fn increment_amount(&self) -> u16 {
                match self {
                    $(OpCode::$variant => $arg_amount,)*
                }
            }
        }

        impl TryFrom<u16> for OpCode {
            type Error = ();

            fn try_from(value: u16) -> Result<OpCode, ()> {
                match value {
                    $(x if x == $value => Ok(OpCode::$variant),)*
                    _v => Err(()),
                }
            }
        }

        impl From<OpCode> for u16 {
            fn from(opcode: OpCode) -> Self  {
                opcode as u16
            }
        }

        impl std::fmt::Display for OpCode {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                $(Self::$variant { .. } => f.write_str(stringify!($variant))?,)*
            }

            write!(f, ": {self:?}")
            }
        }
    }
}

op_codes! {
    PushRegReg = 0,  amount = 3,
    PushRegVal = 1,  amount = 3,
    AddRegReg  = 3,  amount = 3,
    AddRegNum  = 4,  amount = 3,
    Jump       = 6,  amount = 2,
    PopReg     = 2,  amount =1,
    Call       = 5,  amount = 1,
    Halt       = 7,  amount = 1,
    Ret        = 8,  amount = 1,
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
            Instruction::Ret => Self::Ret,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    PushRegReg(Register, Register),
    PushRegVal(Register, u16),
    PopReg(Register),
    AddRegReg(Register, Register),
    AddRegNum(Register, u16),
    Jump(u16),
    Call,
    Halt,
    Ret,
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
            Instruction::Ret => f.write_str("Ret"),
        }
    }
}
