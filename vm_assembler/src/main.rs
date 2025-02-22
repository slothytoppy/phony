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
    //cpu.dump();
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
        ret
        "#;
        let parser = Parser::new(asm).parse();
        println!("{:?}", parser.insts());
        let mut cpu = vm_cpu::cpu::Cpu::<4096>::new();
        cpu.execute(parser.insts().clone());
        println!("{:?}", cpu.registers());
        //println!("{asm}");
        panic!();
    }
}
