use core::panic;
use std::{num::IntErrorKind, str::FromStr};

use crate::ParseError;
use tracing::trace;
use vm_cpu::registers::Register;

#[derive(Debug, Default, Clone)]
pub(crate) struct Lexer<'a> {
    pub(crate) tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn lex(data: &'a str) -> Lexer<'a> {
        let mut tokenizer = Lexer::default();
        let mut start: Option<usize> = None;
        for (i, c) in data.chars().enumerate() {
            match c {
                '\n' => {
                    if start.is_some() {
                        tokenizer
                            .tokens
                            .push(lex_word(&data[start.unwrap()..i]).unwrap());
                        start = None;
                    }
                }
                ',' => {
                    if start.is_some() {
                        tokenizer
                            .tokens
                            .push(lex_word(&data[start.unwrap()..i]).unwrap());
                        start = None;
                    }
                    tokenizer.tokens.push(Token::Comma);
                }
                ' ' => {
                    if start.is_none() {
                        tokenizer.tokens.push(Token::Space);
                    } else {
                        tokenizer
                            .tokens
                            .push(lex_word(&data[start.unwrap()..i]).unwrap());
                        start = None;
                        tokenizer.tokens.push(Token::Space);
                    }
                }
                _ => {
                    if start.is_none() {
                        start = Some(i);
                    }
                }
            }
        }
        if let Some(start1) = start {
            tokenizer.tokens.push(lex_word(&data[start1..]).unwrap());
        }

        tokenizer
    }

    #[allow(unused)]
    pub fn iter(&self) -> LexerIterator<'_> {
        LexerIterator {
            idx: 0,
            tokens: &self.tokens,
        }
    }
}

#[derive(Debug)]
pub struct LexerIterator<'a> {
    tokens: &'a [Token<'a>],
    idx: usize,
}

impl<'a> Iterator for LexerIterator<'a> {
    type Item = &'a Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.tokens.get(self.idx);
        self.idx += 1;
        item
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
                        IntErrorKind::Zero => Ok(Number::U16(0)),
                        _ => todo!(),
                    },
                },
                IntErrorKind::Zero => Ok(Number::U8(0)),
                _ => todo!(),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Address(pub u32);

impl From<u8> for Address {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Self(value as u32)
    }
}

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<i32> for Address {
    fn from(value: i32) -> Self {
        Self(value as u32)
    }
}

impl Address {
    fn parse(s: &str) -> Result<Self, ParseError<'_>> {
        let s = s.trim();

        let mut start = None;

        for (i, ch) in s.chars().enumerate() {
            match ch {
                '[' => {
                    start = Some(i + 1);
                }
                ']' => {
                    assert!(start.is_some());
                    trace!("{:?}", &s[start.unwrap()..i]);
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
pub enum Token<'a> {
    Register(Register),
    Number(Number),
    Address(Address),
    Identifier(&'a str),
    Comma,
    Space,
}

fn lex_word(word: &str) -> Result<Token, ParseError> {
    let mut start = None;

    for (i, c) in word.chars().enumerate() {
        match c {
            ',' => {
                if let Some(s) = start {
                    println!("{:?}", &word[s..s + 1]);
                }
                return Ok(Token::Comma);
            }
            ' ' => return Ok(Token::Space),
            '[' => start = Some(i),
            ']' => {
                return Ok(Token::Address(
                    Address::parse(&word[start.unwrap()..i + 1]).unwrap(),
                ))
            }
            'a'..='z' => {
                if start.is_none() {
                    start = Some(i);
                }
            }
            _ => {}
        }
    }

    if let Ok(reg) = Register::from_str(word) {
        return Ok(Token::Register(reg));
    }

    match Address::parse(word) {
        Ok(addr) => return Ok(Token::Address(addr)),
        Err(_e) => {}
    }

    if let Ok(num) = Number::parse(word) {
        return Ok(Token::Number(num));
    }

    Ok(Token::Identifier(word))
}

#[cfg(test)]
mod lexer_test {
    use std::str::FromStr;

    use vm_cpu::registers::Register;

    use crate::{
        tokens::{Address, Lexer, Number},
        Token,
    };

    #[test]
    fn registers() {
        let regs = [
            "ip", "sp", "fp", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8",
        ];

        for reg in regs {
            assert!(
                Lexer::lex(reg).tokens.first().unwrap()
                    == &Token::Register(Register::from_str(reg).unwrap())
            );
        }
    }

    #[test]
    fn numbers() {
        let nums = "0 1 100000 val";

        let lexer = Lexer::lex(nums);
        let ast = lexer.iter().collect::<Vec<_>>();

        let expected = [
            &Token::Number(Number::U8(0)),
            &Token::Space,
            &Token::Number(Number::U8(1)),
            &Token::Space,
            &Token::Number(Number::U32(100000)),
            &Token::Space,
            &Token::Identifier("val"),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn address() {
        let addrs = "[1] [2] 1";

        let lexer = Lexer::lex(addrs);
        let ast = lexer.iter().collect::<Vec<_>>();

        let expected = [
            &Token::Address(Address(1)),
            &Token::Space,
            &Token::Address(Address(2)),
            &Token::Space,
            &Token::Number(Number::U8(1)),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn comma() {
        let comma = ",";

        let lexer = Lexer::lex(comma);
        let ast = lexer.iter().collect::<Vec<_>>();

        assert!(*ast.first().unwrap() == &Token::Comma);
    }

    #[test]
    fn space() {
        let space = " ";

        let lexer = Lexer::lex(space);
        let ast = lexer.iter().collect::<Vec<_>>();

        assert!(*ast.first().unwrap() == &Token::Space);
    }

    #[test]
    fn ident() {
        let ident = "val foo foo:";

        let lexer = Lexer::lex(ident);
        let ast = lexer.iter().collect::<Vec<_>>();

        let expected = [
            &Token::Identifier("val"),
            &Token::Space,
            &Token::Identifier("foo"),
            &Token::Space,
            &Token::Identifier("foo:"),
        ];

        assert_eq!(ast, expected);
    }
}
