use std::ops::{Index, IndexMut};

use crate::registers::Register;
use crate::{
    address::Address,
    memory::{Error, Memory},
};

#[derive(Debug)]
pub struct Stack<const SIZE: usize> {
    memory: [u16; SIZE],
}

impl<const SIZE: usize> Memory for Stack<SIZE> {
    fn read<A>(&self, address: A) -> std::result::Result<u16, Error>
    where
        A: Into<Address> + Copy,
    {
        let byte = self.memory[usize::from(address.into())];
        Ok(byte)
    }

    fn write<A>(&mut self, address: A, byte: impl Into<u16>) -> std::result::Result<(), Error>
    where
        A: Into<Address> + Copy,
    {
        let byte = byte.into();
        let addr: Address = address.into();
        if addr < Address::from(SIZE as u16) {
            self.memory[usize::from(address.into())] = byte;
        } else {
            self.memory[usize::from(address.into()).saturating_sub(1)] = byte;
        }
        Ok(())
    }

    fn get<A>(&self, start: A, end: A) -> std::result::Result<&[u16], Error>
    where
        A: Into<Address> + Copy,
    {
        let start = start.into();
        let end = end.into();
        Ok(&self.memory[usize::from(start)..usize::from(end)])
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<const SIZE: usize> Index<Register> for Stack<SIZE> {
    type Output = u16;

    fn index(&self, index: Register) -> &Self::Output {
        &self.memory[index as usize]
    }
}

impl<const SIZE: usize> IndexMut<Register> for Stack<SIZE> {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.memory[index as usize]
    }
}

impl<const SIZE: usize> Default for Stack<SIZE> {
    fn default() -> Self {
        Self { memory: [0; SIZE] }
    }
}

impl<const SIZE: usize> Stack<SIZE> {
    pub fn new() -> Self {
        Self::default()
    }
}
