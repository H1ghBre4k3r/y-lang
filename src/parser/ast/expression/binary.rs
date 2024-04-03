use crate::lexer::Span;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryExpression<T> {
    Addition {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
    Substraction {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
    Multiplication {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
    Division {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
    Equal {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
    GreaterThan {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
    LessThen {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
    GreaterOrEqual {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
    LessOrEqual {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
        position: Span,
    },
}

impl<T> BinaryExpression<T>
where
    T: Clone,
{
    pub fn inner(&self) -> (Expression<T>, Expression<T>) {
        match self {
            Self::Addition {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
            Self::Substraction {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
            Self::Multiplication {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
            Self::Division {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
            Self::Equal {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
            Self::GreaterThan {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
            Self::LessThen {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
            Self::GreaterOrEqual {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
            Self::LessOrEqual {
                left: lhs,
                right: rhs,
                ..
            } => (lhs.clone(), rhs.clone()),
        }
    }

    pub fn get_info(&self) -> T {
        match self {
            BinaryExpression::Addition { info, .. } => info.clone(),
            BinaryExpression::Substraction { info, .. } => info.clone(),
            BinaryExpression::Multiplication { info, .. } => info.clone(),
            BinaryExpression::Division { info, .. } => info.clone(),
            BinaryExpression::Equal { info, .. } => info.clone(),
            BinaryExpression::GreaterThan { info, .. } => info.clone(),
            BinaryExpression::LessThen { info, .. } => info.clone(),
            BinaryExpression::GreaterOrEqual { info, .. } => info.clone(),
            BinaryExpression::LessOrEqual { info, .. } => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            BinaryExpression::Addition { position, .. } => position.clone(),
            BinaryExpression::Substraction { position, .. } => position.clone(),
            BinaryExpression::Multiplication { position, .. } => position.clone(),
            BinaryExpression::Division { position, .. } => position.clone(),
            BinaryExpression::Equal { position, .. } => position.clone(),
            BinaryExpression::GreaterThan { position, .. } => position.clone(),
            BinaryExpression::LessThen { position, .. } => position.clone(),
            BinaryExpression::GreaterOrEqual { position, .. } => position.clone(),
            BinaryExpression::LessOrEqual { position, .. } => position.clone(),
        }
    }
}

impl BinaryExpression<()> {
    pub fn converter(
        &self,
    ) -> impl Fn(Expression<()>, Expression<()>, Span) -> BinaryExpression<()> {
        match self {
            Self::Addition {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::Addition {
                left,
                right,
                info: (),
                position,
            },
            Self::Substraction {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::Substraction {
                left,
                right,
                info: (),
                position,
            },
            Self::Multiplication {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::Multiplication {
                left,
                right,
                info: (),
                position,
            },
            Self::Division {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::Division {
                left,
                right,
                info: (),
                position,
            },
            Self::Equal {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::Equal {
                left,
                right,
                info: (),
                position,
            },
            Self::GreaterThan {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::GreaterThan {
                left,
                right,
                info: (),
                position,
            },
            Self::LessThen {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::LessThen {
                left,
                right,
                info: (),
                position,
            },
            Self::GreaterOrEqual {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::GreaterOrEqual {
                left,
                right,
                info: (),
                position,
            },
            Self::LessOrEqual {
                left: _, right: _, ..
            } => |left, right, position| BinaryExpression::LessOrEqual {
                left,
                right,
                info: (),
                position,
            },
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
        use BinaryExpression::*;

        match self {
            Addition { .. } | Substraction { .. } => 1,
            Multiplication { .. } | Division { .. } => 2,
            Equal { .. }
            | GreaterThan { .. }
            | LessThen { .. }
            | GreaterOrEqual { .. }
            | LessOrEqual { .. } => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Span,
        parser::ast::{Expression, Num},
    };

    use super::BinaryExpression;

    #[test]
    fn test_simple_balance() {
        let testee = BinaryExpression::Multiplication {
            left: Expression::Num(Num::Integer(42, (), Span::default())),
            right: Expression::Binary(Box::new(BinaryExpression::Addition {
                left: Expression::Num(Num::Integer(1, (), Span::default())),
                right: Expression::Num(Num::Integer(2, (), Span::default())),
                info: (),
                position: Span::default(),
            })),
            info: (),
            position: Span::default(),
        };

        let expected = BinaryExpression::Addition {
            left: Expression::Binary(Box::new(BinaryExpression::Multiplication {
                left: Expression::Num(Num::Integer(42, (), Span::default())),
                right: Expression::Num(Num::Integer(1, (), Span::default())),
                info: (),
                position: Span::default(),
            })),
            right: Expression::Num(Num::Integer(2, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        assert_eq!(expected, testee.balance());
    }

    #[test]
    fn test_unneeded_balance() {
        let testee = BinaryExpression::Addition {
            left: Expression::Binary(Box::new(BinaryExpression::Multiplication {
                left: Expression::Num(Num::Integer(42, (), Span::default())),
                right: Expression::Num(Num::Integer(1, (), Span::default())),
                info: (),
                position: Span::default(),
            })),
            right: Expression::Num(Num::Integer(2, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        assert_eq!(testee, testee.balance());
    }
}
