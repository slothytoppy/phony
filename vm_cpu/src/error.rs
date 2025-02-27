use crate::memory;
use crate::opcodes;
use crate::registers;

#[derive(Debug)]
pub enum Error {
    MemError(memory::Error),
    OpCodeError(opcodes::Error),
    RegisterError(registers::Error),
}

impl From<opcodes::Error> for Error {
    fn from(val: opcodes::Error) -> Self {
        Error::OpCodeError(val)
    }
}

impl From<memory::Error> for Error {
    fn from(value: memory::Error) -> Self {
        Error::MemError(value)
    }
}

impl From<registers::Error> for Error {
    fn from(value: registers::Error) -> Self {
        Error::RegisterError(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
