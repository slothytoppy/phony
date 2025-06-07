use std::fmt::Display;

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

#[derive(Clone, Debug)]
pub enum Token<'a> {
    LCarrot,
    RCarrot,
    EqSign,
    Space,
    Number(Number),
    Lparen,
    Rparen,
    Ident(&'a str),
}

#[derive(Clone, Debug)]
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
                if let Ok(num) = src[start.unwrap()..i].parse::<u8>() {
                    return (Number::U8(num), i);
                } else if let Ok(num) = src[start.unwrap()..i].parse::<u16>() {
                    return (Number::U16(num), i);
                } else if let Ok(num) = src[start.unwrap()..i].parse::<u32>() {
                    return (Number::U32(num), i);
                }
            }
        }
    }

    if let Ok(num) = src[start.unwrap()..].parse::<u8>() {
        (Number::U8(num), src.len())
    } else if let Ok(num) = src[start.unwrap()..].parse::<u16>() {
        (Number::U16(num), src.len())
    } else if let Ok(num) = src[start.unwrap()..].parse::<u32>() {
        (Number::U32(num), src.len())
    } else {
        panic!()
    }
}

fn lex_ident(src: &str) -> &str {
    let mut start = None;
    for (i, ch) in src.chars().enumerate() {
        match ch {
            'a'..='z' | 'A'..='Z' => {
                if start.is_none() {
                    start = Some(i)
                }
            }
            _ => return &src[start.unwrap()..i],
        }
    }
    src
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data }
    }

    pub fn lex(self) -> Result<Vec<Token<'a>>, LexError> {
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
                'a'..='z' | 'A'..='Z' => {
                    let ident = lex_ident(&self.data[i..]);
                    idx += ident.len().saturating_sub(1);
                    Token::Ident(ident)
                }
                '(' => Token::Lparen,
                ')' => Token::Rparen,
                _ => return Err(LexError::InvalidToken(ch.to_string())),
            };

            tokens.push(tok);
        }

        Ok(tokens)
    }
}
