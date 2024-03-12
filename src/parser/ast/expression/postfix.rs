use super::{Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Postfix<T> {
    Call {
        expr: Box<Expression<T>>,
        args: Vec<Expression<T>>,
        info: T,
    },
    Index {
        expr: Box<Expression<T>>,
        index: Box<Expression<T>>,
        info: T,
    },
    PropertyAccess {
        expr: Box<Expression<T>>,
        property: Id<T>,
        info: T,
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
}
