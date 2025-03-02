use std::str::FromStr;

use vm_cpu::{
    address::Address, error::Error, memory::Memory, opcodes::Instruction, registers::Register,
};

#[derive(Debug)]
pub struct Parser {
    data: String,
    tokens: Vec<Token>,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidKeyWord,
    InvalidRegister,
}

#[derive(Debug)]
pub enum Token {
    KeyWord(KeyWord),
    Register(Register),
    U16(u16),
}

impl Parser {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
            tokens: vec![],
        }
    }

    pub fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        for word in self.data.split_whitespace() {
            self.tokens.push(Token::from_str(word)?);
            //println!("line {line}");
        }
        Ok(())
    }
}

impl FromStr for Parser {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(s);
        parser.parse()?;

        Ok(parser)
    }
}

impl FromStr for Token {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let res = KeyWord::from_str(s);
        let Ok(_) = res else {
            let Ok(num) = s.parse::<u16>() else {
                let Some(idx) = s.find(',') else {
                    let reg = Register::from_str(s).map_err(|_f| ParseError::InvalidRegister)?;
                    return Ok(Token::Register(reg));
                };
                let reg =
                    Register::from_str(&s[0..idx]).map_err(|_f| ParseError::InvalidRegister)?;
                return Ok(Token::Register(reg));
            };
            return Ok(Token::U16(num));
        };
        Ok(Token::KeyWord(res?))
    }
}

keywords! {
    Mov, amount = 3,
    Add, amount = 3,
    Load, amount = 3,
    Jump, amount = 2,
    Push, amount = 2,
    Pop, amount = 2,
    Call, amount = 2,
    Ret, amount = 1,
    Halt, amount = 1,
}
