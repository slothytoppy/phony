use crate::{memory::Address, registers::Register};

#[derive(Debug)]
pub enum Error {
    InvalidOpCode(u8),
    InvalidComparison(u8),
}

macro_rules! op_codes {
    ($($variant:ident, $amount:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[repr(u8)]
        #[rustfmt::skip]
        pub enum OpCode {
            $($variant),*
        }

        impl OpCode {
            pub fn increment_amount(&self) -> u8 {
                match self {
                    $(OpCode::$variant => $amount+1,)*
                }
            }
        }

        impl TryFrom<u8> for OpCode {
            type Error = Error;

            fn try_from(value: u8) -> Result<OpCode, Error> {
                match value {
                    $(x if x == OpCode::$variant as u8 => Ok(OpCode::$variant),)*
                    _v => Err(Error::InvalidOpCode(value)),
                }
            }
        }

        impl From<OpCode> for u8 {
            fn from(opcode: OpCode) -> Self  {
                opcode as u8
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
    };
}

op_codes! {
    MovRegMem, 5,
    MovRegReg, 2,
    MovRegU8, 2,
    MovRegU16, 3,
    MovRegU32, 5,
    MovMemReg, 5,
    MovMemU8, 5,
    MovMemU16, 6,
    MovMemU32, 8,

    AddRegReg, 2,
    AddRegMem, 5,
    AddMemReg, 5,
    AddRegU8, 2,
    AddRegU16, 3,
    AddRegU32, 5,

    IncReg, 1,
    IncMem, 4,

    PushReg,   1,
    PushMem,   4,
    PushU8, 1,
    PushU16, 2,
    PushU32, 4,

    PopReg,    1,

    CmpReg,    2,
    CmpU8,     2,
    CmpU16,    3,
    CmpU32,    5,

    Jump,      4,
    JumpGe,    4,
    JumpGte,    4,
    JumpLe,    4,
    JumpLte,    4,

    Call,      4,

    Load, 4,

    Halt,      0,
    Ret,       0,

    Interrupt, 4,
    InterruptReg, 5,

    StoreReg, 5,
    StoreU8, 5,
    StoreU16, 6,
    StoreU32, 8,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Comparison {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
}

impl TryFrom<u8> for Comparison {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Eq,
            1 => Self::Ne,
            2 => Self::Lt,
            3 => Self::Lte,
            4 => Self::Gt,
            5 => Self::Gte,

            _ => return Err(Error::InvalidComparison(value)),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    MovRegMem(Register, Address),
    MovRegReg(Register, Register),
    MovRegNum(Register, Value),

    MovMemReg(Address, Register),
    MovMemNum(Address, Value),

    AddRegReg(Register, Register),
    AddRegNum(Register, Value),
    AddRegMem(Register, Address),
    AddMemReg(Address, Register),

    IncReg(Register),
    IncMem(Address),

    PushReg(Register),
    PushMem(Address),
    PushVal(Value),

    PopReg(Register),

    CmpReg(Register, Register),
    CmpVal(Value, Value),

    Jump(Address),
    JumpGe(Address),
    JumpGte(Address),
    JumpLt(Address),
    JumpLte(Address),

    Call(Address),

    Load(Register, Address),

    StoreReg(Address, Register),
    StoreVal(Address, Value),

    Interrupt(u32),
    InterruptReg(Register),

    Halt,
    Ret,
}

impl From<Instruction> for OpCode {
    fn from(value: Instruction) -> Self {
        use Instruction::*;
        match value {
            MovRegReg(..) => OpCode::MovRegReg,
            MovRegMem(_, _) => OpCode::MovRegMem,

            MovRegNum(_, val) => match val {
                Value::U8(_) => OpCode::MovRegU8,
                Value::U16(_) => OpCode::MovRegU16,
                Value::U32(_) => OpCode::MovRegU32,
            },

            MovMemReg(_, _) => OpCode::MovMemReg,

            MovMemNum(_, val) => match val {
                Value::U8(_) => OpCode::MovMemU8,
                Value::U16(_) => OpCode::MovMemU16,
                Value::U32(_) => OpCode::MovMemU32,
            },

            PushReg(_) => OpCode::PushReg,
            PushMem(_) => OpCode::PushMem,

            PushVal(val) => match val {
                Value::U8(_) => OpCode::PushU8,
                Value::U16(_) => OpCode::PushU16,
                Value::U32(_) => OpCode::PushU32,
            },

            PopReg(_) => OpCode::PopReg,

            AddRegReg(_, _) => OpCode::AddRegReg,
            AddRegMem(_, _) => OpCode::AddRegMem,
            AddMemReg(_, _) => OpCode::AddMemReg,

            AddRegNum(_, val) => match val {
                Value::U8(_) => OpCode::AddRegU8,
                Value::U16(_) => OpCode::AddRegU16,
                Value::U32(_) => OpCode::AddRegU32,
            },

            Interrupt(_) => OpCode::Interrupt,
            InterruptReg(_) => OpCode::InterruptReg,

            CmpReg(_, _) => OpCode::CmpReg,
            CmpVal(val, _) => match val {
                Value::U8(_) => OpCode::CmpU8,
                Value::U16(_) => OpCode::CmpU16,
                Value::U32(_) => OpCode::CmpU32,
            },

            Jump(_) => OpCode::Jump,
            JumpGe(_) => OpCode::JumpGe,
            JumpGte(_) => OpCode::JumpGte,
            JumpLt(_) => OpCode::JumpLe,
            JumpLte(_) => OpCode::JumpLte,

            Call(_) => OpCode::Call,

            IncReg(_) => OpCode::IncReg,
            IncMem(_) => OpCode::IncMem,

            Halt => OpCode::Halt,
            Ret => OpCode::Ret,

            Load(_, _) => OpCode::Load,

            StoreReg(_, _) => OpCode::StoreReg,

            StoreVal(_, val) => match val {
                Value::U8(_) => OpCode::StoreU8,
                Value::U16(_) => OpCode::StoreU16,
                Value::U32(_) => OpCode::StoreU32,
            },
        }
    }
}

impl From<&Instruction> for OpCode {
    fn from(value: &Instruction) -> Self {
        OpCode::from(*value)
    }
}
