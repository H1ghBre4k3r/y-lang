use std::{cell::RefCell, rc::Rc};

use crate::parser::ast::{Expression, Statement};
use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::Block,
    typechecker::{TypeCheckable, TypeInformation, TypeResult, context::Context, types::Type},
};

impl TypeCheckable for Block<()> {
    type Typed = Block<TypeInformation>;

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
        typechecker::{TypeCheckable, context::Context, types::Type},
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
