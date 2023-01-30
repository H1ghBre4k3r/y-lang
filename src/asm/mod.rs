mod reg;

use std::fmt::Display;

pub use self::reg::*;

#[derive(Debug)]
pub enum LoadSource {
    Register(Reg),
    Immediate(i64),
    Identifier(String),
}

impl Display for LoadSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            LoadSource::Register(reg) => reg.to_string(),
            LoadSource::Immediate(val) => format!("{}", val),
            LoadSource::Identifier(ident) => format!("{}", ident),
        })
    }
}

#[derive(Debug)]
pub enum Instruction {
    Label(String),
    Lea(Reg, LoadSource),
    Mov(Reg, LoadSource),
    Syscall,
    Ret,
    Call(String),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Instruction::Label(label) => format!("{}:", label),
            Instruction::Lea(target, source) => format!("\tlea {}, {}", target, source),
            Instruction::Mov(target, source) => format!("\tmov {}, {}", target, source),
            Instruction::Syscall => "\tsyscall".to_string(),
            Instruction::Ret => "\tret".to_string(),
            Instruction::Call(name) => format!("\tcall {}", name),
        };
        f.write_str(&value)
    }
}
