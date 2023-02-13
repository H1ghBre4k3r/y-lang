use std::fmt::Display;

use crate::typechecker::TypeInfo;

#[derive(Debug, Clone, Copy)]
pub enum Reg {
    /// 1. function argument
    Rdi,
    Edi,
    Di,
    Dil,

    /// 2. function argument
    Rsi,
    /// Return value
    Rax,
    Eax,
    Ax,
    Al,

    /// Preserved. Sometimes used to store the old value of the stack pointer
    Rbp,
    /// Stack pointer
    Rsp,

    /// Scratch register
    Rcx,
    /// Scratch register
    Rdx,

    R8,
    R9,
    R10,
    R11,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Reg::Rdi => "rdi",
            Reg::Edi => "edi",
            Reg::Di => "di",
            Reg::Dil => "dil",

            Reg::Rax => "rax",
            Reg::Eax => "eax",
            Reg::Ax => "ax",
            Reg::Al => "al",

            Reg::Rsi => "rsi",
            Reg::Rdx => "rdx",
            Reg::Rbp => "rbp",
            Reg::Rsp => "rsp",
            Reg::Rcx => "rcx",

            Reg::R8 => "r8",
            Reg::R9 => "r9",
            Reg::R10 => "r10",
            Reg::R11 => "r11",
        })
    }
}

impl Reg {
    pub fn to_sized(self, info: &TypeInfo) -> Self {
        use Reg::*;
        match self {
            Rdi | Edi | Di | Dil => match info.var_size() {
                8 => Rdi,
                4 => Edi,
                2 => Di,
                1 => Dil,
                _ => unimplemented!(),
            },
            Rsi => match info.var_size() {
                8 => Rsi,
                _ => unimplemented!(),
            },
            Rax | Eax | Ax | Al => match info.var_size() {
                8 => Rax,
                4 => Eax,
                2 => Ax,
                1 => Al,
                _ => unimplemented!(),
            },
            Rbp => match info.var_size() {
                8 => Rbp,
                _ => unimplemented!(),
            },
            Rsp => match info.var_size() {
                8 => Rsp,
                _ => unimplemented!(),
            },
            Rcx => match info.var_size() {
                8 => Rcx,
                _ => unimplemented!(),
            },
            Rdx => match info.var_size() {
                8 => Rdx,
                _ => unimplemented!(),
            },
            R8 => match info.var_size() {
                8 => R8,
                _ => unimplemented!(),
            },
            R9 => match info.var_size() {
                8 => R9,
                _ => unimplemented!(),
            },
            R10 => match info.var_size() {
                8 => R10,
                _ => unimplemented!(),
            },
            R11 => match info.var_size() {
                8 => R11,
                _ => unimplemented!(),
            },
        }
    }
}
