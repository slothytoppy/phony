use std::{fmt::Debug, ops::ControlFlow};

use crate::{
    address::Address,
    error::Error,
    memory::{self, Memory},
    opcodes::{Instruction, OpCode},
    registers::{Register, Registers},
    stack::{self},
};

#[derive(Debug)]
pub struct Cpu<M: Memory> {
    registers: Registers,
    memory: M,
}

impl<M: Memory> Cpu<M> {
    pub fn new(memory: M, program_start: u16, stack_start: u16) -> Self {
        Self {
            memory,
            registers: Registers::new(program_start, stack_start),
        }
    }

    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut M {
        &mut self.memory
    }

    pub fn next_instruction(&mut self, op: OpCode) -> Result<Instruction, memory::Error> {
        let start_amount = self.registers.get(Register::IP) as usize;
        let bytecode = self.memory.get(
            Address::from(start_amount as u16),
            (start_amount as u16 + op.increment_amount()).into(),
        )?;
        self.registers
            .set(Register::IP, start_amount as u16 + op.increment_amount());
        match op {
            OpCode::PushReg => {
                let arg = bytecode[1];
                let arg = Register::try_from(arg).unwrap();
                Ok(Instruction::PushReg(arg))
            }
            OpCode::PushVal => Ok(Instruction::PushVal(bytecode[1])),
            OpCode::MovRegReg => {
                println!("{bytecode:?}");
                let left = Register::try_from(bytecode[1]).unwrap();
                let right = Register::try_from(bytecode[2]).unwrap();

                Ok(Instruction::MovRegReg(left, right))
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

                Ok(Instruction::MovRegVal(left, right))
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

                Ok(Instruction::AddRegReg(left, right))
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

                Ok(Instruction::AddRegVal(left, right))
            }
            OpCode::PopReg => {
                let reg = bytecode[1];
                let reg = Register::try_from(reg).unwrap();

                Ok(Instruction::PopReg(reg))
            }
            OpCode::Jump => Ok(Instruction::Jump(bytecode[1].into())),
            OpCode::Call => Ok(Instruction::Call),
            OpCode::CallAddr => {
                let addr = bytecode[1];
                Ok(Instruction::CallAddr(Address::from(addr)))
            }
            OpCode::Halt => Ok(Instruction::Halt),
            OpCode::Ret => Ok(Instruction::Ret),
            OpCode::Load => {
                let reg = bytecode[1];
                let addr = bytecode[2];
                let Ok(reg) = Register::try_from(reg) else {
                    println!("printing converting {reg:?} to register");
                    panic!();
                };
                Ok(Instruction::Load(reg, addr.into()))
            }
        }
    }

    pub fn execute(&mut self) -> ControlFlow<(), ()> {
        loop {
            match self.step() {
                Ok(flow) => match flow {
                    ControlFlow::Continue(_) => continue,
                    ControlFlow::Break(_) => return ControlFlow::Break(()),
                },
                Err(e) => panic!("{e:?}"),
            };
        }
    }

    pub fn write_instruction_to_memory(&mut self, inst: &Instruction) -> stack::Result<()> {
        let ip = self.registers.get(Register::IP);
        let mut ip_addr = Address::from(ip);
        self.memory.write(ip_addr, OpCode::from(inst))?;
        ip_addr = ip_addr.next()?;
        match inst {
            Instruction::MovRegReg(register, register1) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, self.registers.get(*register1))?;
                _ = ip_addr.next()?;
            }
            Instruction::MovRegVal(register, val) => {
                self.memory.write(ip_addr, *register)?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, *val)?;
                _ = ip_addr.next()?;
            }
            Instruction::PushReg(register) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                _ = ip_addr.next()?;
            }
            Instruction::PushVal(val) => {
                self.memory.write(ip_addr, *val)?;
                _ = ip_addr.next()?;
            }
            Instruction::PopReg(register) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                _ = ip_addr.next()?;
            }
            Instruction::AddRegReg(register, register1) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, self.registers.get(*register1))?;
                _ = ip_addr.next()?;
            }
            Instruction::AddRegVal(register, val) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, *val)?;
                _ = ip_addr.next()?;
            }
            Instruction::Jump(address) => {
                self.memory.write(ip_addr, *address)?;
                _ = ip_addr.next()?;
            }
            Instruction::Load(register, address) => {
                self.memory.write(ip_addr, *register)?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, *address)?;
                _ = ip_addr.next()?;
            }
            Instruction::Call => {
                todo!()
            }
            Instruction::CallAddr(address) => {
                self.memory.write(ip_addr, *address)?;
                ip_addr.next()?;
            }
            Instruction::Halt => {}
            Instruction::Ret => {}
        }
        self.registers[Register::IP] += OpCode::from(inst).increment_amount();
        Ok(())
    }

    pub fn write_instructions_to_memory(&mut self, insts: &[Instruction]) -> stack::Result<()> {
        for inst in insts {
            self.write_instruction_to_memory(inst)?;
        }
        self.registers.set(Register::IP, 0);

        Ok(())
    }

    fn push_stack(&mut self, val: u16) -> stack::Result<()> {
        let sp = self.registers()[Register::SP];
        self.memory.write(Address::from(sp), val)?;
        self.registers[Register::SP] = sp - 1;
        Ok(())
    }

    fn pop_stack(&mut self, reg: Register) -> stack::Result<()> {
        let sp = self.registers()[Register::SP];
        self.registers.set(reg, self.memory.read(sp)?);
        self.registers.set(Register::SP, sp + 1);
        Ok(())
    }

    pub fn step(&mut self) -> Result<ControlFlow<(), ()>, Error> {
        let byte = self.memory.read(self.registers()[Register::IP])?;

        let code = OpCode::try_from(byte)?;
        let inst = self.next_instruction(code)?;

        match inst {
            Instruction::MovRegReg(register, register1) => {
                self.registers[register] = self.registers.get(register1)
            }

            Instruction::MovRegVal(register, val) => self.registers[register] = val,
            Instruction::PushReg(register) => self.push_stack(self.registers.get(register))?,
            Instruction::PushVal(val) => self.push_stack(val)?,
            Instruction::PopReg(register) => self.pop_stack(register)?,
            Instruction::AddRegReg(register, register1) => {
                self.registers[register] += self.registers.get(register1)
            }

            Instruction::AddRegVal(register, val) => self.registers[register] += val,
            Instruction::Jump(address) => self.registers[Register::IP] = address.into(),
            Instruction::Load(register, address) => {
                self.registers[register] = self.memory.read(address)?
            }
            Instruction::Call => todo!(),
            Instruction::CallAddr(_) => todo!(),
            Instruction::Halt => return Ok(ControlFlow::Break(())),
            Instruction::Ret => {} // where should the return address be
        }
        Ok(ControlFlow::Continue(()))
    }
}
