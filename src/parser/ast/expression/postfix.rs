use super::{Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Postfix {
    Call {
        expr: Box<Expression>,
        args: Vec<Expression>,
    },
    Index {
        expr: Box<Expression>,
        index: Box<Expression>,
    },
    PropertyAccess {
        expr: Box<Expression>,
        property: Id,
    },
}
