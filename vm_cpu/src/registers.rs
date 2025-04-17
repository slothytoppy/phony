use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

use tracing::trace;

#[derive(Debug)]
pub enum Error {
    InvalidRegister(u8),
    InvalidConversion(String),
}

/// Registers r1-r4 are nonvolatile, r5-r8 are volatile
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    IP,
    SP,
    FP,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

impl Register {
    pub const fn len() -> usize {
        11
    }
}

impl FromStr for Register {
    type Err = Error;

    // should it work for things like ip && IP but not iP or Ip?
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ip" => Ok(Self::IP),
            "sp" => Ok(Self::SP),
            "fp" => Ok(Self::FP),
            "r1" => Ok(Self::R1),
            "r2" => Ok(Self::R2),
            "r3" => Ok(Self::R3),
            "r4" => Ok(Self::R4),
            "r5" => Ok(Self::R5),
            "r6" => Ok(Self::R6),
            "r7" => Ok(Self::R7),
            "r8" => Ok(Self::R8),
            _ => Err(Self::Err::InvalidConversion(s.to_string())),
        }
    }
}

pub trait WordSize {
    type Output;

    fn upper(&self) -> Self::Output;
    fn lower(&self) -> Self::Output;
}

impl WordSize for u16 {
    type Output = u8;

    fn upper(&self) -> Self::Output {
        let bytes = self.to_le_bytes();

        Self::Output::from_le_bytes([bytes[1]])
    }

    fn lower(&self) -> Self::Output {
        let bytes = self.to_le_bytes();

        Self::Output::from_le_bytes([bytes[0]])
    }
}

impl WordSize for u32 {
    type Output = u16;

    fn upper(&self) -> Self::Output {
        let bytes = self.to_le_bytes();

        Self::Output::from_le_bytes([bytes[2], bytes[3]])
    }

    fn lower(&self) -> Self::Output {
        let bytes = self.to_le_bytes();

        Self::Output::from_le_bytes([bytes[0], bytes[1]])
    }
}

/// made in case i decide to increase bitsize of my vm
macro_rules! register_impl {
    ($($variant:ty),* $(,)?) => {

        impl TryFrom<$($variant)?> for Register{
            type Error = Error;

            fn try_from(value: $($variant)?) -> Result<Register, Error> {
                match value as u8 {
                    0 => Ok(Register::IP),
                    1 => Ok(Register::SP),
                    2 => Ok(Register::FP),
                    3 => Ok(Register::R1),
                    4 => Ok(Register::R2),
                    5 => Ok(Register::R3),
                    6 => Ok(Register::R4),
                    7 => Ok(Register::R5),
                    8 => Ok(Register::R6),
                    9 => Ok(Register::R7),
                    10 => Ok(Register::R8),
                    _ => Err(Error::InvalidRegister(value as u8))
                }
            }
        }

        impl TryFrom<&$($variant)?> for Register{
            type Error = Error;

            fn try_from(value: &$($variant)?) -> Result<Register, Error> {
                Register::try_from(*value)
            }
        }

        impl From<Register> for $($variant)?{
            fn from(value: Register) -> $($variant)?{
                value as $($variant)?
            }
        }
    };
}

register_impl!(u8);
register_impl!(u16);
register_impl!(u32);

#[derive(Debug)]
pub struct Registers([u32; Register::len()]);

impl Default for Registers {
    fn default() -> Self {
        let mut register = [0_u32; Register::len()];
        register[Register::SP as usize] = u32::MAX;
        Self(register)
    }
}

impl Registers {
    pub fn program_start(&mut self, start: u32) {
        self.0[Register::IP as usize] = start;
    }

    pub fn program_end(&mut self, end: u32) {
        self.0[Register::SP as usize] = end;
    }

    pub fn new(program_start: u32, stack_start: u32) -> Self {
        let mut registers = Self::default();
        registers.0[Register::IP as usize] = program_start;
        registers.0[Register::SP as usize] = stack_start;
        registers.0[Register::FP as usize] = stack_start - 2;

        registers
    }

    pub fn get(&self, register: Register) -> u32 {
        self[register]
    }

    pub fn set(&mut self, register: Register, val: u32) {
        self[register] = val;
    }

    pub fn as_slice(&self) -> &[u32; Register::len()] {
        &self.0
    }

    pub fn get_upper(&self, register: Register) -> u16 {
        let upper = self[register] << 16;

        (upper >> 16) as u16
    }

    pub fn get_lower(&self, register: Register) -> u16 {
        let lower = self[register] >> 16;

        lower as u16
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "IP {}", self.0[0])?;
        writeln!(f, "SP {}", self.0[1])?;
        writeln!(f, "R1 {}", self.0[2])?;
        writeln!(f, "R2 {}", self.0[3])?;
        writeln!(f, "R3 {}", self.0[4])?;
        writeln!(f, "R4 {}", self.0[5])?;
        writeln!(f, "R5 {}", self.0[6])?;
        writeln!(f, "R6 {}", self.0[7])?;
        writeln!(f, "R7 {}", self.0[8])?;
        writeln!(f, "R8 {}", self.0[9])?;

        Ok(())
    }
}

impl Index<Register> for Registers {
    type Output = u32;

    #[track_caller]
    fn index(&self, index: Register) -> &Self::Output {
        trace!("indexing register {index:?} got {}", self.0[index as usize]);
        &self.0[index as usize]
    }
}

impl IndexMut<Register> for Registers {
    #[track_caller]
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        trace!("{}", self.0[index as usize]);
        &mut self.0[index as usize]
    }
}

impl PartialEq for Registers {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
mod register_tests {
    use super::*;
    use Register::*;

    #[test]
    fn from_u16() {
        let r = Register::try_from(0_u16).unwrap();
        assert!(r == IP);
    }

    #[test]
    fn upper() {
        let mut regs = Registers::default();
        regs[R1] = u32::MAX;
        println!("{:?}", regs[R1].upper());
        assert!(regs.get_upper(R1) == 65535);
    }

    #[test]
    fn lower() {
        let mut regs = Registers::default();
        regs[R1] = u16::MAX as u32;
        assert!(regs.get_upper(R1) == 65535);
    }

    #[test]
    fn upper_and_lower() {
        let x = 0xAAAAEEEE;
        let mut regs = Registers::default();
        regs[R1] = x;

        let upper = regs[R1].upper();
        let lower = regs[R1].lower();
        println!("upper {upper:x} lower {lower:x}");
        assert_eq!(upper, 0xAAAA);
        assert_eq!(lower, 0xEEEE);
    }

    #[test]
    fn len() {
        assert!(Register::len() == 11);
    }
}
