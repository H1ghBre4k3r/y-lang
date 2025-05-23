use crate::lexer::Span;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Prefix<T> {
    Negation {
        expr: Box<Expression<T>>,
        position: Span,
    },
    Minus {
        expr: Box<Expression<T>>,
        position: Span,
    },
}

impl<T> Prefix<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Prefix::Negation { expr, .. } => expr.get_info(),
            Prefix::Minus { expr, .. } => expr.get_info(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Prefix::Negation { position, .. } => position.clone(),
            Prefix::Minus { position, .. } => position.clone(),
        }
    }
}
