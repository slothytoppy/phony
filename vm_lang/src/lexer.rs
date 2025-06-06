use std::{fmt::Display, ops::Range};

#[derive(Debug)]
pub enum LexError {
    InvalidToken(String),
}

impl std::error::Error for LexError {}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::InvalidToken(tok) => write!(f, "{tok}"),
        }
    }
}

#[derive(Debug)]
pub enum Token {
    LCarrot,
    RCarrot,
    EqSign,
    Space,
    Number(Number),
}

#[derive(Debug)]
pub enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Debug)]
pub struct Lexer<'a> {
    data: &'a str,
}

fn lex_number(src: &str) -> (Number, usize) {
    let mut start = None;
    for (i, ch) in src.chars().enumerate() {
        match ch {
            '0'..='9' => {
                if start.is_none() {
                    start = Some(i)
                }
            }
            _ => {
                return (
                    Number::U32(src[start.unwrap()..i].parse::<u32>().unwrap()),
                    i,
                )
            }
        }
    }

    (
        Number::U32(src[start.unwrap()..].parse::<u32>().unwrap()),
        src.len(),
    )
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data }
    }

    pub fn lex(&self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::default();

        let mut idx = 0;
        let mut chars = self.data.chars().enumerate();
        loop {
            let Some((i, ch)) = chars.nth(idx) else { break };
            let tok = match ch {
                ' ' => Token::Space,
                '>' => Token::RCarrot,
                '<' => Token::LCarrot,
                '=' => Token::EqSign,
                '0'..='9' => {
                    let (num, amount) = lex_number(&self.data[i..]);
                    idx += amount;
                    Token::Number(num)
                }
                _ => return Err(LexError::InvalidToken(ch.to_string())),
            };

            tokens.push(tok);
        }
        // for ch in self.data.chars() {
        //     let tok = match ch {
        //         ' ' => Token::Space,
        //         '>' => Token::RCarrot,
        //         '<' => Token::LCarrot,
        //         '=' => Token::EqSign,
        //         '0'..='9' => match ch {
        //             _ => unreachable!(),
        //         },
        //         _ => return Err(LexError::InvalidToken(ch.to_string())),
        //     };
        //
        //     tokens.push(tok);
        // }

        Ok(tokens)
    }
}
