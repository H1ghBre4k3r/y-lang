use std::fmt::Display;

use crate::typechecker::TypeInfo;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum InstructionSize {
    None = 0,
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
            InstructionSize::None => "",
            InstructionSize::Byte => "byte",
            InstructionSize::Word => "word",
            InstructionSize::Dword => "dword",
            InstructionSize::Qword => "qword",
        })
    }
}

impl From<TypeInfo> for InstructionSize {
    fn from(value: TypeInfo) -> Self {
        use InstructionSize::*;

        match value.var_size() {
            1 => Byte,
            4 => Dword,
            8 => Qword,
            _ => unimplemented!("Variables of type '{value:?}' are currently not supported"),
        }
    }
}
