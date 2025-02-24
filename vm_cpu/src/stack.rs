use std::{ops::Index, slice::SliceIndex};

#[derive(Debug)]
pub enum Error {
    MemError,
    StackOverflow,
    StackUnderFlow,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Stack<const SIZE: usize> {
    memory: [Option<u16>; SIZE],
}

impl<const SIZE: usize> Index<u16> for Stack<SIZE> {
    type Output = Option<u16>;

    fn index(&self, index: u16) -> &Self::Output {
        if index < SIZE as u16 {
            &self.memory[index as usize]
        } else {
            &None
        }
    }
}

impl<const SIZE: usize> Default for Stack<SIZE> {
    fn default() -> Self {
        Self {
            memory: [None; SIZE],
        }
    }
}

impl<const SIZE: usize> Stack<SIZE> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, address: u16, val: u16) -> Result<()> {
        if address > SIZE as u16 {
            return Err(Error::MemError);
        }
        self.memory[address.saturating_sub(1) as usize] = Some(val);
        Ok(())
    }

    pub fn memory(&self) -> &[Option<u16>; SIZE] {
        &self.memory
    }

    pub fn get<I>(&self, range: I) -> &I::Output
    where
        I: SliceIndex<[Option<u16>]>,
    {
        &self.memory[range]
    }
}
