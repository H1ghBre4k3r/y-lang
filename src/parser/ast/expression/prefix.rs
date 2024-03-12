use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prefix<T> {
    Negation { expr: Box<Expression<T>> },
    Minus { expr: Box<Expression<T>> },
}

impl<T> Prefix<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Prefix::Negation { expr } => expr.get_info(),
            Prefix::Minus { expr } => expr.get_info(),
        }
    }
}
