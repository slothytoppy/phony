use std::ops::Index;

use vm_cpu::{opcodes::Instruction, registers::Register};

#[derive(Debug)]
pub struct Parser {
    data: String,
    insts: Vec<Instruction>,
}

#[derive(Debug)]
pub enum InvalidArgument {
    Register,
    Number,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidArgument(InvalidArgument),
    InvalidKeyWord,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

macro_rules! keywords {
    ($($variant:ident, $amount:ident = $arg_amount:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy)]
        #[repr(u8)]
        #[rustfmt::skip]
        pub enum KeyWord {
            $($variant),*
        }

        impl KeyWord {
            pub fn increment_amount(&self) -> u16 {
                match self {
                    $(KeyWord::$variant => $arg_amount,)*
                }
            }
        }

        impl std::fmt::Display for KeyWord{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                $(Self::$variant { .. } => f.write_str(stringify!($variant))?,)*
            }

            write!(f, ": {self:?}")
            }
        }
    }
}

keywords! {
    Mov, amount = 3,
    Add, amount = 3,
    Pop, amount = 2,
    Ret, amount = 1,
    Halt, amount = 1,
    Jump, amount = 2,
    Load, amount = 3,
    Push, amount = 2,
}

#[derive(Debug)]
enum Arg {
    Register(Register),
    U16(u16),
}

