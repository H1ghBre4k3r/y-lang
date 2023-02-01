use std::fmt::Display;

#[derive(Debug)]
pub enum Reg {
    /// 1. function argument
    RDI,
    /// 2. function argument
    RSI,
    /// Return value
    RAX,

    /// Preserved. Sometimes used to store the old value of the stack pointer
    RBP,
    /// Stack pointer
    RSP,

    /// Scratch register
    RCX,
    /// Scratch register
    RDX,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Reg::RDI => "rdi",
            Reg::RAX => "rax",
            Reg::RSI => "rsi",
            Reg::RDX => "rdx",
            Reg::RBP => "rbp",
            Reg::RSP => "rsp",
            Reg::RCX => "rcx",
        })
    }
}
