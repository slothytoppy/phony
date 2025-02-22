pub mod cpu;
pub mod opcodes;
pub mod registers;
pub mod stack;

//fn main() {
//    let mut cpu = Cpu::<4096>::new();
//    let reg: u16 = registers::Register::R1.into();
//    let mut start = 0;
//    for i in 0..=4 {
//        cpu.memory_mut().write(i + start, OpCode::PushRegVal.into());
//        cpu.memory_mut().write(i + start + 1, reg + i);
//        cpu.memory_mut().write(i + start + 2, 40);
//        println!(
//            "writing to {:?} {:?}",
//            i + start..=i + start + 2,
//            registers::Register::try_from(reg + i)
//        );
//        start += 3;
//    }
//    let program_end = 20;
//    cpu.memory_mut().write(program_end, OpCode::Halt.into());
//    let insts = cpu.parse();
//    cpu.execute(insts);
//    println!("{:?}", cpu.registers());
//    println!("{:?}", cpu.memory_mut().get(0..14));
//    cpu.dump();
//}

#[cfg(test)]
mod tests {
    use crate::cpu::Cpu;

    #[test]
    fn push_reg_val() {
        let _cpu = Cpu::<{ u16::MAX as usize }>::new();
    }
}
