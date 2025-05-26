use crate::tokens::Lexer;
use crate::ParseError;
use crate::Token;

use tracing::info;
use tracing::instrument;
use vm_cpu::memory::Address;

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseToken<'a> {
    Token(Token<'a>),
    Label(Address),
    Ident(&'a str),
}

impl ParseToken<'_> {
    fn byte_size(&self) -> Option<usize> {
        match self {
            ParseToken::Token(token) => match token {
                Token::Register(_) => Some(1),
                Token::Number(number) => match number {
                    crate::tokens::Number::U8(_) => Some(1),
                    crate::tokens::Number::U16(_) => Some(2),
                    crate::tokens::Number::U32(_) => Some(4),
                },
                Token::Address(_) => Some(4),
                Token::Identifier(s) => Some(s.len()),
                Token::Comma => None,
                Token::Space => None,
            },
            ParseToken::Label(_) => None,
            ParseToken::Ident(_) => todo!(),
        }
    }
}

impl<'a> From<Token<'a>> for ParseToken<'a> {
    fn from(value: Token<'a>) -> Self {
        ParseToken::Token(value)
    }
}

impl<'a> From<&Token<'a>> for ParseToken<'a> {
    fn from(value: &Token<'a>) -> Self {
        ParseToken::from(value.clone())
    }
}

#[derive(Debug, Default)]
pub struct Parser<'a> {
    tokens: Vec<ParseToken<'a>>,
}

#[derive(Debug)]
pub struct ParserIterator<'a> {
    parser: Parser<'a>,
    idx: usize,
}

impl<'a> Iterator for ParserIterator<'a> {
    type Item = ParseToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.parser.tokens.get(self.idx).cloned();
        self.idx += 1;

        item
    }
}

impl<'a> IntoIterator for Parser<'a> {
    type Item = ParseToken<'a>;
    type IntoIter = ParserIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ParserIterator {
            parser: self,
            idx: 0,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn push(&mut self, token: impl Into<ParseToken<'a>>) {
        self.tokens.push(token.into());
    }

    #[instrument]
    pub fn parse(data: &'a str) -> Result<Parser<'a>, ParseError<'a>> {
        if data.is_empty() {
            return Err(ParseError::EmptyFile);
        }

        let tokenizer = Lexer::lex(data).into_iter();

        let mut parser = Parser::default();

        info!(?tokenizer);

        let mut addr: usize = 0;

        for token in tokenizer {
            match token {
                Token::Identifier(s) => {
                    if s.ends_with(":") {
                        parser.push(ParseToken::Label(addr.into()));
                        addr += 4;
                    } else {
                        parser.push(ParseToken::Ident(s));
                    }
                }

                _ => {
                    let tok = ParseToken::from(token);

                    if let Some(amount) = tok.byte_size() {
                        addr += amount;
                    }

                    parser.push(tok);
                }
            }
        }

        Ok(parser)
    }
}

#[cfg(test)]
mod test {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::util::SubscriberInitExt;
    use vm_cpu::registers::Register;

    use crate::{
        tokens::{Address, Number},
        Token,
    };

    use super::{ParseToken, Parser};

    fn init_logger() {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(LevelFilter::INFO)
            .finish()
            .try_init();
    }

    #[test]
    fn label() {
        let src = "urmom urmom:";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("urmom"),
            Token::Space.into(),
            ParseToken::Label(0.into()),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn call_label() {
        let src = "call urmom";
        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("call"),
            Token::Space.into(),
            ParseToken::Ident("urmom"),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn add() {
        let src = "add r1, 10";
        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("add"),
            Token::Space.into(),
            Token::Register(Register::R1).into(),
            Token::Comma.into(),
            Token::Space.into(),
            Token::Number(Number::U8(10)).into(),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn mov_val() {
        init_logger();
        let src = "mov r1, 0";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("mov"),
            Token::Space.into(),
            Token::Register(Register::R1).into(),
            Token::Comma.into(),
            Token::Space.into(),
            Token::Number(Number::U8(0)).into(),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn mov_reg() {
        init_logger();
        let src = "mov r1, r2";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("mov"),
            Token::Space.into(),
            Token::Register(Register::R1).into(),
            Token::Comma.into(),
            Token::Space.into(),
            Token::Register(Register::R2).into(),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn mov_mem() {
        init_logger();
        let src = "mov r1, [10]";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("mov"),
            Token::Space.into(),
            Token::Register(Register::R1).into(),
            Token::Comma.into(),
            Token::Space.into(),
            Token::Address(Address::from(10)).into(),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn mem_mov_reg() {
        init_logger();
        let src = "mov [10], r1";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("mov"),
            Token::Space.into(),
            Token::Address(Address::from(10)).into(),
            Token::Comma.into(),
            Token::Space.into(),
            Token::Register(Register::R1).into(),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn mem_mov_val() {
        init_logger();
        let src = "mov [10], 10";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("mov"),
            Token::Space.into(),
            Token::Address(Address::from(10)).into(),
            Token::Comma.into(),
            Token::Space.into(),
            Token::Number(Number::U8(10)).into(),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn cmp() {
        init_logger();
        let src = "cmp r1";
        let parser = Parser::parse(src).unwrap();

        let ast = parser.into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Ident("cmp"),
            Token::Space.into(),
            ParseToken::Token(crate::Token::Register(Register::R1)),
        ];

        assert_eq!(ast, expected);
    }

    #[test]
    fn labels_test() {
        let src = "foo:\nbar:\nbaz:";
        let parser = Parser::parse(src).unwrap();

        let ast = parser.into_iter().collect::<Vec<_>>();

        let expected = [
            ParseToken::Label(0.into()),
            ParseToken::Label(4.into()),
            ParseToken::Label(8.into()),
        ];

        assert_eq!(ast, expected);
    }

    #[test]
    fn unresolved_label() {
        let src = "foo\nfoo:";
        let parser = Parser::parse(src).unwrap();

        let ast = parser.into_iter().collect::<Vec<_>>();

        let expected = [ParseToken::Ident("foo"), ParseToken::Label(0.into())];

        assert_eq!(ast, expected);
    }
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
