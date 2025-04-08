use crate::{memory::Address, registers::Register};

#[derive(Debug)]
pub enum Error {
    InvalidOpCode(u8),
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

    MovMemMem, 8,
    MovMemReg, 5,

    MovMemU8, 5,
    MovMemU16, 6,
    MovMemU32, 8,

    AddRegReg, 2,
    AddRegMem, 5,

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

    Jump,      4,
    Call,      4,

    Load, 4,

    Halt,      0,
    Ret,       0,

    Interrupt, 1,
    InterruptReg, 1,

    StoreReg, 2,

    StoreU8, 5,
    StoreU16, 6,
    StoreU32, 8,
}

#[derive(Debug, Clone, Copy)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    MovRegMem(Register, Address),
    MovRegReg(Register, Register),
    MovRegNum(Register, Value),

    MovMemMem(Address, Address),
    MovMemReg(Address, Register),
    MovMemVal(Address, Value),

    AddRegReg(Register, Register),
    AddRegNum(Register, Value),
    AddRegMem(Register, Address),

    IncReg(Register),
    IncMem(Address),

    PushReg(Register),
    PushMem(Address),
    PushVal(Value),

    PopReg(Register),

    Jump(Address),
    Call(Address),

    Load(Register, Address),

    StoreReg(Address, Register),
    StoreVal(Address, Value),

    Interrupt(u8),
    InterruptReg(Register),

    Halt,
    Ret,
}

impl TryFrom<&[u8]> for Instruction {
    type Error = crate::error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let opcode = OpCode::try_from(value[0])?;

        println!("converting {opcode:?} into instruction");

        Ok(match opcode {
            OpCode::PushReg => {
                let arg = Register::try_from(value[1])?;
                Instruction::PushReg(arg)
            }
            OpCode::MovRegReg => {
                let left = Register::try_from(value[1])?;

                let right = Register::try_from(value[2])?;

                Instruction::MovRegReg(left, right)
            }
            OpCode::MovRegU8 => {
                let left = Register::try_from(value[0])
                    .unwrap_or_else(|_| panic!("failed to convert {} into register", value[1]));

                let right = value[2];

                Instruction::MovRegNum(left, Value::U8(right))
            }

            OpCode::MovRegU16 => {
                let left = Register::try_from(value[0])
                    .unwrap_or_else(|_| panic!("failed to convert {} into register", value[1]));

                let right = u16::from_le_bytes([value[2], value[3]]);

                Instruction::MovRegNum(left, Value::U16(right))
            }

            OpCode::MovRegU32 => {
                let left = Register::try_from(value[0])
                    .unwrap_or_else(|_| panic!("failed to convert {} into register", value[1]));

                let right = u32::from_le_bytes([value[2], value[3], value[4], value[5]]);

                Instruction::MovRegNum(left, Value::U32(right))
            }

            OpCode::AddRegReg => {
                let left = Register::try_from(value[1])?;
                let right = Register::try_from(value[2])?;

                Instruction::AddRegReg(left, right)
            }
            OpCode::AddRegU8 => {
                let left = Register::try_from(value[1])
                    .unwrap_or_else(|_| panic!("failed to convert {} into register", value[1]));

                let right = value[2];

                Instruction::AddRegNum(left, Value::U8(right))
            }

            OpCode::AddRegU16 => {
                let left = Register::try_from(value[1])
                    .unwrap_or_else(|_| panic!("failed to convert {} into register", value[1]));

                let right = u16::from_le_bytes([value[2], value[3]]);

                Instruction::AddRegNum(left, Value::U16(right))
            }

            OpCode::AddRegU32 => {
                let left = Register::try_from(value[1])
                    .unwrap_or_else(|_| panic!("failed to convert {} into register", value[1]));

                let right = u32::from_le_bytes([value[2], value[3], value[4], value[5]]);

                Instruction::AddRegNum(left, Value::U32(right))
            }

            OpCode::PopReg => {
                let reg = Register::try_from(value[1])?;

                Instruction::PopReg(reg)
            }
            OpCode::Jump => Instruction::Jump(value[1].into()),
            OpCode::Call => Instruction::Call(value[1].into()),
            OpCode::Halt => Instruction::Halt,
            OpCode::Ret => Instruction::Ret,
            OpCode::Load => {
                let reg = Register::try_from(value[1])?;
                let addr = u32::from_le_bytes([value[2], value[3], value[4], value[5]]);
                Instruction::Load(reg, addr.into())
            }
            OpCode::MovRegMem => {
                let reg = Register::try_from(value[1])?;
                let addr = u32::from_le_bytes([value[2], value[3], value[4], value[5]]);

                Instruction::MovRegMem(reg, addr.into())
            }
            OpCode::MovMemMem => {
                let l_addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);
                let r_addr = u32::from_le_bytes([value[5], value[6], value[7], value[8]]);

                Instruction::MovMemMem(Address::from(l_addr), r_addr.into())
            }
            OpCode::MovMemReg => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);
                let reg = Register::try_from(value[5])?;

