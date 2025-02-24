use parser::Parser;
use vm_cpu::cpu::*;

mod parser;

fn main() {
    let mut cpu = Cpu::<4096>::new();
    let parser = Parser::new(
        r#"
        mov r1, 40 
        mov r1, r2
        push 10
        pop r3
        halt
        "#,
    )
    .parse()
    .unwrap();
    let insts = parser.insts();
    println!("{insts:?}");
    println!("{:?}", cpu.registers());
    cpu.execute(insts);
    println!("{:?}", cpu.memory_mut().get(0..20));
    let sp = cpu.registers().get(vm_cpu::registers::Register::SP) as usize;
    println!("{:?}", cpu.memory_mut().get(sp..));
    println!("{:?}", cpu.registers());
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
        let parser = Parser::new(asm).parse().unwrap();
        let mut cpu = vm_cpu::cpu::Cpu::<4096>::new();
        if cpu.write_instructions_to_memory(parser.insts()).is_err() {
            panic!()
        }
        //println!("{:?}", parser.insts());
        println!("memory {:?}", cpu.memory_mut().get(0..=18));
        cpu.execute(parser.insts());
        println!("{:?}", cpu.registers());
        assert!(cpu.registers().into_slice() == &[19, 35, 40, 40, 40, 65535]);
        //println!("{asm}");
    }
}
