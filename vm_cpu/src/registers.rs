use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

#[derive(Debug)]
pub enum Error {
    InvalidRegister(u16),
    InvalidConversion(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    IP,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    SP,
}

impl FromStr for Register {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ip" => Ok(Self::IP),
            "r1" => Ok(Self::R1),
            "r2" => Ok(Self::R2),
            "r3" => Ok(Self::R3),
            "r4" => Ok(Self::R4),
            "r5" => Ok(Self::R5),
            "r6" => Ok(Self::R6),
            "r7" => Ok(Self::R7),
            "r8" => Ok(Self::R8),
            "sp" => Ok(Self::SP),
            _ => Err(Self::Err::InvalidConversion(s.to_string())),
        }
    }
}

impl Register {
    pub const fn len() -> usize {
        10
    }
}

impl From<Register> for u16 {
    fn from(value: Register) -> Self {
        value as u16
    }
}

impl TryFrom<u16> for Register {
    type Error = Error;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::IP),
            1 => Ok(Register::R1),
            2 => Ok(Register::R2),
            3 => Ok(Register::R3),
            4 => Ok(Register::R4),
            6 => Ok(Register::R5),
            7 => Ok(Register::R6),
            8 => Ok(Register::R7),
            9 => Ok(Register::R8),
            10 => Ok(Register::SP),
            _ => Err(Error::InvalidRegister(value)),
        }
    }
}

impl TryFrom<&u16> for Register {
    type Error = Error;

    fn try_from(value: &u16) -> Result<Self, Self::Error> {
        Register::try_from(*value)
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub struct Registers([u16; Register::len()]);

impl Default for Registers {
    fn default() -> Self {
        let mut register = [0_u16; Register::len()];
        register[Register::SP as usize] = u16::MAX;
        Self(register)
    }
}

impl Registers {
    pub fn new(program_start: u16, stack_start: u16) -> Self {
        let mut registers = Self::default();
        registers.0[Register::IP as usize] = program_start;
        registers.0[Register::SP as usize] = stack_start;

        registers
    }

    pub fn get(&self, register: Register) -> u16 {
        self[register]
    }

    pub fn set(&mut self, register: Register, val: u16) {
        self[register] = val;
    }

    pub fn as_slice(&self) -> &[u16; Register::len()] {
        &self.0
    }
}

impl Index<Register> for Registers {
    type Output = u16;

    #[track_caller]
    fn index(&self, index: Register) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<Register> for Registers {
    #[track_caller]
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl PartialEq for Registers {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
