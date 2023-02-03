use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Reg {
    /// 1. function argument
    Rdi,
    /// 2. function argument
    Rsi,
    /// Return value
    Rax,

    /// Preserved. Sometimes used to store the old value of the stack pointer
    Rbp,
    /// Stack pointer
    Rsp,

    /// Scratch register
    Rcx,
    /// Scratch register
    Rdx,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Reg::Rdi => "rdi",
            Reg::Rax => "rax",
            Reg::Rsi => "rsi",
            Reg::Rdx => "rdx",
            Reg::Rbp => "rbp",
            Reg::Rsp => "rsp",
            Reg::Rcx => "rcx",
        })
    }
}
