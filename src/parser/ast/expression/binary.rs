use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryExpression {
    Addition(Expression, Expression),
    Substraction(Expression, Expression),
    Multiplication(Expression, Expression),
    Division(Expression, Expression),
    Equal(Expression, Expression),
    GreaterThan(Expression, Expression),
    LessThen(Expression, Expression),
    GreaterOrEqual(Expression, Expression),
    LessOrEqual(Expression, Expression),
}

impl BinaryExpression {
    pub fn inner(&self) -> (Expression, Expression) {
        match self {
            Self::Addition(lhs, rhs) => (lhs.clone(), rhs.clone()),
            Self::Substraction(lhs, rhs) => (lhs.clone(), rhs.clone()),
            Self::Multiplication(lhs, rhs) => (lhs.clone(), rhs.clone()),
            Self::Division(lhs, rhs) => (lhs.clone(), rhs.clone()),
            Self::Equal(lhs, rhs) => (lhs.clone(), rhs.clone()),
            Self::GreaterThan(lhs, rhs) => (lhs.clone(), rhs.clone()),
            Self::LessThen(lhs, rhs) => (lhs.clone(), rhs.clone()),
            Self::GreaterOrEqual(lhs, rhs) => (lhs.clone(), rhs.clone()),
            Self::LessOrEqual(lhs, rhs) => (lhs.clone(), rhs.clone()),
        }
    }

    pub fn converter(&self) -> impl Fn(Expression, Expression) -> BinaryExpression {
        match self {
            Self::Addition(_, _) => BinaryExpression::Addition,
            Self::Substraction(_, _) => BinaryExpression::Substraction,
            Self::Multiplication(_, _) => BinaryExpression::Multiplication,
            Self::Division(_, _) => BinaryExpression::Division,
            Self::Equal(_, _) => BinaryExpression::Equal,
            Self::GreaterThan(_, _) => BinaryExpression::GreaterThan,
            Self::LessThen(_, _) => BinaryExpression::LessThen,
            Self::GreaterOrEqual(_, _) => BinaryExpression::GreaterOrEqual,
            Self::LessOrEqual(_, _) => BinaryExpression::LessOrEqual,
        }
    }

    /// This function balances a binary expresion according the precedence of the operators.
    ///
    /// Attetention: This function assumes the left hand side to be a non-binary expression!
    pub fn balance(&self) -> BinaryExpression {
        let converter = self.converter();
        let (mut lhs, mut rhs) = self.inner();

        if let Expression::Binary(rhs_binary) = rhs {
            let precedence = rhs_binary.precedence();
            let (inner_lhs, inner_rhs) = rhs_binary.inner();
            let inner_converter = rhs_binary.converter();

            if precedence < self.precedence() {
                lhs = Expression::Binary(Box::new(converter(lhs, inner_lhs).balance()));
                rhs = inner_rhs;
                return inner_converter(lhs, rhs);
            }
        }
        self.clone()
    }

    pub fn precedence(&self) -> usize {
        use BinaryExpression::*;

        match self {
            Addition(_, _) | Substraction(_, _) => 1,
            Multiplication(_, _) | Division(_, _) => 2,
            Equal(_, _)
            | GreaterThan(_, _)
            | LessThen(_, _)
            | GreaterOrEqual(_, _)
            | LessOrEqual(_, _) => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::{Expression, Num};

    use super::BinaryExpression;

    #[test]
    fn test_simple_balance() {
        let testee = BinaryExpression::Multiplication(
            Expression::Num(Num::Integer(42)),
            Expression::Binary(Box::new(BinaryExpression::Addition(
                Expression::Num(Num::Integer(1)),
                Expression::Num(Num::Integer(2)),
            ))),
        );

        let expected = BinaryExpression::Addition(
            Expression::Binary(Box::new(BinaryExpression::Multiplication(
                Expression::Num(Num::Integer(42)),
                Expression::Num(Num::Integer(1)),
            ))),
            Expression::Num(Num::Integer(2)),
        );

        assert_eq!(expected, testee.balance());
    }

    #[test]
    fn test_unneeded_balance() {
        let testee = BinaryExpression::Addition(
            Expression::Binary(Box::new(BinaryExpression::Multiplication(
                Expression::Num(Num::Integer(42)),
                Expression::Num(Num::Integer(1)),
            ))),
            Expression::Num(Num::Integer(2)),
        );

        assert_eq!(testee, testee.balance());
    }
}
