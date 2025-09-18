//! # Binary Expression Type Checking: Operator Overloading Prevention
//!
//! Y deliberately avoids operator overloading to maintain predictable performance
//! characteristics. Binary operations are restricted to primitive types because:
//!
//! - LLVM can generate optimal assembly for primitive operations
//! - No hidden method calls or allocations behind operators
//! - Compile-time operation cost is always known
//! - Prevents accidental expensive operations disguised as simple syntax
//!
//! This design philosophy prioritizes explicitness over convenience, ensuring
//! that complex operations require explicit method calls rather than operators.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::error::UnsupportedBinaryOperation;
use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::{BinaryExpression, BinaryOperator},
    typechecker::{
        context::Context, error::TypeCheckError, types::Type, TypeCheckable, TypeInformation,
        TypeResult,
    },
};

// TODO lome: this should maybe only be possible for integer and floats
impl TypeCheckable for BinaryExpression<()> {
    type Typed = BinaryExpression<TypeInformation>;

    /// Type compatibility is strictly enforced to prevent subtle runtime errors.
    ///
    /// Y requires explicit type conversions rather than implicit coercion because
    /// implicit conversions can hide performance costs and introduce unexpected
    /// precision loss. This design prevents common programming errors while
    /// maintaining zero-cost abstractions.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();
        let BinaryExpression {
            left,
            right,
            operator,
            position,
            ..
        } = self;

        // Type check both operands recursively
        let left = left.check(ctx)?;
        let right = right.check(ctx)?;

        // Extract the concrete types from both operands
        let left_type = { left.get_info().type_id.borrow() }.clone();
        let right_type = { right.get_info().type_id.borrow() }.clone();

        // Verify that both operands have compatible types (if known)
        let compount_type = if let (Some(left_type), Some(right_type)) = (left_type, right_type) {
            if !left_type.does_eq(&right_type) {
                return Err(TypeCheckError::UnsupportedBinaryOperation(
                    UnsupportedBinaryOperation {
                        operands: (left_type, right_type),
                    },
                    position,
                ));
            }
            Some(left_type)
        } else {
            // If either operand has unknown type, defer type checking
            None
        };

        // Ensure binary operations are only performed on supported primitive types
        if let Some(t) = &compount_type {
            match t {
                Type::Integer | Type::FloatingPoint | Type::Boolean => {}
                _ => {
                    return Err(TypeCheckError::UnsupportedBinaryOperation(
                        UnsupportedBinaryOperation {
                            operands: (t.clone(), t.clone()),
                        },
                        position,
                    ));
                }
            }
        }

        // Determine the result type based on the operator
        let type_id = match operator {
            // Arithmetic operations preserve the operand type
            BinaryOperator::Add
            | BinaryOperator::Substract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide => compount_type,
            // Comparison operations always produce boolean results
            BinaryOperator::Equals
            | BinaryOperator::NotEquals
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterOrEqual
            | BinaryOperator::LessOrEqual => Some(Type::Boolean),
        };

        Ok(BinaryExpression {
            left,
            right,
            operator,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(type_id)),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let BinaryExpression {
            left,
            right,
            operator,
            position,
            ..
        } = this;

        BinaryExpression {
            left: TypeCheckable::revert(left),
            right: TypeCheckable::revert(right),
            operator: *operator,
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for BinaryExpression<TypeInformation> {
    type Validated = BinaryExpression<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let BinaryExpression {
            left,
            right,
            operator,
            info,
            position,
        } = self;

        Ok(BinaryExpression {
            left: left.validate()?,
            right: right.validate()?,
            operator,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{BinaryExpression, BinaryOperator, Expression, Num},
        typechecker::{
            context::Context,
            error::{TypeCheckError, UnsupportedBinaryOperation},
            types::Type,
            TypeCheckable,
        },
    };

    #[test]
    fn test_simple_addition() -> Result<()> {
        let mut ctx = Context::default();
        let exp = BinaryExpression {
            left: Expression::Num(Num::Integer(42, (), Span::default())),
            right: Expression::Num(Num::Integer(1337, (), Span::default())),
            operator: BinaryOperator::Add,
            info: (),
            position: Span::default(),
        };

        let exp = exp.check(&mut ctx)?;

        assert_eq!(exp.info.type_id, Rc::new(RefCell::new(Some(Type::Integer))));

        Ok(())
    }

    #[test]
    fn test_simple_equality() -> Result<()> {
        let mut ctx = Context::default();
        let exp = BinaryExpression {
            left: Expression::Num(Num::Integer(42, (), Span::default())),
            right: Expression::Num(Num::Integer(1337, (), Span::default())),
            operator: BinaryOperator::Equals,
            info: (),
            position: Span::default(),
        };

        let exp = exp.check(&mut ctx)?;

        assert_eq!(exp.info.type_id, Rc::new(RefCell::new(Some(Type::Boolean))));

        Ok(())
    }

    #[test]
    fn test_addition_with_incompatible_types() -> Result<()> {
        let mut ctx = Context::default();
        let exp = BinaryExpression {
            left: Expression::Num(Num::Integer(42, (), Span::default())),
            right: Expression::Num(Num::FloatingPoint(1337.0, (), Span::default())),
            operator: BinaryOperator::Add,
            info: (),
            position: Span::default(),
        };

        let res = exp.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::UnsupportedBinaryOperation(
                UnsupportedBinaryOperation {
                    operands: (Type::Integer, Type::FloatingPoint)
                },
                Span::default()
            ))
        );

        Ok(())
    }
}
