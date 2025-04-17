use tracing::info;

use crate::memory::{Address, Error, Memory};

#[derive(Debug)]
pub struct Stack<const SIZE: usize> {
    memory: [u8; SIZE],
}

impl<const SIZE: usize> Memory for Stack<SIZE> {
    fn read<A>(&self, address: A) -> std::result::Result<u8, Error>
    where
        A: Into<Address> + Copy,
    {
        let byte = self.memory[usize::from(address.into())];
        Ok(byte)
    }

    fn write<A>(&mut self, address: A, byte: impl Into<u8>) -> std::result::Result<(), Error>
    where
        A: Into<Address> + Copy,
    {
        let byte = byte.into();
        let addr: Address = address.into();
        if addr < Address::from(SIZE) {
            self.memory[usize::from(addr)] = byte;
        } else {
            self.memory[usize::from(addr).saturating_sub(1)] = byte;
        }
        Ok(())
    }

    // fn write_bytes<'a, A>(&mut self, address: A, bytes: impl Into<&'a [u8]>) -> Result<()>
    // where
    //     A: Into<Address> + Copy,
    // {
    //     let addr = address.into();
    //     let bytes: &[u8] = bytes.into();
    //
    //     info!("writing bytes {bytes:?} into mem at address {addr}");
    //
    //     for (i, byte) in bytes.iter().enumerate() {
    //         self.write(addr - Address::from(i), *byte)?;
    //     }
    //
    //     Ok(())
    // }

    fn get<A>(&self, start: A, end: A) -> std::result::Result<&[u8], Error>
    where
        A: Into<Address> + Copy,
    {
        let start = start.into();
        let end = end.into();
        Ok(&self.memory[usize::from(start)..usize::from(end)])
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<const SIZE: usize> Default for Stack<SIZE> {
    fn default() -> Self {
        const { assert!(SIZE > 0) };
        Self { memory: [0; SIZE] }
    }
}

impl<const SIZE: usize> Stack<SIZE> {
    pub fn new() -> Self {
        Self::default()
    }
}
