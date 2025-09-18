//! # While Loop Type Checking: Boolean-Gated Control Flow
//!
//! While loops in Y provide conditional iteration with strict boolean condition
//! requirements. This design ensures predictable control flow behavior and
//! eliminates common programming errors:
//!
//! - Boolean-only conditions prevent truthiness confusion from other languages
//! - Explicit type checking catches condition type errors at compile time
//! - LLVM can optimize boolean conditions more efficiently than general expressions
//! - Clear semantics enable static analysis and loop optimization opportunities
//!
//! The strict boolean requirement maintains Y's philosophy of explicit behavior
//! over implicit conversions that might hide logical errors.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::WhileLoop,
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for WhileLoop<()> {
    type Typed = WhileLoop<TypeInformation>;

    /// While loop type checking enforces boolean conditions for predictable control flow.
    ///
    /// The strict boolean requirement prevents common errors from truthy value
    /// semantics in other languages. Condition type validation occurs before
    /// body checking to provide clear error messages for type mismatches.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let WhileLoop {
            condition,
            block,
            position,
            ..
        } = self;

        let context = ctx.clone();

        // Step 1: Type check the loop condition expression
        // While loops require a boolean condition to determine when to continue looping
        let condition = condition.check(ctx)?;

        // Step 2: Verify the condition expression has boolean type
        // While loop conditions must evaluate to true/false to control loop execution
        match &*condition.get_info().type_id.borrow() {
            Some(Type::Boolean) => {}
            Some(other) => {
                // Condition has non-boolean type - this is a type error
                return Err(TypeCheckError::TypeMismatch(
                    TypeMismatch {
                        expected: Type::Boolean,
                        actual: other.clone(),
                    },
                    condition.position(),
                ));
            }
            _ => {}
        };

        // Step 3: Type check the loop body block
        // The loop body can contain any statements and expressions
        // Step 4: Return the typed while loop with void type for the statement itself
        // While loops are statements and don't yield values
        Ok(WhileLoop {
            condition,
            block: block.check(ctx)?,
            info: TypeInformation {
                // While loops always have Void type as they are statements
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let WhileLoop {
            condition,
            block,
            position,
            ..
        } = this;

        WhileLoop {
            condition: TypeCheckable::revert(condition),
            block: TypeCheckable::revert(block),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for WhileLoop<TypeInformation> {
    type Validated = WhileLoop<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let WhileLoop {
            condition,
            block,
            info,
            position,
        } = self;

        Ok(WhileLoop {
            condition: condition.validate()?,
            block: block.validate()?,
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
        parser::ast::{Block, Expression, Id, Num, Statement, WhileLoop},
        typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation},
    };

    #[test]
    fn test_empty_while_loop() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Boolean))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;

        let while_l = WhileLoop {
            condition: Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
            block: Block {
                statements: vec![],
                info: (),
                position: Span::default(),
            },
            info: (),
            position: Span::default(),
        };

        let while_l = while_l.check(&mut ctx)?;

        assert_eq!(
            while_l.info.type_id,
            Rc::new(RefCell::new(Some(Type::Void)))
        );

        Ok(())
    }

    #[test]
    fn test_non_empty_while_loop() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Boolean))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;

        let while_l = WhileLoop {
            condition: Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
            block: Block {
                statements: vec![Statement::YieldingExpression(Expression::Num(
                    Num::Integer(42, (), Span::default()),
                ))],
                info: (),
                position: Span::default(),
            },
            info: (),
            position: Span::default(),
        };

        let while_l = while_l.check(&mut ctx)?;

        assert_eq!(
            while_l.info.type_id,
            Rc::new(RefCell::new(Some(Type::Void)))
        );

        Ok(())
    }
}
