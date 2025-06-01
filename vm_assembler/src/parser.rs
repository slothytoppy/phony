use crate::tokens::Lexer;
use crate::ParseError;
use crate::Token;

use tracing::info;
use tracing::instrument;
use vm_cpu::memory::Address;

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode<'a> {
    Token(Token<'a>),
    Label(Address),
    Ident(&'a str),
}

impl AstNode<'_> {
    fn byte_size(&self) -> Option<usize> {
        match self {
            AstNode::Token(token) => match token {
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
            AstNode::Label(_) => None,
            AstNode::Ident(_) => todo!(),
        }
    }
}

impl<'a> From<Token<'a>> for AstNode<'a> {
    fn from(value: Token<'a>) -> Self {
        AstNode::Token(value)
    }
}

impl<'a> From<&Token<'a>> for AstNode<'a> {
    fn from(value: &Token<'a>) -> Self {
        AstNode::from(value.clone())
    }
}

#[derive(Debug, Default)]
pub struct Ast<'a> {
    nodes: Vec<AstNode<'a>>,
}

impl<'a> Ast<'a> {
    pub fn push(&mut self, node: AstNode<'a>) {
        self.nodes.push(node);
    }

    pub fn get(&mut self, idx: usize) -> Option<&AstNode<'a>> {
        self.nodes.get(idx)
    }
}

#[derive(Debug, Default)]
pub struct Parser<'a> {
    ast: Ast<'a>,
}

impl<'a> Parser<'a> {
    pub fn push(&mut self, token: impl Into<AstNode<'a>>) {
        self.ast.push(token.into());
    }

    #[instrument]
    pub fn parse(data: &'a str) -> Result<Parser<'a>, ParseError<'a>> {
        if data.is_empty() {
            return Err(ParseError::EmptyFile);
        }

        let lexer = Lexer::lex(data).into_iter();

        let mut parser = Parser::default();

        info!(?lexer);

        let mut addr: usize = 0;

        for token in lexer {
            match token {
                Token::Identifier(s) => {
                    if s.ends_with(":") {
                        parser.push(AstNode::Label(addr.into()));
                        addr += 4;
                    } else {
                        parser.push(AstNode::Ident(s));
                    }
                }

                _ => {
                    let tok = AstNode::from(token);

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

#[derive(Debug)]
pub struct ParserIterator<'a> {
    parser: Parser<'a>,
    idx: usize,
}

impl<'a> Iterator for ParserIterator<'a> {
    type Item = AstNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.parser.ast.get(self.idx).cloned();
        self.idx += 1;

        item
    }
}

impl<'a> IntoIterator for Parser<'a> {
    type Item = AstNode<'a>;
    type IntoIter = ParserIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ParserIterator {
            parser: self,
            idx: 0,
        }
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

    use super::{AstNode, Parser};

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
            AstNode::Ident("urmom"),
            Token::Space.into(),
            AstNode::Label(0.into()),
        ];

        assert_eq!(ast.as_slice(), expected)
    }

    #[test]
    fn call_label() {
        let src = "call urmom";
        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            AstNode::Ident("call"),
            Token::Space.into(),
            AstNode::Ident("urmom"),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn add() {
        let src = "add r1, 10";
        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            AstNode::Ident("add"),
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
            AstNode::Ident("mov"),
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
            AstNode::Ident("mov"),
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
            AstNode::Ident("mov"),
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
            AstNode::Ident("mov"),
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
            AstNode::Ident("mov"),
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

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            AstNode::Ident("cmp"),
            Token::Space.into(),
            AstNode::Token(crate::Token::Register(Register::R1)),
        ];

        assert_eq!(ast, expected);
    }

    #[test]
    fn labels_test() {
        let src = "foo:\nbar:\nbaz:";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            AstNode::Label(0.into()),
            AstNode::Label(4.into()),
            AstNode::Label(8.into()),
        ];

        assert_eq!(ast, expected);
    }

    #[test]
    fn unresolved_label() {
        let src = "foo\nfoo:";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [AstNode::Ident("foo"), AstNode::Label(0.into())];

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
