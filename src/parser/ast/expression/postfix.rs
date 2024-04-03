use crate::lexer::Span;

use super::{Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Postfix<T> {
    Call {
        expr: Box<Expression<T>>,
        args: Vec<Expression<T>>,
        info: T,
        position: Span,
    },
    Index {
        expr: Box<Expression<T>>,
        index: Box<Expression<T>>,
        info: T,
        position: Span,
    },
    PropertyAccess {
        expr: Box<Expression<T>>,
        property: Id<T>,
        info: T,
        position: Span,
    },
}

impl<T> Postfix<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Postfix::Call { info, .. } => info.clone(),
            Postfix::Index { info, .. } => info.clone(),
            Postfix::PropertyAccess { info, .. } => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Postfix::Call { position, .. } => position.clone(),
            Postfix::Index { position, .. } => position.clone(),
            Postfix::PropertyAccess { position, .. } => position.clone(),
        }
    }
}
