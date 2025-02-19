use std::fmt::Debug;

use crate::{
    opcodes::{Instruction, OpCode},
    registers::{Register, Registers},
    stack::Stack,
};

#[derive(Debug)]
pub struct Cpu<const SIZE: usize> {
    registers: Registers,
    memory: Stack<SIZE>,
}

impl<const SIZE: usize> Cpu<SIZE> {
    pub fn new() -> Self {
        let memory = Stack::new();
        Self {
            memory,
            registers: Registers::default(),
        }
    }

    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    pub fn memory_mut(&mut self) -> &mut Stack<SIZE> {
        &mut self.memory
    }

    pub fn dump(&self) {
        self.memory.print();
    }

    pub fn next_instruction(&mut self, op: OpCode) {
        match op {
            OpCode::PushRegReg => *self.registers.get_mut(Register::IP) += 3,
            OpCode::PushRegVal => *self.registers.get_mut(Register::IP) += 3,
            OpCode::PopReg => *self.registers.get_mut(Register::IP) += 2,
            OpCode::AddRegReg => *self.registers.get_mut(Register::IP) += 3,
            OpCode::AddRegNum => *self.registers.get_mut(Register::IP) += 3,
            OpCode::Call => *self.registers.get_mut(Register::IP) += 1,
            OpCode::Jump => *self.registers.get_mut(Register::IP) += 2,
            OpCode::Halt => {}
        }
    }

    pub fn parse(&self) -> Vec<Instruction> {
        let mut start = 0;
        let mut instructions = vec![];

        while let Some(byte) = self.memory.memory().get(start) {
            let Some(byte) = byte else {
                break;
            };
            match <u16 as TryInto<OpCode>>::try_into(*byte) {
                Ok(op) => {
                    //println!("opcode {op:?}");
                    match op {
                        OpCode::PushRegVal => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let left = left.unwrap();
                            let Some(right) = self.memory.memory().get(start + 2) else {
                                panic!();
                            };
                            let right = right.unwrap();
                            //println!("left {left:?} right {right:?}");

                            instructions.push(Instruction::PushRegVal(
                                Register::try_from(left).unwrap(),
                                right as usize,
                            ));
                            start += 3;
                            //println!(
                            //    "indexing {:?}",
                            //    self.memory.memory().get(start).unwrap().unwrap()
                            //);
                        }
                        OpCode::PushRegReg => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let left = left.unwrap();
                            let Some(right) = self.memory.memory().get(start + 2) else {
                                panic!();
                            };
                            let right = right.unwrap();
                            //println!("left {left:?} right {right:?}");

                            instructions.push(Instruction::PushRegReg(
                                Register::try_from(left).unwrap(),
                                Register::try_from(right).unwrap(),
                            ));
                            start += 2
                        }
                        OpCode::AddRegReg => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let left = left.unwrap();
                            let Some(right) = self.memory.memory().get(start + 2) else {
                                panic!();
                            };
                            let right = right.unwrap();
                            //println!("left {left:?} right {right:?}");

                            instructions.push(Instruction::AddRegReg(
                                Register::try_from(left).unwrap(),
                                Register::try_from(right).unwrap(),
                            ));
                            start += 2
                        }
                        OpCode::AddRegNum => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let left = left.unwrap();
                            let Some(right) = self.memory.memory().get(start + 2) else {
                                panic!();
                            };
                            let right = right.unwrap();
                            //println!("left {left:?} right {right:?}");

                            instructions.push(Instruction::PushRegVal(
                                Register::try_from(left).unwrap(),
                                right as usize,
                            ));
                            start += 2
                        }
                        OpCode::Jump => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let left = left.unwrap();
                            instructions.push(Instruction::Jump(left as usize));
                            start += 1
                        }
                        OpCode::Call => {
                            instructions.push(Instruction::Call);
                            start += 1
                        }
                        OpCode::Halt => {
                            instructions.push(Instruction::Halt);
                            start += 1
                        }
                        OpCode::PopReg => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let left = left.unwrap();
                            match Register::try_from(left) {
                                Ok(val) => {
                                    instructions.push(Instruction::PopReg(val));
                                }
                                Err(_e) => {}
                            }

                            start += 1
                        }
                    }
                }
                Err(_e) => {}
            }
        }

        instructions
    }

    pub fn execute(&mut self, insts: Vec<Instruction>) {
        let mut op_addr = 1;
        println!("{insts:?}");
        for inst in insts {
            let mut should_call_next_instruction = true;
            match inst {
                Instruction::PushRegReg(register, register1) => {
                    *self.registers.get_mut(register) = self.registers.get(register1);
                }
                Instruction::PushRegVal(register, val) => *self.registers.get_mut(register) = val,
                Instruction::PopReg(register) => todo!(),
                Instruction::AddRegReg(register, register1) => {
                    self.registers
                        .get_mut(register)
                        .saturating_add(self.registers.get(register1));
                }
                Instruction::AddRegNum(register, val) => {
                    self.registers.get_mut(register).saturating_add(val);
                }

                Instruction::Call => should_call_next_instruction = false,
                Instruction::Jump(addr) => {
                    should_call_next_instruction = false;
                    *self.registers.get_mut(Register::IP) = addr;
                }
                Instruction::Halt => return,
            }
            if should_call_next_instruction {
                self.next_instruction(OpCode::from(inst));
            }
        }
    }
}
