use std::collections::HashMap;
use std::str::FromStr;

use crate::tokens::KeyWord;
// use crate::tokens::Label;
use crate::tokens::Number;
use crate::tokens::Tokenizer;
use crate::ParseError;
use crate::Token;

use tracing::info;
use tracing::instrument;
use vm_cpu::memory::address::Address;
use vm_cpu::opcodes::Instruction;
use vm_cpu::opcodes::OpCode;
use vm_cpu::opcodes::Value;

#[derive(Debug)]
pub struct Parser {
    data: String,
    insts: Vec<Instruction>,
    symbol_table: HashMap<String, Address>,
    patchlist: Option<Vec<usize>>,
}

impl Parser {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.trim().to_string(),
            insts: vec![],
            symbol_table: HashMap::default(),
            patchlist: None,
        }
    }

    pub fn insts(&self) -> &[Instruction] {
        &self.insts
    }

    #[instrument(skip(self))]
    pub fn parse(&mut self) -> Result<&[Instruction], ParseError> {
        if self.data.is_empty() {
            return Err(ParseError::EmptyFile);
        }

        let mut address = 0;

        info!(?self.data);

        let tokenizer = Tokenizer::default().tokenize(&self.data);

        // let tokens = &mut self
        //     .data
        //     .split_whitespace()
        //     .map_while(|word| {
        //         if let Ok(mut token) = Token::from_str(word) {
        //             match token {
        //                 Token::KeyWord(key_word) => address += key_word.increment_amount() as usize,
        //                 Token::Label(ref label) => match label {
        //                     Label::Unresolved(name) => {
        //                         if let Some(address) = self.symbol_table.get(name) {
        //                             token = Token::Number(Number::U16(u16::from(*address)));
        //                         } else if let Some(ref mut list) = &mut self.patchlist {
        //                             list.push(address)
        //                         } else {
        //                             self.patchlist = Some(vec![address]);
        //                         }
        //                     }
        //                     Label::Resolved(name) => {
        //                         self.symbol_table.insert(
        //                             name[0..name.len().saturating_sub(1)].to_string(),
        //                             Address::from(address as u16),
        //                         );
        //                     }
        //                 },
        //                 _ => {}
        //             }
        //
        //             return Some(token);
        //         }
        //         None
        //     })
        //     .collect::<Vec<_>>();

        info!(?tokenizer);

        // if let Some(list) = &self.patchlist {
        //     info!(?list);
        //     for patch in list {
        //         if let Token::Label(label) = &tokens[*patch] {
        //             info!(?label);
        //             match label {
        //                 Label::Unresolved(name) => {
        //                     if let Some(addr) = self.symbol_table.get(name) {
        //                         tokens[*patch] = Token::Number(Number::U16(u16::from(*addr)));
        //                     }
        //                 }
        //                 Label::Resolved(name) => {
        //                     self.symbol_table.insert(
        //                         name[0..name.len().saturating_sub(1)].to_string(),
        //                         Address::from(address as u16),
        //                     );
        //                 }
        //             }
        //         }
        //     }
        // }
        //
        // let mut idx = 0;
        //
        // while let Some(token) = tokens.get(idx) {
        //     info!(?token);
        //     let keyword = match token {
        //         Token::KeyWord(key_word) => *key_word,
        //         Token::Label(_) => {
        //             idx += 1;
        //             continue;
        //         }
        //         _ => panic!("expected keyword found token {token:?}"),
        //     };
        //
        //     let Some(tokens) = tokens.get(idx..idx + keyword.increment_amount() as usize) else {
        //         break;
        //     };
        //     let inst = tokens_to_instructions(tokens)?;
        //     self.insts.push(inst);
        //     idx += keyword.increment_amount() as usize;
        // }

        Ok(self.insts())
    }
}

