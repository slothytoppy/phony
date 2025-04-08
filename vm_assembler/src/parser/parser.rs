use std::collections::HashMap;
use std::str::FromStr;

use crate::parser::ast::Ast;
use crate::parser::ast::AstNode;
use crate::parser::tokens::KeyWord;
use crate::parser::tokens::Label;

use super::ParseError;
use super::Token;

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
        let mut ast = Ast::default();

        if self.data.is_empty() {
            return Err(ParseError::EmptyFile);
        }

        let mut address = 0;

        let tokens = &mut self
            .data
            .split_whitespace()
            .map_while(|word| {
                ast.push(AstNode::from_str(word).unwrap());
                if let Ok(mut token) = Token::from_str(word) {
                    match token {
                        Token::KeyWord(key_word) => address += key_word.increment_amount() as usize,
                        Token::Label(ref label) => match label {
                            Label::Unresolved(name) => {
                                if let Some(address) = self.symbol_table.get(name) {
                                    token = Token::U16(u16::from(*address));
                                } else if let Some(ref mut list) = &mut self.patchlist {
                                    list.push(address)
                                } else {
                                    self.patchlist = Some(vec![address]);
                                }
                            }
                            Label::Resolved(name) => {
                                self.symbol_table.insert(
                                    name[0..name.len().saturating_sub(1)].to_string(),
                                    Address::from(address as u16),
                                );
                            }
                        },
                        _ => {}
                    }

                    return Some(token);
                }
                None
            })
            .collect::<Vec<_>>();

        info!(?tokens);
        info!(?ast);

        if let Some(list) = &self.patchlist {
            info!(?list);
            for patch in list {
                if let Token::Label(label) = &tokens[*patch] {
                    info!(?label);
                    match label {
                        Label::Unresolved(name) => {
                            if let Some(addr) = self.symbol_table.get(name) {
                                tokens[*patch] = Token::U16(u16::from(*addr));
                            }
                        }
                        Label::Resolved(name) => {
                            self.symbol_table.insert(
                                name[0..name.len().saturating_sub(1)].to_string(),
                                Address::from(address as u16),
                            );
                        }
                    }
                }
            }
        }

        let mut idx = 0;

        while let Some(token) = tokens.get(idx) {
            info!(?token);
            let keyword = match token {
                Token::KeyWord(key_word) => *key_word,
                Token::Label(_) => {
                    idx += 1;
                    continue;
                }
                _ => panic!("expected keyword found token {token:?}"),
            };

            let Some(tokens) = tokens.get(idx..idx + keyword.increment_amount() as usize) else {
                break;
            };
            let inst = tokens_to_instructions(tokens)?;
            self.insts.push(inst);
            idx += keyword.increment_amount() as usize;
        }

        Ok(self.insts())
    }
}

fn tokens_to_instructions(tokens: &[Token]) -> Result<Instruction, ParseError> {
    assert!(!tokens.is_empty());
    match tokens[0] {
        Token::KeyWord(key_word) => match key_word {
            KeyWord::Mov => {
                assert!(tokens.len() == OpCode::MovRegReg.increment_amount() as usize);
                match tokens[1] {
                    Token::Register(reg) => match tokens[2] {
                        Token::Register(register) => Ok(Instruction::MovRegReg(reg, register)),
                        Token::U16(val) => Ok(Instruction::MovRegNum(reg, Value::U16(val))),
                        _ => panic!("invalid token type"),
                    },
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Add => {
                assert!(tokens.len() == OpCode::AddRegNum.increment_amount() as usize);
                match tokens[1] {
                    Token::Register(reg) => match tokens[2] {
                        Token::Register(register) => Ok(Instruction::AddRegReg(reg, register)),
                        Token::U16(val) => Ok(Instruction::AddRegNum(reg, Value::U16(val))),
                        _ => panic!("invalid token type"),
                    },
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Load => {
                assert!(tokens.len() == OpCode::Load.increment_amount() as usize);
                match tokens[1] {
                    Token::Register(reg) => match tokens[2] {
                        Token::U16(addr) => Ok(Instruction::Load(reg, addr.into())),
                        _ => panic!("invalid token type"),
                    },
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Jump => {
                assert!(tokens.len() == OpCode::Jump.increment_amount() as usize);
                match &tokens[1] {
                    Token::U16(addr) => Ok(Instruction::Jump(Address::from(*addr))),
                    Token::Label(label) => todo!("{label}"),
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Push => {
                assert!(tokens.len() == OpCode::PushReg.increment_amount() as usize);
                match &tokens[1] {
                    Token::Register(reg) => Ok(Instruction::PushReg(*reg)),
                    Token::U16(val) => Ok(Instruction::PushVal(Value::U16(*val))),
                    Token::Label(label) => todo!("{label}"),
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Pop => {
                assert!(tokens.len() == OpCode::PopReg.increment_amount() as usize);
                match tokens[1] {
                    Token::Register(reg) => Ok(Instruction::PopReg(reg)),
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Call => {
                assert!(tokens.len() <= OpCode::Call.increment_amount() as usize);
                if tokens.len() == 1 {
                    return Err(ParseError::InvalidToken(Token::KeyWord(KeyWord::Call)));
                }
                match &tokens[1] {
                    Token::U16(addr) => Ok(Instruction::Call(Address::from(*addr))),
                    Token::Label(label) => todo!("calling a label: {label}"),
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
