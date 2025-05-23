use std::{cell::RefCell, rc::Rc};

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

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let WhileLoop {
            condition,
            block,
            position,
            ..
        } = self;

        let context = ctx.clone();

        let condition = condition.check(ctx)?;

        match &*condition.get_info().type_id.borrow() {
            Some(Type::Boolean) => {}
            Some(other) => {
                return Err(TypeCheckError::TypeMismatch(
                    TypeMismatch {
                        expected: Type::Boolean,
                        actual: other.clone(),
                    },
                    condition.position(),
                ))
            }
            _ => {}
        };

        Ok(WhileLoop {
            condition,
            block: block.check(ctx)?,
            info: TypeInformation {
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
