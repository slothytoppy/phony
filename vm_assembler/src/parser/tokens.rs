use std::str::FromStr;

use crate::parser::ParseError;
use vm_cpu::registers::Register;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Label {
    Unresolved(String),
    Resolved(String),
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Label {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.ends_with(':') {
            return Ok(Self::Resolved(String::from(s)));
        }
        Ok(Self::Unresolved(String::from(s)))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    KeyWord(KeyWord),
    Register(Register),
    U16(u16),
    Label(Label),
}

impl FromStr for Token {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Ok(num) = s.parse::<u16>() {
            return Ok(Token::U16(num));
        }
        if let Ok(keyword) = KeyWord::from_str(s) {
            return Ok(Token::KeyWord(keyword));
        }
        if let Some(idx) = s.find(',') {
            let reg = Register::from_str(&s[0..idx])
                .map_err(|_| ParseError::InvalidRegister(s[0..idx].to_string()))?;
            return Ok(Token::Register(reg));
        }
        if let Ok(reg) = Register::from_str(s) {
            Ok(Token::Register(reg))
        } else {
            let label = Label::from_str(&s[0..s.len()])
                .map_err(|_| ParseError::InvalidKeyWord(s[0..s.len()].to_string()))?;
            Ok(Token::Label(label))
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
    Call, amount = 2,
    Ret, amount = 1,
    Halt, amount = 1,
}
