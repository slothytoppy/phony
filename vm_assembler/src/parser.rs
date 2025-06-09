use crate::tokens::Lexer;
use crate::ParseError;
use crate::Token;

use tracing::info;
use tracing::instrument;
use vm_cpu::memory::Address;

use std::collections::HashMap;
use std::fmt::Display;
use std::num;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode<'a> {
    Token(Token<'a>),
    Label(Address),
    Ident(&'a str),
    KeyWord(KeyWord),
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
                Token::Address(_addr) => Some(4),
                Token::Identifier(_ident) => None,
                Token::Comma => None,
                Token::Space => None,
            },
            AstNode::Label(_) => None,
            AstNode::Ident(_) => None,
            AstNode::KeyWord(_) => None,
        }
    }
}

impl<'a> From<KeyWord> for AstNode<'a> {
    fn from(value: KeyWord) -> Self {
        AstNode::KeyWord(value)
    }
}

impl<'a> From<Token<'a>> for AstNode<'a> {
    fn from(value: Token<'a>) -> Self {
        match value {
            Token::Address(address) => AstNode::Label(address),
            Token::Identifier(ident) => AstNode::Ident(ident),
            _ => AstNode::Token(value),
        }
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

    pub fn get(&self, idx: usize) -> Option<&AstNode<'a>> {
        self.nodes.get(idx)
    }

    #[allow(unused)]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut AstNode<'a>> {
        self.nodes.get_mut(idx)
    }

    pub fn set(&mut self, idx: usize, node: AstNode<'a>) {
        if let Some(old_node) = self.nodes.get_mut(idx) {
            *old_node = node;
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        let nodes = self.nodes;

        let mut output = Vec::new();

        for node in nodes {
            match node {
                AstNode::Token(token) => match token {
                    Token::Register(register) => output.push(register as u8),
                    Token::Number(number) => match number {
                        crate::tokens::Number::U8(val) => output.push(val),
                        crate::tokens::Number::U16(val) => {
                            let bytes = val.to_le_bytes();

                            for byte in bytes {
                                output.push(byte);
                            }
                        }
                        crate::tokens::Number::U32(val) => {
                            let bytes = val.to_le_bytes();

                            for byte in bytes {
                                output.push(byte);
                            }
                        }
                    },
                    Token::Address(address) => {
                        let val = u32::from(address).to_le_bytes();

                        for byte in val {
                            output.push(byte);
                        }
                    }
                    // these tokens can not be turned into bytes as they do not conform to what
                    // the cpu expects
                    Token::Comma | Token::Space | Token::Identifier(_) => {}
                },
                AstNode::Label(address) => {
                    let bytes = u32::from(address).to_le_bytes();

                    for byte in bytes {
                        output.push(byte)
                    }
                }
                // these nodes can not be turned into bytes as they do not conform to what
                // the cpu expects
                AstNode::Ident(_) | AstNode::KeyWord(_) => {}
            }
        }

        output
    }
}

#[derive(Debug, Default)]
pub struct Parser<'a> {
    ast: Ast<'a>,
    resolved_labels: HashMap<&'a str, Address>,
}

impl Display for Parser<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Ast Nodes:")?;
        for node in &self.ast.nodes {
            match node {
                AstNode::Token(token) => match token {
                    Token::Comma | Token::Space => {}
                    _ => writeln!(f, "{token:?}")?,
                },
                _ => writeln!(f, "{node:?}")?,
            }
        }
        writeln!(f, "Resolved Labels:")?;
        for key in self.resolved_labels.iter() {
            writeln!(f, "{key:?}")?
        }
        Ok(())
    }
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

        let lexer = Lexer::lex(data);

        let mut parser = Parser::default();

        let mut addr: usize = 0;

        for token in lexer.tokens.iter() {
            match token {
                Token::Identifier(s) => match KeyWord::from_str(s) {
                    Ok(keyword) => parser.push(AstNode::KeyWord(keyword)),
                    Err(e) => {
                        match e {
                            ParseError::InvalidKeyWord(_) => {}
                            _ => panic!("{e:?}"),
                        };

                        if s.ends_with(":") {
                            parser
                                .resolved_labels
                                .insert(&s[0..s.len().saturating_sub(1)], Address::from(addr));
                            addr += 4;
                        } else {
                            parser.push(AstNode::Ident(s));
                        }
                    }
                },
                _ => {
                    let tok = AstNode::from(token.clone());
                    if let Some(amount) = tok.byte_size() {
                        addr += amount;
                    }

                    parser.push(tok);
                }
            }
        }

        for idx in 0..parser.ast.nodes.len() {
            let Some(token) = parser.ast.get(idx) else {
                break;
            };

            match token {
                AstNode::Label(address) => info!(?address),
                AstNode::Ident(ident) => {
                    if let Some(addr) = parser.resolved_labels.get(ident) {
                        info!(?ident, ?addr);
                        info!(?token);
                        parser.ast.set(idx, AstNode::Label(*addr));
                    }
                }
                _ => {}
            }
        }

        info!(%parser);

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
    use vm_cpu::{memory::Address, registers::Register};

    use crate::{parser::KeyWord, tokens::Number, Token};

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
            AstNode::Label(vm_cpu::memory::Address::from(0)),
            Token::Space.into(),
        ];

        assert_eq!(ast.as_slice(), expected)
    }

    #[test]
    fn call_label() {
        let src = "call urmom";
        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            AstNode::KeyWord(KeyWord::Call),
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
            AstNode::KeyWord(KeyWord::Add),
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
            AstNode::KeyWord(KeyWord::Mov),
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
            AstNode::KeyWord(KeyWord::Mov),
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
            AstNode::KeyWord(KeyWord::Mov),
            Token::Space.into(),
            Token::Register(Register::R1).into(),
            Token::Comma.into(),
            Token::Space.into(),
            AstNode::Label(Address::from(10)),
        ];

        assert_eq!(ast, expected)
    }

    #[test]
    fn mem_mov_reg() {
        init_logger();
        let src = "mov [10], r1";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            AstNode::KeyWord(KeyWord::Mov),
            Token::Space.into(),
            AstNode::Label(Address::from(10)),
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
            AstNode::KeyWord(KeyWord::Mov),
            Token::Space.into(),
            AstNode::Label(Address::from(10)),
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
            AstNode::KeyWord(KeyWord::Cmp),
            Token::Space.into(),
            AstNode::Token(crate::Token::Register(Register::R1)),
        ];

        assert_eq!(ast, expected);
    }

    #[test]
    fn labels_test() {
        let src = "foo:\nbar:\nbaz:\nfoo\nbar\nbaz";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [
            AstNode::Label(vm_cpu::memory::Address::from(0)),
            AstNode::Label(vm_cpu::memory::Address::from(4)),
            AstNode::Label(vm_cpu::memory::Address::from(8)),
        ];

        assert_eq!(ast, expected);
    }

    #[test]
    fn unresolved_label() {
        let src = "foo";

        let ast = Parser::parse(src).unwrap().into_iter().collect::<Vec<_>>();

        let expected = [AstNode::Ident("foo")];

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
