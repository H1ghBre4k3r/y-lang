use super::{ComparisonOperation, Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryExpression {
    Addition(Expression, Expression),
    Substraction(Expression, Expression),
    Multiplication(Expression, Expression),
    Division(Expression, Expression),
    Comparison {
        lhs: Expression,
        rhs: Expression,
        operation: ComparisonOperation,
    },
}

impl BinaryExpression {
    pub fn precedence(&self) -> usize {
        match self {
            BinaryExpression::Addition(_, _) | BinaryExpression::Substraction(_, _) => 1,
            BinaryExpression::Multiplication(_, _) | BinaryExpression::Division(_, _) => 2,
            BinaryExpression::Comparison { .. } => 0,
        }
    }
}
