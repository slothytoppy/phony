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

impl Node {
    fn skip_amount(&self) -> Option<usize> {
        match self {
            Node::U8(val) => match val {
                0..=9 => Some(1),
                10..=99 => Some(2),
                _ => Some(3),
            },
            Node::U16(val) => match val {
                0..=9 => Some(1),
                10..=99 => Some(2),
                100..=999 => Some(3),
                1000..=9999 => Some(4),
                10000..=u16::MAX => Some(5),
            },
            Node::U32(val) => match val {
                0..=9 => Some(1),
                10..=99 => Some(2),
                100..=999 => Some(3),
                1000..=9999 => Some(4),
                10000..=99999 => Some(5),
                100_000..=999_999 => Some(6),
                1_000_000..=u32::MAX => Some(7),
            },
            _ => None,
        }
    }
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

                if let Some(amount) = peeked_node.skip_amount() {
                    idx += amount;
                }
                info!(?peeked_node);
                ast.push(peeked_node);
            } else {
                let node = match token {
                    Token::LCarrot => Node::Lt,
                    Token::RCarrot => Node::Gt,
                    Token::EqSign => Node::Eq,
                    Token::Space => Node::Space,
                    Token::Number(val) => match val {
                        crate::lexer::Number::U8(num) => {
                            idx += 3;
                            Node::U8(*num)
                        }
                        crate::lexer::Number::U16(num) => {
                            idx += 4;
                            Node::U16(*num)
                        }
                        crate::lexer::Number::U32(num) => {
                            idx += 8;
                            Node::U32(*num)
                        }
                    },
                };

                if let Some(amount) = node.skip_amount() {
                    idx += amount;
                }

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
