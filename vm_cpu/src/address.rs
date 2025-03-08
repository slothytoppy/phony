use crate::memory::{self};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Address(u16);

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Address> for u16 {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl From<Address> for usize {
    fn from(value: Address) -> Self {
        value.0 as usize
    }
}

impl Address {
    pub fn next(&self) -> Result<Address, memory::Error> {
        let Some(addr) = self.0.checked_add(1) else {
            return Err(memory::Error::StackOverflow);
        };
        Ok(Address(addr))
    }

    pub fn prev(&self) -> Result<Address, memory::Error> {
        let Some(addr) = self.0.checked_sub(1) else {
            return Err(memory::Error::StackUnderflow);
        };
        Ok(Address(addr))
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
