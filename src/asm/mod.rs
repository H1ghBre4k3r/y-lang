mod reg;

use std::fmt::Display;

pub use self::reg::*;

#[derive(Debug, Clone)]
pub enum InstructionSize {
    /// size for working with 1 byte
    Byte = 1,

    /// size for working with 2 bytes
    Word = 2,

    /// size for working with 4 bytes
    Dword = 4,

    /// size for working with 8 bytes
    Qword = 8,
}

impl Display for InstructionSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InstructionSize::Byte => "byte",
            InstructionSize::Word => "word",
            InstructionSize::Dword => "dword",
            InstructionSize::Qword => "qword",
        })
    }
}

#[derive(Debug, Clone)]
pub enum InstructionOperand {
    Register(Reg),
    Immediate(i64),
    Memory(InstructionSize, String),
    Identifier(String),
}

impl Display for InstructionOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            InstructionOperand::Register(reg) => reg.to_string(),
            InstructionOperand::Immediate(val) => format!("{val}"),
            InstructionOperand::Identifier(ident) => ident.to_string(),
            InstructionOperand::Memory(size, location) => format!("{size} [{location}]"),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Comment(String),
    Label(String),
    Lea(InstructionOperand, InstructionOperand),
    Mov(InstructionOperand, InstructionOperand),
    Add(InstructionOperand, InstructionOperand),
    Sub(InstructionOperand, InstructionOperand),
    Xor(InstructionOperand, InstructionOperand),
    Cmp(InstructionOperand, InstructionOperand),
    Je(String),
    Jmp(String),
    Inc(Reg),
    Syscall,
    Ret,
    Call(String),
    Push(Reg),
    Pop(Reg),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Instruction::Comment(comment) => format!(" ; {comment}"),
            Instruction::Label(label) => format!("\n{label}:"),
            Instruction::Lea(target, source) => format!("\tlea \t{target}, \t{source}"),
            Instruction::Mov(target, source) => format!("\tmov \t{target}, \t{source}"),
            Instruction::Add(target, source) => format!("\tadd \t{target}, \t{source}"),
            Instruction::Sub(target, source) => format!("\tsub \t{target}, \t{source}"),
            Instruction::Xor(target, source) => format!("\txor \t{target}, \t{source}"),
            Instruction::Cmp(target, source) => format!("\tcmp \t{target}, \t{source}"),
            Instruction::Je(target) => format!("\tje {target}"),
            Instruction::Jmp(target) => format!("\tjmp {target}"),
            Instruction::Inc(target) => format!("\tinc {target}"),
            Instruction::Syscall => "\tsyscall".to_string(),
            Instruction::Ret => "\tret".to_string(),
            Instruction::Call(name) => format!("\tcall \t{name}"),
            Instruction::Push(source) => format!("\tpush \t{source}"),
            Instruction::Pop(target) => format!("\tpop \t{target}"),
        };
        f.write_str(&value)
    }
}
