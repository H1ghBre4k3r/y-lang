use std::fmt::Display;

pub enum Reg {
    RDI,
    RAX,
    RSI,
    RDX,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Reg::RDI => "rdi",
            Reg::RAX => "rax",
            Reg::RSI => "RSI",
            Reg::RDX => "rdx",
        })
    }
}
