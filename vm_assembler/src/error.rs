use std::{fmt::Display, num::IntErrorKind};

use super::Token;

#[derive(Debug)]
pub enum ParseError {
    InvalidKeyWord(String),
    InvalidToken(Token),
    InvalidRegister(String),
    InvalidNumber(IntErrorKind),
    EmptyFile,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidKeyWord(word) => write!(f, "invalid keyword {word}"),
            ParseError::InvalidRegister(register) => write!(f, "invalid register {register}"),
            ParseError::InvalidToken(token) => write!(f, "invalid token {token:?}"),
            ParseError::InvalidNumber(error) => write!(f, "invalid number {error:?}"),
            ParseError::EmptyFile => write!(f, "Attempted to parse empty file"),
        }
    }
}

impl std::error::Error for ParseError {}
