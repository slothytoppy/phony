use std::collections::HashMap;
use std::str::FromStr;

use crate::parser::tokens::Label;
use crate::parser::KeyWord;

use super::ParseError;
use super::Token;

use vm_cpu::address::Address;
use vm_cpu::memory::Memory;
use vm_cpu::opcodes::Instruction;
use vm_cpu::opcodes::OpCode;

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
            data: data.to_string(),
            insts: vec![],
            symbol_table: HashMap::default(),
            patchlist: None,
        }
    }

    pub fn insts(&self) -> &[Instruction] {
        &self.insts
    }

    pub fn parse(&mut self) -> Result<&[Instruction], ParseError> {
        let mut address = 0;

        let tokens = &mut self
            .data
            .split_whitespace()
            .map_while(|word| {
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

        if let Some(list) = &self.patchlist {
            for patch in list {
                if let Token::Label(label) = &tokens[*patch] {
                    println!("label {label}");
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

        println!("symbols {:?}", self.symbol_table);

        let mut idx = 0;

        while let Some(token) = tokens.get(idx) {
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

    fn add_symbols(&mut self, tokens: &mut [Token]) {
        let mut address = 0;

        for keyword in tokens.iter() {
            match keyword {
                Token::KeyWord(key_word) => address += key_word.increment_amount(),
                Token::Label(label) => {
                    match label {
                        Label::Unresolved(_) => {}
                        Label::Resolved(name) => {
                            self.symbol_table.insert(
                                name[0..name.len().saturating_sub(1)].to_string(),
                                Address::from(address),
                            );
                        }
                    };
                }
                _ => {}
            }
        }

        for token in tokens.iter_mut() {
            if let Token::Label(label) = token {
                match label {
                    Label::Unresolved(label) => {
                        let addr = self.symbol_table.get_mut(label);
                        match addr {
                            Some(addr) => {
                                println!("before change: {token:?}");
                                *token = Token::U16(u16::from(*addr));
                                println!("after change: {token:?}");
                            }
                            None => panic!("unknown symbol {label}"),
                        }
                    }
                    Label::Resolved(_) => {}
                }
            }
        }
    }
}

impl From<Parser> for &[Instruction] {
    fn from(value: Parser) -> Self {
        &[]
    }
}

fn tokens_to_instructions(tokens: &[Token]) -> Result<Instruction, ParseError> {
    assert!(!tokens.is_empty());
    match tokens[0] {
        Token::KeyWord(key_word) => match key_word {
            KeyWord::Mov => {
                assert!(tokens.len() == OpCode::MovRegReg.increment_amount().into());
                match tokens[1] {
                    Token::Register(reg) => match tokens[2] {
                        Token::Register(register) => Ok(Instruction::MovRegReg(reg, register)),
                        Token::U16(val) => Ok(Instruction::MovRegVal(reg, val)),
                        _ => panic!("invalid token type"),
                    },
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Add => {
                assert!(tokens.len() == OpCode::AddRegNum.increment_amount().into());
                match tokens[1] {
                    Token::Register(reg) => match tokens[2] {
                        Token::Register(register) => Ok(Instruction::AddRegReg(reg, register)),
                        Token::U16(val) => Ok(Instruction::AddRegVal(reg, val)),
                        _ => panic!("invalid token type"),
                    },
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Load => {
                assert!(tokens.len() == OpCode::Load.increment_amount().into());
                match tokens[1] {
                    Token::Register(reg) => match tokens[2] {
                        Token::U16(addr) => Ok(Instruction::Load(reg, addr.into())),
                        _ => panic!("invalid token type"),
                    },
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Jump => {
                assert!(tokens.len() == OpCode::Jump.increment_amount().into());
                match &tokens[1] {
                    Token::U16(addr) => Ok(Instruction::Jump(Address::from(*addr))),
                    Token::Label(label) => todo!("{label}"),
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Push => {
                assert!(tokens.len() == OpCode::PushReg.increment_amount().into());
                match &tokens[1] {
                    Token::Register(reg) => Ok(Instruction::PushReg(*reg)),
                    Token::U16(val) => Ok(Instruction::PushVal(*val)),
                    Token::Label(label) => todo!("{label}"),
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Pop => {
                assert!(tokens.len() == OpCode::PopReg.increment_amount().into());
                match tokens[1] {
                    Token::Register(reg) => Ok(Instruction::PopReg(reg)),
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Call => {
                assert!(tokens.len() <= OpCode::CallAddr.increment_amount().into());
                if tokens.len() == 1 {
                    return Ok(Instruction::Call);
                }
                match &tokens[1] {
                    Token::U16(addr) => Ok(Instruction::CallAddr(Address::from(*addr))),
                    Token::Label(label) => todo!("calling a label: {label}"),
                    _ => panic!("invalid token type"),
                }
            }
            KeyWord::Ret => {
                assert!(tokens.len() == OpCode::Ret.increment_amount().into());
                Ok(Instruction::Ret)
            }
            KeyWord::Halt => {
                assert!(tokens.len() == OpCode::Halt.increment_amount().into());
                Ok(Instruction::Halt)
            }
        },
        _ => panic!("Expected Keyword"),
    }
}

pub fn write_instructions_to_memory<M>(
    memory: &mut M,
    instructions: &[Instruction],
) -> Result<(), vm_cpu::memory::Error>
where
    M: Memory,
{
    let mut offset = 0;
    for inst in instructions {
        memory.write(offset, OpCode::from(inst))?;
        offset += 1;
        match inst {
            Instruction::MovRegReg(register, register1) => {
                memory.write(offset, *register)?;
                offset += 1;
                memory.write(offset, *register1)?;
                offset += 1;
            }
            Instruction::MovRegVal(register, val) => {
                memory.write(offset, *register)?;
                offset += 1;
                memory.write(offset, *val)?;
                offset += 1;
            }
            Instruction::PushReg(register) => {
                memory.write(offset, *register)?;
                offset += 1;
            }
            Instruction::PushVal(val) => {
                memory.write(offset, *val)?;
                offset += 1;
            }
            Instruction::PopReg(register) => {
                memory.write(offset, *register)?;
                offset += 1;
            }
            Instruction::AddRegReg(register, register1) => {
                memory.write(offset, *register)?;
                offset += 1;
                memory.write(offset, *register1)?;
                offset += 1;
            }
            Instruction::AddRegVal(register, val) => {
                memory.write(offset, *register)?;
                offset += 1;
                memory.write(offset, *val)?;
                offset += 1;
            }
            Instruction::Jump(address) => {
                memory.write(offset, *address)?;
                offset += 1;
            }
            Instruction::Load(register, address) => {
                memory.write(offset, *register)?;
                offset += 1;
                memory.write(offset, *address)?;
                offset += 1;
            }
            Instruction::Call => {}
            Instruction::CallAddr(address) => {
                memory.write(offset, *address)?;
                offset += 1;
            }
            Instruction::Halt => {}
            Instruction::Ret => {}
        }
    }
    Ok(())
}

impl FromStr for Parser {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(s);
        parser.parse()?;

        Ok(parser)
    }
}
