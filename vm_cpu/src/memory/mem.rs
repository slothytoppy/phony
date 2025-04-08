use crate::memory::Address;

#[derive(Debug)]
pub enum Error {
    InvalidAddress(u32),
    StackOverflow,
    StackUnderflow,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Memory {
    #[track_caller]
    #[allow(unused_variables)]
    fn read<A>(&self, address: A) -> Result<u8>
    where
        A: Into<Address> + Copy;

    #[track_caller]
    #[allow(unused_variables)]
    fn write<A>(&mut self, address: A, byte: impl Into<u8>) -> Result<()>
    where
        A: Into<Address> + Copy;

    #[track_caller]
    #[allow(unused_variables)]
    fn get<A>(&self, address_start: A, address_end: A) -> Result<&[u8]>
    where
        A: Into<Address> + Copy,
    {
        todo!()
    }
}
