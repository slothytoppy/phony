use std::fmt::Display;

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::util::SubscriberInitExt;

use crate::lexer::{LexError, Lexer, Token};

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
pub struct Parser {}

#[derive(PartialEq, Debug)]
pub enum Node {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Space,
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Default, Debug)]
pub struct Ast {
    nodes: Vec<Node>,
}

impl Ast {
    pub fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }
}

impl Parser {
    pub fn parse(&mut self, data: &str) -> Result<Ast, ParserError> {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(LevelFilter::INFO)
            .finish()
            .try_init();

        let tokens = Lexer::new(data).lex()?;
        info!(?tokens);
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
    use crate::parser::Node;

    use super::Parser;

    #[test]
    pub fn gte() {
        assert_eq!(Parser::default().parse(">=").unwrap().nodes, [Node::Gte]);
    }

    #[test]
    pub fn gt() {
        assert_eq!(Parser::default().parse(">").unwrap().nodes, [Node::Gt]);
    }

    #[test]
    pub fn gte_space() {
        assert_eq!(
            Parser::default().parse(">= ").unwrap().nodes,
            [Node::Gte, Node::Space]
        );
    }

    #[test]
    pub fn gt_space() {
        assert_eq!(
            Parser::default().parse("> ").unwrap().nodes,
            [Node::Gt, Node::Space]
        );
    }

    #[test]
    pub fn lte() {
        assert_eq!(Parser::default().parse("<=").unwrap().nodes, [Node::Lte]);
    }

    #[test]
    pub fn lt() {
        assert_eq!(Parser::default().parse("<").unwrap().nodes, [Node::Lt]);
    }

    #[test]
    pub fn lte_space() {
        assert_eq!(
            Parser::default().parse("<= ").unwrap().nodes,
            [Node::Lte, Node::Space]
        );
    }

    #[test]
    pub fn lt_space() {
        assert_eq!(
            Parser::default().parse("< ").unwrap().nodes,
            [Node::Lt, Node::Space]
        );
    }

    #[test]
    pub fn eq() {
        assert_eq!(Parser::default().parse("=").unwrap().nodes, [Node::Eq]);
    }

    #[test]
    pub fn eq_space() {
        assert_eq!(
            Parser::default().parse("= ").unwrap().nodes,
            [Node::Eq, Node::Space]
        );
    }

    #[test]
    pub fn nums() {
        assert_eq!(
            Parser::default().parse("12345").unwrap().nodes,
            [Node::U16(12345)]
        );
    }
}
