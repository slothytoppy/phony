use std::ops::Index;

use vm_cpu::{opcodes::Instruction, registers::Register};

#[derive(Debug, Clone)]
pub struct Parser {
    data: String,
    idx: usize,
    insts: Vec<Instruction>,
}

#[derive(Debug, Clone)]
enum KeyWord {
    Mov,
    Pop,
    Ret,
    Halt,
    Jump,
}

impl KeyWord {
    pub fn increment_amount(&self) -> u16 {
        match self {
            KeyWord::Mov => 3,
            KeyWord::Pop => 2,
            KeyWord::Ret => 1,
            KeyWord::Halt => 1,
            KeyWord::Jump => 2,
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
            idx: 0,
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
            println!("keyword: {keyword:?}");
            let bytecode = &words.get(idx..idx + incr_amount as usize);
            println!("bytecode {bytecode:?}");
            if let Some(code) = bytecode {
                let args = clone.get_arg(&keyword, code);
                if let Some(args) = args {
                    println!("args {args:?}");
                    let inst = clone.args_to_inst(keyword, &args);
                    println!("inst {inst:?}");
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
            "mov" => Some((KeyWord::Mov, 3)),
            "pop" => Some((KeyWord::Pop, 2)),
            "jump" => Some((KeyWord::Jump, 2)),
            "halt" => Some((KeyWord::Halt, 1)),
            "ret" => Some((KeyWord::Ret, 1)),
            _ => None,
        }
    }

    fn get_arg(&self, word: &KeyWord, bytecode: &[&str]) -> Option<Vec<Arg>> {
        let mut vec = Vec::new();
        match word {
            KeyWord::Mov => {
                let left = bytecode[1];
                let left = {
                    let idx = left.find(',');
                    if let Some(idx) = idx {
                        left.index(0..idx)
                    } else {
                        left
                    }
                };
                let left_arg = match left {
                    "r1" => Arg::Register(Register::R1),
                    "r2" => Arg::Register(Register::R2),
                    "r3" => Arg::Register(Register::R3),
                    "r4" => Arg::Register(Register::R4),
                    _ => {
                        println!("left {left:?}");
                        match left.parse::<u16>() {
                            Ok(num) => Arg::U16(num),
                            Err(e) => {
                                println!("{e:?} {left}");
                                panic!();
                            }
                        }
                    }
                };

                vec.push(left_arg);

                let right = bytecode[2];
                let right_arg = match right {
                    "r1" => Arg::Register(Register::R1),
                    "r2" => Arg::Register(Register::R2),
                    "r3" => Arg::Register(Register::R3),
                    "r4" => Arg::Register(Register::R4),
                    _ => match right.parse::<u16>() {
                        Ok(num) => Arg::U16(num),
                        Err(e) => panic!("{e}"),
                    },
                };
                vec.push(right_arg);
                return Some(vec);
            }
            KeyWord::Pop => {}
            KeyWord::Ret => {}
            KeyWord::Halt => {}
            KeyWord::Jump => {}
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
        }
    }
}
