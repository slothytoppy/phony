use std::{fmt::Debug, ops::ControlFlow};

use crate::{
    opcodes::{Instruction, OpCode},
    registers::{Register, Registers},
    stack::{self, Stack},
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
            registers: Registers::new(SIZE as u16),
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

    pub fn memory(&self) -> &Stack<SIZE> {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Stack<SIZE> {
        &mut self.memory
    }

    pub fn next_instruction(&mut self, op: OpCode) -> Instruction {
        let start_amount = self.registers.get(Register::IP) as usize;
        println!("starting at {start_amount}");
        let bytecode = self
            .memory
            .get(start_amount..start_amount + op.increment_amount() as usize);
        self.registers
            .set(Register::IP, start_amount as u16 + op.increment_amount());
        match op {
            OpCode::PushReg => {
                let arg = bytecode[1];
                let arg = Register::try_from(arg).unwrap();
                Instruction::PushReg(arg)
            }
            OpCode::PushVal => {
                println!("pushing val {bytecode:?}");
                let val = bytecode[1];
                Instruction::PushVal(val)
            }
            OpCode::MovRegReg => {
                let (mut left, mut right) = (Register::IP, Register::IP);
                for (i, code) in bytecode[1..=2].iter().enumerate() {
                    if i == 0 {
                        left = Register::try_from(code).unwrap();
                    } else {
                        right = Register::try_from(code).unwrap();
                    }
                }

                Instruction::MovRegReg(left, right)
            }
            OpCode::MovRegVal => {
                let (mut left, mut right) = (Register::IP, 0);
                for (i, code) in bytecode[1..=2].iter().enumerate() {
                    if i == 0 {
                        left = Register::try_from(code).unwrap();
                    } else {
                        right = *code
                    }
                }

                Instruction::MovRegVal(left, right)
            }
            OpCode::AddRegReg => {
                let (mut left, mut right) = (Register::IP, Register::IP);
                for (i, code) in bytecode[1..=2].iter().enumerate() {
                    if i == 0 {
                        left = Register::try_from(code).unwrap();
                    } else {
                        right = Register::try_from(code).unwrap();
                    }
                }

                Instruction::AddRegReg(left, right)
            }
            OpCode::AddRegNum => {
                let (mut left, mut right) = (Register::IP, 0);
                for (i, code) in bytecode[1..=2].iter().enumerate() {
                    if i == 0 {
                        left = Register::try_from(code).unwrap();
                    } else {
                        right = *code
                    }
                }

                Instruction::AddRegNum(left, right)
            }
            OpCode::PopReg => {
                let reg = bytecode[1];
                let reg = Register::try_from(reg).unwrap();

                Instruction::PopReg(reg)
            }
            OpCode::Jump => {
                let code = bytecode[1];
                Instruction::Jump(code)
            }
            OpCode::Call => Instruction::Call,
            OpCode::Halt => Instruction::Halt,
            OpCode::Ret => Instruction::Ret,
            OpCode::Load => {
                let reg = bytecode[1];
                let addr = bytecode[2];
                let Ok(reg) = Register::try_from(reg) else {
                    println!("printing converting {reg:?} to register");
                    panic!();
                };
                Instruction::Load(reg, addr)
            }
        }
    }

    pub fn execute(&mut self, insts: &Vec<Instruction>) {
        let res = self.write_instructions_to_memory(insts);
        if res.is_err() {
            panic!("{res:?}");
        }

        for inst in insts {
            match self.step(inst) {
                ControlFlow::Continue(_) => {}
                ControlFlow::Break(_) => break,
            }
        }
    }

    pub fn write_instructions_to_memory(&mut self, insts: &[Instruction]) -> stack::Result<()> {
        for inst in insts {
            match *inst {
                Instruction::MovRegReg(register, register1) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::MovRegReg.into())?;
                    self.memory_mut().write(ip + 1, register.into())?;
                    self.memory_mut().write(ip + 2, register1.into())?;
                }
                Instruction::MovRegVal(register, val) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::MovRegVal.into())?;
                    self.memory_mut().write(ip + 1, register.into())?;
                    self.memory_mut().write(ip + 2, val)?;
                }
                Instruction::PopReg(register) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::PopReg.into())?;
                    self.memory_mut().write(ip + 1, register.into())?;
                }
                Instruction::AddRegReg(register, register1) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::AddRegReg.into())?;
                    self.memory_mut().write(ip + 1, register.into())?;
                    self.memory_mut().write(ip + 2, register1.into())?;
                }
                Instruction::AddRegNum(register, val) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::AddRegNum.into())?;
                    self.memory_mut().write(ip + 1, register.into())?;
                    self.memory_mut().write(ip + 2, val)?;
                }
                Instruction::Jump(addr) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Jump.into())?;
                    self.memory_mut().write(ip + 1, addr)?;
                }
                Instruction::Load(register, addr) => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Load.into())?;
                    self.memory_mut().write(ip + 1, register.into())?;
                    self.memory_mut().write(ip + 2, addr)?;
                }
                Instruction::Call => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Call.into())?;
                }
                Instruction::Halt => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Halt.into())?;
                }
                Instruction::Ret => {
                    let ip = self.registers.get(Register::IP);
                    self.memory_mut().write(ip, OpCode::Ret.into())?;
                }
                Instruction::PushReg(reg) => {
                    let reg = self.registers().get(reg);
                    let sp = self.registers().get(Register::SP);
                    self.memory_mut().write(sp, reg)?;
                }
                Instruction::PushVal(val) => {
                    let sp = self.registers().get(Register::SP);
                    self.memory_mut().write(sp, val)?;
                }
            }
            self.registers.set(
                Register::IP,
                self.registers.get(Register::IP) + OpCode::from(*inst).increment_amount(),
            )
        }
        Ok(())
    }

    fn push_stack(&mut self, val: u16) -> stack::Result<()> {
        let sp = self.registers().get(Register::SP);
        if sp < SIZE as u16 / 4 {
            return Err(stack::Error::StackUnderFlow);
        }
        self.memory.write(sp, val)?;
        self.registers.set(Register::SP, sp - 1);
        Ok(())
    }

    fn pop_stack(&mut self) -> stack::Result<()> {
        let sp = self.registers().get(Register::SP);
        if sp + 2 > SIZE as u16 {
            return Err(stack::Error::StackOverflow);
        }
        self.registers.set(Register::SP, sp + 2);
        Ok(())
    }

    pub fn step(&mut self, inst: &Instruction) -> ControlFlow<(), ()> {
        match *inst {
            Instruction::PushReg(reg) => {
                let reg = self.registers().get(reg);
                self.push_stack(reg).unwrap();
            }
            Instruction::PushVal(val) => self.push_stack(val).unwrap(),
            Instruction::MovRegReg(register, register1) => {
                self.registers.set(register1, self.registers.get(register));
            }
            Instruction::MovRegVal(register, val) => {
                self.registers.set(register, val);
            }
            Instruction::PopReg(register) => {
                let addr = self.registers.get(Register::SP);
                let val = self.memory.get(addr as usize);
                println!("pop reg {addr} {val}");
                self.registers.set(register, *val);
            }
            Instruction::AddRegReg(register, register1) => {
                println!(
                    "{register} = {} {register1} = {}",
                    self.registers.get(register),
                    self.registers.get(register1)
                );
                println!(
                    "val {}",
                    self.registers.get(register) + self.registers.get(register1)
                );
                self.registers.set(
                    register,
                    self.registers.get(register) + self.registers.get(register1),
                );
            }

            Instruction::AddRegNum(register, val) => self
                .registers
                .set(register, self.registers.get(register).saturating_add(val)),

            Instruction::Call => {}
            Instruction::Jump(addr) => {
                self.registers.set(Register::IP, addr);
                return ControlFlow::Break(());
            }
            Instruction::Halt => return ControlFlow::Break(()),

            Instruction::Ret => return ControlFlow::Break(()),
            Instruction::Load(reg, addr) => {
                let val = self.memory.get(addr as usize);
                self.registers.set(reg, *val);
            }
        }
        ControlFlow::Continue(())
    }
}
