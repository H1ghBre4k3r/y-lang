use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Assignment, LValue},
    typechecker::{
        context::Context,
        error::{ImmutableReassign, TypeCheckError, TypeMismatch},
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Assignment<()> {
    type Typed = Assignment<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();
        let Assignment {
            lvalue,
            rvalue,
            position,
            ..
        } = self;

        let name = lvalue.get_original_variable_name().name;
        if let Some(false) = ctx.scope.is_variable_mutable(&name) {
            return Err(TypeCheckError::ImmutableReassign(
                ImmutableReassign {
                    variable_name: name,
                },
                position,
            ));
        }

        let lvalue = lvalue.check(ctx)?;

        let mut rvalue = rvalue.check(ctx)?;
        let info = rvalue.get_info();

        let variable_type_id = { lvalue.get_info().type_id.borrow().clone() };
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

        Ok(Assignment {
            lvalue,
            rvalue,
            info: TypeInformation {
                type_id: info.type_id.clone(),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
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

impl TypedConstruct for Assignment<TypeInformation> {
    type Validated = Assignment<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Assignment {
            lvalue,
            rvalue,
            info,
            position,
        } = self;

        Ok(Assignment {
            lvalue: lvalue.validate()?,
            rvalue: rvalue.validate()?,
            info: info.validate(&position)?,
            position,
        })
    }
}

impl TypeCheckable for LValue<()> {
    type Typed = LValue<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        match self {
            LValue::Id(id) => Ok(LValue::Id(id.check(ctx)?)),
            LValue::Postfix(postfix) => Ok(LValue::Postfix(postfix.check(ctx)?)),
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            LValue::Id(id) => LValue::Id(TypeCheckable::revert(id)),
            LValue::Postfix(postfix) => LValue::Postfix(TypeCheckable::revert(postfix)),
        }
    }
}

impl TypedConstruct for  LValue<TypeInformation> {
    type Validated = LValue<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            LValue::Id(id) => Ok(LValue::Id(id.validate()?)),
            LValue::Postfix(postfix) => Ok(LValue::Postfix(postfix.validate()?))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{
            Assignment, Expression, Id, LValue, Num, Postfix, StructFieldInitialisation,
            StructInitialisation,
        },
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
            lvalue: LValue::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
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
            lvalue: LValue::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
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
            lvalue: LValue::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
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
            lvalue: LValue::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
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

    #[test]
    fn test_struct_property_assign() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_type(
            "Foo",
            Type::Struct("Foo".to_string(), vec![("bar".to_string(), Type::Integer)]),
        )?;

        ctx.scope.add_variable(
            "foo",
            Expression::StructInitialisation(StructInitialisation {
                id: Id {
                    name: "foo".to_string(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Struct(
                            "Foo".to_string(),
                            vec![("bar".to_string(), Type::Integer)],
                        )))),
                        context: ctx.clone(),
                    },
                    position: Span::default(),
                },
                fields: vec![StructFieldInitialisation {
                    name: Id {
                        name: "bar".into(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: ctx.clone(),
                        },
                        position: Span::default(),
                    },
                    value: Expression::Num(Num::Integer(
                        1337,
                        TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: ctx.clone(),
                        },
                        Span::default(),
                    )),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: ctx.clone(),
                    },
                    position: Span::default(),
                }],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Struct(
                        "Foo".to_string(),
                        vec![("bar".to_string(), Type::Integer)],
                    )))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            true,
        )?;

        let assignment = Assignment {
            lvalue: LValue::Postfix(Postfix::PropertyAccess {
                expr: Box::new(Expression::Id(Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default(),
                })),
                property: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                },
                info: (),
                position: Span::default(),
            }),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        assignment.check(&mut ctx)?;

        Ok(())
    }
    #[test]
    fn test_immutable_struct_property_assign_error() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_type(
            "Foo",
            Type::Struct("Foo".to_string(), vec![("bar".to_string(), Type::Integer)]),
        )?;

        ctx.scope.add_variable(
            "foo",
            Expression::StructInitialisation(StructInitialisation {
                id: Id {
                    name: "foo".to_string(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Struct(
                            "Foo".to_string(),
                            vec![("bar".to_string(), Type::Integer)],
                        )))),
                        context: ctx.clone(),
                    },
                    position: Span::default(),
                },
                fields: vec![StructFieldInitialisation {
                    name: Id {
                        name: "bar".into(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: ctx.clone(),
                        },
                        position: Span::default(),
                    },
                    value: Expression::Num(Num::Integer(
                        1337,
                        TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: ctx.clone(),
                        },
                        Span::default(),
                    )),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: ctx.clone(),
                    },
                    position: Span::default(),
                }],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Struct(
                        "Foo".to_string(),
                        vec![("bar".to_string(), Type::Integer)],
                    )))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;

        let assignment = Assignment {
            lvalue: LValue::Postfix(Postfix::PropertyAccess {
                expr: Box::new(Expression::Id(Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default(),
                })),
                property: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                },
                info: (),
                position: Span::default(),
            }),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let res = assignment.check(&mut ctx);

        assert!(res.is_err());

        Ok(())
    }
}
