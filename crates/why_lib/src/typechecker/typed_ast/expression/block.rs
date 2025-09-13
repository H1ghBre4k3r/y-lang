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

/*
* use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::Block,
    typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for Block<()> {
    type Typed = Block<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // Default to non-yielding context for backward compatibility
        self.check_with_yielding_context(ctx, false)
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

impl Block<()> {
    /// Check a block with explicit yielding context
    ///
    /// # Arguments
    /// * `ctx` - The type checking context
    /// * `is_yielding_context` - Whether this block is in a position where it should yield a value
    ///   - `true` for function bodies and other expression contexts
    ///   - `false` for standalone statement blocks
    pub fn check_with_yielding_context(
        self,
        ctx: &mut Context,
        is_yielding_context: bool
    ) -> TypeResult<Block<TypeInformation>> {
        let Block {
            statements,
            position,
            ..
        } = self;
        let context = ctx.clone();

        let mut checked_statements = vec![];

        for stmt in statements.into_iter() {
            checked_statements.push(stmt.check(ctx)?);
        }

        let type_id = if is_yielding_context {
            // In yielding context, return the type of the last statement if it's yielding
            checked_statements
                .last()
                .map(|last| match last {
                    crate::parser::ast::Statement::YieldingExpression(_) => {
                        // Only yield if it's explicitly a yielding expression
                        last.get_info().type_id
                    }
                    crate::parser::ast::Statement::Return(_) => {
                        // Return statements also yield their value
                        last.get_info().type_id
                    }
                    _ => {
                        // Other statements (including Expression) don't yield
                        Rc::new(RefCell::new(Some(Type::Void)))
                    }
                })
                .unwrap_or(Rc::new(RefCell::new(Some(Type::Void))))
        } else {
            // In non-yielding context, blocks always have void type
            Rc::new(RefCell::new(Some(Type::Void)))
        };

        Ok(Block {
            statements: checked_statements,
            info: TypeInformation { type_id, context },
            position,
        })
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
*/
