use std::ops::Index;

use vm_cpu::{opcodes::Instruction, registers::Register};

#[derive(Debug, Clone)]
pub struct Parser {
    data: String,
    insts: Vec<Instruction>,
}

#[derive(Debug, Clone)]
enum KeyWord {
    Mov,
    Pop,
    Ret,
    Halt,
    Jump,
    Load,
}

impl KeyWord {
    pub fn increment_amount(&self) -> u16 {
        match self {
            KeyWord::Mov => 3,
            KeyWord::Pop => 2,
            KeyWord::Ret => 1,
            KeyWord::Halt => 1,
            KeyWord::Jump => 2,
            KeyWord::Load => 3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Arg {
    Register(Register),
    U16(u16),
}

impl Parser {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.trim().to_string(),
            insts: Vec::new(),
        }
    }

    pub fn insts(&self) -> &Vec<Instruction> {
        &self.insts
    }

    pub fn parse(self) -> Self {
        let mut clone = self;
        let mut words: Vec<&str> = Vec::new();
        let mut idx = 0;
        for line in clone.data.lines() {
            for word in line.split_whitespace() {
                words.push(word);
            }
        }
        for word in &words {
            let Some((keyword, incr_amount)) = clone.get_keyword(word) else {
                continue;
            };
            //println!("keyword: {keyword:?}");
            let bytecode = &words.get(idx..idx + incr_amount as usize);
            //println!("bytecode {bytecode:?}");
            if let Some(code) = bytecode {
                let args = clone.get_args(&keyword, code);
                if let Some(args) = args {
                    //println!("args {args:?}");
                    let inst = clone.args_to_inst(keyword, &args);
                    //println!("inst {inst:?}");
                    match inst {
                        Some(inst) => clone.insts.push(inst),
                        None => continue,
                    }
                } else {
                    match keyword {
                        KeyWord::Ret => clone.insts.push(Instruction::Ret),
                        KeyWord::Halt => clone.insts.push(Instruction::Halt),
                        _ => {}
                    }
                }

                idx += incr_amount as usize;
            }
        }
        clone
    }

    fn get_keyword(&self, word: &str) -> Option<(KeyWord, u16)> {
        match word {
            "mov" => Some((KeyWord::Mov, KeyWord::Mov.increment_amount())),
            "pop" => Some((KeyWord::Pop, KeyWord::Pop.increment_amount())),
            "jump" => Some((KeyWord::Jump, KeyWord::Jump.increment_amount())),
            "halt" => Some((KeyWord::Halt, KeyWord::Halt.increment_amount())),
            "ret" => Some((KeyWord::Ret, KeyWord::Ret.increment_amount())),
            "load" => Some((KeyWord::Load, KeyWord::Load.increment_amount())),
            _ => None,
        }
    }

    fn get_arg(&self, word: &KeyWord, bytecode: &str) -> Option<Arg> {
        let arg = bytecode;
        let arg = {
            let idx = arg.find(',');
            if let Some(idx) = idx {
                arg.index(0..idx)
            } else {
                arg
            }
        };
        match arg {
            "r1" => Some(Arg::Register(Register::R1)),
            "r2" => Some(Arg::Register(Register::R2)),
            "r3" => Some(Arg::Register(Register::R3)),
            "r4" => Some(Arg::Register(Register::R4)),
            _ => {
                //println!("arg {arg:?}");
                match arg.parse::<u16>() {
                    Ok(num) => Some(Arg::U16(num)),
                    Err(e) => {
                        println!("{e:?} {arg}");
                        None
                    }
                }
            }
        }
    }

    fn get_args(&self, word: &KeyWord, bytecode: &[&str]) -> Option<Vec<Arg>> {
        let mut vec = Vec::new();
        match word {
            KeyWord::Mov => {
                let left = self.get_arg(word, bytecode[1])?;
                match left {
                    Arg::Register(_) => vec.push(left),
                    Arg::U16(_) => panic!(),
                }
                let right = self.get_arg(word, bytecode[2])?;

                vec.push(right);
                return Some(vec);
            }
            KeyWord::Pop => {
                let arg = self.get_arg(word, bytecode[1])?;
                match arg {
                    Arg::Register(register) => todo!(),
                    Arg::U16(_) => vec.push(arg),
                }
            }
            KeyWord::Ret => return None,
            KeyWord::Halt => return None,
            KeyWord::Jump => {
                let arg = self.get_arg(word, bytecode[1]);
                if let Some(arg) = arg {
                    match arg {
                        Arg::U16(addr) => {
                            vec.push(arg);
                        }
                        Arg::Register(reg) => {
                            panic!()
                        }
                    }
                }
            }
            KeyWord::Load => {
                let (left, right) = (
                    self.get_arg(word, bytecode[1])?,
                    self.get_arg(word, bytecode[2])?,
                );
                match left {
                    Arg::Register(register) => vec.push(left),
                    Arg::U16(_) => panic!("Expected register found u16"),
                }
                match right {
                    Arg::Register(register) => panic!("Expected u16 found register: {register}"),
                    Arg::U16(addr) => vec.push(right),
                }
                return Some(vec);
            }
        }
        None
    }

    fn args_to_inst(&self, keyword: KeyWord, args: &[Arg]) -> Option<Instruction> {
        match keyword {
            KeyWord::Mov => {
                let reg = match args[0] {
                    Arg::Register(reg) => reg,
                    Arg::U16(num) => panic!("Expected Argument: Register found number: {num}"),
                };
                match args[1] {
                    Arg::Register(register) => Some(Instruction::PushRegReg(reg, register)),
                    Arg::U16(num) => Some(Instruction::PushRegVal(reg, num)),
                }
            }
            KeyWord::Pop => match args[0] {
                Arg::Register(register) => Some(Instruction::PopReg(register)),
                Arg::U16(num) => panic!("Expected Argument: Register found number {num}"),
            },
            KeyWord::Ret => None,
            KeyWord::Halt => None,
            KeyWord::Jump => match args[0] {
                Arg::Register(register) => {
                    panic!("Expected Argument: number found register {register}")
                }
                Arg::U16(num) => Some(Instruction::Jump(num)),
            },
            KeyWord::Load => {
                let reg = match args[0] {
                    Arg::Register(reg) => reg,
                    Arg::U16(num) => panic!("Expected Argument: Register found number: {num}"),
                };
                match args[1] {
                    Arg::Register(register) => {
                        panic!("Expected Argument: address found register: {register}")
                    }
                    Arg::U16(num) => Some(Instruction::Load(reg, num)),
                }
            }
        }
    }
}
