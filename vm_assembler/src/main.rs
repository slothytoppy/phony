use parser::Parser;
use vm_cpu::{
    address::Address,
    memory::{self, Memory},
    stack::Stack,
};

mod parser;

fn main() -> Result<(), memory::Error> {
    let mem = Stack::<65535>::new();
    let mut cpu = vm_cpu::cpu::Cpu::new(mem, 0, u16::MAX - 8000);
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
    cpu.write_instructions_to_memory(insts)?;
    println!("{insts:?}");
    println!("{:?}", cpu.registers());
    cpu.execute();
    println!("{:?}", cpu.memory().get(0, 20));
    let sp = cpu.registers().get(vm_cpu::registers::Register::SP);
    println!(
        "{:?}",
        cpu.memory().get(Address::from(sp), Address::from(u16::MAX))
    );
    println!("{:?}", cpu.registers());
    Ok(())
}

#[cfg(test)]
mod test {
    use vm_cpu::{
        memory::{Error, Memory},
        registers::Register,
        stack::Stack,
    };

    use crate::parser::Parser;

    #[test]
    fn basic_asm() -> Result<(), Error> {
        let asm = r#"
        mov r1, 40 
        mov r2, 40 
        mov r3, 40 
        mov r4, 40 
        push 10
        load r1, 57535 
        halt
        "#;

        let parser = Parser::new(asm).parse().unwrap();
        let mem = Stack::<65535>::new();
        let mut cpu = vm_cpu::cpu::Cpu::new(mem, 0, u16::MAX - 8000);
        println!("instructions: {:?}", parser.insts());
        cpu.write_instructions_to_memory(parser.insts())?;
        println!("memory {:?}", cpu.memory().get(0, 20));
        cpu.execute();

        println!("reg {:?}", cpu.registers());
        assert!(cpu.registers().as_slice() == &[18, 10, 40, 40, 40, cpu.registers()[Register::SP]]);
        Ok(())
    }
}
