mod cpu;
mod opcodes;
mod registers;
mod stack;

use cpu::Cpu;
use opcodes::*;

fn main() {
    let mut cpu = Cpu::<4096>::new();
    cpu.memory_mut().write(0, OpCode::PushRegVal.into());
    cpu.memory_mut().write(1, registers::Register::R3.into());
    cpu.memory_mut().write(2, 40);
    cpu.memory_mut().write(3, OpCode::Halt.into());
    let insts = cpu.parse();
    cpu.execute(insts);
    println!("{:?}", cpu.registers());
    //cpu.dump();
}

#[cfg(test)]
mod tests {
    use crate::cpu::Cpu;

    #[test]
    fn push_reg_val() {
        let _cpu = Cpu::<{ u16::MAX as usize }>::new();
    }
}
