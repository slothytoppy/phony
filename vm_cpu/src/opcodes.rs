use std::fmt::Display;

use crate::{address::Address, registers::Register};

#[derive(Debug)]
pub enum Error {
    InvalidOpCode(u16),
}

macro_rules! op_codes {
    ($($variant:ident = $value:expr, $amount:ident = $arg_amount:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
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
            type Error = Error;

            fn try_from(value: u16) -> Result<OpCode, Error> {
                match value {
                    $(x if x == $value => Ok(OpCode::$variant),)*
                    _v => Err(Error::InvalidOpCode(value)),
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
    MovRegReg = 1,  amount = 3,
    MovRegVal = 2,  amount = 3,
    AddRegReg  = 3,  amount = 3,
    AddRegNum  = 4,  amount = 3,
    Jump       = 5,  amount = 2,
    PopReg     = 6,  amount = 2,
    Call       = 7,  amount = 1,
    CallAddr   = 8, amount = 2,
    Halt       = 9,  amount = 1,
    Ret        = 10,  amount = 1,
    Load       = 11,  amount = 3,
    PushReg = 12, amount = 2,
    PushVal = 13, amount = 2,
}

impl From<Instruction> for OpCode {
    fn from(value: Instruction) -> Self {
        match value {
            Instruction::MovRegReg(_, _) => Self::MovRegReg,
            Instruction::MovRegVal(_, _) => Self::MovRegVal,
            Instruction::PopReg(_) => Self::PopReg,
            Instruction::AddRegReg(_, _) => Self::AddRegReg,
            Instruction::AddRegVal(_, _) => Self::AddRegNum,
            Instruction::Call => Self::Call,
            Instruction::CallAddr(_) => Self::CallAddr,
            Instruction::Jump(_) => Self::Jump,
            Instruction::Halt => Self::Halt,
            Instruction::Ret => Self::Ret,
            Instruction::Load(_, _) => Self::Load,
            Instruction::PushReg(_) => Self::PushReg,
            Instruction::PushVal(_) => Self::PushVal,
        }
    }
}

impl From<&Instruction> for OpCode {
    fn from(value: &Instruction) -> Self {
        OpCode::from(*value)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    MovRegReg(Register, Register),
    MovRegVal(Register, u16),
    PushReg(Register),
    PushVal(u16),
    PopReg(Register),
    AddRegReg(Register, Register),
    AddRegVal(Register, u16),
    Jump(Address),
    Load(Register, Address),
    Call, // reserved: for future implementation of a syscall like feature
    CallAddr(Address),
    Halt,
    Ret,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::MovRegReg(register, register1) => {
                write!(f, "Pushing({register1} into {register})")
            }
            Instruction::MovRegVal(register, val) => {
                write!(f, "Pushing({val} into {register})")
            }
            Instruction::PopReg(register) => write!(f, "Popping {register}"),
            Instruction::AddRegReg(register, register1) => {
                write!(f, "Adding({register} and {register1})")
            }
            Instruction::AddRegVal(register, val) => {
                write!(f, "Adding({val} to {register})")
            }
            Instruction::Call => write!(f, "Calling"),
            Instruction::CallAddr(address) => write!(f, "Calling {address:?}"),
            Instruction::Jump(addr) => write!(f, "Jumping to address {addr:#02x?}"),
            Instruction::Halt => f.write_str("Halt"),
            Instruction::Ret => f.write_str("Ret"),
            Instruction::Load(reg, addr) => write!(f, "loading address: {addr:?} into {reg}"),
            Instruction::PushReg(reg) => write!(f, "pushing {reg} onto the stack"),
            Instruction::PushVal(val) => write!(f, "pushing {val} onto the stack"),
        }
    }
}
