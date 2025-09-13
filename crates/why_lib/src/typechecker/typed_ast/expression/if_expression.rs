use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::If,
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for If<()> {
    type Typed = If<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let If {
            condition,
            then_block,
            else_block,
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

        let checked_then_block = then_block.check(ctx)?;
        let checked_else_block = else_block.check(ctx)?;

        let type_id = match (
            checked_then_block.info.type_id.borrow().clone(),
            checked_else_block.info.type_id.borrow().clone(),
        ) {
            (first_type, last_type) => {
                // check, if types of if and else match
                match (first_type, last_type) {
                    (Some(first_type), Some(last_type)) => {
                        // if they do not match, we have a fucky wucky
                        if first_type != last_type {
                            return Err(TypeCheckError::TypeMismatch(
                                TypeMismatch {
                                    expected: first_type,
                                    actual: last_type,
                                },
                                checked_else_block.position,
                            ));
                        }
                        // otherwise (e.g., in case of both being None), we simply return the type
                        // of the if branch
                        Rc::new(RefCell::new(Some(first_type)))
                    }
                    _ => Rc::new(RefCell::new(None)),
                }
            }
            // if we do not have if & else, we simply return void as a type
            _ => Rc::new(RefCell::new(Some(Type::Void))),
        };

        Ok(If {
            condition: Box::new(condition),
            then_block: checked_then_block,
            else_block: checked_else_block,
            info: TypeInformation { type_id, context },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let If {
            condition,
            then_block,
            else_block,
            position,
            ..
        } = this;

        If {
            condition: Box::new(TypeCheckable::revert(condition.as_ref())),
            then_block: TypeCheckable::revert(then_block),
            else_block: TypeCheckable::revert(else_block),
            position: position.clone(),
            info: (),
        }
    }
}

impl TypedConstruct for If<TypeInformation> {
    type Validated = If<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let If {
            condition,
            then_block,
            else_block,
            info,
            position,
        } = self;

        Ok(If {
            condition: Box::new(condition.validate()?),
            then_block: then_block.validate()?,
            else_block: else_block.validate()?,
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
        parser::ast::{Block, Expression, Id, If, Statement},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_empty_if() -> Result<()> {
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

        let if_exp = If {
            condition: Box::new(Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            })),
            then_block: Block {
                statements: vec![],
                position: Span::default(),
                info: (),
            },
            else_block: Block {
                statements: vec![],
                position: Span::default(),
                info: (),
            },
            info: (),
            position: Span::default(),
        };

        let if_exp = if_exp.check(&mut ctx)?;

        assert_eq!(if_exp.info.type_id, Rc::new(RefCell::new(Some(Type::Void))));

        Ok(())
    }

    #[test]
    fn test_error_on_non_boolean_condition() -> Result<()> {
        let mut ctx = Context::default();
        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;

        let if_exp = If {
            condition: Box::new(Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            })),
            then_block: Block {
                statements: vec![],
                position: Span::default(),
                info: (),
            },
            else_block: Block {
                statements: vec![],
                position: Span::default(),
                info: (),
            },
            info: (),
            position: Span::default(),
        };

        let result = if_exp.check(&mut ctx);

        assert_eq!(
            result,
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

    #[test]
    fn test_error_on_if_else_missmatch() -> Result<()> {
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
        ctx.scope.add_variable(
            "bar",
            Expression::Id(Id {
                name: "bar".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::FloatingPoint))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;
        ctx.scope.add_variable(
            "baz",
            Expression::Id(Id {
                name: "baz".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;

        let if_exp = If {
            condition: Box::new(Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            })),
            then_block: Block {
                statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                }))],
                info: (),
                position: Span::default(),
            },
            else_block: Block {
                statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                    name: "baz".into(),
                    info: (),
                    position: Span::default(),
                }))],
                info: (),
                position: Span::default(),
            },
            info: (),
            position: Span::default(),
        };

        let result = if_exp.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::FloatingPoint,
                    actual: Type::Integer
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_void_on_empty_else() -> Result<()> {
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
        ctx.scope.add_variable(
            "bar",
            Expression::Id(Id {
                name: "bar".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::FloatingPoint))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;

        let if_exp = If {
            condition: Box::new(Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            })),
            then_block: Block {
                statements: vec![Statement::Expression(Expression::Id(Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                }))],
                info: (),
                position: Span::default(),
            },
            else_block: Block {
                statements: vec![],
                info: (),
                position: Span::default(),
            },
            info: (),
            position: Span::default(),
        };

        let result = if_exp.check(&mut ctx)?;

        assert_eq!(result.info.type_id, Rc::new(RefCell::new(Some(Type::Void))));

        Ok(())
    }
}
