#[cfg(test)]
mod test {

    use vm_cpu::{
        memory::{self, Memory},
        registers::Register,
        stack::Stack,
    };

    use crate::error::ParseError;
    use crate::parser::Parser;

    #[derive(Debug)]
    enum Error {
        ParseError(()),
        VmError(()),
    }

    impl From<memory::Error> for Error {
        fn from(_: memory::Error) -> Self {
            Self::VmError(())
        }
    }

    impl From<ParseError> for Error {
        fn from(_: ParseError) -> Self {
            Self::ParseError(())
        }
    }

    impl From<vm_cpu::error::Error> for Error {
        fn from(_: vm_cpu::error::Error) -> Self {
            Self::VmError(())
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
        let mem = Stack::<65535>::new();
        let mut cpu = vm_cpu::cpu::Cpu::new(mem, 0, (u16::MAX - 8000) as u32, 0.into());
        // write_instructions_to_memory(parser.insts())?;
        println!("instructions: {:?}", parser.insts());
        println!("memory {:?}", cpu.memory().get(0_u16.into()..20_u16.into()));
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
                    0,
                    cpu.registers()[Register::SP]
                ]
        );
        Ok(())
    }

    #[test]
    fn label() -> Result<(), Error> {
        // let asm = r#"
        // bai:
        // load r1, 57535
        // halt
        // load r1, 200
        // call bai
        // "#;

        // let parser: Parser = asm.parse()?;
        // println!("insts {:?}", parser.insts());
        // let stack = Stack::<{ u16::MAX as usize }>::new();
        // let mut cpu = vm_cpu::cpu::Cpu::new(stack, 0, u16::MAX as u32, 0.into());
        // cpu.write_instructions_to_memory(parser.insts())?;
        // todo!();
        // println!("mem {:?}", cpu.memory().get(0_u16.into()..20_u16.into())?);
        // cpu.execute();
        // println!("{:?}", cpu.registers());
        Ok(())
    }
}
