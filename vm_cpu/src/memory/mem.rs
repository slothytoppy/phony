use std::fmt::Debug;

use tracing::info;

use crate::memory::Address;

#[derive(Debug)]
pub enum Error {
    InvalidAddress(u32),
    StackOverflow,
    StackUnderflow,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Memory: Debug + Default {
    #[track_caller]
    #[allow(unused_variables)]
    fn read<A>(&self, address: A) -> Result<u8>
    where
        A: Into<Address> + Copy;

    fn read_u16<A>(&self, address: A) -> Result<u16>
    where
        A: Into<Address> + Copy,
    {
        let address: Address = address.into();
        let low = self.read(address)?;
        let high = self.read(address.next()?)?;

        Ok(u16::from_le_bytes([low, high]))
    }

    fn read_u32<A>(&self, address: A) -> Result<u32>
    where
        A: Into<Address> + Copy,
    {
        let address: Address = address.into();
        let low_u16 = self.read(address)?;
        let address = address.next()?;
        let high_u16 = self.read(address)?;

        let low_u32 = self.read(address)?;
        let address = address.next()?;
        let high_u32 = self.read(address)?;

        Ok(u32::from_le_bytes([low_u16, high_u16, low_u32, high_u32]))
    }

    #[track_caller]
    #[allow(unused_variables)]
    fn write<A>(&mut self, address: A, byte: impl Into<u8>) -> Result<()>
    where
        A: Into<Address> + Copy;

    fn write_bytes<'a, A>(&mut self, address: A, bytes: impl Into<&'a [u8]>) -> Result<()>
    where
        A: Into<Address> + Copy,
    {
        let addr = address.into();
        let bytes: &[u8] = bytes.into();

        info!("writing bytes {bytes:?} into mem at address {addr}");

        for (i, byte) in bytes.iter().enumerate() {
            self.write(addr + Address::from(i), *byte)?;
        }

        Ok(())
    }

    fn write_u16<A>(&mut self, address: A, num: u16) -> Result<()>
    where
        A: Into<Address> + Copy,
    {
        let bytes = num.to_le_bytes();

        self.write_bytes(address, &bytes as &[u8])
    }

    fn write_u32<A>(&mut self, address: A, num: u32) -> Result<()>
    where
        A: Into<Address> + Copy,
    {
        let bytes = num.to_le_bytes();

        self.write_bytes(address, &bytes as &[u8])
    }

    #[track_caller]
    #[allow(unused_variables)]
    fn get<A>(&self, address_start: A, address_end: A) -> Result<&[u8]>
    where
        A: Into<Address> + Copy,
    {
        todo!()
    }
}
