use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Expression, Id, Lambda, LambdaParameter},
    typechecker::{
        context::Context,
        error::{RedefinedConstant, TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Lambda<()> {
    type Output = Lambda<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let Lambda {
            parameters,
            expression,
            ..
        } = self;

        let context = ctx.clone();

        ctx.scope.enter_scope();

        let mut checked_parameters = vec![];

        for param in parameters.into_iter() {
            checked_parameters.push(param.check(ctx)?);
        }

        let checked_expression = expression.check(ctx)?;

        ctx.scope.exit_scope();

        Ok(Lambda {
            parameters: checked_parameters,
            expression: Box::new(checked_expression),
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(None)),
                context,
            },
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let Lambda {
            parameters,
            expression,
            ..
        } = this;

        Lambda {
            parameters: parameters.iter().map(TypeCheckable::revert).collect(),
            expression: Box::new(TypeCheckable::revert(expression.as_ref())),
            info: (),
        }
    }
}

impl TypedConstruct for Lambda<TypeInformation> {
    fn update_type(&mut self, type_id: Type) -> Result<(), TypeCheckError> {
        let err = Err(TypeCheckError::TypeMismatch(TypeMismatch {
            expected: Type::Function {
                params: vec![Type::Unknown; self.parameters.len()],
                return_value: Box::new(Type::Unknown),
            },
            actual: type_id.clone(),
        }));

        // check, if we have function
        let Type::Function {
            params,
            return_value,
        } = type_id.clone()
        else {
            return err;
        };

        if let Some(current_type) = self.info.type_id.borrow().as_ref() {
            if *current_type == type_id {
                return Ok(());
            }

            // TODO: maybe use different error for this
            return Err(TypeCheckError::TypeMismatch(TypeMismatch {
                expected: current_type.clone(),
                actual: type_id,
            }));
        }

        // check for correct arity
        if params.len() != self.parameters.len() {
            return err;
        }

        // clone context to mess nothing up
        let mut ctx = self.info.context.clone();

        ctx.scope.enter_scope();

        // enter all parameters with their respective types into the scope
        for (i, t) in params.iter().enumerate() {
            let name = &self.parameters[i].name.name;

            if ctx
                .scope
                .add_variable(
                    name,
                    Expression::Id(Id {
                        name: name.clone(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(t.clone()))),
                            context: ctx.clone(),
                        },
                    }),
                )
                .is_err()
            {
                return Err(TypeCheckError::RedefinedConstant(RedefinedConstant {
                    constant_name: name.to_string(),
                }));
            }
        }

        // check (the reverted) expression
        let expr =
            <Expression<()> as TypeCheckable>::revert(self.expression.as_ref()).check(&mut ctx)?;

        // check, if return types match
        if let Some(expr_type) = expr.get_info().type_id.borrow_mut().as_ref() {
            if *expr_type != *return_value {
                return Err(TypeCheckError::TypeMismatch(TypeMismatch {
                    expected: expr_type.clone(),
                    actual: *return_value.clone(),
                }));
            }
        }

        // update types of parameters accordingly
        for (i, t) in params.iter().enumerate() {
            self.parameters[i].update_type(t.to_owned())?;
        }

        Ok(())
    }
}

impl TypeCheckable for LambdaParameter<()> {
    type Output = LambdaParameter<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let LambdaParameter { name, .. } = self;

        let name = name.name;

        let type_id = Rc::new(RefCell::new(None));

        let id = Id {
            name,
            info: TypeInformation {
                type_id: type_id.clone(),
                context: ctx.clone(),
            },
        };

        if ctx
            .scope
            .add_variable(&id.name, Expression::Id(id.clone()))
            .is_err()
        {
            return Err(TypeCheckError::RedefinedConstant(RedefinedConstant {
                constant_name: id.name,
            }));
        }

        Ok(LambdaParameter {
            name: id,
            info: TypeInformation {
                type_id,
                context: ctx.clone(),
            },
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let LambdaParameter { name, .. } = this;

        LambdaParameter {
            name: TypeCheckable::revert(name),
            info: (),
        }
    }
}

impl TypedConstruct for LambdaParameter<TypeInformation> {
    fn update_type(&mut self, type_id: Type) -> std::result::Result<(), TypeCheckError> {
        *self.info.type_id.borrow_mut() = Some(type_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use anyhow::Result;

    use crate::{
        parser::ast::{Expression, Id, Initialisation, Lambda, LambdaParameter, Num},
        typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation},
    };

    #[test]
    fn test_parameter_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let param = LambdaParameter {
            name: Id {
                name: "foo".into(),
                info: (),
            },
            info: (),
        };

        let param = param.check(&mut ctx)?;

        assert_eq!(
            param,
            LambdaParameter {
                name: Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(None)),
                        context: Context::default(),
                    }
                },
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(None)),
                    context: Context::default(),
                }
            }
        );

        Ok(())
    }

    #[test]
    fn test_parameter_in_scope_insertion() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let param = LambdaParameter {
            name: Id {
                name: "foo".into(),
                info: (),
            },
            info: (),
        };

        param.check(&mut ctx)?;

        assert_eq!(
            ctx.scope.get_variable("foo"),
            Some(Rc::new(RefCell::new(None)))
        );

        Ok(())
    }

    #[test]
    fn test_lambda_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let lambda = Lambda {
            parameters: vec![],
            expression: Box::new(Expression::Num(Num::Integer(42, ()))),
            info: (),
        };

        let lambda = lambda.check(&mut ctx)?;

        assert_eq!(
            lambda,
            Lambda {
                parameters: vec![],
                expression: Box::new(Expression::Num(Num::Integer(
                    42,
                    TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: Context::default(),
                    }
                ))),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(None)),
                    context: Context::default(),
                }
            }
        );

        Ok(())
    }

    #[test]
    fn test_error_on_type_update() -> Result<()> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
            },
            mutable: false,
            type_name: None,
            value: Expression::Lambda(Lambda {
                parameters: vec![LambdaParameter {
                    name: Id {
                        name: "x".into(),
                        info: (),
                    },
                    info: (),
                }],
                expression: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                })),
                info: (),
            }),
            info: (),
        };

        init.check(&mut ctx)?;

        ctx.scope.update_variable(
            "foo",
            Type::Function {
                params: vec![Type::Integer],
                return_value: Box::new(Type::Integer),
            },
        )?;

        ctx.scope.update_variable(
            "foo",
            Type::Function {
                params: vec![Type::Integer],
                return_value: Box::new(Type::Integer),
            },
        )?;

        assert!(ctx
            .scope
            .update_variable(
                "foo",
                Type::Function {
                    params: vec![Type::FloatingPoint],
                    return_value: Box::new(Type::Integer),
                },
            )
            .is_err());
        Ok(())
    }
}
