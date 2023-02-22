use std::fmt::Display;

use crate::typechecker::TypeInfo;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Reg {
    /// 1. function argument
    Rdi,
    Edi,
    Di,
    Dil,

    /// 2. function argument
    Rsi,
    Esi,
    Si,
    Sil,

    /// Return value
    Rax,
    Eax,
    Ax,
    Al,

    /// Preserved. Sometimes used to store the old value of the stack pointer
    Rbp,
    Ebp,
    Bp,
    Bpl,

    /// Stack pointer
    Rsp,
    Esp,
    Sp,
    Spl,

    /// Scratch register
    Rcx,
    Ecx,
    Cx,
    Cl,

    /// Scratch register
    Rdx,
    Edx,
    Dx,
    Dl,

    /// Scratch register
    R8,
    R8d,
    R8w,
    R8b,

    /// Scratch register
    R9,
    R9d,
    R9w,
    R9b,

    /// Scratch register
    R10,
    R10d,
    R10w,
    R10b,

    /// Scratch register
    R11,
    R11d,
    R11w,
    R11b,
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
            Reg::Esi => "esi",
            Reg::Si => "si",
            Reg::Sil => "sil",

            Reg::Rdx => "rdx",
            Reg::Edx => "edx",
            Reg::Dx => "dx",
            Reg::Dl => "dl",

            Reg::Rbp => "rbp",
            Reg::Ebp => "ebp",
            Reg::Bp => "bp",
            Reg::Bpl => "bpl",

            Reg::Rsp => "rsp",
            Reg::Esp => "esp",
            Reg::Sp => "sp",
            Reg::Spl => "spl",

            Reg::Rcx => "rcx",
            Reg::Ecx => "ecx",
            Reg::Cx => "cx",
            Reg::Cl => "cl",

            Reg::R8 => "r8",
            Reg::R8d => "r8d",
            Reg::R8w => "r8w",
            Reg::R8b => "r8b",

            Reg::R9 => "r9",
            Reg::R9d => "r9d",
            Reg::R9w => "r9w",
            Reg::R9b => "r9b",

            Reg::R10 => "r10",
            Reg::R10d => "r10d",
            Reg::R10w => "r10w",
            Reg::R10b => "r10b",

            Reg::R11 => "r11",
            Reg::R11d => "r11d",
            Reg::R11w => "r11w",
            Reg::R11b => "r11b",
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
            Rsi | Esi | Si | Sil => match info.var_size() {
                8 => Rsi,
                4 => Esi,
                2 => Si,
                1 => Sil,
                _ => unimplemented!(),
            },
            Rax | Eax | Ax | Al => match info.var_size() {
                8 => Rax,
                4 => Eax,
                2 => Ax,
                1 => Al,
                _ => unimplemented!(),
            },
            Rbp | Ebp | Bp | Bpl => match info.var_size() {
                8 => Rbp,
                4 => Ebp,
                2 => Bp,
                1 => Bpl,
                _ => unimplemented!(),
            },
            Rsp | Esp | Sp | Spl => match info.var_size() {
                8 => Rsp,
                4 => Esp,
                2 => Sp,
                1 => Spl,
                _ => unimplemented!(),
            },
            Rcx | Ecx | Cx | Cl => match info.var_size() {
                8 => Rcx,
                4 => Ecx,
                2 => Cx,
                1 => Cl,
                _ => unimplemented!(),
            },
            Rdx | Edx | Dx | Dl => match info.var_size() {
                8 => Rdx,
                4 => Edx,
                2 => Dx,
                1 => Dl,
                _ => unimplemented!(),
            },
            R8 | R8d | R8w | R8b => match info.var_size() {
                8 => R8,
                4 => R8d,
                2 => R8w,
                1 => R8b,
                _ => unimplemented!(),
            },
            R9 | R9d | R9w | R9b => match info.var_size() {
                8 => R9,
                4 => R9d,
                2 => R9w,
                1 => R9b,
                _ => unimplemented!(),
            },
            R10 | R10d | R10w | R10b => match info.var_size() {
                8 => R10,
                4 => R10d,
                2 => R10w,
                1 => R10b,
                _ => unimplemented!(),
            },
            R11 | R11d | R11w | R11b => match info.var_size() {
                8 => R11,
                4 => R11d,
                2 => R11w,
                1 => R11b,
                _ => unimplemented!(),
            },
        }
    }
}
