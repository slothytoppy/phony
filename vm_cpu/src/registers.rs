use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

#[derive(Debug)]
pub enum Error {
    InvalidRegister(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    IP,
    R1,
    R2,
    R3,
    R4,
    SP,
}

impl Register {
    pub const fn len() -> usize {
        6
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
            5 => Ok(Register::SP),
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
        match self {
            Register::IP => f.write_str("IP"),
            Register::R1 => f.write_str("R1"),
            Register::R2 => f.write_str("R2"),
            Register::R3 => f.write_str("R3"),
            Register::R4 => f.write_str("R4"),
            Register::SP => f.write_str("SP"),
        }
    }
}

#[derive(Debug)]
pub struct Registers([u16; Register::len()]);

impl Registers {
    pub fn new(program_start: u16, stack_start: u16) -> Self {
        println!("stack_start = {stack_start}");
        let mut register = [0; Register::len()];
        register[Register::IP as usize] = program_start;
        register[Register::SP as usize] = stack_start;
        Self(register)
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

impl Default for Registers {
    fn default() -> Self {
        let mut register = [0_u16; Register::len()];
        register[Register::SP as usize] = u16::MAX;
        Self(register)
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
