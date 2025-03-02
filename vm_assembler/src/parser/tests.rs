#[cfg(test)]
mod test {

    use vm_cpu::{
        memory::{self, Memory},
        registers::Register,
        stack::Stack,
    };

    use crate::parser::{self, Parser};

    #[derive(Debug)]
    enum Error {
        ParseError(parser::ParseError),
        VmError(vm_cpu::error::Error),
    }

    impl From<memory::Error> for Error {
        fn from(value: memory::Error) -> Self {
            Self::VmError(vm_cpu::error::Error::MemError(value))
        }
    }

    impl From<parser::ParseError> for Error {
        fn from(value: parser::ParseError) -> Self {
            Self::ParseError(value)
        }
    }

    impl From<vm_cpu::error::Error> for Error {
        fn from(value: vm_cpu::error::Error) -> Self {
            Self::VmError(value)
        }
    }

    #[test]
    fn basic_asm() -> Result<(), Error> {
        let asm = r#"
        mov r1, r1
        mov r2, 40 
        mov r3, 40 
        mov r4, 40 
        push 10
        load r1, 57535 
        halt
        "#;

        let parser: Parser = asm.parse()?;
        println!("parser {:#?}", parser.tokens());
        //let mut mem = Stack::<65535>::new();
        //parser.write_instructions_to_memory(&mut mem)?;
        //let mut cpu = vm_cpu::cpu::Cpu::new(mem, 0, u16::MAX - 8000);
        //println!("instructions: {:?}", parser.symbols());
        //println!("memory {:?}", cpu.memory().get(0, 20));
        //cpu.execute();
        //
        //println!("reg {:?}", cpu.registers());
        //assert!(
        //    cpu.registers().as_slice()
        //        == &[
        //            18,
        //            10,
        //            40,
        //            40,
        //            40,
        //            0,
        //            0,
        //            0,
        //            0,
        //            cpu.registers()[Register::SP]
        //        ]
        //);
        Ok(())
    }

    #[test]
    fn label_test() -> Result<(), Error> {
        let asm = r#"
        bai:
        load r1, 57535 
        halt
        hai:
        load r1, 200
        "#;

        let parser: Parser = asm.parse()?;
        //println!("insts {:?}", parser.symbols());
        //let mut stack = Stack::<{ u16::MAX as usize }>::new();
        //parser.write_instructions_to_memory(&mut stack)?;
        //let mut cpu = vm_cpu::cpu::Cpu::new(stack, 0, u16::MAX);
        //println!("mem {:?}", cpu.memory().get(0, 20)?);
        //cpu.execute();
        //println!("{:?}", cpu.registers());
        Ok(())
    }
}
