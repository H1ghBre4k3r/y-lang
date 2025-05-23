use std::{cell::RefCell, rc::Rc};

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

        for stmt in statements.into_iter() {
            checked_statements.push(stmt.check(ctx)?);
        }

        let type_id = checked_statements
            .last()
            .map(|last| last.get_info().type_id)
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
