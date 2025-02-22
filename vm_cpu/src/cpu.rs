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

impl<const SIZE: usize> Default for Cpu<SIZE> {
    fn default() -> Self {
        let memory = Stack::new();
        Self {
            memory,
            registers: Registers::default(),
        }
    }
}

impl<const SIZE: usize> Cpu<SIZE> {
    pub fn new() -> Self {
        Self::default()
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

    pub fn next_instruction(&mut self, op: OpCode) -> Instruction {
        let start_amount = self.registers.get(Register::IP);
        let bytecode = self
            .memory
            .get(start_amount..start_amount + op.increment_amount() as usize);
        *self.registers.get_mut(Register::IP) += op.increment_amount() as usize;
        match op {
            OpCode::PushRegReg => {
                let (mut left, mut right) = (Register::IP, Register::IP);
                for (i, code) in bytecode[1..=2].iter().enumerate() {
                    if let Some(code) = code {
                        if i == 0 {
                            left = Register::try_from(code).unwrap();
                        } else {
                            right = Register::try_from(code).unwrap();
                        }
                    }
                }

                Instruction::PushRegReg(left, right)
            }
            OpCode::PushRegVal => {
                let (mut left, mut right) = (Register::IP, 0);
                for (i, code) in bytecode[1..=2].iter().enumerate() {
                    if let Some(code) = code {
                        if i == 0 {
                            left = Register::try_from(code).unwrap();
                        } else {
                            right = *code
                        }
                    }
                }

                Instruction::PushRegVal(left, right)
            }
            OpCode::AddRegReg => {
                let (mut left, mut right) = (Register::IP, Register::IP);
                for (i, code) in bytecode[1..=2].iter().enumerate() {
                    if let Some(code) = code {
                        if i == 0 {
                            left = Register::try_from(code).unwrap();
                        } else {
                            right = Register::try_from(code).unwrap();
                        }
                    }
                }

                Instruction::AddRegReg(left, right)
            }
            OpCode::AddRegNum => {
                let (mut left, mut right) = (Register::IP, 0);
                for (i, code) in bytecode[1..=2].iter().enumerate() {
                    if let Some(code) = code {
                        if i == 0 {
                            left = Register::try_from(code).unwrap();
                        } else {
                            right = *code
                        }
                    }
                }

                Instruction::AddRegNum(left, right)
            }
            OpCode::PopReg => {
                let Some(code) = bytecode[1] else {
                    panic!();
                };

                let reg = Register::try_from(code).unwrap();
                Instruction::PopReg(reg)
            }
            OpCode::Jump => match bytecode[1] {
                Some(code) => Instruction::Jump(code),
                None => {
                    panic!()
                }
            },
            OpCode::Call => Instruction::Call,
            OpCode::Halt => Instruction::Halt,
            OpCode::Ret => Instruction::Ret,
        }
    }

    pub fn parse(&self) -> Vec<Instruction> {
        let mut start = 0;
        let mut instructions = vec![];

        while let Some(byte) = self.memory.memory().get(start) {
            if start >= SIZE {
                break;
            }
            let Some(byte) = byte else {
                start += 1;
                continue;
            };
            match <u16 as TryInto<OpCode>>::try_into(*byte) {
                Ok(op) => {
                    //println!("opcode {op:?}");
                    match op {
                        OpCode::PushRegVal => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let Some(right) = self.memory.memory().get(start + 2) else {
                                panic!();
                            };
                            //println!("left {left:?} right {right:?}");
                            let left = left.unwrap();
                            let right = right.unwrap();

                            instructions.push(Instruction::PushRegVal(
                                Register::try_from(left).unwrap(),
                                right,
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
                            start += 3
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
                            start += 3
                        }
                        OpCode::AddRegNum => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let left = left.unwrap();
                            let Some(right) = self.memory.memory().get(start + 2) else {
                                panic!();
                            };
                            println!("left {left:?} right {right:?}");
                            let right = right.unwrap();

                            instructions.push(Instruction::PushRegVal(
                                Register::try_from(left).unwrap(),
                                right,
                            ));
                            start += 2
                        }
                        OpCode::Jump => {
                            let Some(left) = self.memory.memory().get(start + 1) else {
                                panic!()
                            };
                            let left = left.unwrap();
                            instructions.push(Instruction::Jump(left));
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
                        }
                        OpCode::Ret => {}
                    }
                }
                Err(_e) => {
                    println!("failed to convert {byte} into opcode");
                }
            }
            start += 1;
        }

        instructions
    }

    pub fn execute(&mut self, insts: Vec<Instruction>) {
        for inst in insts {
            let mut should_call_next_instruction = true;
            match inst {
                Instruction::PushRegReg(register, register1) => {
                    *self.registers.get_mut(register1) = self.registers.get(register);
                }
                Instruction::PushRegVal(register, val) => {
                    *self.registers.get_mut(register) = val as usize
                }
                Instruction::PopReg(_register) => todo!(),
                Instruction::AddRegReg(register, register1) => {
                    self.registers
                        .get_mut(register)
                        .saturating_add(self.registers.get(register1));
                }
                Instruction::AddRegNum(register, val) => {
                    self.registers
                        .get_mut(register)
                        .saturating_add(val as usize);
                }

                Instruction::Call => should_call_next_instruction = false,
                Instruction::Jump(addr) => {
                    should_call_next_instruction = false;
                    *self.registers.get_mut(Register::IP) = addr as usize;
                }
                Instruction::Halt => return,
                Instruction::Ret => {}
            }
            if should_call_next_instruction {
                self.next_instruction(OpCode::from(inst));
            }
        }
    }
}