fn tokens_to_instructions(tokens: &[Token]) -> Result<Instruction, ParseError> {
    assert!(!tokens.is_empty());

    match tokens[0] {
        Token::KeyWord(key_word) => match key_word {
            KeyWord::Mov => match tokens[1] {
                Token::Register(reg) => match &tokens[2] {
                    Token::Register(register) => Ok(Instruction::MovRegReg(reg, *register)),
                    Token::Number(num) => match num {
                        Number::U8(val) => Ok(Instruction::MovRegNum(reg, Value::U8(*val))),
                        Number::U16(val) => Ok(Instruction::MovRegNum(reg, Value::U16(*val))),
                        Number::U32(val) => Ok(Instruction::MovRegNum(reg, Value::U32(*val))),
                    },
                    _ => panic!("invalid token type"),
                },
                _ => panic!("invalid token type"),
            },
            KeyWord::Add => match tokens[1] {
                Token::Register(reg) => match &tokens[2] {
                    Token::Register(register) => Ok(Instruction::AddRegReg(reg, *register)),
                    Token::Number(num) => match num {
                        Number::U8(val) => Ok(Instruction::AddRegNum(reg, Value::U8(*val))),
                        Number::U16(val) => Ok(Instruction::AddRegNum(reg, Value::U16(*val))),
                        Number::U32(val) => Ok(Instruction::AddRegNum(reg, Value::U32(*val))),
                    },
                    _ => panic!("invalid token type"),
                },
                _ => panic!("invalid token type"),
            },
            KeyWord::Load => match tokens[1] {
                Token::Register(reg) => match &tokens[2] {
                    Token::Number(num) => match num {
                        Number::U8(val) => Ok(Instruction::Load(reg, Address::from(val))),
                        Number::U16(val) => Ok(Instruction::Load(reg, Address::from(val))),
                        Number::U32(val) => Ok(Instruction::Load(reg, Address::from(val))),
                    },
                    _ => panic!("invalid token type"),
                },
                _ => panic!("invalid token type"),
            },
            KeyWord::Jump => match &tokens[1] {
                Token::Number(num) => match num {
                    Number::U8(val) => Ok(Instruction::Jump(Address::from(val))),
                    Number::U16(val) => Ok(Instruction::Jump(Address::from(val))),
                    Number::U32(val) => Ok(Instruction::Jump(Address::from(val))),
                },
                // Token::Label(label) => todo!("{label}"),
                _ => panic!("invalid token type"),
            },
            KeyWord::Push => match &tokens[1] {
                Token::Register(reg) => Ok(Instruction::PushReg(*reg)),
                Token::Number(num) => match num {
                    Number::U8(val) => Ok(Instruction::PushVal(Value::U8(*val))),
                    Number::U16(val) => Ok(Instruction::PushVal(Value::U16(*val))),
                    Number::U32(val) => Ok(Instruction::PushVal(Value::U32(*val))),
                },
                // Token::Label(label) => todo!("{label}"),
                _ => panic!("invalid token type"),
            },
            KeyWord::Pop => match tokens[1] {
                Token::Register(reg) => Ok(Instruction::PopReg(reg)),
                _ => panic!("invalid token type"),
            },
            KeyWord::Call => {
                if tokens.len() == 1 {
                    return Err(ParseError::InvalidToken(Token::KeyWord(KeyWord::Call)));
                }
                match &tokens[1] {
                    Token::Number(Number::U16(addr)) => Ok(Instruction::Call(Address::from(*addr))),
                    // Token::Label(label) => todo!("calling a label: {label}"),
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Ret => {
                assert!(tokens.len() == OpCode::Ret.increment_amount() as usize);
                Ok(Instruction::Ret)
            }
            KeyWord::Halt => {
                assert!(tokens.len() == OpCode::Halt.increment_amount() as usize);
                Ok(Instruction::Halt)
            }
            KeyWord::Cmp => todo!(),
        },
        _ => panic!("Expected Keyword"),
    }
}

impl FromStr for Parser {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(s);
        parser.parse()?;

        Ok(parser)
    }
}

#[cfg(test)]
mod test {
    use tracing::{info, level_filters::LevelFilter};
    use tracing_subscriber::util::SubscriberInitExt;

    use super::Parser;

    fn parse(src: &str) {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(LevelFilter::INFO)
            .finish()
            .try_init();

        let mut parser = Parser::new(src);

        let parser = parser.parse();

        assert!(parser.is_ok())
    }

    #[test]
    #[ignore = "unsupported"]
    fn label() {
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
}
