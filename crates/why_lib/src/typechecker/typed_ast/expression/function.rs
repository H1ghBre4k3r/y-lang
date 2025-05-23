use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Expression, Function, FunctionParameter, Id, Statement},
    typechecker::{
        context::Context,
        error::{
            RedefinedConstant, RedefinedFunction, TypeCheckError, TypeMismatch, UndefinedType,
        },
        types::Type,
        ShallowCheck, TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Function<()> {
    type Typed = Function<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // at start of function, enter scope
        ctx.scope.enter_scope();

        let Function {
            id,
            parameters,
            return_type,
            statements,
            position,
            ..
        } = self;

        let mut checked_parameters = vec![];
        let mut param_types = vec![];

        for param in parameters.into_iter() {
            let param = param.check(ctx)?;
            let Some(param_type) = { param.info.type_id.borrow() }.clone() else {
                todo!()
            };

            checked_parameters.push(param);
            param_types.push(param_type);
        }

        let Ok(return_type_id) = Type::try_from((&return_type, &*ctx)) else {
            let position = return_type.position();
            return Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: return_type,
                },
                position,
            ));
        };

        let mut checked_statements = vec![];

        for stmt in statements.into_iter() {
            checked_statements.push(stmt.check(ctx)?);
        }

        match checked_statements.last_mut() {
            Some(
                last_stmt @ Statement::YieldingExpression(_) | last_stmt @ Statement::Return(_),
            ) => {
                let last_stmt_type = last_stmt.get_info().type_id.clone();
                let inner = { last_stmt_type.borrow().clone() };

                match inner {
                    Some(inner_type) => {
                        if inner_type != return_type_id {
                            return Err(TypeCheckError::TypeMismatch(
                                TypeMismatch {
                                    expected: return_type_id,
                                    actual: inner_type.clone(),
                                },
                                last_stmt.position(),
                            ));
                        }
                    }
                    None if return_type_id == Type::Void => {}
                    None => {
                        last_stmt.update_type(return_type_id.clone())?;
                    }
                }
            }
            _ if return_type_id == Type::Void => {}
            _ => {
                return Err(TypeCheckError::TypeMismatch(
                    TypeMismatch {
                        expected: return_type_id,
                        actual: Type::Void,
                    },
                    return_type.position(),
                ));
            }
        }

        ctx.scope.exit_scope();

        let function_type_id = Type::Function {
            params: param_types,
            return_value: Box::new(return_type_id),
        };

        let function_type = Rc::new(RefCell::new(Some(function_type_id.clone())));

        let info = TypeInformation {
            type_id: function_type.clone(),
            context: ctx.clone(),
        };

        let id = Id {
            name: id.name,
            position: id.position,
            info: info.clone(),
        };

        let func = Function {
            id: id.clone(),
            parameters: checked_parameters,
            return_type,
            statements: checked_statements,
            info,
            position: position.clone(),
        };

        Ok(func)
    }

    fn revert(this: &Self::Typed) -> Self {
        let Function {
            id,
            parameters,
            return_type,
            statements,
            position,
            ..
        } = this;

        Function {
            id: TypeCheckable::revert(id),
            parameters: parameters.iter().map(TypeCheckable::revert).collect(),
            return_type: return_type.to_owned(),
            statements: statements.iter().map(TypeCheckable::revert).collect(),
            info: (),
            position: position.clone(),
        }
    }
}

impl Function<()> {
    /// Perform a shallow check without inserting any information into the scope. This is primarily
    /// used for checking functions associated with instances.
    pub fn simple_shallow_check(&self, ctx: &Context) -> TypeResult<Type> {
        let Function {
            parameters,
            return_type,
            ..
        } = self;

        let mut param_types = vec![];

        for FunctionParameter { type_name, .. } in parameters.iter() {
            let Ok(param_type) = Type::try_from((type_name, ctx)) else {
                return Err(TypeCheckError::UndefinedType(
                    UndefinedType {
                        type_name: type_name.clone(),
                    },
                    type_name.position(),
                ));
            };

            param_types.push(param_type);
        }

        let Ok(return_type) = Type::try_from((return_type, ctx)) else {
            return Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: return_type.clone(),
                },
                return_type.position(),
            ));
        };

        Ok(Type::Function {
            params: param_types,
            return_value: Box::new(return_type),
        })
    }
}

impl ShallowCheck for Function<()> {
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let Function { id, position, .. } = self;

        let type_id = self.simple_shallow_check(&*ctx)?;

        if ctx.scope.add_constant(&id.name, type_id).is_err() {
            return Err(TypeCheckError::RedefinedFunction(
                RedefinedFunction {
                    function_name: id.name.clone(),
                },
                position.clone(),
            ));
        }

        Ok(())
    }
}

impl TypedConstruct for Function<TypeInformation> {
    type Validated = Function<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Function {
            id,
            parameters,
            return_type,
            statements,
            info,
            position,
        } = self;

        let mut validated_parameters = vec![];
        for param in parameters {
            validated_parameters.push(param.validate()?);
        }

