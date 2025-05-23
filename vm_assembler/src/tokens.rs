use core::panic;
use std::{num::IntErrorKind, str::FromStr};

use crate::ParseError;
use tracing::{error, info};
use vm_cpu::registers::Register;

#[derive(Debug, Default, Clone)]
pub(crate) struct Tokenizer<'a> {
    tokens: Vec<Token<'a>>,
}

fn tokenize_word(word: &str) -> Result<Token, ParseError> {
    let word = word.trim();

    if let Ok(keyword) = KeyWord::from_str(word) {
        return Ok(Token::KeyWord(keyword));
    }

    if let Some(idx) = word.find(',') {
        let reg = Register::from_str(&word[0..idx])
            .map_err(|_| ParseError::InvalidRegister(word[0..idx].to_string()))?;
        return Ok(Token::Register(reg));
    }

    if let Ok(reg) = Register::from_str(word) {
        return Ok(Token::Register(reg));
    }

    match Address::parse(word) {
        Ok(addr) => return Ok(Token::Address(addr)),
        Err(e) => error!(?e, ?word),
    }

    if let Ok(num) = Number::parse(word) {
        return Ok(Token::Number(num));
    }

    if let Ok(label) = Ident::parse(word) {
        return Ok(Token::Identifier(label));
    }

    panic!("{word:?}");
}

impl<'a> IntoIterator for Tokenizer<'a> {
    type Item = Token<'a>;
    type IntoIter = TokenizerIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TokenizerIterator {
            idx: 0,
            tokenizer: self,
        }
    }
}

impl<'a> Tokenizer<'a> {
    pub fn tokenize(data: &'a str) -> Tokenizer<'a> {
        let mut tokenizer = Tokenizer::default();
        data.split_whitespace().for_each(|word| {
            if let Ok(token) = tokenize_word(word) {
                tokenizer.tokens.push(token);
            }
        });

        tokenizer
    }

    pub fn get(&self, idx: usize) -> Option<&Token> {
        self.tokens.get(idx)
    }
}

#[derive(Debug)]
pub struct TokenizerIterator<'a> {
    tokenizer: Tokenizer<'a>,
    idx: usize,
}

impl<'a> Iterator for TokenizerIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.tokenizer.tokens.len() {
            let next = Some(&self.tokenizer.tokens[self.idx]);
            self.idx += 1;
            return next.cloned();
        }
        None
    }

    fn count(self) -> usize {
        self.tokenizer.tokens.len()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
}

impl<'a> Number {
    fn parse(s: &'a str) -> Result<Self, ParseError<'a>> {
        let s = s.trim();

        match s.parse::<u8>() {
            Ok(num) => Ok(Number::U8(num)),
            Err(e) => match e.kind() {
                IntErrorKind::Empty => panic!(),
                IntErrorKind::InvalidDigit => Err(ParseError::InvalidNumber(e.kind().clone())),
                IntErrorKind::PosOverflow => match s.parse::<u16>() {
                    Ok(num) => Ok(Number::U16(num)),
                    Err(e) => match e.kind() {
                        IntErrorKind::Empty => panic!(),
                        IntErrorKind::InvalidDigit => {
                            Err(ParseError::InvalidNumber(e.kind().clone()))
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

impl Address {
    fn parse(s: &str) -> Result<Self, ParseError<'_>> {
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
pub enum Ident<'a> {
    // tokenizer is not responsible for resolving labels, only for collecting them
    Ident(&'a str),
}

impl<'a> Ident<'a> {
    fn parse(s: &'a str) -> Result<Self, ParseError<'a>> {
        let s = s.trim();

        // if its not a keyword, it should be an Ident since it also shouldnt be anything else
        if KeyWord::from_str(s).is_err() {
            Ok(Ident::Ident(s))
        } else {
            Err(ParseError::InvalidIdent(s.to_string()))
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    KeyWord(KeyWord),
    Register(Register),
    Number(Number),
    Address(Address),
    Identifier(Ident<'a>),
}

macro_rules! keywords {
    ($($variant:ident),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        #[rustfmt::skip]
        pub enum KeyWord {
            $($variant),*
        }

        impl FromStr for KeyWord {
            type Err = ParseError<'static>;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                $(
                    if value == stringify!($variant).to_lowercase() {
                        return Ok(KeyWord::$variant);
                    }
                )*
            return Err(ParseError::InvalidKeyWord(value.to_string()));
            }
        }
    }
}

keywords! {
    Mov,
    Add,
    Load,
    Jump,
    Push,
    Pop,
    Call,
    Ret,
    Halt,
    Cmp,
    Inc,
    Store,
    Interrupt,
}
