use std::{fmt::Debug, ops::ControlFlow};

use crate::{
    address::Address,
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
        //println!("starting at {start_amount}");
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
            OpCode::PushVal => {
                //println!("pushing val {bytecode:?}");
                let val = bytecode[1];
                Ok(Instruction::PushVal(val))
            }
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

                Ok(Instruction::AddRegNum(left, right))
            }
            OpCode::PopReg => {
                let reg = bytecode[1];
                let reg = Register::try_from(reg).unwrap();

                Ok(Instruction::PopReg(reg))
            }
            OpCode::Jump => {
                let code = bytecode[1];
                Ok(Instruction::Jump(code.into()))
            }
            OpCode::Call => Ok(Instruction::Call),
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
            self.step()?
        }
    }

    pub fn write_instruction_to_memory(&mut self, inst: &Instruction) -> stack::Result<()> {
        let ip = self.registers.get(Register::IP);
        let mut ip_addr = Address::from(ip);
        self.memory.write(ip_addr, OpCode::from(inst))?;
        ip_addr = ip_addr.next()?;
        //println!("idx {ip_addr}");
        match inst {
            Instruction::MovRegReg(register, register1) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, self.registers.get(*register1))?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::MovRegVal(register, val) => {
                self.memory.write(ip_addr, *register)?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, *val)?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::PushReg(register) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::PushVal(val) => {
                self.memory.write(ip_addr, *val)?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::PopReg(register) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::AddRegReg(register, register1) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, self.registers.get(*register1))?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::AddRegNum(register, val) => {
                self.memory.write(ip_addr, self.registers.get(*register))?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, *val)?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::Jump(address) => {
                self.memory.write(ip_addr, *address)?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::Load(register, address) => {
                self.memory.write(ip_addr, *register)?;
                ip_addr = ip_addr.next()?;
                self.memory.write(ip_addr, *address)?;
                ip_addr = ip_addr.next()?;
            }
            Instruction::Call => {}
            Instruction::Halt => {}
            Instruction::Ret => {}
        }
        //println!("ip {:?}", self.registers[Register::IP]..ip_addr.into());
        self.registers[Register::IP] += OpCode::from(inst).increment_amount();
        //let ip_addr: u16 = ip_addr.into();
        //println!("memory {:?}", self.memory.get(0, ip_addr));
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
        println!("attempting to write {val} to {sp}");
        self.memory.write(Address::from(sp), val)?;
        println!("sp = {sp}");
        self.registers[Register::SP] = sp + 1;
        Ok(())
    }

    fn pop_stack(&mut self, reg: Register) -> stack::Result<()> {
        let sp = self.registers().get(Register::SP);
        self.registers.set(reg, self.memory.read(sp)?);
        self.registers.set(Register::SP, sp - 1);
        Ok(())
    }

    pub fn step(&mut self) -> ControlFlow<(), ()> {
        let byte = self
            .memory
            .read(self.registers().get(Register::IP))
            .unwrap();
        let code = OpCode::try_from(byte).unwrap();
        let ip = self.registers().get(Register::IP);
        let bytecode = self
            .memory
            .get(ip + 1, ip + code.increment_amount())
            .unwrap();
        match code {
            OpCode::MovRegReg => todo!(),
            OpCode::MovRegVal => {
                let (reg, val) = (Register::try_from(bytecode[0]), bytecode[1]);
                let reg = reg.unwrap();
                self.registers[reg] = val;
            }
            OpCode::AddRegReg => todo!(),
            OpCode::AddRegNum => todo!(),
            OpCode::Jump => todo!(),
            OpCode::PopReg => todo!(),
            OpCode::Call => todo!(),
            OpCode::Halt => return ControlFlow::Break(()),
            OpCode::Ret => return ControlFlow::Break(()),
            OpCode::Load => {
                println!("bytecode {bytecode:?}");
                let (reg, addr) = (Register::try_from(bytecode[0]), bytecode[1]);
                let reg = reg.unwrap();
                let val = self.memory.read(addr).unwrap();
                self.registers[reg] = val;
            }
            OpCode::PushReg => todo!(),
            OpCode::PushVal => {
                let val = bytecode[0];
                self.push_stack(val).unwrap();
            }
        };
        let amount = self.registers.get(Register::IP) + code.increment_amount();
        self.registers.set(Register::IP, amount);
        ControlFlow::Continue(())
    }
}
