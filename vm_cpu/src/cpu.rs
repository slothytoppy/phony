use std::{fmt::Debug, ops::ControlFlow};
use tracing::{info, instrument, trace, warn};

use crate::{
    error::Error,
    memory::{self, Address, CpuMemory},
    opcodes::{Comparison, Instruction, OpCode, Value},
    registers::{Register, Registers},
};

#[derive(Debug)]
pub struct Flags {
    cmp: Comparison,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            cmp: Comparison::Eq,
        }
    }
}

#[derive(Default, Debug)]
pub struct Cpu {
    flags: Flags,
    registers: Registers,
    interrupt_table: Address,
    in_interrupt: bool,
    program_start: Address,
    memory: CpuMemory,
}

impl Cpu {
    pub fn new(
        memory: CpuMemory,
        program_start: u32,
        stack_start: u32,
        interrupt_table: Address,
    ) -> Self {
        Self {
            memory,
            registers: Registers::new(program_start, stack_start),
            interrupt_table,
            in_interrupt: false,
            program_start: program_start.into(),
            flags: Flags::default(),
        }
    }

    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    pub fn memory(&self) -> &CpuMemory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut CpuMemory {
        &mut self.memory
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

    fn fetch_instruction(&mut self) -> Result<Instruction, Error> {
        let ip = self.registers[Register::IP];
        let byte = self.memory.read(ip)?;
        let op = OpCode::try_from(byte)?;

        let ip = ip + 1; // to skip the opcode and only deal with bytecodes
        let bytecode = self
            .memory
            .get(ip.into()..(ip + op.increment_amount() as u32).into())?;

        Ok(match op {
            OpCode::MovRegMem => {
                let reg = Register::try_from(bytecode[0])?;
                let addr = u32::from_le_bytes([bytecode[1], bytecode[2], bytecode[3], bytecode[4]]);

                Instruction::MovRegMem(reg, addr.into())
            }
            OpCode::MovRegReg => {
                let left = Register::try_from(bytecode[0])?;

                let right = Register::try_from(bytecode[1])?;

                Instruction::MovRegReg(left, right)
            }
            OpCode::MovRegU8 => {
                let left = Register::try_from(bytecode[0])?;

                let right = bytecode[1];

                Instruction::MovRegNum(left, Value::U8(right))
            }
            OpCode::MovRegU16 => {
                let left = Register::try_from(bytecode[0])?;

                let right = u16::from_le_bytes([bytecode[1], bytecode[2]]);

                Instruction::MovRegNum(left, Value::U16(right))
            }
            OpCode::MovRegU32 => {
                let left = Register::try_from(bytecode[0])?;

                let right =
                    u32::from_le_bytes([bytecode[1], bytecode[2], bytecode[3], bytecode[4]]);

                Instruction::MovRegNum(left, Value::U32(right))
            }

            OpCode::MovMemReg => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);
                let reg = Register::try_from(bytecode[4])?;

                Instruction::MovMemReg(addr.into(), reg)
            }
            OpCode::MovMemU8 => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::MovMemNum(addr.into(), Value::U8(bytecode[4]))
            }
            OpCode::MovMemU16 => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);
                let val = u16::from_le_bytes([bytecode[4], bytecode[5]]);

