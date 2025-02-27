use crate::address::Address;

#[derive(Debug)]
pub enum Error {
    InvalidAddress(u16),
    StackOverflow,
    StackUnderflow,
}

pub trait Memory {
    #[track_caller]
    fn read<A>(&self, address: A) -> Result<u16, Error>
    where
        A: Into<Address> + Copy;

    #[track_caller]
    fn write<A>(&mut self, address: A, byte: impl Into<u16>) -> Result<(), Error>
    where
        A: Into<Address> + Copy;

    #[track_caller]
    fn get<A>(&self, address_start: A, address_end: A) -> Result<&[u16], Error>
    where
        A: Into<Address> + Copy;
}
