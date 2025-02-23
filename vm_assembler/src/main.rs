use vm_cpu::cpu::*;
use vm_cpu::opcodes::*;
use vm_cpu::registers::*;

mod parser;

fn main() {
    let mut cpu = Cpu::<4096>::new();
    let reg: u16 = Register::R1.into();
    let mut start = 0;
    for i in 0..=4 {
        cpu.memory_mut().write(i + start, OpCode::PushRegVal.into());
        cpu.memory_mut().write(i + start + 1, reg + i);
        cpu.memory_mut().write(i + start + 2, 40);
        println!(
            "writing to {:?} {:?}",
            i + start..=i + start + 2,
            Register::try_from(reg + i)
        );
        start += 3;
    }
    let program_end = 20;
    cpu.memory_mut().write(program_end, OpCode::Halt.into());
    let insts = cpu.parse();
    cpu.execute(insts);
    println!("{}", OpCode::PushRegReg.increment_amount());
    println!("{:?}", cpu.registers());
    println!("{:?}", cpu.memory_mut().get(0..program_end as usize));
}

#[cfg(test)]
mod test {
    use crate::parser::Parser;

    #[test]
    fn basic_asm() {
        let asm = r#"
        mov r1, 40 
        mov r2, 40 
        mov r3, 40 
        mov r4, 40 
        load r1, 0
        mov r1, 35
        ret
        "#;
        let parser = Parser::new(asm).parse();
        let mut cpu = vm_cpu::cpu::Cpu::<4096>::new();
        cpu.write_instructions_to_memory(parser.insts().clone());
        //println!("{:?}", parser.insts());
        println!("memory {:?}", cpu.memory_mut().get(0..=18));
        cpu.execute(parser.insts().clone());
        println!("{:?}", cpu.registers());
        assert!(cpu.registers().into_slice() == &[19, 35, 40, 40, 40, 65535]);
        //println!("{asm}");
    }
}