                Instruction::MovMemNum(addr.into(), Value::U16(val))
            }
            OpCode::MovMemU32 => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);
                let val = u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]);

                Instruction::MovMemNum(addr.into(), Value::U32(val))
            }

            OpCode::AddRegReg => {
                let left = Register::try_from(bytecode[0])?;
                let right = Register::try_from(bytecode[1])?;

                Instruction::AddRegReg(left, right)
            }
            OpCode::AddRegMem => {
                let reg = Register::try_from(bytecode[0])?;
                let addr = u32::from_le_bytes([bytecode[1], bytecode[2], bytecode[3], bytecode[4]]);

                Instruction::AddRegMem(reg, addr.into())
            }
            OpCode::AddMemReg => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);
                let reg = Register::try_from(bytecode[4])?;

                Instruction::AddMemReg(addr.into(), reg)
            }
            OpCode::AddRegU8 => {
                let left = Register::try_from(bytecode[0])?;

                let right = bytecode[1];

                Instruction::AddRegNum(left, Value::U8(right))
            }
            OpCode::AddRegU16 => {
                let left = Register::try_from(bytecode[0])?;

                let right = u16::from_le_bytes([bytecode[1], bytecode[2]]);

                Instruction::AddRegNum(left, Value::U16(right))
            }
            OpCode::AddRegU32 => {
                let left = Register::try_from(bytecode[0])?;

                let right =
                    u32::from_le_bytes([bytecode[1], bytecode[2], bytecode[3], bytecode[4]]);

                Instruction::AddRegNum(left, Value::U32(right))
            }

            OpCode::IncReg => {
                let reg = Register::try_from(bytecode[0])?;

                Instruction::IncReg(reg)
            }
            OpCode::IncMem => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::IncMem(addr.into())
            }

            OpCode::PushReg => {
                let arg = Register::try_from(bytecode[0])?;
                Instruction::PushReg(arg)
            }
            OpCode::PushMem => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::PushMem(addr.into())
            }
            OpCode::PushU8 => {
                let val = Value::U8(bytecode[0]);

                Instruction::PushVal(val)
            }
            OpCode::PushU16 => {
                let val = u16::from_le_bytes([bytecode[0], bytecode[1]]);
                let val = Value::U16(val);

                Instruction::PushVal(val)
            }
            OpCode::PushU32 => {
                let val = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);
                let val = Value::U32(val);

                Instruction::PushVal(val)
            }

            OpCode::PopReg => {
                let reg = Register::try_from(bytecode[0])?;

                Instruction::PopReg(reg)
            }

            OpCode::CmpReg => Instruction::CmpReg(
                Register::try_from(bytecode[0])?,
                Register::try_from(bytecode[1])?,
            ),

            OpCode::CmpU8 => Instruction::CmpVal(Value::U8(bytecode[0]), Value::U8(bytecode[0])),
            OpCode::CmpU16 => Instruction::CmpVal(
                Value::U16(u16::from_le_bytes([bytecode[0], bytecode[1]])),
                Value::U16(u16::from_le_bytes([bytecode[2], bytecode[3]])),
            ),

            OpCode::CmpU32 => Instruction::CmpVal(
                Value::U32(u32::from_le_bytes([
                    bytecode[0],
                    bytecode[1],
                    bytecode[2],
                    bytecode[3],
                ])),
                Value::U32(u32::from_le_bytes([
                    bytecode[4],
                    bytecode[5],
                    bytecode[6],
                    bytecode[7],
                ])),
            ),

            OpCode::Jump => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::Jump(addr.into())
            }
            OpCode::JumpGe => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::JumpGe(addr.into())
            }
            OpCode::JumpGte => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::JumpGte(addr.into())
            }
            OpCode::JumpLe => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::JumpLt(addr.into())
            }
            OpCode::JumpLte => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::JumpLte(addr.into())
            }

            OpCode::Call => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                Instruction::Call(addr.into())
            }

            OpCode::Load => {
                let reg = Register::try_from(bytecode[0])?;
                let addr = u32::from_le_bytes([bytecode[1], bytecode[2], bytecode[3], bytecode[4]]);
                Instruction::Load(reg, addr.into())
            }

            OpCode::StoreReg => {
                let reg = Register::try_from(bytecode[0])?;
                let addr = u32::from_le_bytes([bytecode[1], bytecode[2], bytecode[3], bytecode[4]]);

                warn!(?reg, ?addr);

                Instruction::StoreReg(addr.into(), reg)
            }
            OpCode::StoreU8 => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);
                let val = Value::U8(bytecode[4]);

                Instruction::StoreVal(addr.into(), val)
            }
            OpCode::StoreU16 => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                let val = u16::from_le_bytes([bytecode[4], bytecode[5]]);
                let val = Value::U16(val);

                Instruction::StoreVal(addr.into(), val)
            }
            OpCode::StoreU32 => {
                let addr = u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]);

                let val = u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]);
                let val = Value::U32(val);

                Instruction::StoreVal(addr.into(), val)
            }

            OpCode::Interrupt => Instruction::Interrupt(u32::from_le_bytes([
                bytecode[0],
                bytecode[1],
                bytecode[2],
                bytecode[3],
            ])),
            OpCode::InterruptReg => Instruction::InterruptReg(Register::try_from(bytecode[0])?),

            OpCode::Halt => Instruction::Halt,
            OpCode::Ret => Instruction::Ret,
        })
    }

    fn execute_instruction(&mut self, inst: Instruction) -> Result<ControlFlow<(), ()>, Error> {
        match inst {
            Instruction::MovRegMem(register, address) => self
                .memory
                .write_u32(self.program_start + address, self.registers[register])?,
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

            Instruction::MovMemReg(address, register) => {
                self.registers[register] = self.memory.read_u32(self.program_start + address)?
            }
            Instruction::MovMemNum(address, val) => match val {
                Value::U8(val) => self.memory.write(self.program_start + address, val),
                Value::U16(val) => self.memory.write_u16(self.program_start + address, val),
                Value::U32(val) => self.memory.write_u32(self.program_start + address, val),
            }?,

            Instruction::AddRegReg(register, register1) => {
                self.registers[register] += self.registers[register1]
            }
            Instruction::AddRegNum(register, val) => match val {
                Value::U8(val) => self.registers[register] += val as u32,
                Value::U16(val) => self.registers[register] += val as u32,
                Value::U32(val) => self.registers[register] += val,
            },
            Instruction::AddRegMem(register, address) => {
                self.registers[register] += self.memory.read_u32(self.program_start + address)?
            }
            Instruction::AddMemReg(address, register) => {
                let val = self.memory.read_u32(address)?;
                self.memory
                    .write_u32(address, val + self.registers[register])?;
            }

            Instruction::IncReg(register) => {
                self.registers[register] += 1;
            }

            // should it incremement the address or what it points to?
            Instruction::IncMem(_address) => todo!(),

            Instruction::PushReg(register) => self.push_stack(self.registers[register])?,
            Instruction::PushMem(address) => {
                let val = self.memory.read_u32(self.program_start + address)?;

                self.push_stack(val)?
            }
            Instruction::PushVal(val) => {
                let val = match val {
                    Value::U8(val) => val as u32,
                    Value::U16(val) => val as u32,
                    Value::U32(val) => val,
                };
                self.push_stack(val)?
            }

            Instruction::PopReg(register) => {
                let val = self.pop_stack()?;
                self.registers[register] = val;
            }

            Instruction::CmpReg(reg, reg1) => {
                if reg == reg1 {
                    self.flags.cmp = Comparison::Eq
                }

                let val = self.registers[reg];
                let val1 = self.registers[reg1];

                if val > val1 {
                    self.flags.cmp = Comparison::Gt
                }

                if val < val1 {
                    self.flags.cmp = Comparison::Lt
                }
            }
            Instruction::CmpVal(val, val1) => {
                if val == val1 {
                    self.flags.cmp = Comparison::Eq
                }

                if val > val1 {
                    self.flags.cmp = Comparison::Gt
                }

                if val < val1 {
                    self.flags.cmp = Comparison::Lt
                }
            }

            Instruction::Jump(address) => {
                self.registers[Register::IP] = (self.program_start + address).into()
            }
            Instruction::JumpGe(address) => {
                if self.flags.cmp == Comparison::Gt {
                    self.registers[Register::IP] = (self.program_start + address).into()
                }
            }

            Instruction::JumpGte(address) => {
                if self.flags.cmp == Comparison::Gt || self.flags.cmp == Comparison::Gte {
                    self.registers[Register::IP] = (self.program_start + address).into()
                }
            }
            Instruction::JumpLt(address) => {
                if self.flags.cmp == Comparison::Lt {
                    self.registers[Register::IP] = (self.program_start + address).into()
                }
            }
            Instruction::JumpLte(address) => {
                if self.flags.cmp == Comparison::Lt || self.flags.cmp == Comparison::Lte {
                    self.registers[Register::IP] = (self.program_start + address).into()
                }
            }

            Instruction::Call(addr) => {
                self.registers[Register::SP] = self.registers[Register::IP];
                self.registers[Register::IP] = (self.program_start + addr).into();
                self.push_stack(self.registers[Register::R1])?;
                self.push_stack(self.registers[Register::R2])?;
                self.push_stack(self.registers[Register::R3])?;
                self.push_stack(self.registers[Register::R4])?;
                self.push_stack(self.registers[Register::IP])?;

                self.registers[Register::FP] = self.registers[Register::SP].saturating_sub(1);
            }

            Instruction::Load(register, address) => {
                self.registers[register] = self.memory.read_u32(self.program_start + address)?
            }

            Instruction::StoreReg(address, register) => {
                let num = self.registers[register];
                self.memory.write_u32(self.program_start + address, num)?
            }
            Instruction::StoreVal(address, bytecode) => match bytecode {
                Value::U8(num) => self.memory.write(self.program_start + address, num)?,
                Value::U16(num) => self.memory.write_u16(self.program_start + address, num)?,
                Value::U32(num) => self.memory.write_u32(self.program_start + address, num)?,
            },

            Instruction::Interrupt(idx) => self.handle_interrupt(idx)?,
            Instruction::InterruptReg(register) => {
                let idx = self.registers[register];
                self.handle_interrupt(idx)?
            }

            Instruction::Halt => return Ok(ControlFlow::Break(())),

            Instruction::Ret => self.restore_stack()?,
        }

        Ok(ControlFlow::Continue(()))
    }

    #[instrument(skip(self))]
    pub fn step(&mut self) -> Result<ControlFlow<(), ()>, Error> {
        let inst = self.fetch_instruction()?;

        let res = self.execute_instruction(inst);

        self.registers[Register::IP] += OpCode::from(inst).increment_amount() as u32;

        res
    }

    fn write_val(&mut self, address: Address, val: u32) -> Result<(), memory::Error> {
        trace!("writing to address {address} value {val}");

        let sp: Address = self.registers()[Register::SP].into();

        let bytes = val.to_le_bytes();

        for (i, byte) in bytes.iter().rev().enumerate() {
            info!(?byte, ?i);
            self.memory.write(sp - i.into(), *byte)?;
        }

        Ok(())
    }

    fn push_stack(&mut self, val: u32) -> Result<(), memory::Error> {
        let sp: Address = self.registers()[Register::SP].into();

        self.write_val(sp, val)?;

        let prev: u32 = sp.prev()?.into();
        self.registers[Register::SP] -= prev.saturating_sub(4);
        Ok(())
    }

    #[allow(unused)]
    fn read_mem_u8<A>(&self, address: A) -> Result<u8, memory::Error>
    where
        A: Into<Address>,
    {
        let address = address.into();

        let val = self.memory.read(address)?;

        Ok(val)
    }

    fn read_mem_u32<A>(&self, address: A) -> Result<u32, memory::Error>
    where
        A: Into<Address>,
    {
        let address = address.into();

        let l8bit = self.memory.read(address)?;
        let address = address.next()?;

        let h8bit = self.memory.read(address)?;
        let address = address.next()?;

        let address = address.next()?;
        let l16bit = self.memory.read(address)?;

        let h16bit = self.memory.read(address)?;

        let val = u32::from_le_bytes([l8bit, h8bit, l16bit, h16bit]);

        Ok(val)
    }

    fn pop_stack(&mut self) -> Result<u32, memory::Error> {
        let sp: Address = self.registers[Register::SP].into();

        let val = self.read_mem_u32(sp)?;

        let sp = sp.next()?;

        self.registers.set(Register::SP, sp.into());
        Ok(val)
    }

    fn save_stack(&mut self) -> Result<(), memory::Error> {
        self.push_stack(self.registers[Register::R1])?;
        self.push_stack(self.registers[Register::R2])?;
        self.push_stack(self.registers[Register::R3])?;
        self.push_stack(self.registers[Register::R4])?;
        self.push_stack(self.registers[Register::IP])?;

        let sp = self.registers[Register::SP];
        let fp = self.registers[Register::FP];

        let next_fp = sp.saturating_sub(1);
        let fp_size = fp - next_fp;

        self.memory.write_u32(Address::from(sp), fp_size)?;

        Ok(())
    }

    fn restore_stack(&mut self) -> Result<(), memory::Error> {
        let fp = self.registers[Register::FP];

        self.registers[Register::SP] = fp;

        let frame_size = self.pop_stack()?;
        let ip = self.pop_stack()?;
        let r4 = self.pop_stack()?;
        let r3 = self.pop_stack()?;
        let r2 = self.pop_stack()?;
        let r1 = self.pop_stack()?;

        self.registers[Register::IP] = ip;
        self.registers[Register::R4] = r4;
        self.registers[Register::R3] = r3;
        self.registers[Register::R2] = r2;
        self.registers[Register::R1] = r1;

        let prev_frame_ptr = fp + frame_size;
        self.registers[Register::FP] = prev_frame_ptr;

        Ok(())
    }

    fn handle_interrupt(&mut self, idx: u32) -> Result<(), memory::Error> {
        let ptr = self.interrupt_table + idx.into();

        if !self.in_interrupt {
            self.save_stack()?;
        }

        let fp = self.memory.read_u32(ptr)?;

        self.in_interrupt = true;
        self.registers[Register::IP] = fp;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::ops::ControlFlow;

    use tracing::{info, level_filters::LevelFilter, trace};
    use tracing_subscriber::util::SubscriberInitExt;

    use crate::{cpu::CpuMemory, memory::Address, opcodes::OpCode, registers::Register};

    use super::Cpu;

    fn setup_logger() {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(LevelFilter::INFO)
            .finish()
            .try_init();
    }

    fn setup_cpu(bytes: &[u8]) -> Cpu {
        let mut mem = super::CpuMemory::default();

        mem.write_bytes(0, bytes as &[u8]).unwrap();

        Cpu::new(mem, 0, u16::MAX as u32, 0.into())
    }

    #[test]
    fn mov_reg_mem() {}

    #[test]
    fn mov_reg_reg() {}
    #[test]
    fn mov_reg_num() {}

    #[test]
    fn mov_mem_reg() {}
    #[test]
    fn mov_mem_num() {}

    #[test]
    fn add_reg_reg() {}
    #[test]
    fn add_reg_num() {}
    #[test]
    fn add_reg_mem() {}
    #[test]
    fn add_mem_reg() {}

    #[test]
    fn inc_reg() {}
    #[test]
    fn inc_mem() {}

    #[test]
    fn push_reg() {
        setup_logger();

        let mut cpu = setup_cpu(&[
            OpCode::MovRegU8 as u8,
            Register::R1 as u8,
            10,
            OpCode::PushReg as u8,
            Register::R1 as u8,
            OpCode::Halt as u8,
        ]);

        cpu.execute();

        assert_eq!(cpu.read_mem_u8(u16::MAX as usize - 1).unwrap(), 10);
    }
    #[test]
    fn push_mem() {
        setup_logger();

        let mut mem = CpuMemory::default();

        let bytes = &[OpCode::PushMem as u8, 6, 0, 0, 0, OpCode::Halt as u8, 90];
        mem.write_bytes(0, bytes as &[u8]).unwrap();

        let mut cpu = Cpu::new(mem, 0, u16::MAX as u32, 0.into());

        tracing::info!("{:?}", cpu.memory.get(0.into()..10.into()));

        cpu.execute();
    }
    #[test]
    fn push_val() {}

    #[test]
    fn pop_reg() {}

    #[test]
    fn cmp_reg() {}

    #[test]
    fn cmp_val() {}

    #[test]
    fn jump() {}
    #[test]
    fn jump_ge() {}
    #[test]
    fn jump_gte() {}
    #[test]
    fn jump_lt() {}
    #[test]
    fn jump_lte() {}

    #[test]
    fn call() {}

    #[test]
    fn load() {}

    #[test]
    fn store_reg() {
        setup_logger();

        let mut cpu = setup_cpu(&[
            OpCode::MovRegU8 as u8,
            Register::R1 as u8,
            9,
            OpCode::StoreReg as u8,
            Register::R1 as u8,
            10,
            0,
            0,
            0,
            OpCode::Halt as u8,
        ]);

        cpu.execute();

        assert_eq!(cpu.memory.read_u32(10).unwrap(), 9);
    }

    #[test]
    fn store_val() {
        setup_logger();

        let mut cpu = setup_cpu(&[OpCode::StoreU8 as u8, 5, 0, 0, 0, 10]);
        trace!("{:?}", cpu.memory.get(Address::from(0)..10.into()));

        let _ = cpu.step().unwrap();

        trace!("{:?}", cpu.memory.get(Address::from(0)..10.into()));

        assert!(cpu.memory.read(5).unwrap() == 10);
    }

    #[test]
    fn interrupt() {
        setup_logger();

        let mut mem = CpuMemory::default();

        let bytes: &[u8] = &[
            OpCode::Interrupt as u8,
            0,
            0,
            0,
            1,
            OpCode::IncReg as u8,
            Register::R1 as u8,
            OpCode::Halt as u8,
        ];

        mem.write_bytes(0, bytes as &[u8]).unwrap();

        let mut cpu = Cpu::new(mem, 0, u16::MAX as u32, 10.into());

        for i in 0..3 {
            let hlt = cpu.step().unwrap();
            info!(?hlt);
            if i == 3 {
                assert!(hlt == ControlFlow::Break(()));
            }
        }
    }

    #[test]
    fn interrupt_reg() {
        setup_logger();

        let mut mem = CpuMemory::default();

        let bytes: &[u8] = &[
            OpCode::MovRegU8 as u8,
            Register::R1 as u8,
            6,
            OpCode::InterruptReg as u8,
            Register::R1 as u8,
            OpCode::IncReg as u8,
            Register::R1 as u8,
            OpCode::Halt as u8,
        ];

        mem.write_bytes(0, bytes as &[u8]).unwrap();

        let mut cpu = Cpu::new(mem, 0, u16::MAX as u32, 10.into());

        for i in 0..3 {
            let hlt = cpu.step().unwrap();
            info!(?hlt);
            if i == 3 {
                assert!(hlt == ControlFlow::Break(()));
            }
        }
    }

    #[test]
    fn halt() {
        setup_logger();

        let mut cpu = setup_cpu(&[OpCode::Halt as u8]);

        assert!(cpu.step().unwrap() == ControlFlow::Break(()));
    }

    #[test]
    fn ret() {
        todo!()
    }

    #[test]
    fn read_mem_u32() {
        setup_logger();

        let mut mem = CpuMemory::default();

        let bytes: &[u8] = &[6, 0, 0, 0];
        mem.write_bytes(0, bytes as &[u8]).unwrap();

        let cpu = Cpu::new(mem, 0, u16::MAX as u32, 0.into());

        let val = cpu.read_mem_u32(0).unwrap();

        assert!(val == 6)
    }

    // #[test]
    // fn mem_test() {
    //     setup_logger();
    //
    //     let mut mem = CpuMemory::default();
    //
    //     let addr = mem.alloc(10);
    //
    //     mem.memcpy(addr.unwrap(), None);
    // }
    //
    // #[test]
    // fn page_test() {
    //     setup_logger();
    //
    //     let mut mem = CpuMemory::default();
    //
    //     let addr = mem.alloc(4096);
    //     let addr1 = mem.alloc(4096);
    //
    //     info!(?addr, ?addr1);
    //
    //     mem.memcpy(addr.unwrap(), Some(&[1]));
    //     mem.memcpy(addr1.unwrap(), Some(&[2]));
    //
    //     info!(?mem);
    // }
    //
    // #[test]
    // fn pages_test() {
    //     setup_logger();
    //
    //     let mut mem = CpuMemory::default();
    //
    //     let addr = mem.alloc(4096);
    //     mem.memcpy(addr, Some(&[1; 4096]));
    //     let addr = mem.alloc(4095 * 3);
    //     let addr = mem.alloc(4096);
    //     let addr = mem.alloc(4096);
    //     let addr = mem.alloc(4096);
    //     let addr = mem.alloc(4096);
    //     let addr = mem.alloc(4096);
    //     let addr = mem.alloc(4096);
    //     let addr = mem.alloc(4096);
    //
    //     mem.memcpy(addr, Some(&[1; 40]));
    // }
}
