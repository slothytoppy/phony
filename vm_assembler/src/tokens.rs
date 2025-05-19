use core::panic;
use std::{num::IntErrorKind, str::FromStr};

use crate::ParseError;
use tracing::{error, info, warn};
use vm_cpu::registers::Register;

#[derive(Debug, Default)]
pub(crate) struct Tokenizer {
    tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn tokenize(&mut self, data: &str) -> Tokenizer {
        let mut tokenizer = Tokenizer::default();

        data.split_whitespace().for_each(|word| {
            if let Ok(token) = Token::from_str(word) {
                tokenizer.tokens.push(token);
            }
        });

        tokenizer
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
}

impl FromStr for Number {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match s.parse::<u8>() {
            Ok(num) => Ok(Number::U8(num)),
            Err(e) => match e.kind() {
                IntErrorKind::Empty => panic!(),
                IntErrorKind::InvalidDigit => {
                    Err(ParseError::InvalidNumber(IntErrorKind::InvalidDigit))
                }
                IntErrorKind::PosOverflow => match s.parse::<u16>() {
                    Ok(num) => Ok(Number::U16(num)),
                    Err(e) => match e.kind() {
                        IntErrorKind::Empty => panic!(),
                        IntErrorKind::InvalidDigit => {
                            Err(ParseError::InvalidNumber(IntErrorKind::InvalidDigit))
                        }
                        IntErrorKind::PosOverflow => match s.parse::<u32>() {
                            Ok(num) => Ok(Number::U32(num)),
                            Err(e) => match e.kind() {
                                IntErrorKind::Empty => panic!(),
                                IntErrorKind::InvalidDigit => {
                                    Err(ParseError::InvalidNumber(IntErrorKind::InvalidDigit))
                                }
                                IntErrorKind::PosOverflow => panic!(),
                                IntErrorKind::NegOverflow => todo!(),
                                IntErrorKind::Zero => Ok(Number::U32(0)),
                                _ => todo!(),
                            },
                        },
                        IntErrorKind::NegOverflow => todo!(),
                        IntErrorKind::Zero => Ok(Number::U16(0)),
                        _ => todo!(),
                    },
                },
                IntErrorKind::NegOverflow => todo!(),
                IntErrorKind::Zero => Ok(Number::U8(0)),
                _ => todo!(),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Address(u32);

impl FromStr for Address {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let mut start = None;

        info!(s);

        for (i, ch) in s.chars().enumerate() {
            match ch {
                '[' => {
                    start = Some(i + 1);
                }
                ']' => {
                    assert!(start.is_some());
                    info!("{:?}", &s[start.unwrap()..i]);
                    return match s[start.unwrap()..i].parse::<u32>() {
                        Ok(val) => Ok(Address(val)),
                        Err(e) => return Err(ParseError::InvalidNumber(e.kind().clone())),
                    };
                }
                _ => {}
            }
        }

        Err(ParseError::InvalidNumber(IntErrorKind::InvalidDigit))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    KeyWord(KeyWord),
    Register(Register),
    Number(Number),
    Address(Address),
    // Label(Label),
}

impl FromStr for Token {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Ok(keyword) = KeyWord::from_str(s) {
            return Ok(Token::KeyWord(keyword));
        }

        if let Some(idx) = s.find(',') {
            let reg = Register::from_str(&s[0..idx])
                .map_err(|_| ParseError::InvalidRegister(s[0..idx].to_string()))?;
            return Ok(Token::Register(reg));
        }

        if let Ok(reg) = Register::from_str(s) {
            return Ok(Token::Register(reg));
        }

        match Address::from_str(s) {
            Ok(addr) => return Ok(Token::Address(addr)),
            Err(e) => error!(?e, ?s),
        }

        if let Ok(num) = Number::from_str(s) {
            return Ok(Token::Number(num));
        }

        panic!("{s:?}");
    }
}

macro_rules! keywords {
    ($($variant:ident, $amount:ident = $arg_amount:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

        impl FromStr for KeyWord {
            type Err = ParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    "mov" => Ok(KeyWord::Mov),
                    "add" => Ok(KeyWord::Add),
                    "pop" => Ok(KeyWord::Pop),
                    "ret" => Ok(KeyWord::Ret),
                    "halt" => Ok(KeyWord::Halt),
                    "jump" => Ok(KeyWord::Jump),
                    "load" => Ok(KeyWord::Load),
                    "push" => Ok(KeyWord::Push),
                    "call" => Ok(KeyWord::Call),

                    _ => Err(Self::Err::InvalidKeyWord(value.to_string()))
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
    Call, amount = 2,
    Ret, amount = 1,
    Halt, amount = 1,
    Cmp, amount = 2,
}
