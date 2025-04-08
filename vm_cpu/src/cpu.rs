use core::panic;
use std::{fmt::Debug, ops::ControlFlow};

use tracing::{info, instrument, trace};

use crate::{
    error::Error,
    memory::{mem, Address, Memory},
    opcodes::{Instruction, OpCode, Value},
    registers::{Register, Registers, WordSize},
    stack::{self},
};

#[derive(Debug)]
pub struct Cpu<M: Memory> {
    registers: Registers,
    memory: M,
}

impl<M: Memory + Debug> Cpu<M> {
    pub fn new(memory: M, program_start: u32, stack_start: u32) -> Self {
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

    #[instrument(skip(self))]
    fn next_instruction(&mut self, op: OpCode) -> Result<Instruction, Error> {
        let ip = self.registers[Register::IP];

        trace!(ip = ip, ?op, "opcode inc amount {}", op.increment_amount());

        let bytecode = self.memory.get(
            Address::from(ip),
            Address::from(ip + op.increment_amount() as u32),
        )?;

        self.registers[Register::IP] += op.increment_amount() as u32;

        trace!(ip = self.registers[Register::IP]);

        trace!(?bytecode);

        Instruction::try_from(bytecode)
    }

    pub fn execute(&mut self) {
        loop {
            match self.step() {
                Ok(flow) => match flow {
                    ControlFlow::Continue(_) => continue,
                    ControlFlow::Break(_) => return,
                },
                Err(e) => panic!("{e:?}"),
            };
        }
    }

    #[instrument(skip(self))]
    pub fn step(&mut self) -> Result<ControlFlow<(), ()>, Error> {
        let byte = self.memory.read(self.registers[Register::IP])?;

        let code = OpCode::try_from(byte)?;

        match self.next_instruction(code)? {
            Instruction::MovRegReg(register, register1) => {
                self.registers[register] = self.registers[register1]
            }
            Instruction::MovRegNum(register, val) => {
                let val = match val {
                    Value::U8(val) => val as u32,
                    Value::U16(val) => val as u32,
                    Value::U32(val) => val,
                };
                self.registers[register] = val
            }
            Instruction::PushReg(register) => self.push_stack(self.registers[register])?,
            Instruction::PushVal(val) => {
                let val = match val {
                    Value::U8(val) => val as u32,
                    Value::U16(val) => val as u32,
                    Value::U32(val) => val,
                };
                self.push_stack(val)?
            }
            Instruction::PopReg(register) => self.pop_stack(register)?,
            Instruction::AddRegReg(register, register1) => {
                self.registers[register] += self.registers[register1]
            }
            Instruction::AddRegNum(register, val) => match val {
                Value::U8(val) => self.registers[register] += val as u32,
                Value::U16(val) => self.registers[register] += val as u32,
                Value::U32(val) => self.registers[register] += val,
            },
            Instruction::Jump(address) => self.registers[Register::IP] = address.into(),
            Instruction::Load(register, address) => {
                self.registers[register] = self.memory.read(address)?.into()
            }
            Instruction::Halt => return Ok(ControlFlow::Break(())),

            Instruction::Call(_addr) => {
                todo!()
            }
            Instruction::Ret => {
                todo!()
            }

            Instruction::MovRegMem(_register, _address) => todo!(),
            Instruction::MovMemMem(_address, _address1) => todo!(),
            Instruction::MovMemReg(_address, _register) => todo!(),
            Instruction::MovMemVal(_address, _value) => todo!(),
            Instruction::AddRegMem(_register, _address) => todo!(),
            Instruction::IncReg(register) => {
                self.registers[register] += 1;
            }
            Instruction::IncMem(_address) => todo!(),
            Instruction::PushMem(_address) => todo!(),
            Instruction::Interrupt(_value) => todo!(),
            Instruction::InterruptReg(_register) => todo!(),
            Instruction::StoreReg(address, register) => {
                let num = self.registers[register];
                let upper = num.upper();
                let lower = num.lower();

                self.memory.write(address, lower.lower())?;
                self.memory.write(address, lower.upper())?;

                self.memory.write(address, upper.lower())?;
                self.memory.write(address, upper.upper())?;
            }
            Instruction::StoreVal(address, value) => match value {
                Value::U8(num) => self.memory.write(address, num)?,
                Value::U16(num) => {
                    self.memory.write(address, num.lower())?;
                    self.memory.write(address, num.upper())?;
                }
                Value::U32(num) => {
                    let upper = num.upper();
                    let lower = num.lower();

                    self.memory.write(address, lower.lower())?;
                    self.memory.write(address, lower.upper())?;

                    self.memory.write(address, upper.lower())?;
                    self.memory.write(address, upper.upper())?;
                }
            },
        }
        Ok(ControlFlow::Continue(()))
    }

    pub fn write_instructions_to_memory(&mut self, insts: &[Instruction]) -> mem::Result<()> {
        for inst in insts {
            let ip = self.registers[Register::IP];
            let mut ip_addr = Address::from(ip);
            self.memory.write(ip_addr, OpCode::from(inst))?;
            ip_addr = ip_addr.next()?;
            match inst {
                Instruction::MovRegReg(register, register1) => {
                    let reg_val = self.registers.get(*register);
                    let reg1_val = self.registers.get(*register1);
                    info!("{} {} {}", reg_val, reg_val >> 8, reg_val << 8);
                    self.memory.write(ip_addr, reg_val as u8)?;
                    ip_addr = ip_addr.next()?;
                    self.memory.write(ip_addr, reg1_val as u8)?;
                    _ = ip_addr.next()?;
                }
                Instruction::MovRegNum(register, val) => {
                    self.memory.write(ip_addr, *register)?;
                    ip_addr = ip_addr.next()?;
                    match val {
                        Value::U8(val) => self.memory.write(ip_addr, *val)?,
                        Value::U16(_) => todo!(),
                        Value::U32(_) => todo!(),
                    }
                    _ = ip_addr.next()?;
                }
                Instruction::PushReg(register) => {
                    self.memory
                        .write(ip_addr, self.registers.get(*register) as u8)?;
                    _ = ip_addr.next()?;
                }
                Instruction::PushVal(val) => {
                    match val {
                        Value::U8(val) => self.memory.write(ip_addr, *val)?,
                        Value::U16(_) => todo!(),
                        Value::U32(_) => todo!(),
                    }
                    _ = ip_addr.next()?;
                }
                Instruction::PopReg(register) => {
                    self.memory
                        .write(ip_addr, self.registers.get(*register) as u8)?;
                    _ = ip_addr.next()?;
                }
                Instruction::AddRegReg(register, register1) => {
                    self.memory
                        .write(ip_addr, self.registers.get(*register) as u8)?;
                    ip_addr = ip_addr.next()?;
                    self.memory
                        .write(ip_addr, self.registers.get(*register1) as u8)?;
                    _ = ip_addr.next()?;
                }
                Instruction::AddRegNum(register, val) => {
                    self.memory
                        .write(ip_addr, self.registers.get(*register) as u8)?;
                    ip_addr = ip_addr.next()?;
                    match val {
                        Value::U8(val) => self.memory.write(ip_addr, *val)?,
                        Value::U16(_) => todo!(),
                        Value::U32(_) => todo!(),
                    }
                    _ = ip_addr.next()?;
                }
                Instruction::Jump(address) => {
                    self.memory.write(ip_addr, u8::from(*address))?;
                    _ = ip_addr.next()?;
                }
                Instruction::Load(_register, _address) => {
                    todo!()
                    //self.memory.write(ip_addr, *register)?;
                    //ip_addr = ip_addr.next()?;
                    //self.memory.write(ip_addr, u8::from(*address))?;
                    //_ = ip_addr.next()?;
                }
                Instruction::Call(address) => {
                    self.memory.write(ip_addr, u8::from(*address))?;
                    ip_addr.next()?;
                    todo!()
                }
                Instruction::Halt => {}
                Instruction::Ret => {}
                Instruction::MovRegMem(_register, _address) => todo!(),
                Instruction::MovMemMem(_address, _address1) => todo!(),
                Instruction::MovMemReg(_address, _register) => todo!(),
                Instruction::MovMemVal(_address, _value) => todo!(),
                Instruction::AddRegMem(_register, _address) => todo!(),
                Instruction::IncReg(_register) => todo!(),
                Instruction::IncMem(_address) => todo!(),
                Instruction::PushMem(_address) => todo!(),
                Instruction::Interrupt(_value) => todo!(),
                Instruction::InterruptReg(_register) => todo!(),
                Instruction::StoreReg(_address, _register) => todo!(),
                Instruction::StoreVal(_address, _value) => todo!(),
            }
            self.registers[Register::IP] += OpCode::from(inst).increment_amount() as u32;
        }
        self.registers[Register::IP] = 0;
        Ok(())
    }

    fn push_stack(&mut self, val: u32) -> stack::Result<()> {
        let sp = self.registers()[Register::SP];
        self.memory.write(Address::from(sp), val as u8)?;
        self.registers[Register::SP] = sp - 1;
        Ok(())
    }

    fn pop_stack(&mut self, reg: Register) -> stack::Result<()> {
        let sp = self.registers()[Register::SP];
        self.registers.set(reg, self.memory.read(sp)? as u32);
        self.registers.set(Register::SP, sp + 1);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::ops::ControlFlow;

    use tracing::info;
    use tracing::level_filters::LevelFilter;
    use tracing::trace;
    use tracing_subscriber::util::SubscriberInitExt;

    use crate::{
        memory::{Address, Memory},
        opcodes::OpCode,
        registers::Register,
        stack::Stack,
    };

    use super::Cpu;

    #[test]
    fn step() {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(LevelFilter::TRACE)
            .finish()
            .try_init();

        let mut mem = Stack::<{ u16::MAX as usize }>::new();

        mem.write(0, OpCode::PushU32).unwrap();
        mem.write(1, 10).unwrap();
        mem.write(2, 0).unwrap();
        mem.write(3, 0).unwrap();
        mem.write(4, 0).unwrap();
        mem.write(5, OpCode::Halt).unwrap();

        let mut cpu = Cpu::new(mem, 0, u16::MAX as u32);

        cpu.step().expect("step failed");
        trace!("{}", cpu.registers);
        info!("{}", cpu.registers);
        info!(
            "memory at IP {:?}",
            cpu.memory()
                .read(Address::from(cpu.registers[Register::IP]))
        );
        info!("memory {:?}", cpu.memory.get(Address::from(0), 10.into()));
        let res = cpu.step().expect("step failed");

        assert!(res == ControlFlow::<(), ()>::Break(()))
    }
}
