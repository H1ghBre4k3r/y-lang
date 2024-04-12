use std::{error::Error, fmt::Display};

use crate::{lexer::Span, parser::ast::TypeName};

use super::types::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeCheckError {
    TypeMismatch(TypeMismatch, Span),
    UndefinedVariable(UndefinedVariable, Span),
    UndefinedType(UndefinedType, Span),
    InvalidConstantType(InvalidConstantType, Span),
    RedefinedConstant(RedefinedConstant, Span),
    ImmutableReassign(ImmutableReassign, Span),
}

impl Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.span().to_string(self.err()).as_str())
    }
}

impl TypeCheckError {
    fn span(&self) -> Span {
        match self {
            TypeCheckError::TypeMismatch(_, span) => span.clone(),
            TypeCheckError::UndefinedVariable(_, span) => span.clone(),
            TypeCheckError::UndefinedType(_, span) => span.clone(),
            TypeCheckError::InvalidConstantType(_, span) => span.clone(),
            TypeCheckError::RedefinedConstant(_, span) => span.clone(),
            TypeCheckError::ImmutableReassign(_, span) => span.clone(),
        }
    }

    fn err(&self) -> Box<dyn Error> {
        match self {
            TypeCheckError::TypeMismatch(e, _) => Box::new(e.clone()),
            TypeCheckError::UndefinedVariable(e, _) => Box::new(e.clone()),
            TypeCheckError::UndefinedType(e, _) => Box::new(e.clone()),
            TypeCheckError::InvalidConstantType(e, _) => Box::new(e.clone()),
            TypeCheckError::RedefinedConstant(e, _) => Box::new(e.clone()),
            TypeCheckError::ImmutableReassign(e, _) => Box::new(e.clone()),
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
        f.write_fmt(format_args!("Undefined type {}", self.type_name))
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
            "Constant '{}' needs to have a valid annotated type",
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImmutableReassign {
    pub variable_name: String,
}

impl Display for ImmutableReassign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Can not reassign immutable variable '{}'",
            self.variable_name
        ))
    }
}

impl Error for ImmutableReassign {}
