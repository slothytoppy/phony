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
        let start_amount = self.registers.get(Register::IP) as usize;
        println!("starting at {start_amount}");
        let bytecode = self
            .memory
            .get(start_amount..start_amount + op.increment_amount() as usize);
        self.registers.set(
            Register::IP,
            self.registers.get(Register::IP) + op.increment_amount(),
        );
        println!("bytecode {bytecode:?}");
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
            OpCode::Load => {
                let Some(reg) = bytecode[1] else {
                    println!("printing left {bytecode:?}");
                    panic!();
                };
                let Some(addr) = bytecode[2] else {
                    println!("printing right {bytecode:?}");
                    panic!();
                };
                let Ok(reg) = Register::try_from(reg) else {
                    println!("printing converting {reg:?} to register");
                    panic!();
                };
                Instruction::Load(reg, addr)
            }
        }
    }

    pub fn parse(&mut self) -> Vec<Instruction> {
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
                        OpCode::Load => {
                            instructions.push(self.next_instruction(op));
                        }
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
        loop {
            match self.memory[self.registers().get(Register::IP)] {
                Some(code) => {
                    println!("reading {:?}", code);
                    let code: OpCode = code.try_into().unwrap();
                    self.registers.set(
                        Register::IP,
                        self.registers.get(Register::IP) + code.increment_amount(),
                    );
                }
                None => {
                    println!("leaving loop {}", self.registers.get(Register::IP));

                    break;
                }
            }
        }
        self.registers.set(Register::IP, 0);
        for inst in insts {
            let mut should_call_next_instruction = true;
            match inst {
                Instruction::PushRegReg(register, register1) => {
                    self.registers.set(register1, self.registers.get(register));
                }
                Instruction::PushRegVal(register, val) => {
                    self.registers.set(register, val);
                }
                Instruction::PopReg(_register) => todo!(),
                Instruction::AddRegReg(register, register1) => self.registers.set(
                    register,
                    self.registers
                        .get(register)
                        .saturating_add(self.registers.get(register1)),
                ),
                Instruction::AddRegNum(register, val) => self
                    .registers
                    .set(register, self.registers.get(register).saturating_add(val)),

                Instruction::Call => should_call_next_instruction = false,
                Instruction::Jump(addr) => {
                    should_call_next_instruction = false;
                    self.registers.set(Register::IP, addr);
                }
                Instruction::Halt => return,
                Instruction::Ret => {}
                Instruction::Load(reg, addr) => {
                    let val = self.memory.get(addr as usize);
                    if let Some(val) = val {
                        self.registers.set(reg, *val);
                    }
                }
            }
            if should_call_next_instruction {
                self.next_instruction(OpCode::from(inst));
            }
        }
    }

    pub fn write_instructions_to_memory(&mut self, insts: Vec<Instruction>) {
        for inst in insts {
            match inst {
                Instruction::PushRegReg(register, register1) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::PushRegReg.into());
                    self.memory_mut().write(ip + 1, register.into());
                    self.memory_mut().write(ip + 2, register1.into());
                }
                Instruction::PushRegVal(register, val) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::PushRegVal.into());
                    self.memory_mut().write(ip + 1, register.into());
                    self.memory_mut().write(ip + 2, val);
                }
                Instruction::PopReg(register) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::PopReg.into());
                    self.memory_mut().write(ip + 1, register.into());
                }
                Instruction::AddRegReg(register, register1) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::AddRegReg.into());
                    self.memory_mut().write(ip + 1, register.into());
                    self.memory_mut().write(ip + 2, register1.into());
                }
                Instruction::AddRegNum(register, val) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::AddRegNum.into());
                    self.memory_mut().write(ip + 1, register.into());
                    self.memory_mut().write(ip + 2, val);
                }
                Instruction::Jump(addr) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Jump.into());
                    self.memory_mut().write(ip + 1, addr);
                }
                Instruction::Load(register, addr) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Load.into());
                    self.memory_mut().write(ip + 1, register.into());
                    self.memory_mut().write(ip + 2, addr);
                }
                Instruction::Call => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Call.into());
                }
                Instruction::Halt => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Halt.into());
                }
                Instruction::Ret => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Ret.into());
                }
            }
            self.registers.set(
                Register::IP,
                self.registers.get(Register::IP) + OpCode::from(inst).increment_amount(),
            );
        }
    }
}
