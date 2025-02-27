use vm_cpu::{address::Address, opcodes::Instruction, registers::Register};

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

        impl TryFrom<&str> for KeyWord {
            type Error = ParseError;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                //println!("converting {value}");
                match value {
                    "mov" => Ok(KeyWord::Mov),
                    "add" => Ok(KeyWord::Add),
                    "pop" => Ok(KeyWord::Pop),
                    "ret" => Ok(KeyWord::Ret),
                    "halt" => Ok(KeyWord::Halt),
                    "jump" => Ok(KeyWord::Jump),
                    "load" => Ok(KeyWord::Load),
                    "push" => Ok(KeyWord::Push),
                    _ => Err(Self::Error::InvalidKeyWord)
                }
            }
        }

        impl std::fmt::Display for KeyWord {
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
    Load, amount = 3,
    Jump, amount = 2,
    Push, amount = 2,
    Pop, amount = 2,
    Ret, amount = 1,
    Halt, amount = 1,
}

#[derive(Debug)]
enum Token {
    KeyWord(KeyWord),
    Register(Register),
    U16(u16),
}

impl TryFrom<&str> for Token {
    type Error = ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().parse::<u16>() {
            Ok(num) => return Ok(Token::U16(num)),
            Err(_e) => {}
        };
        match value {
            "r1," => Ok(Token::Register(Register::R1)),
            "r2," => Ok(Token::Register(Register::R2)),
            "r3," => Ok(Token::Register(Register::R3)),
            "r4," => Ok(Token::Register(Register::R4)),
            _ => {
                let Ok(keyword) = KeyWord::try_from(value) else {
                    return Err(ParseError::InvalidKeyWord);
                };
                Ok(Token::KeyWord(keyword))
            }
        }
    }
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

    fn parse_token(&self, data: &str) -> Result<(Token, usize), ParseError> {
        for (i, ch) in data.chars().enumerate() {
            if ch == ' ' {
                let Some(data) = data.get(0..i) else {
                    break;
                };
                return Ok((Token::try_from(data)?, i));
            }
        }

        Ok((Token::try_from(data)?, data.len()))
    }

    pub fn parse(mut self) -> Result<Self, ParseError> {
        let mut tokens = vec![];
        let mut idx = 0;
        loop {
            let res = self.parse_token(&self.data[idx..]);
            //println!("{res:?}");
            if idx >= self.data.len() {
                break;
            }
            if let Ok((word, i)) = res {
                idx += i;
                tokens.push(word);
            } else {
                idx += 1;
            }
        }

        //println!("tokens {tokens:?}");

        idx = 0;

        loop {
            let Some(token) = tokens.get(idx) else {
                break;
            };
            if let Token::KeyWord(key_word) = token {
                let Some(code) = tokens.get(idx + 1..idx + key_word.increment_amount() as usize)
                else {
                    return Err(ParseError::InvalidArgument(InvalidArgument::Number));
                };
                println!("keyword {key_word:?} code {code:?}");
                assert!(code.len() == key_word.increment_amount().saturating_sub(1).into());
                let inst = match key_word {
                    KeyWord::Mov => {
                        let reg = match &code[0] {
                            Token::KeyWord(keyword) => {
                                panic!("Should be Register not KeyWord {keyword}")
                            }
                            Token::Register(register) => register,
                            Token::U16(_) => panic!("Should be Register not U16"),
                        };
                        match &code[1] {
                            Token::KeyWord(keyword) => {
                                panic!("Should be Register not KeyWord {keyword}")
                            }
                            Token::Register(register) => Instruction::MovRegReg(*reg, *register),
                            Token::U16(val) => Instruction::MovRegVal(*reg, *val),
                        }
                    }
                    KeyWord::Add => {
                        let reg = match &code[0] {
                            Token::KeyWord(_) => {
                                panic!("Should be Register not KeyWord")
                            }
                            Token::Register(register) => register,
                            Token::U16(_) => panic!("Should be Register not U16"),
                        };
                        match &code[1] {
                            Token::KeyWord(_) => {
                                panic!("Should be Register not KeyWord")
                            }
                            Token::Register(register) => Instruction::AddRegReg(*reg, *register),
                            Token::U16(val) => Instruction::AddRegNum(*reg, *val),
                        }
                    }
                    KeyWord::Pop => match &code[0] {
                        Token::KeyWord(_) => {
                            panic!("Should be Register not KeyWord")
                        }
                        Token::Register(register) => Instruction::PopReg(*register),
                        Token::U16(_) => panic!("Should be Register not U16"),
                    },
                    KeyWord::Jump => match &code[0] {
                        Token::KeyWord(_) => {
                            panic!("Should be Register not KeyWord")
                        }
                        Token::Register(register) => Instruction::PopReg(*register),
                        Token::U16(_) => panic!("Should be Register not U16"),
                    },
                    KeyWord::Load => {
                        let reg = match &code[0] {
                            Token::KeyWord(_) => {
                                panic!("Should be Register not KeyWord")
                            }
                            Token::Register(register) => register,
                            Token::U16(_) => panic!("Should be Register not U16"),
                        };
                        match &code[1] {
                            Token::KeyWord(_) => {
                                panic!("Should be Register not KeyWord")
                            }
                            Token::Register(_) => {
                                panic!("Should be U16 not Register")
                            }
                            Token::U16(val) => Instruction::Load(*reg, Address::from(*val)),
                        }
                    }
                    KeyWord::Push => match &code[0] {
                        Token::KeyWord(keyword) => {
                            panic!("Should be Register not KeyWord {keyword}")
                        }
                        Token::Register(register) => Instruction::PushReg(*register),
                        Token::U16(val) => Instruction::PushVal(*val),
                    },
                    KeyWord::Ret => Instruction::Ret,
                    KeyWord::Halt => Instruction::Halt,
                };
                idx += key_word.increment_amount() as usize;
                self.insts.push(inst);
            } else {
                panic!()
            }
        }

        Ok(self)
    }
}
