mod reg;

use std::fmt::Display;

pub use self::reg::*;

pub enum LoadSource {
    Register(Reg),
    Immediate(i64),
}

impl Display for LoadSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            LoadSource::Register(reg) => reg.to_string(),
            LoadSource::Immediate(val) => format!("{}", val),
        })
    }
}

pub enum Instruction {
    Lea(Reg, LoadSource),
    Mov(Reg, LoadSource),
    Syscall,
    Ret,
    Call(String),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Instruction::Lea(target, source) => format!("lea {}, {}", target, source),
            Instruction::Mov(target, source) => format!("mov {}, {}", target, source),
            Instruction::Syscall => "syscall".to_string(),
            Instruction::Ret => "ret".to_string(),
            Instruction::Call(name) => format!("call {}", name),
        };
        f.write_str(&value)
    }
}