        let mut validated_statements = vec![];
        for statement in statements {
            validated_statements.push(statement.validate()?);
        }

        Ok(Function {
            id: id.validate()?,
            parameters: validated_parameters,
            return_type,
            statements: validated_statements,
            info: info.validate(&position)?,
            position,
        })
    }
}

impl TypeCheckable for FunctionParameter<()> {
    type Typed = FunctionParameter<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let FunctionParameter {
            name,
            type_name,
            position: param_position,
            ..
        } = self;

        let Id {
            name,
            position: id_position,
            ..
        } = name;

        let info = TypeInformation {
            type_id: Rc::new(RefCell::new(None)),
            context: ctx.clone(),
        };

        match Type::try_from((&type_name, &*ctx)) {
            Ok(type_id) => *info.type_id.borrow_mut() = Some(type_id),
            Err(e) => {
                return Err(e);
            }
        };

        let id = Id {
            name: name.clone(),
            info: info.clone(),
            position: id_position.clone(),
        };

        if ctx
            .scope
            .add_variable(&id.name, Expression::Id(id.clone()), false)
            .is_err()
        {
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: id.name,
                },
                id_position,
            ));
        };

        Ok(FunctionParameter {
            name: id,
            type_name,
            info,
            position: param_position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let FunctionParameter {
            name,
            type_name,
            position,
            ..
        } = this;

        FunctionParameter {
            name: TypeCheckable::revert(name),
            type_name: type_name.to_owned(),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for FunctionParameter<TypeInformation> {
    type Validated = FunctionParameter<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let FunctionParameter {
            name,
            type_name,
            info,
            position,
        } = self;

        Ok(FunctionParameter {
            name: name.validate()?,
            type_name,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{
            BinaryExpression, BinaryOperator, Expression, Function, FunctionParameter, Id, Num,
            Statement, TypeName,
        },
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            ShallowCheck, TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_parameter_type_insertion() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let param = FunctionParameter {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let param = param.check(&mut ctx)?;

        assert_eq!(
            param.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            }
        );

        assert_eq!(
            param.name.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            }
        );

        assert_eq!(
            ctx.scope.resolve_name("foo"),
            Some(Rc::new(RefCell::new(Some(Type::Integer))))
        );

        Ok(())
    }

    #[test]
    fn test_function_type_insertion() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let func = Function {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            parameters: vec![FunctionParameter {
                name: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                },
                type_name: TypeName::Literal("f64".into(), Span::default()),
                info: (),
                position: Span::default(),
            }],
            statements: vec![Statement::YieldingExpression(Expression::Num(
                Num::Integer(42, (), Span::default()),
            ))],
            return_type: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let func = func.check(&mut ctx)?;

        assert_eq!(
            func.id,
            Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Function {
                        params: vec![Type::FloatingPoint],
                        return_value: Box::new(Type::Integer)
                    }))),
                    context: Context::default(),
                },
                position: Span::default()
            }
        );

        assert_eq!(
            func.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Function {
                    params: vec![Type::FloatingPoint],
                    return_value: Box::new(Type::Integer)
                }))),
                context: Context::default(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_function_add_to_scope() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let func = Function {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            parameters: vec![FunctionParameter {
                name: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                },
                type_name: TypeName::Literal("f64".into(), Span::default()),
                info: (),
                position: Span::default(),
            }],
            statements: vec![Statement::YieldingExpression(Expression::Num(
                Num::Integer(42, (), Span::default()),
            ))],
            return_type: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        func.shallow_check(&mut ctx)?;

        let type_id = ctx.scope.resolve_name("foo");

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
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            parameters: vec![],
            statements: vec![Statement::YieldingExpression(Expression::Num(
                Num::Integer(42, (), Span::default()),
            ))],
            return_type: TypeName::Literal("void".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let res = func.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::Void,
                    actual: Type::Integer
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_simple_add_function() -> Result<()> {
        let mut ctx = Context::default();

        let func = Function {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            parameters: vec![
                FunctionParameter {
                    name: Id {
                        name: "x".into(),
                        position: Span::default(),
                        info: (),
                    },
                    type_name: TypeName::Literal("i64".into(), Span::default()),
                    info: (),
                    position: Span::default(),
                },
                FunctionParameter {
                    name: Id {
                        name: "y".into(),
                        position: Span::default(),
                        info: (),
                    },
                    type_name: TypeName::Literal("i64".into(), Span::default()),
                    info: (),
                    position: Span::default(),
                },
            ],
            statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                BinaryExpression {
                    left: Expression::Id(Id {
                        name: "x".into(),
                        position: Span::default(),
                        info: (),
                    }),
                    right: Expression::Id(Id {
                        name: "y".into(),
                        position: Span::default(),
                        info: (),
                    }),
                    operator: BinaryOperator::Add,
                    position: Span::default(),
                    info: (),
                },
            )))],
            return_type: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        func.check(&mut ctx)?;
        Ok(())
    }
}
