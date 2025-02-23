use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    IP,
    R1,
    R2,
    R3,
    R4,
    SP,
}

impl From<Register> for u16 {
    fn from(value: Register) -> Self {
        value as u16
    }
}

impl TryFrom<u16> for Register {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::IP),
            1 => Ok(Register::R1),
            2 => Ok(Register::R2),
            3 => Ok(Register::R3),
            4 => Ok(Register::R4),
            5 => Ok(Register::SP),
            _ => Err(()),
        }
    }
}

impl TryFrom<&u16> for Register {
    type Error = ();
    fn try_from(value: &u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::IP),
            1 => Ok(Register::R1),
            2 => Ok(Register::R2),
            3 => Ok(Register::R3),
            4 => Ok(Register::R4),
            5 => Ok(Register::SP),
            _ => Err(()),
        }
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

impl Register {
    pub const fn len() -> usize {
        6
    }
}

#[derive(Debug)]
pub struct Registers {
    register: [usize; Register::len()],
}

impl PartialEq for Registers {
    fn eq(&self, other: &Self) -> bool {
        self.register == other.register
    }
}

impl Default for Registers {
    fn default() -> Self {
        let mut register = [0; Register::len()];
        register[Register::SP as usize] = u16::MAX as usize;
        Self { register }
    }
}

impl Registers {
    pub fn get(&self, register: Register) -> u16 {
        self.register[register as usize] as u16
    }

    pub fn set(&mut self, register: Register, val: u16) {
        self.register[register as usize] = val as usize
    }

    pub fn into_slice(&self) -> &[usize; Register::len()] {
        &self.register
    }
}
