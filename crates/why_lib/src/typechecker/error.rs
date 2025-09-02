use std::{error::Error, fmt::Display};

use crate::{lexer::Span, parser::ast::TypeName};

use super::types::Type;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TypeCheckError {
    TypeMismatch(TypeMismatch, Span),
    UndefinedVariable(UndefinedVariable, Span),
    UndefinedType(UndefinedType, Span),
    MissingInitialisationType(MissingInitialisationType, Span),
    InvalidConstantType(InvalidConstantType, Span),
    RedefinedConstant(RedefinedConstant, Span),
    RedefinedFunction(RedefinedFunction, Span),
    RedefinedMethod(RedefinedMethod, Span),
    ImmutableReassign(ImmutableReassign, Span),
    MissingMainFunction(MissingMainFunction),
    InvalidMainSignature(InvalidMainSignature, Span),
    UnsupportedBinaryOperation(UnsupportedBinaryOperation, Span),
}

impl Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.span().to_string(self.err()).as_str())
    }
}

impl TypeCheckError {
    pub fn span(&self) -> Span {
        match self {
            TypeCheckError::TypeMismatch(_, span) => span.clone(),
            TypeCheckError::UndefinedVariable(_, span) => span.clone(),
            TypeCheckError::UndefinedType(_, span) => span.clone(),
            TypeCheckError::MissingInitialisationType(_, span) => span.clone(),
            TypeCheckError::InvalidConstantType(_, span) => span.clone(),
            TypeCheckError::RedefinedConstant(_, span) => span.clone(),
            TypeCheckError::RedefinedFunction(_, span) => span.clone(),
            TypeCheckError::RedefinedMethod(_, span) => span.clone(),
            TypeCheckError::ImmutableReassign(_, span) => span.clone(),
            TypeCheckError::MissingMainFunction(_) => Span::default(),
            TypeCheckError::InvalidMainSignature(_, span) => span.clone(),
            TypeCheckError::UnsupportedBinaryOperation(_, span) => span.clone(),
        }
    }

    pub fn err(&self) -> Box<dyn Error + Send> {
        match self {
            TypeCheckError::TypeMismatch(e, _) => Box::new(e.clone()),
            TypeCheckError::UndefinedVariable(e, _) => Box::new(e.clone()),
            TypeCheckError::UndefinedType(e, _) => Box::new(e.clone()),
            TypeCheckError::MissingInitialisationType(e, _) => Box::new(e.clone()),
            TypeCheckError::InvalidConstantType(e, _) => Box::new(e.clone()),
            TypeCheckError::RedefinedConstant(e, _) => Box::new(e.clone()),
            TypeCheckError::RedefinedFunction(e, _) => Box::new(e.clone()),
            TypeCheckError::RedefinedMethod(e, _) => Box::new(e.clone()),
            TypeCheckError::ImmutableReassign(e, _) => Box::new(e.clone()),
            TypeCheckError::MissingMainFunction(e) => Box::new(e.clone()),
            TypeCheckError::InvalidMainSignature(e, _) => Box::new(e.clone()),
            TypeCheckError::UnsupportedBinaryOperation(e, _) => Box::new(e.clone()),
        }
    }
}

impl Error for TypeCheckError {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UndefinedType {
    pub type_name: TypeName,
}

impl Display for UndefinedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Undefined type {}", self.type_name))
    }
}

impl Error for UndefinedType {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MissingInitialisationType;

impl Display for MissingInitialisationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("There must be a type known at compile time!")
    }
}

impl Error for MissingInitialisationType {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RedefinedFunction {
    pub function_name: String,
}

impl Display for RedefinedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Function '{}' is already defined",
            self.function_name
        ))
    }
}

impl Error for RedefinedFunction {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RedefinedMethod {
    pub type_id: Type,
    pub function_name: String,
}

impl Display for RedefinedMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Method '{}' is already defined for type '{:?}'",
            self.function_name, self.type_id
        ))
    }
}

impl Error for RedefinedMethod {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MissingMainFunction;

impl Display for MissingMainFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Missing main function!")
    }
}

impl Error for MissingMainFunction {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InvalidMainSignature;

impl Display for InvalidMainSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("The main function does not have a valid signature. It must not accept any arguments and must return either void or an integer!")
    }
}

impl Error for InvalidMainSignature {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnsupportedBinaryOperation {
    pub operands: (Type, Type),
}

impl Display for UnsupportedBinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UnsupportedBinaryOperation {
            operands: (left, right),
        } = self;

        f.write_fmt(format_args!(
            "This binary operation is not supported for types '{left:?}' and '{right:?}'",
        ))
    }
}

impl Error for UnsupportedBinaryOperation {}
