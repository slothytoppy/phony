use crate::tokens::Tokenizer;
use crate::ParseError;
use crate::Token;

use tracing::info;
use tracing::instrument;
use vm_cpu::memory::Address;

#[derive(Debug)]
pub enum Label {
    Unresolved(usize),
    Resolved(Address),
}

#[derive(Debug)]
pub enum ParseToken<'a> {
    Token(Token<'a>),
    Label(Label),
}

impl ParseToken<'_> {
    fn size(&self) -> Option<usize> {
        match self {
            ParseToken::Token(token) => match token {
                Token::KeyWord(_) => Some(1),
                Token::Register(_) => Some(1),
                Token::Number(number) => match number {
                    crate::tokens::Number::U8(_) => Some(1),
                    crate::tokens::Number::U16(_) => Some(2),
                    crate::tokens::Number::U32(_) => Some(4),
                },
                Token::Address(_) => Some(4),
                Token::Identifier(_) => None,
            },
            ParseToken::Label(_) => None,
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

impl<'a> Parser<'a> {
    pub fn push(&mut self, token: impl Into<ParseToken<'a>>) {
        self.tokens.push(token.into());
    }

    #[instrument(skip(self))]
    pub fn parse(&'a mut self, data: &'a str) -> Result<(), ParseError<'a>> {
        if data.is_empty() {
            return Err(ParseError::EmptyFile);
        }

        let tokenizer = Tokenizer::tokenize(data);

        info!(?tokenizer);

        let mut addr: usize = 0;
        let mut idx = 0;

        for token in tokenizer {
            match token {
                Token::Identifier(ident) => {
                    match ident {
                        crate::tokens::Ident::Ident(s) => {
                            if s.ends_with(":") {
                                self.push(ParseToken::Label(Label::Resolved(addr.into())));
                                addr += 4;
                            } else {
                                self.push(ParseToken::Label(Label::Unresolved(idx)));
                            }
                        }
                    };
                }

                _ => {
                    let tok = ParseToken::from(token);

                    if let Some(amount) = tok.size() {
                        addr += amount;
                    }

                    self.push(tok);
                }
            }
            idx += 1;
        }

        info!(?self);

        for token in &self.tokens {
            match token {
                ParseToken::Token(_token) => {}
                ParseToken::Label(ref label) => match label {
                    Label::Unresolved(unres) => {
                        let res = tokenizer.get(*unres).unwrap();
                        info!(?res)
                    }
                    Label::Resolved(_address) => {
                        info!(?token)
                    }
                },
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::util::SubscriberInitExt;

    use super::Parser;

    fn parse(src: &str) -> Parser {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(LevelFilter::INFO)
            .finish()
            .try_init();

        let mut parser = Parser::default();

        _ = parser.parse(src);

        parser
    }

    #[test]
    fn label() {
        parse("urmom:");
    }

    #[test]
    fn call_label() {
        parse("urmom");
    }

    #[test]
    fn add() {
        parse("add r1, 10");
    }

    #[test]
    fn mov_val() {
        parse("mov r1, 0");
    }

    #[test]
    fn mov_reg() {
        parse("mov r1, r2");
    }

    #[test]
    fn mov_mem() {
        parse("mov r1, [10]");
    }

    #[test]
    fn mem_mov_reg() {
        parse("mov [10], r1");
    }

    #[test]
    fn mem_mov_val() {
        parse("mov [10], 10");
    }

    #[test]
    fn cmp() {
        parse("cmp r1");
    }

    #[test]
    fn labels_test() {
        parse("foo:\nbar:\nbaz:");
    }

    #[test]
    fn unresolved_label() {
        let parser = parse("foo\nfoo:");
    }
}
