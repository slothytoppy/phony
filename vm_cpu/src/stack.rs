use crate::opcodes::{Instruction, OpCode};

#[derive(Debug)]
pub struct Stack<const SIZE: usize> {
    memory: [Option<u16>; SIZE],
}

impl<const SIZE: usize> Stack<SIZE> {
    pub const fn new() -> Self {
        Self {
            memory: [None; SIZE],
        }
    }

    pub fn read(&self, address: u16) -> Option<u16> {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, val: u16) {
        self.memory[address as usize] = Some(val)
    }

    pub fn print(&self) {
        println!("reading memory:");
        for byte in self.memory.iter() {
            if let Some(byte) = *byte {
                let op: Result<OpCode, ()> = byte.try_into();
                println!("byte {:?}", op)
            } else {
                break;
            }
        }
    }
}