                Instruction::MovMemReg(addr.into(), reg)
            }
            OpCode::AddRegMem => {
                let reg = Register::try_from(value[1])?;
                let addr = u32::from_le_bytes([value[2], value[3], value[4], value[5]]);

                Instruction::AddRegMem(reg, addr.into())
            }
            OpCode::IncReg => {
                let reg = Register::try_from(value[1])?;

                Instruction::IncReg(reg)
            }
            OpCode::IncMem => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);

                Instruction::IncMem(addr.into())
            }
            OpCode::PushMem => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);

                Instruction::PushMem(addr.into())
            }
            OpCode::Interrupt => Instruction::Interrupt(value[1]),
            OpCode::InterruptReg => Instruction::InterruptReg(Register::try_from(value[1])?),
            OpCode::StoreReg => {
                let reg = Register::try_from(value[1])?;
                let addr = u32::from_le_bytes([value[2], value[3], value[4], value[5]]);

                Instruction::StoreReg(addr.into(), reg)
            }
            OpCode::MovMemU8 => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);

                Instruction::MovMemVal(addr.into(), Value::U8(value[5]))
            }
            OpCode::MovMemU16 => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);
                let val = u16::from_le_bytes([value[5], value[6]]);

                Instruction::MovMemVal(addr.into(), Value::U16(val))
            }
            OpCode::MovMemU32 => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);
                let val = u32::from_le_bytes([value[5], value[6], value[7], value[8]]);

                Instruction::MovMemVal(addr.into(), Value::U32(val))
            }
            OpCode::PushU8 => {
                let val = Value::U8(value[1]);

                Instruction::PushVal(val)
            }
            OpCode::PushU16 => {
                let val = u16::from_le_bytes([value[1], value[2]]);
                let val = Value::U16(val);

                Instruction::PushVal(val)
            }
            OpCode::PushU32 => {
                let val = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);
                let val = Value::U32(val);

                Instruction::PushVal(val)
            }
            OpCode::StoreU8 => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);
                let val = Value::U8(value[5]);

                Instruction::StoreVal(addr.into(), val)
            }
            OpCode::StoreU16 => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);

                let val = u16::from_le_bytes([value[5], value[6]]);
                let val = Value::U16(val);

                Instruction::StoreVal(addr.into(), val)
            }
            OpCode::StoreU32 => {
                let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);

                let val = u32::from_le_bytes([value[5], value[6], value[7], value[8]]);
                let val = Value::U32(val);

                Instruction::StoreVal(addr.into(), val)
            }
        })

        // Ok(match opcode {
        //     OpCode::MovRegMem => {
        //         let addr = u32::from_le_bytes([value[2], value[3], value[4], value[5]]);
        //         Instruction::MovRegMem(Register::try_from(value[1])?, Address::from(addr))
        //     }
        //     OpCode::MovRegReg => {
        //         let r1 = Register::try_from(value[1])?;
        //         let r2 = Register::try_from(value[1])?;
        //
        //         Instruction::MovRegReg(r1, r2)
        //     }
        //     OpCode::MovRegNum => {},
        //     OpCode::MovMemMem => {
        //         let l_addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);
        //         let r_addr = u32::from_le_bytes([value[5], value[6], value[7], value[8]]);
        //
        //         Instruction::MovMemMem(Address::from(l_addr), Address::from(r_addr))
        //     }
        //     OpCode::MovMemReg => {
        //         let addr = u32::from_le_bytes([value[1], value[2], value[3], value[4]]);
        //         let reg = Register::try_from(value[5])?;
        //
        //         Instruction::MovMemReg(Address::from(addr), reg)
        //     }
        //     OpCode::MovMemU8 => {},
        //     OpCode::MovMemU16 => {},
        //     OpCode::MovMemU32 => {},
        //     OpCode::AddRegReg => {},
        //     OpCode::AddRegNum => {},
        //     OpCode::AddRegMem => {},
        //     OpCode::AddRegU8 => {},
        //     OpCode::AddRegU16 => {},
        //     OpCode::AddRegU32 => {},
        //     OpCode::IncReg => Instruction::IncReg(Register::try_from(value[1])?),
        //     OpCode::IncMem => {},
        //     OpCode::PushReg => {},
        //     OpCode::PushMem => {},
        //     OpCode::PushU8 => {},
        //     OpCode::PushU16 => {},
        //     OpCode::PushU32 => {},
        //     OpCode::PopReg => {},
        //     OpCode::Jump => {},
        //     OpCode::Call => {},
        //     OpCode::Load => {},
        //     OpCode::Halt => Instruction::Halt,
        //     OpCode::Ret => Instruction::Ret,
        //     OpCode::Interrupt => {},
        //     OpCode::InterruptReg => {},
        //     OpCode::StoreReg => {},
        //     OpCode::StoreU8 => {},
        //     OpCode::StoreU16 => {},
        //     OpCode::StoreU32 => {},
        // })
    }
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

            MovMemMem(_, _) => OpCode::MovMemMem,
            MovMemReg(_, _) => OpCode::MovMemReg,

            MovMemVal(_, val) => match val {
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

            AddRegNum(_, val) => match val {
                Value::U8(_) => OpCode::AddRegU8,
                Value::U16(_) => OpCode::AddRegU16,
                Value::U32(_) => OpCode::AddRegU32,
            },

            Interrupt(_) => OpCode::Interrupt,
            InterruptReg(_) => OpCode::InterruptReg,

            Jump(_) => OpCode::Jump,
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
