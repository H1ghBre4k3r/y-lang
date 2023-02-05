use std::fmt::Display;

#[derive(Debug, Clone)]
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
