use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryExpression<T> {
    Addition {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
    },
    Substraction {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
    },
    Multiplication {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
    },
    Division {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
    },
    Equal {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
    },
    GreaterThan {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
    },
    LessThen {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
    },
    GreaterOrEqual {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
    },
    LessOrEqual {
        left: Expression<T>,
        right: Expression<T>,
        info: T,
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
}

impl BinaryExpression<()> {
    pub fn converter(&self) -> impl Fn(Expression<()>, Expression<()>) -> BinaryExpression<()> {
        match self {
            Self::Addition {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::Addition {
                left,
                right,
                info: (),
            },
            Self::Substraction {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::Substraction {
                left,
                right,
                info: (),
            },
            Self::Multiplication {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::Multiplication {
                left,
                right,
                info: (),
            },
            Self::Division {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::Division {
                left,
                right,
                info: (),
            },
            Self::Equal {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::Equal {
                left,
                right,
                info: (),
            },
            Self::GreaterThan {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::GreaterThan {
                left,
                right,
                info: (),
            },
            Self::LessThen {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::LessThen {
                left,
                right,
                info: (),
            },
            Self::GreaterOrEqual {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::GreaterOrEqual {
                left,
                right,
                info: (),
            },
            Self::LessOrEqual {
                left: _, right: _, ..
            } => |left, right| BinaryExpression::LessOrEqual {
                left,
                right,
                info: (),
            },
        }
    }

    /// This function balances a binary expresion according the precedence of the operators.
    ///
    /// Attetention: This function assumes the left hand side to be a non-binary expression!
    pub fn balance(&self) -> BinaryExpression<()> {
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
    use crate::parser::ast::{Expression, Num};

    use super::BinaryExpression;

    #[test]
    fn test_simple_balance() {
        let testee = BinaryExpression::Multiplication {
            left: Expression::Num(Num::Integer(42, ())),
            right: Expression::Binary(Box::new(BinaryExpression::Addition {
                left: Expression::Num(Num::Integer(1, ())),
                right: Expression::Num(Num::Integer(2, ())),
                info: (),
            })),
            info: (),
        };

        let expected = BinaryExpression::Addition {
            left: Expression::Binary(Box::new(BinaryExpression::Multiplication {
                left: Expression::Num(Num::Integer(42, ())),
                right: Expression::Num(Num::Integer(1, ())),
                info: (),
            })),
            right: Expression::Num(Num::Integer(2, ())),
            info: (),
        };

        assert_eq!(expected, testee.balance());
    }

    #[test]
    fn test_unneeded_balance() {
        let testee = BinaryExpression::Addition {
            left: Expression::Binary(Box::new(BinaryExpression::Multiplication {
                left: Expression::Num(Num::Integer(42, ())),
                right: Expression::Num(Num::Integer(1, ())),
                info: (),
            })),
            right: Expression::Num(Num::Integer(2, ())),
            info: (),
        };

        assert_eq!(testee, testee.balance());
    }
}
