use crate::lexer::Span;

use super::Expression;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BinaryOperator {
    Add,
    Substract,
    Multiply,
    Divide,
    Equals,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BinaryExpression<T> {
    pub left: Expression<T>,
    pub right: Expression<T>,
    pub operator: BinaryOperator,
    pub info: T,
    pub position: Span,
}

impl<T> BinaryExpression<T>
where
    T: Clone,
{
    pub fn inner(&self) -> (Expression<T>, Expression<T>) {
        let BinaryExpression { left, right, .. } = self;

        (left.clone(), right.clone())
    }

    pub fn get_info(&self) -> T {
        self.info.clone()
    }

    pub fn position(&self) -> Span {
        self.position.clone()
    }
}

impl BinaryExpression<()> {
    pub fn converter(
        &self,
    ) -> impl Fn(Expression<()>, Expression<()>, Span) -> BinaryExpression<()> {
        let operator = self.operator;
        move |left, right, position| BinaryExpression {
            left,
            right,
            operator,
            info: (),
            position,
        }
    }

    /// This function balances a binary expresion according the precedence of the operators.
    ///
    /// Attetention: This function assumes the left hand side to be a non-binary expression!
    pub fn balance(&self) -> BinaryExpression<()> {
        let position = self.position();
        let converter = self.converter();
        let (mut lhs, mut rhs) = self.inner();

        if let Expression::Binary(rhs_binary) = rhs {
            let precedence = rhs_binary.precedence();
            let (inner_lhs, inner_rhs) = rhs_binary.inner();
            let inner_converter = rhs_binary.converter();
            let inner_position = rhs_binary.position();

            if precedence < self.precedence() {
                lhs = Expression::Binary(Box::new(converter(lhs, inner_lhs, position).balance()));
                rhs = inner_rhs;
                return inner_converter(lhs, rhs, inner_position);
            }
        }
        self.clone()
    }

    pub fn precedence(&self) -> usize {
        match self.operator {
            BinaryOperator::Add | BinaryOperator::Substract => 1,
            BinaryOperator::Multiply | BinaryOperator::Divide => 2,
            BinaryOperator::Equals
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterOrEqual
            | BinaryOperator::LessOrEqual => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Span,
        parser::ast::{BinaryOperator, Expression, Num},
    };

    use super::BinaryExpression;

    #[test]
    fn test_simple_balance() {
        let testee = BinaryExpression {
            left: Expression::Num(Num::Integer(42, (), Span::default())),
            right: Expression::Binary(Box::new(BinaryExpression {
                left: Expression::Num(Num::Integer(1, (), Span::default())),
                right: Expression::Num(Num::Integer(2, (), Span::default())),
                operator: BinaryOperator::Add,
                info: (),
                position: Span::default(),
            })),
            operator: BinaryOperator::Multiply,
            info: (),
            position: Span::default(),
        };

        let expected = BinaryExpression {
            left: Expression::Binary(Box::new(BinaryExpression {
                left: Expression::Num(Num::Integer(42, (), Span::default())),
                right: Expression::Num(Num::Integer(1, (), Span::default())),
                operator: BinaryOperator::Multiply,
                info: (),
                position: Span::default(),
            })),
            right: Expression::Num(Num::Integer(2, (), Span::default())),
            operator: BinaryOperator::Add,
            info: (),
            position: Span::default(),
        };

        assert_eq!(expected, testee.balance());
    }

    #[test]
    fn test_unneeded_balance() {
        let testee = BinaryExpression {
            left: Expression::Binary(Box::new(BinaryExpression {
                left: Expression::Num(Num::Integer(42, (), Span::default())),
                right: Expression::Num(Num::Integer(1, (), Span::default())),
                operator: BinaryOperator::Multiply,
                info: (),
                position: Span::default(),
            })),
            right: Expression::Num(Num::Integer(2, (), Span::default())),
            operator: BinaryOperator::Add,
            info: (),
            position: Span::default(),
        };

        assert_eq!(testee, testee.balance());
    }
}
