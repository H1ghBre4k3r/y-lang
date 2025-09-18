//! # Block Expression Type Checking: Expression-Oriented Design
//!
//! Blocks in Y embody the expression-oriented programming paradigm where every
//! construct can potentially yield a value. This design choice enables several
//! architectural benefits:
//!
//! - Functional programming patterns with immutable data flow
//! - Consistent expression composition without special syntax
//! - LLVM can optimize expression sequences more effectively
//! - Eliminates the statement/expression distinction for cleaner semantics
//!
//! The yielding/non-yielding distinction exists because LLVM requires explicit
//! control over when values are produced versus when side effects occur.

use std::{cell::RefCell, rc::Rc};

use crate::parser::ast::{Expression, Statement};
use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::Block,
    typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for Block<()> {
    type Typed = Block<TypeInformation>;

    /// Blocks require careful type inference because Y's expression-oriented design
    /// means blocks can yield values like any other expression.
    ///
    /// The key design constraint is maintaining LLVM's requirement for explicit
    /// value production - intermediate expressions cannot yield values unless
    /// they're the final statement, which prevents ambiguous code generation.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Block {
            statements,
            position,
            ..
        } = self;
        let context = ctx.clone();

        let mut checked_statements = vec![];
        let len = statements.len();

        // Process each statement in the block sequentially
        // Type check statements in order to maintain scope and context changes
        for (i, stmt) in statements.into_iter().enumerate() {
            // Type check the current statement using the accumulated context
            let stmt = stmt.check(ctx)?;

            // Handle yielding expressions that appear before the last statement
            // In blocks, only the final statement can yield a value - intermediate
            // yielding expressions must be converted to non-yielding expressions
            match stmt {
                // Convert intermediate yielding blocks to non-yielding expressions
                Statement::YieldingExpression(Expression::Block(block)) if i < len - 1 => {
                    checked_statements.push(Statement::Expression(Expression::Block(block)))
                }
                // Convert intermediate yielding if-expressions to non-yielding expressions
                Statement::YieldingExpression(Expression::If(if_expression)) if i < len - 1 => {
                    checked_statements.push(Statement::Expression(Expression::If(if_expression)))
                }
                // Convert intermediate yielding lambdas to non-yielding expressions
                Statement::YieldingExpression(Expression::Lambda(lambda)) if i < len - 1 => {
                    checked_statements.push(Statement::Expression(Expression::Lambda(lambda)))
                }
                // Other intermediate yielding expressions are not allowed
                Statement::YieldingExpression(other) if i < len - 1 => {
                    todo!("yielding expression is only allowed at the end of a function {other:?}");
                }
                // Keep all other statements as-is (including the final yielding expression)
                _ => checked_statements.push(stmt),
            }
        }

        // Determine the type of the entire block based on its last statement
        // Block type inference rules:
        // 1. Empty block: void type (no value produced)
        // 2. Last statement is non-yielding expression: void type (side-effect only)
        // 3. Last statement is yielding expression: inherits that expression's type
        // 4. Last statement is declaration/assignment: void type
        let type_id = checked_statements
            .last()
            .map(|last| match last {
                // Non-yielding expressions produce void (they don't yield values)
                crate::parser::ast::Statement::Expression(_) => {
                    Rc::new(RefCell::new(Some(Type::Void)))
                }
                // Yielding expressions and other statements provide their own type information
                last => last.get_info().type_id,
            })
            // Empty blocks default to void type
            .unwrap_or(Rc::new(RefCell::new(Some(Type::Void))));

        Ok(Block {
            statements: checked_statements,
            info: TypeInformation { type_id, context },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Block {
            statements,
            position,
            ..
        } = this;

        Block {
            statements: statements.iter().map(TypeCheckable::revert).collect(),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Block<TypeInformation> {
    type Validated = Block<ValidatedTypeInformation>;

    fn update_type(
        &mut self,
        type_id: Type,
    ) -> Result<(), crate::typechecker::error::TypeCheckError> {
        // Update type information when the block is used in a context that expects a specific type
        // This propagates type constraints down to the block's yielding expression

        // Check if the block has any statements to update
        if let Some(last_stmt) = self.statements.last_mut() {
            match last_stmt {
                // If the last statement yields a value, propagate the expected type to it
                // This allows the expression to refine its type based on usage context
                crate::parser::ast::Statement::YieldingExpression(expr) => {
                    expr.update_type(type_id.clone())?;
                }
                _ => {
                    // If the last statement doesn't yield a value but we expect a non-void type,
                    // this represents a type mismatch that will be caught during validation
                    // We don't error here to allow the validation phase to handle it properly
                }
            }
        }

        // Update the block's own type information to reflect the expected type
        // This ensures the block's type matches what the surrounding context expects
        *self.info.type_id.borrow_mut() = Some(type_id);
        Ok(())
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Block {
            statements,
            info,
            position,
        } = self;

        let mut validated_statements = vec![];
        for statement in statements {
            validated_statements.push(statement.validate()?);
        }

        Ok(Block {
            statements: validated_statements,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::{Ok, Result};

    use crate::{
        lexer::Span,
        parser::ast::{Block, Expression, Num, Statement},
        typechecker::{context::Context, types::Type, TypeCheckable},
    };

    #[test]
    fn test_empty_block() -> Result<()> {
        let mut ctx = Context::default();

        let block = Block {
            statements: vec![],
            info: (),
            position: Span::default(),
        };

        let block = block.check(&mut ctx)?;

        assert_eq!(block.info.type_id, Rc::new(RefCell::new(Some(Type::Void))));

        Ok(())
    }

    #[test]
    fn test_return_type_of_last_statement() -> Result<()> {
        let mut ctx = Context::default();

        let block = Block {
            statements: vec![Statement::YieldingExpression(Expression::Num(
                Num::Integer(42, (), Span::default()),
            ))],
            info: (),
            position: Span::default(),
        };

        let block = block.check(&mut ctx)?;

        assert_eq!(
            block.info.type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        Ok(())
    }
}
