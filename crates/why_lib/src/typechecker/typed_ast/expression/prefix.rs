use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::Prefix,
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for Prefix<()> {
    type Typed = Prefix<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // Prefix operators modify their operand expressions in specific ways
        // Both negation (!) and minus (-) have strict type requirements for their operands
        match self {
            // Logical negation operator (!expr) - requires boolean operand
            Prefix::Negation { expr, position } => {
                // First type check the operand expression
                let expr = expr.check(ctx)?;

                // Extract the operand's type to verify it's boolean
                let info = expr.get_info();
                let type_id_ref = info.type_id;
                let type_id = type_id_ref.borrow().clone();

                // Verify the operand has boolean type - negation only works on booleans
                if let Some(type_id) = type_id {
                    if type_id != Type::Boolean {
                        // Operand is not boolean - this is a type error
                        return Err(TypeCheckError::TypeMismatch(
                            TypeMismatch {
                                expected: Type::Boolean,
                                actual: type_id,
                            },
                            expr.position(),
                        ));
                    }
                }

                // Negation result inherits the boolean type from its operand
                Ok(Prefix::Negation {
                    expr: Box::new(expr),
                    position,
                })
            }
            // Arithmetic negation operator (-expr) - requires numeric operand
            Prefix::Minus { expr, position } => {
                // First type check the operand expression
                let expr = expr.check(ctx)?;

                // Extract the operand's type to verify it's numeric
                let info = expr.get_info();
                let type_id_ref = info.type_id;
                let type_id = type_id_ref.borrow().clone();

                // Verify the operand has numeric type - minus only works on numbers
                if let Some(type_id) = type_id {
                    if type_id != Type::Integer && type_id != Type::FloatingPoint {
                        // Operand is not numeric - this is a type error
                        return Err(TypeCheckError::TypeMismatch(
                            TypeMismatch {
                                expected: Type::Integer,
                                actual: type_id,
                            },
                            expr.position(),
                        ));
                    }
                }

                // Minus result inherits the numeric type from its operand
                Ok(Prefix::Minus {
                    expr: Box::new(expr),
                    position,
                })
            }
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            Prefix::Negation { expr, position } => Prefix::Negation {
                expr: Box::new(TypeCheckable::revert(expr.as_ref())),
                position: position.clone(),
            },
            Prefix::Minus { expr, position } => Prefix::Minus {
                expr: Box::new(TypeCheckable::revert(expr.as_ref())),
                position: position.clone(),
            },
        }
    }
}

impl TypedConstruct for Prefix<TypeInformation> {
    type Validated = Prefix<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            Prefix::Negation { expr, position } => Ok(Prefix::Negation {
                expr: Box::new(expr.validate()?),
                position,
            }),
            Prefix::Minus { expr, position } => Ok(Prefix::Minus {
                expr: Box::new(expr.validate()?),
                position,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{BinaryExpression, BinaryOperator, Expression, Num, Prefix},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            TypeCheckable,
        },
    };

    #[test]
    fn test_simple_prefix_minus() -> Result<()> {
        let mut ctx = Context::default();

        let pref = Prefix::Minus {
            expr: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
            position: Span::default(),
        };

        let pref = pref.check(&mut ctx)?;

        assert_eq!(
            pref.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );
        Ok(())
    }

    #[test]
    fn test_simple_prefix_negation() -> Result<()> {
        let mut ctx = Context::default();

        let pref = Prefix::Negation {
            expr: Box::new(Expression::Binary(Box::new(BinaryExpression {
                left: Expression::Num(Num::Integer(42, (), Span::default())),
                right: Expression::Num(Num::Integer(1337, (), Span::default())),
                operator: BinaryOperator::LessThan,
                info: (),
                position: Span::default(),
            }))),
            position: Span::default(),
        };

        let pref = pref.check(&mut ctx)?;

        assert_eq!(
            pref.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Boolean)))
        );
        Ok(())
    }

    #[test]
    fn test_error_in_non_numeric_minus() -> Result<()> {
        let mut ctx = Context::default();

        let pref = Prefix::Minus {
            expr: Box::new(Expression::Binary(Box::new(BinaryExpression {
                left: Expression::Num(Num::Integer(42, (), Span::default())),
                right: Expression::Num(Num::Integer(1337, (), Span::default())),
                operator: BinaryOperator::LessThan,
                info: (),
                position: Span::default(),
            }))),
            position: Span::default(),
        };

        let res = pref.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::Integer,
                    actual: Type::Boolean
                },
                Span::default()
            ))
        );
        Ok(())
    }

    #[test]
    fn test_error_in_non_boolean_negation() -> Result<()> {
        let mut ctx = Context::default();

        let pref = Prefix::Negation {
            expr: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
            position: Span::default(),
        };

        let res = pref.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::Boolean,
                    actual: Type::Integer
                },
                Span::default()
            ))
        );
        Ok(())
    }
}
