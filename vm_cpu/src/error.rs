pub enum Error {
    MemError(crate::memory::Error),
    OpCodeError(crate::opcodes::Error),
    RegisterError(crate::registers::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