impl Parser {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.trim().to_string(),
            insts: Vec::new(),
        }
    }

    pub fn insts(&self) -> &Vec<Instruction> {
        &self.insts
    }

    pub fn parse(self) -> Result<Self, ParseError> {
        let mut clone = self;
        let mut words: Vec<&str> = Vec::new();
        let mut idx = 0;
        for line in clone.data.lines() {
            for word in line.split_whitespace() {
                words.push(word);
            }
        }
        loop {
            if idx >= words.len() {
                break;
            }
            let word = words[idx];
            let keyword = clone.get_keyword(word);
            println!("{word}");
            println!("{keyword:?}");
            let keyword = keyword?;
            let bytecode = &words.get(idx..idx + keyword.increment_amount() as usize);
            if let Some(code) = bytecode {
                let args = clone.get_args(&keyword, code);
                println!("keyword {keyword:?} args {args:?}");
                if let Some(args) = args {
                    let inst = clone.args_to_inst(keyword, &args);
                    match inst {
                        Some(inst) => clone.insts.push(inst),
                        None => continue,
                    }
                } else {
                    match keyword {
                        KeyWord::Ret => clone.insts.push(Instruction::Ret),
                        KeyWord::Halt => clone.insts.push(Instruction::Halt),
                        _ => {}
                    }
                }

                idx += keyword.increment_amount() as usize;
            }
        }
        Ok(clone)
    }

    fn get_keyword(&self, word: &str) -> Result<KeyWord, ParseError> {
        match word {
            "mov" => Ok(KeyWord::Mov),
            "pop" => Ok(KeyWord::Pop),
            "push" => Ok(KeyWord::Push),
            "jump" => Ok(KeyWord::Jump),
            "halt" => Ok(KeyWord::Halt),
            "ret" => Ok(KeyWord::Ret),
            "load" => Ok(KeyWord::Load),
            "add" => Ok(KeyWord::Add),
            _ => Err(ParseError::InvalidKeyWord),
        }
    }

    fn get_arg(&self, bytecode: &str) -> Option<Arg> {
        let arg = bytecode;
        let arg = {
            let idx = arg.find(',');
            if let Some(idx) = idx {
                arg.index(0..idx)
            } else {
                arg
            }
        };
        match arg {
            "r1" => Some(Arg::Register(Register::R1)),
            "r2" => Some(Arg::Register(Register::R2)),
            "r3" => Some(Arg::Register(Register::R3)),
            "r4" => Some(Arg::Register(Register::R4)),
            _ => {
                //println!("arg {arg:?}");
                match arg.parse::<u16>() {
                    Ok(num) => Some(Arg::U16(num)),
                    Err(e) => {
                        println!("{e:?} {arg}");
                        None
                    }
                }
            }
        }
    }

    fn get_args(&self, word: &KeyWord, bytecode: &[&str]) -> Option<Vec<Arg>> {
        let mut vec = Vec::new();
        match word {
            KeyWord::Mov => {
                let left = self.get_arg(bytecode[1])?;
                match left {
                    Arg::Register(_) => vec.push(left),
                    Arg::U16(_) => panic!(),
                }
                let right = self.get_arg(bytecode[2])?;

                vec.push(right);
            }
            KeyWord::Add => {
                let left = self.get_arg(bytecode[1])?;
                match left {
                    Arg::Register(_) => vec.push(left),
                    Arg::U16(_) => {
                        panic!("{}", ParseError::InvalidArgument(InvalidArgument::Number))
                    }
                }
                let right = self.get_arg(bytecode[2])?;

                vec.push(right);
            }
            KeyWord::Pop => {
                let arg = self.get_arg(bytecode[1])?;
                match arg {
                    Arg::Register(_register) => vec.push(arg),
                    Arg::U16(_) => {
                        panic!("{}", ParseError::InvalidArgument(InvalidArgument::Number))
                    }
                }
            }
            KeyWord::Ret => return None,
            KeyWord::Halt => return None,
            KeyWord::Jump => {
                let arg = self.get_arg(bytecode[1]);
                if let Some(arg) = arg {
                    match arg {
                        Arg::U16(_addr) => {
                            vec.push(arg);
                        }
                        Arg::Register(_reg) => {
                            panic!("{}", ParseError::InvalidArgument(InvalidArgument::Register))
                        }
                    }
                }
            }
            KeyWord::Load => {
                let (left, right) = (self.get_arg(bytecode[1])?, self.get_arg(bytecode[2])?);
                match left {
                    Arg::Register(_register) => vec.push(left),
                    Arg::U16(_) => {
                        panic!("{}", ParseError::InvalidArgument(InvalidArgument::Number))
                    }
                }
                match right {
                    Arg::Register(_register) => {
                        panic!("{}", ParseError::InvalidArgument(InvalidArgument::Register))
                    }
                    Arg::U16(_addr) => vec.push(right),
                }
            }
            KeyWord::Push => {
                let arg = self.get_arg(bytecode[1])?;
                vec.push(arg);
            }
        }
        Some(vec)
    }

    fn args_to_inst(&self, keyword: KeyWord, args: &[Arg]) -> Option<Instruction> {
        match keyword {
            KeyWord::Mov => {
                let reg = match args[0] {
                    Arg::Register(reg) => reg,
                    Arg::U16(num) => panic!("Expected Argument: Register found number: {num}"),
                };
                match args[1] {
                    Arg::Register(register) => Some(Instruction::MovRegReg(reg, register)),
                    Arg::U16(num) => Some(Instruction::MovRegVal(reg, num)),
                }
            }
            KeyWord::Add => {
                let reg = match args[0] {
                    Arg::Register(reg) => reg,
                    Arg::U16(num) => panic!("Expected Argument: Register found number: {num}"),
                };
                match args[1] {
                    Arg::Register(register) => Some(Instruction::AddRegReg(reg, register)),
                    Arg::U16(num) => Some(Instruction::AddRegNum(reg, num)),
                }
            }
            KeyWord::Pop => match args[0] {
                Arg::Register(register) => Some(Instruction::PopReg(register)),
                Arg::U16(num) => panic!("Expected Argument: Register found number {num}"),
            },
            KeyWord::Ret => None,
            KeyWord::Halt => None,
            KeyWord::Jump => match args[0] {
                Arg::Register(register) => {
                    panic!("Expected Argument: number found register {register}")
                }
                Arg::U16(num) => Some(Instruction::Jump(num)),
            },
            KeyWord::Load => {
                let reg = match args[0] {
                    Arg::Register(reg) => reg,
                    Arg::U16(num) => panic!("Expected Argument: Register found number: {num}"),
                };
                match args[1] {
                    Arg::Register(register) => {
                        panic!("Expected Argument: address found register: {register}")
                    }
                    Arg::U16(num) => Some(Instruction::Load(reg, num)),
                }
            }
            KeyWord::Push => match args[0] {
                Arg::Register(register) => Some(Instruction::PushReg(register)),
                Arg::U16(val) => Some(Instruction::PushVal(val)),
            },
        }
    }
}
