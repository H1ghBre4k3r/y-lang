use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Function, FunctionParameter, Id},
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch, UndefinedType},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Function<()> {
    type Output = Function<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        // at start of function, enter scope
        ctx.scope.enter_scope();

        let Function {
            id,
            parameters,
            return_type,
            statements,
            ..
        } = self;

        let mut checked_parameters = vec![];
        let mut param_types = vec![];

        for param in parameters.into_iter() {
            let param = param.check(ctx)?;
            let Some(param_type) = param.info.type_id.clone().take() else {
                todo!()
            };

            checked_parameters.push(param);
            param_types.push(param_type);
        }

        let Ok(return_type_id) = Type::try_from((&return_type, &*ctx)) else {
            return Err(TypeCheckError::UndefinedType(UndefinedType {
                type_name: return_type,
            }));
        };

        let mut checked_statements = vec![];

        for stmt in statements.into_iter() {
            checked_statements.push(stmt.check(ctx)?);
        }

        match checked_statements.last() {
            Some(last_stmt) => {
                let last_stmt_type = last_stmt.get_info().type_id.clone();
                let inner = last_stmt_type.borrow_mut();

                match inner.as_ref() {
                    Some(inner_type) => {
                        if *inner_type != return_type_id {
                            return Err(TypeCheckError::TypeMismatch(TypeMismatch {
                                expected: return_type_id,
                                actual: inner_type.clone(),
                            }));
                        }
                    }
                    None if return_type_id == Type::Void => {}
                    None => {
                        return Err(TypeCheckError::TypeMismatch(TypeMismatch {
                            expected: return_type_id,
                            actual: Type::Void,
                        }))
                    }
                }
            }
            None if return_type_id == Type::Void => {}
            None => {
                return Err(TypeCheckError::TypeMismatch(TypeMismatch {
                    expected: return_type_id,
                    actual: Type::Void,
                }))
            }
        }

        ctx.scope.exit_scope();

        let function_type = Rc::new(RefCell::new(Some(Type::Function {
            params: param_types,
            return_value: Box::new(return_type_id),
        })));

        let info = TypeInformation {
            type_id: function_type.clone(),
        };

        let id = if let Some(Id { name, .. }) = id {
            ctx.scope.add_variable(&name, function_type);
            Some(Id {
                name,
                info: info.clone(),
            })
        } else {
            None
        };

        Ok(Function {
            id,
            parameters: checked_parameters,
            return_type,
            statements: checked_statements,
            info,
        })
    }
}

impl TypedConstruct for Function<TypeInformation> {}

impl TypeCheckable for FunctionParameter<()> {
    type Output = FunctionParameter<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let FunctionParameter {
            name, type_name, ..
        } = self;

        let name = name.name;

        let info = TypeInformation {
            type_id: Rc::new(RefCell::new(None)),
        };

        match Type::try_from((&type_name, &*ctx)) {
            Ok(type_id) => *info.type_id.borrow_mut() = Some(type_id),
            Err(e) => {
                unimplemented!("{e}")
            }
        };

        ctx.scope.add_variable(&name, info.type_id.clone());

        Ok(FunctionParameter {
            name: Id {
                name,
                info: info.clone(),
            },
            type_name,
            info,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        parser::ast::{Expression, Function, FunctionParameter, Id, Num, Statement, TypeName},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_parameter_type_insertion() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let param = FunctionParameter {
            name: Id {
                name: "foo".into(),
                info: (),
            },
            type_name: TypeName::Literal("i64".into()),
            info: (),
        };

        let param = param.check(&mut ctx)?;

        assert_eq!(
            param.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer)))
            }
        );

        assert_eq!(
            param.name.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer)))
            }
        );

        assert_eq!(
            ctx.scope.get_variable("foo"),
            Some(Rc::new(RefCell::new(Some(Type::Integer))))
        );

        Ok(())
    }

    #[test]
    fn test_function_type_insertion() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let func = Function {
            id: Some(Id {
                name: "foo".into(),
                info: (),
            }),
            parameters: vec![FunctionParameter {
                name: Id {
                    name: "bar".into(),
                    info: (),
                },
                type_name: TypeName::Literal("f64".into()),
                info: (),
            }],
            statements: vec![Statement::YieldingExpression(Expression::Num(
                Num::Integer(42, ()),
            ))],
            return_type: TypeName::Literal("i64".into()),
            info: (),
        };

        let func = func.check(&mut ctx)?;

        assert_eq!(
            func.id,
            Some(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Function {
                        params: vec![Type::FloatingPoint],
                        return_value: Box::new(Type::Integer)
                    })))
                }
            })
        );

        assert_eq!(
            func.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Function {
                    params: vec![Type::FloatingPoint],
                    return_value: Box::new(Type::Integer)
                })))
            }
        );
        Ok(())
    }

    #[test]
    fn test_function_add_to_scope() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let func = Function {
            id: Some(Id {
                name: "foo".into(),
                info: (),
            }),
            parameters: vec![FunctionParameter {
                name: Id {
                    name: "bar".into(),
                    info: (),
                },
                type_name: TypeName::Literal("f64".into()),
                info: (),
            }],
            statements: vec![Statement::YieldingExpression(Expression::Num(
                Num::Integer(42, ()),
            ))],
            return_type: TypeName::Literal("i64".into()),
            info: (),
        };

        func.check(&mut ctx)?;

        let type_id = ctx.scope.get_variable("foo");

        assert_eq!(
            type_id,
            Some(Rc::new(RefCell::new(Some(Type::Function {
                params: vec![Type::FloatingPoint],
                return_value: Box::new(Type::Integer)
            }))))
        );
        Ok(())
    }

    #[test]
    fn test_function_return_type_mismatch() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let func = Function {
            id: Some(Id {
                name: "foo".into(),
                info: (),
            }),
            parameters: vec![],
            statements: vec![Statement::YieldingExpression(Expression::Num(
                Num::Integer(42, ()),
            ))],
            return_type: TypeName::Literal("void".into()),
            info: (),
        };

        let res = func.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatch(TypeMismatch {
                expected: Type::Void,
                actual: Type::Integer
            }))
        );

        Ok(())
    }
}
