use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::Instance,
    typechecker::{
        context::Context,
        error::{RedefinedConstant, RedefinedMethod},
        types::Type,
        ShallowCheck, TypeCheckError, TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for Instance<()> {
    type Output = Instance<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let context = ctx.clone();

        let Instance {
            name,
            functions,
            position,
            ..
        } = self;

        let type_id = match Type::try_from((&name, &*ctx)) {
            Ok(type_id) => type_id,
            Err(e) => return Err(e),
        };

        ctx.scope.enter_scope();
        if ctx.scope.add_constant("this", type_id).is_err() {
            // TODO: use different error
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: "this".into(),
                },
                position,
            ));
        };

        let mut checked_functions = vec![];

        for function in functions.into_iter() {
            checked_functions.push(function.check(ctx)?);
        }

        ctx.scope.exit_scope();

        Ok(Instance {
            name,
            functions: checked_functions,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let Instance {
            name,
            functions,
            position,
            ..
        } = this;

        Instance {
            name: name.clone(),
            functions: functions.iter().map(TypeCheckable::revert).collect(),
            info: (),
            position: position.clone(),
        }
    }
}

impl ShallowCheck for Instance<()> {
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let Instance {
            name, functions, ..
        } = self;

        let type_id = match Type::try_from((name, &*ctx)) {
            Ok(type_id) => type_id,
            Err(e) => return Err(e),
        };

        for function in functions.iter() {
            let function_type = function.simple_shallow_check(ctx)?;
            if ctx
                .scope
                .add_method_to_type(type_id.clone(), &function.id.name, function_type)
                .is_err()
            {
                return Err(TypeCheckError::RedefinedMethod(
                    RedefinedMethod {
                        type_id,
                        function_name: function.id.name.clone(),
                    },
                    function.position.clone(),
                ));
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{Expression, Function, Id, Instance, Postfix, Statement, TypeName},
        typechecker::{
            context::Context, error::UndefinedType, types::Type, TypeCheckError, TypeCheckable,
            TypeInformation,
        },
    };

    #[test]
    fn test_empty_instance_on_intrinsic() -> Result<()> {
        let mut ctx = Context::default();

        let inst = Instance {
            name: TypeName::Literal("i64".into(), Span::default()),
            functions: vec![],
            info: (),
            position: Span::default(),
        };

        let result = inst.check(&mut ctx)?;

        assert_eq!(
            result,
            Instance {
                name: TypeName::Literal("i64".into(), Span::default()),
                functions: vec![],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void))),
                    context: ctx
                },
                position: Span::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_error_on_undefined_type() -> Result<()> {
        let mut ctx = Context::default();

        let inst = Instance {
            name: TypeName::Literal("Foo".into(), Span::default()),
            functions: vec![],
            info: (),
            position: Span::default(),
        };

        let result = inst.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: TypeName::Literal("Foo".into(), Span::default())
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_empty_instance_on_struct() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope
            .add_type("Foo", Type::Struct("Foo".into(), vec![]))?;

        let inst = Instance {
            name: TypeName::Literal("Foo".into(), Span::default()),
            functions: vec![],
            info: (),
            position: Span::default(),
        };

        let result = inst.check(&mut ctx)?;

        assert_eq!(
            result,
            Instance {
                name: TypeName::Literal("Foo".into(), Span::default()),
                functions: vec![],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void))),
                    context: ctx
                },
                position: Span::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_simple_instance_on_intrinsic() -> Result<()> {
        let mut ctx = Context::default();

        let inst = Instance {
            name: TypeName::Literal("i64".into(), Span::default()),
            functions: vec![Function {
                id: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                },
                parameters: vec![],
                return_type: TypeName::Literal("i64".into(), Span::default()),
                statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                    name: "this".into(),
                    info: (),
                    position: Span::default(),
                }))],
                info: (),
                position: Span::default(),
            }],
            info: (),
            position: Span::default(),
        };

        let result = inst.check(&mut ctx)?;

        assert_eq!(
            result,
            Instance {
                name: TypeName::Literal("i64".into(), Span::default()),
                functions: vec![Function {
                    id: Id {
                        name: "bar".into(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Function {
                                params: vec![],
                                return_value: Box::new(Type::Integer)
                            }))),
                            context: Context::default()
                        },
                        position: Span::default(),
                    },
                    parameters: vec![],
                    return_type: TypeName::Literal("i64".into(), Span::default()),
                    statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                        name: "this".into(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: Context::default()
                        },
                        position: Span::default(),
                    }))],
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Integer)
                        }))),
                        context: Context::default()
                    },
                    position: Span::default(),
                }],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void))),
                    context: Context::default()
                },
                position: Span::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_simple_instance_on_struct() -> Result<()> {
        let mut ctx = Context::default();
        ctx.scope.add_type(
            "Foo",
            Type::Struct("Foo".into(), vec![("baz".into(), Type::Integer)]),
        )?;

        let inst = Instance {
            name: TypeName::Literal("Foo".into(), Span::default()),
            functions: vec![Function {
                id: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                },
                parameters: vec![],
                return_type: TypeName::Literal("i64".into(), Span::default()),
                statements: vec![Statement::YieldingExpression(Expression::Postfix(
                    Postfix::PropertyAccess {
                        expr: Box::new(Expression::Id(Id {
                            name: "this".into(),
                            info: (),
                            position: Span::default(),
                        })),
                        property: Id {
                            name: "baz".into(),
                            info: (),
                            position: Span::default(),
                        },
                        info: (),
                        position: Span::default(),
                    },
                ))],
                info: (),
                position: Span::default(),
            }],
            info: (),
            position: Span::default(),
        };

        let result = inst.check(&mut ctx)?;

        assert_eq!(
            result,
            Instance {
                name: TypeName::Literal("Foo".into(), Span::default()),
                functions: vec![Function {
                    id: Id {
                        name: "bar".into(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Function {
                                params: vec![],
                                return_value: Box::new(Type::Integer),
                            }))),
                            context: Context::default(),
                        },
                        position: Span::default(),
                    },
                    parameters: vec![],
                    return_type: TypeName::Literal("i64".into(), Span::default()),
                    statements: vec![Statement::YieldingExpression(Expression::Postfix(
                        Postfix::PropertyAccess {
                            expr: Box::new(Expression::Id(Id {
                                name: "this".into(),
                                info: TypeInformation {
                                    type_id: Rc::new(RefCell::new(Some(Type::Struct(
                                        "Foo".into(),
                                        vec![("baz".into(), Type::Integer)],
                                    )))),
                                    context: Context::default(),
                                },
                                position: Span::default(),
                            })),
                            property: Id {
                                name: "baz".into(),
                                info: TypeInformation {
                                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                                    context: Context::default(),
                                },
                                position: Span::default(),
                            },
                            info: TypeInformation {
                                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                                context: Context::default(),
                            },
                            position: Span::default(),
                        },
                    ))],
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Integer),
                        }))),
                        context: Context::default(),
                    },
                    position: Span::default(),
                }],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void))),
                    context: Context::default(),
                },
                position: Span::default(),
            }
        );

        Ok(())
    }
}
