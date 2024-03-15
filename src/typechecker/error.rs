use std::{error::Error, fmt::Display};

use super::types::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeCheckError {
    TypeMismatch(TypeMismatch),
    UndefinedVariable(UndefinedVariable),
}

impl Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeCheckError::TypeMismatch(e) => f.write_fmt(format_args!("{e}")),
            TypeCheckError::UndefinedVariable(e) => f.write_fmt(format_args!("{e}")),
        }
    }
}

impl Error for TypeCheckError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeMismatch {
    pub expected: Type,
    pub actual: Type,
}

impl Display for TypeMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Expected type '{:?}' but got '{:?}'",
            self.expected, self.actual
        ))
    }
}

impl Error for TypeMismatch {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndefinedVariable {
    pub variable_name: String,
}

impl Display for UndefinedVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Tried to access undefined variable {}",
            self.variable_name
        ))
    }
}

impl Error for UndefinedVariable {}
