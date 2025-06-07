use std::fmt::Display;

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::util::SubscriberInitExt;

use crate::lexer::{LexError, Token};

#[derive(Debug)]
pub enum ParserError {
    InvalidToken(String),
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::InvalidToken(tok) => write!(f, "{tok:?}"),
        }
    }
}

impl std::error::Error for ParserError {}

impl From<LexError> for ParserError {
    fn from(value: LexError) -> Self {
        match value {
            LexError::InvalidToken(tok) => Self::InvalidToken(tok),
        }
    }
}

#[derive(Default, Debug)]
pub struct Parser<'a> {
    _m: std::marker::PhantomData<Ast<'a>>,
}

#[derive(PartialEq, Debug)]
pub enum Node<'a> {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Space,
    U8(u8),
    U16(u16),
    U32(u32),
    Lparen,
    Rparen,
    Ident(&'a str),
}

#[derive(Default, Debug)]
pub struct Ast<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'a> Ast<'a> {
    pub fn push(&mut self, node: Node<'a>) {
        self.nodes.push(node);
    }
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self, tokens: Vec<Token<'a>>) -> Result<Ast<'a>, ParserError> {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(LevelFilter::INFO)
            .finish()
            .try_init();

        let mut ast = Ast::default();

        let mut idx = 0;

        loop {
            let Some(token) = tokens.get(idx) else {
                break;
            };

            if let Some(next_token) = tokens.get(idx + 1) {
                info!(?token, ?next_token);
                let peeked_node = match (token, next_token) {
                    (Token::LCarrot, Token::EqSign) => {
                        idx += 1;
                        Node::Lte
                    }
                    (Token::RCarrot, Token::EqSign) => {
                        idx += 1;
                        Node::Gte
                    }
                    _ => match token {
                        Token::LCarrot => Node::Lt,
                        Token::RCarrot => Node::Gt,
                        Token::EqSign => Node::Eq,
                        Token::Space => Node::Space,
                        Token::Number(val) => match val {
                            crate::lexer::Number::U8(num) => Node::U8(*num),
                            crate::lexer::Number::U16(num) => Node::U16(*num),
                            crate::lexer::Number::U32(num) => Node::U32(*num),
                        },
                        Token::Lparen => Node::Lparen,
                        Token::Rparen => Node::Rparen,
                        Token::Ident(ident) => Node::Ident(ident),
                    },
                };

                info!(?peeked_node);
                ast.push(peeked_node);
            } else {
                let node = match token {
                    Token::LCarrot => Node::Lt,
                    Token::RCarrot => Node::Gt,
                    Token::EqSign => Node::Eq,
                    Token::Space => Node::Space,
                    Token::Number(val) => match val {
                        crate::lexer::Number::U8(num) => Node::U8(*num),
                        crate::lexer::Number::U16(num) => Node::U16(*num),
                        crate::lexer::Number::U32(num) => Node::U32(*num),
                    },
                    Token::Lparen => Node::Lparen,
                    Token::Rparen => Node::Rparen,
                    Token::Ident(ident) => Node::Ident(ident),
                };

                ast.push(node);
            }

            idx += 1;
        }

        Ok(ast)
    }
}

#[cfg(test)]
mod test {

    use crate::{lexer::Lexer, parser::Node};

    use super::{Ast, Parser, ParserError};

    #[derive(Debug, Default)]
    struct TestRunner {}

    impl TestRunner {
        pub fn run(src: &str) -> Result<Ast, ParserError> {
            let lexer = Lexer::new(src);
            let ast = lexer.lex()?.to_vec();
            Parser::default().parse(ast)
        }
    }

    #[test]
    pub fn gte() {
        assert_eq!(TestRunner::run(">=").unwrap().nodes, [Node::Gte]);
    }

    #[test]
    pub fn gt() {
        assert_eq!(TestRunner::run(">").unwrap().nodes, [Node::Gt]);
    }

    #[test]
    pub fn gte_space() {
        assert_eq!(
            TestRunner::run(">= ").unwrap().nodes,
            [Node::Gte, Node::Space]
        );
    }

    #[test]
    pub fn gt_space() {
        assert_eq!(
            TestRunner::run("> ").unwrap().nodes,
            [Node::Gt, Node::Space]
        );
    }

    #[test]
    pub fn lte() {
        assert_eq!(TestRunner::run("<=").unwrap().nodes, [Node::Lte]);
    }

    #[test]
    pub fn lt() {
        assert_eq!(TestRunner::run("<").unwrap().nodes, [Node::Lt]);
    }

    #[test]
    pub fn lte_space() {
        assert_eq!(
            TestRunner::run("<= ").unwrap().nodes,
            [Node::Lte, Node::Space]
        );
    }

    #[test]
    pub fn lt_space() {
        assert_eq!(
            TestRunner::run("< ").unwrap().nodes,
            [Node::Lt, Node::Space]
        );
    }

    #[test]
    pub fn eq() {
        assert_eq!(TestRunner::run("=").unwrap().nodes, [Node::Eq]);
    }

    #[test]
    pub fn eq_space() {
        assert_eq!(
            TestRunner::run("= ").unwrap().nodes,
            [Node::Eq, Node::Space]
        );
    }

    #[test]
    pub fn nums() {
        assert_eq!(TestRunner::run("12345").unwrap().nodes, [Node::U16(12345)]);
    }

    #[test]
    pub fn idents() {
        assert_eq!(
            TestRunner::run("h e l l o").unwrap().nodes,
            [
                Node::Ident("h"),
                Node::Space,
                Node::Ident("e"),
                Node::Space,
                Node::Ident("l"),
                Node::Space,
                Node::Ident("l"),
                Node::Space,
                Node::Ident("o")
            ]
        );
    }
}
