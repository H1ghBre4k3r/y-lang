use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prefix {
    Negation { expr: Box<Expression> },
    Minus { expr: Box<Expression> },
}
