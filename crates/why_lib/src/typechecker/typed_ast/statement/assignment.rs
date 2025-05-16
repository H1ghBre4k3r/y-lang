use crate::{
    parser::ast::{Assignment, Id},
    typechecker::{
        context::Context,
        error::{ImmutableReassign, TypeCheckError, TypeMismatch, UndefinedVariable},
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Assignment<()> {
    type Output = Assignment<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let context = ctx.clone();
        let Assignment {
            lvalue,
            rvalue,
            position,
            ..
        } = self;

        let Id {
            name,
            position: id_position,
            ..
        } = lvalue;

        let Some(variable_type) = ctx.scope.resolve_name(&name) else {
            return Err(TypeCheckError::UndefinedVariable(
                UndefinedVariable {
                    variable_name: name,
                },
                id_position,
            ));
        };

        if let Some(false) = ctx.scope.is_variable_mutable(&name) {
            return Err(TypeCheckError::ImmutableReassign(
                ImmutableReassign {
                    variable_name: name,
                },
                id_position,
            ));
        }

        let mut rvalue = rvalue.check(ctx)?;
        let info = rvalue.get_info();

        let variable_type_id = { variable_type.borrow().clone() };
        let rvalue_type_id = { rvalue.get_info().type_id.borrow().clone() };

        match (variable_type_id, rvalue_type_id) {
            (Some(variable_type_id), Some(rvalue_type_id)) => {
                if variable_type_id != rvalue_type_id {
                    return Err(TypeCheckError::TypeMismatch(
                        TypeMismatch {
                            expected: variable_type_id,
                            actual: rvalue_type_id,
                        },
                        rvalue.position(),
                    ));
                }
            }
            (Some(variable_type_id), None) => {
                rvalue.update_type(variable_type_id.clone())?;

                *info.type_id.borrow_mut() = Some(variable_type_id);
            }
            _ => {}
        }

        if let Err(e) = ctx.scope.add_variable(&name, rvalue.clone(), true) {
            unreachable!("{e}")
        }

        Ok(Assignment {
            lvalue: Id {
                name,
                info: TypeInformation {
                    type_id: info.type_id.clone(),
                    context: context.clone(),
                },
                position: id_position,
            },
            rvalue,
            info: TypeInformation {
                type_id: info.type_id.clone(),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let Assignment {
            lvalue: id,
            rvalue,
            position,
            ..
        } = this;

        Assignment {
            lvalue: TypeCheckable::revert(id),
            rvalue: TypeCheckable::revert(rvalue),
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
        parser::ast::{Assignment, Expression, Id, Num},
        typechecker::{
            context::Context,
            error::{ImmutableReassign, TypeCheckError, TypeMismatch, UndefinedVariable},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_simple_reassign() -> Result<()> {
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
            true,
        )?;

        let ass = Assignment {
            lvalue: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            info: (),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            position: Span::default(),
        };

        ass.check(&mut ctx)?;

        Ok(())
    }

    #[test]
    fn test_assign_type_missmatch() -> Result<()> {
        let mut ctx = Context::default();
        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::FloatingPoint))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            true,
        )?;

        let ass = Assignment {
            lvalue: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            info: (),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            position: Span::default(),
        };

        let result = ass.check(&mut ctx);

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
    fn test_immutable_assign_error() -> Result<()> {
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

        let ass = Assignment {
            lvalue: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            info: (),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            position: Span::default(),
        };

        let result = ass.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::ImmutableReassign(
                ImmutableReassign {
                    variable_name: "foo".into()
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_undefined_reassign_error() -> Result<()> {
        let mut ctx = Context::default();

        let ass = Assignment {
            lvalue: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            info: (),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            position: Span::default(),
        };

        let result = ass.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::UndefinedVariable(
                UndefinedVariable {
                    variable_name: "foo".into()
                },
                Span::default()
            ))
        );

        Ok(())
    }
}
