use std::{ops::Index, slice::SliceIndex};

use crate::opcodes::OpCode;

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

    pub fn write(&mut self, address: u16, val: u16) {
        self.memory[address as usize] = Some(val)
    }

    pub fn print(&self) {
        println!("reading memory:");
        let mut start = 0;
        for byte in &self.memory[start..] {
            if let Some(byte) = *byte {
                println!("byte {byte:?}");
                let op: Result<OpCode, ()> = byte.try_into();
                if let Ok(op) = op {
                    start += op.increment_amount() as usize;
                    println!("byte {:?}", op)
                }
            }
        }
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
