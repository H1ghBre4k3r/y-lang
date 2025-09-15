use std::{cell::RefCell, rc::Rc};

use crate::parser::ast::{Expression, Statement};
use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::Block,
    typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation, TypeResult},
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

        // TODO: this should be done in the Block
        for (i, stmt) in statements.into_iter().enumerate() {
            let stmt = stmt.check(ctx)?;
            match stmt {
                Statement::YieldingExpression(Expression::Block(block)) if i < len - 1 => {
                    checked_statements.push(Statement::Expression(Expression::Block(block)))
                }
                Statement::YieldingExpression(Expression::If(if_expression)) if i < len - 1 => {
                    checked_statements.push(Statement::Expression(Expression::If(if_expression)))
                }
                Statement::YieldingExpression(Expression::Lambda(lambda)) if i < len - 1 => {
                    checked_statements.push(Statement::Expression(Expression::Lambda(lambda)))
                }
                Statement::YieldingExpression(other) if i < len - 1 => {
                    todo!("yielding expression is only allowed at the end of a function {other:?}");
                }
                _ => checked_statements.push(stmt),
            }
        }

        let type_id = checked_statements
            .last()
            .map(|last| match last {
                crate::parser::ast::Statement::Expression(_) => {
                    Rc::new(RefCell::new(Some(Type::Void)))
                }
                last => last.get_info().type_id,
            })
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
        // Propagate the expected type to the last yielding statement in the block
        if let Some(last_stmt) = self.statements.last_mut() {
            match last_stmt {
                crate::parser::ast::Statement::YieldingExpression(expr) => {
                    expr.update_type(type_id.clone())?;

                    // After updating the expression, get its actual type which might be different
                    // (e.g., lambda converted to closure)
                    let actual_type = expr.get_info().type_id.borrow().clone();
                    if let Some(actual_type) = actual_type {
                        *self.info.type_id.borrow_mut() = Some(actual_type);
                        return Ok(());
                    }
                }
                _ => {
                    // If the last statement is not a yielding expression but we expect a non-void type,
                    // this is a type error - but we'll let it be caught by the validation phase
                }
            }
        }

        // Update the block's own type (fallback if no yielding expression or it has no type)
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
