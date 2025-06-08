use std::fmt::Display;

use tracing::info;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    LCarrot,
    RCarrot,
    EqSign,
    Space,
    Number(&'a str),
    Lparen,
    Rparen,
    Ident(&'a str),
    Plus,
    Sub,
    Mult,
    Div,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    data: &'a str,
}

#[tracing::instrument]
fn lex_number(src: &str) -> (Token, usize) {
    let mut start = None;
    for (i, ch) in src.chars().enumerate() {
        match ch {
            '0'..='9' => {
                if start.is_none() {
                    start = Some(i)
                }
            }
            _ => {
                return (Token::Number(&src[start.unwrap()..i]), i);
            }
        }
    }

    (Token::Number(&src[start.unwrap()..]), src.len())
}

#[tracing::instrument]
fn lex_ident(src: &str) -> &str {
    let mut start = None;
    for (i, ch) in src.chars().enumerate() {
        match ch {
            'a'..='z' | 'A'..='Z' => {
                if start.is_none() {
                    start = Some(i)
                }
            }
            '0'..='9' => {
                if start.is_none() {
                    panic!()
                }
            }
            _ => {
                return &src[start.unwrap()..i];
            }
        }
    }
    src
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data }
    }

    #[tracing::instrument]
    pub fn lex(self) -> Result<Vec<Token<'a>>, LexError> {
        let mut tokens = Vec::new();

        let mut idx = 0;

        let mut chars = self.data.chars().collect::<Vec<_>>();

        loop {
            let Some(ch) = chars.get(idx) else { break };
            info!(?ch);
            let tok = match ch {
                ' ' => Token::Space,
                '>' => Token::RCarrot,
                '<' => Token::LCarrot,
                '=' => Token::EqSign,
                '(' => Token::Lparen,
                ')' => Token::Rparen,
                '+' => Token::Plus,
                '-' => Token::Sub,
                '*' => Token::Mult,
                '/' => Token::Div,
                '0'..='9' => {
                    let (num, amount) = lex_number(&self.data[idx..]);
                    idx += amount.saturating_sub(1);
                    num
                }
                'a'..='z' | 'A'..='Z' => {
                    let ident = lex_ident(&self.data[idx..]);
                    idx += ident.len().saturating_sub(1);
                    Token::Ident(ident)
                }

                _ => return Err(LexError::InvalidToken(ch.to_string())),
            };
            idx += 1;

            tokens.push(tok);
        }

        info!(?tokens);

        Ok(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::{Lexer, Token};

    fn lex(data: &str) -> Vec<Token> {
        Lexer::new(data).lex().unwrap()
    }

    #[test]
    fn add() {
        assert_eq!(lex("+"), [Token::Plus])
    }

    #[test]
    fn sub() {
        assert_eq!(lex("-"), [Token::Sub])
    }

    #[test]
    fn mult() {
        assert_eq!(lex("*"), [Token::Mult])
    }

    #[test]
    fn div() {
        assert_eq!(lex("/"), [Token::Div])
    }

    #[test]
    fn lcarrot() {
        assert_eq!(lex("<"), [Token::LCarrot])
    }

    #[test]
    fn rcarrot() {
        assert_eq!(lex(">"), [Token::RCarrot])
    }

    #[test]
    fn lparen() {
        assert_eq!(lex("("), [Token::Lparen])
    }

    #[test]
    fn rparen() {
        assert_eq!(lex(")"), [Token::Rparen])
    }

    #[test]
    fn eq() {
        assert_eq!(lex("="), [Token::EqSign])
    }

    #[test]
    fn number() {
        assert_eq!(lex("123456"), [Token::Number("123456")])
    }

    #[test]
    fn ident() {
        assert_eq!(lex("hello"), [Token::Ident("hello")])
    }
}
