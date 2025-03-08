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
        mov r1, 40
        mov r2, 40 
        mov r3, 40 
        mov r4, 40 
        halt
        "#;

        let parser: Parser = asm.parse()?;
        println!("parser {:?}", parser.insts());
        let mut mem = Stack::<65535>::new();
        parser::parser::write_instructions_to_memory(&mut mem, parser.insts())?;
        let mut cpu = vm_cpu::cpu::Cpu::new(mem, 0, u16::MAX - 8000);
        println!("instructions: {:?}", parser.insts());
        println!("memory {:?}", cpu.memory().get(0, 20));
        cpu.execute();

        println!("reg {:?}", cpu.registers());
        assert!(
            cpu.registers().as_slice()
                == &[
                    cpu.registers()[Register::IP],
                    40,
                    40,
                    40,
                    40,
                    0,
                    0,
                    0,
                    0,
                    cpu.registers()[Register::SP]
                ]
        );
        Ok(())
    }

    #[test]
    fn label() -> Result<(), Error> {
        let asm = r#"
        bai:
        load r1, 57535 
        halt
        load r1, 200
        call bai
        "#;

        let parser: Parser = asm.parse()?;
        println!("insts {:?}", parser.insts());
        let mut stack = Stack::<{ u16::MAX as usize }>::new();
        parser::parser::write_instructions_to_memory(&mut stack, parser.insts())?;
        let mut cpu = vm_cpu::cpu::Cpu::new(stack, 0, u16::MAX);
        println!("mem {:?}", cpu.memory().get(0, 20)?);
        cpu.execute();
        println!("{:?}", cpu.registers());
        Ok(())
    }
}
