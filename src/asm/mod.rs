mod instructionsize;
mod reg;

use std::fmt::Display;

pub use self::instructionsize::InstructionSize;
pub use self::reg::*;

#[cfg(target_os = "macos")]
pub const EXIT_SYSCALL: InstructionOperand = InstructionOperand::Immediate(0x2000001);

#[cfg(target_os = "linux")]
pub const EXIT_SYSCALL: InstructionOperand = InstructionOperand::Immediate(60);

#[cfg(target_os = "macos")]
pub const WRITE_SYSCALL: InstructionOperand = InstructionOperand::Immediate(0x2000004);

#[cfg(target_os = "linux")]
pub const WRITE_SYSCALL: InstructionOperand = InstructionOperand::Immediate(1);

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
    Movzx(InstructionOperand, InstructionOperand),
    Add(InstructionOperand, InstructionOperand),
    Sub(InstructionOperand, InstructionOperand),
    Imul(InstructionOperand, InstructionOperand),
    Idiv(InstructionOperand),
    Xor(InstructionOperand, InstructionOperand),
    Cmp(InstructionOperand, InstructionOperand),
    Sete(InstructionOperand),
    Setl(InstructionOperand),
    Setg(InstructionOperand),
    Je(String),
    Jmp(String),
    Inc(Reg),
    Syscall,
    Ret,
    Call(String),
    Push(Reg),
    Pop(Reg),
    /// NOTE: Do never really use this, except for looooong literal assembly
    Literal(String),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Instruction::Comment(comment) => format!(" ; {comment}"),
            Instruction::Label(label) => format!("\n{label}:"),
            Instruction::Lea(target, source) => format!("\tlea \t{target}, \t{source}"),
            Instruction::Mov(target, source) => format!("\tmov \t{target}, \t{source}"),
            Instruction::Movzx(target, source) => format!("\tmovzx \t{target}, \t{source}"),
            Instruction::Add(target, source) => format!("\tadd \t{target}, \t{source}"),
            Instruction::Imul(target, source) => format!("\timul \t{target}, \t{source}"),
            Instruction::Idiv(source) => format!("\tidiv \t{source}"),
            Instruction::Sub(target, source) => format!("\tsub \t{target}, \t{source}"),
            Instruction::Xor(target, source) => format!("\txor \t{target}, \t{source}"),
            Instruction::Cmp(target, source) => format!("\tcmp \t{target}, \t{source}"),
            Instruction::Sete(target) => format!("\tsete \t{target}"),
            Instruction::Setl(target) => format!("\tsetl \t{target}"),
            Instruction::Setg(target) => format!("\tsetg \t{target}"),
            Instruction::Je(target) => format!("\tje {target}"),
            Instruction::Jmp(target) => format!("\tjmp {target}"),
            Instruction::Inc(target) => format!("\tinc {target}"),
            Instruction::Syscall => "\tsyscall".to_string(),
            Instruction::Ret => "\tret".to_string(),
            Instruction::Call(name) => format!("\tcall \t{name}"),
            Instruction::Push(source) => format!("\tpush \t{source}"),
            Instruction::Pop(target) => format!("\tpop \t{target}"),
            Instruction::Literal(string) => string.to_owned(),
        };
        f.write_str(&value)
    }
}
