mod parser;
use parser::{ParseError, Parser};

use vm_cpu::{
    address::Address,
    error::Error,
    memory::{self, Memory},
    stack::Stack,
};

fn main() -> Result<(), Error> {
    //let asm = r#"
    //    mov r1, 40
    //    mov r1, r2
    //    push 10
    //    pop r3
    //    halt
    //    "#;
    //
    //let mut mem = Stack::<65535>::new();
    //
    //let parser = Parser::new(asm).parse().unwrap();
    //
    //let symbols = parser.symbols();
    //parser.write_instructions_to_memory(&mut mem)?;
    //println!("{symbols:?}");
    //
    //let mut cpu = vm_cpu::cpu::Cpu::new(mem, 0, u16::MAX - 8000);
    //println!("{:?}", cpu.registers());
    //cpu.execute();
    //println!("{:?}", cpu.memory().get(0, 20));
    //let sp = cpu.registers().get(vm_cpu::registers::Register::SP);
    //println!(
    //    "{:?}",
    //    cpu.memory().get(Address::from(sp), Address::from(u16::MAX))
    //);
    //println!("{:?}", cpu.registers());
    Ok(())
}
