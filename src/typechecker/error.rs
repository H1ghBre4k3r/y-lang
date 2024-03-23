use std::{error::Error, fmt::Display};

use crate::parser::ast::TypeName;

use super::types::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeCheckError {
    TypeMismatch(TypeMismatch),
    UndefinedVariable(UndefinedVariable),
    UndefinedType(UndefinedType),
    InvalidConstantType(InvalidConstantType),
    RedefinedConstant(RedefinedConstant),
}

impl Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeCheckError::TypeMismatch(e) => f.write_fmt(format_args!("{e}")),
            TypeCheckError::UndefinedVariable(e) => f.write_fmt(format_args!("{e}")),
            TypeCheckError::UndefinedType(e) => f.write_fmt(format_args!("{e}")),
            TypeCheckError::InvalidConstantType(e) => f.write_fmt(format_args!("{e}")),
            TypeCheckError::RedefinedConstant(e) => f.write_fmt(format_args!("{e}")),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndefinedType {
    pub type_name: TypeName,
}

impl Display for UndefinedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Tried to use undefined type {:?}",
            self.type_name
        ))
    }
}

impl Error for UndefinedType {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidConstantType {
    pub constant_name: String,
}

impl Display for InvalidConstantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Constant '{}' needs to have an annotated type",
            self.constant_name
        ))
    }
}

impl Error for InvalidConstantType {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedefinedConstant {
    pub constant_name: String,
}

impl Display for RedefinedConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Constant '{}' is already defined",
            self.constant_name
        ))
    }
}

impl Error for RedefinedConstant {}
