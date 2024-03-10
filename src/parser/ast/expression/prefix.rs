use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prefix<T> {
    Negation { expr: Box<Expression<T>> },
    Minus { expr: Box<Expression<T>> },
}
