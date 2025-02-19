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

        loop {
            if start == SIZE {
                break;
            }
            match self.memory.read(start as u16) {
                Some(val) => {
                    let op = <u16 as TryInto<OpCode>>::try_into(val);
                    if op.is_ok() {}
                    println!("{op:?}");
                }
                None => {}
            }
            start += 1;
        }

        return vec![];
    }

    pub fn execute(&mut self) {
        let mut op_addr = 1;
        loop {
            let mut should_call_next_instruction = true;
            let exec_addr = self.registers().get(Register::IP);
            if exec_addr == SIZE {
                return;
            }
            let Some(op) = self.memory.read(exec_addr as u16) else {
                continue;
            };
            println!("reading at addr {exec_addr}");
            match op.try_into() {
                Ok(op) => {
                    match op {
                        OpCode::PushRegVal => {
                            op_addr = (<OpCode as Into<u16>>::into(op) - exec_addr as u16)
                                .saturating_sub(1);
                            println!("op addr {op_addr}");
                            let ip = exec_addr + 1;
                            let Ok(register): Result<Register, ()> =
                                (match self.memory.read(ip as u16) {
                                    Some(val) => val.try_into(),
                                    None => panic!(),
                                })
                            else {
                                panic!()
                            };
                            let ip = exec_addr + 2;
                            let Some(val) = self.memory.read(ip as u16) else {
                                panic!();
                            };
                            *self.registers.get_mut(register) = val as usize;
                            println!("reg {register} val {val}");
                        }
                        OpCode::PushRegReg => {
                            let ip = exec_addr + 1;
                            let Ok(left_register): Result<Register, ()> = self
                                .memory
                                .read(ip as u16)
                                .unwrap_or_else(|| panic!())
                                .try_into()
                            else {
                                panic!("invalid register: {}", ip);
                            };
                            let ip = exec_addr + 1;
                            let Ok(right_register): Result<Register, ()> = self
                                .memory
                                .read(ip as u16)
                                .unwrap_or_else(|| panic!())
                                .try_into()
                            else {
                                panic!("invalid register: {}", ip);
                            };
                            *self.registers.get_mut(left_register) =
                                self.registers.get(right_register);
                        }
                        OpCode::AddRegReg => {
                            //let (res, overflow) = self
                            //    .registers
                            //    .get_mut(register)
                            //    .overflowing_add(self.registers.get(register1));
                            //if overflow {
                            //    *self.registers.get_mut(register) = 0;
                            //} else {
                            //    *self.registers.get_mut(register) = res;
                            //}
                        }
                        OpCode::AddRegNum => {
                            //let (res, overflow) = self.registers.get_mut(register).overflowing_add(val);
                            //if overflow {
                            //    *self.registers.get_mut(register) = 0
                            //} else {
                            //    *self.registers.get_mut(register) = res
                            //}
                        }
                        OpCode::Jump => {
                            //println!(
                            //    "jumping to addr {addr:#02x} op at addr {:?}",
                            //    self.memory.read(addr as u16)
                            //);
                            //*self.registers.get_mut(Register::IP) = addr;
                            should_call_next_instruction = false;
                        }
                        OpCode::Call => todo!(),
                        OpCode::PopReg => {
                            //let val = self.registers().get(register);
                            //let addr = self.registers().get(Register::SP);
                            //self.memory.write(addr as u16, op);
                        }
                        OpCode::Halt => {
                            println!("op addr {op_addr}");

                            return;
                        }
                    }
                }
                Err(e) => {}
            }
            if should_call_next_instruction {
                let opcode = OpCode::try_from(op_addr as u16);
                println!("op_addr {:?}", opcode);
                self.next_instruction(opcode.unwrap());
            }
        }
    }
}
