use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Expression, Id, Lambda, LambdaParameter},
    typechecker::{
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
        context::Context,
        error::{RedefinedConstant, TypeCheckError, TypeMismatch},
        types::Type,
    },
};

impl TypeCheckable for Lambda<()> {
    type Typed = Lambda<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Lambda {
            parameters,
            expression,
            position,
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

        // try to infer the type of the lambda
        let type_id = match (
            checked_parameters.len(),
            checked_expression.get_info().type_id.borrow().clone(),
        ) {
            // in the special case where we have no parameters and a distinct return type of the
            // lambda, we can actually infer the type of the entire lambda
            (0, Some(type_id)) => Some(Type::Function {
                params: vec![],
                return_value: Box::new(type_id),
            }),
            _ => None,
        };

        Ok(Lambda {
            parameters: checked_parameters,
            expression: Box::new(checked_expression),
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(type_id)),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Lambda {
            parameters,
            expression,
            position,
            ..
        } = this;

        Lambda {
            parameters: parameters.iter().map(TypeCheckable::revert).collect(),
            expression: Box::new(TypeCheckable::revert(expression.as_ref())),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Lambda<TypeInformation> {
    type Validated = Lambda<ValidatedTypeInformation>;

    fn update_type(&mut self, type_id: Type) -> Result<(), TypeCheckError> {
        let err = Err(TypeCheckError::TypeMismatch(
            TypeMismatch {
                expected: Type::Function {
                    params: vec![Type::Unknown; self.parameters.len()],
                    return_value: Box::new(Type::Unknown),
                },
                actual: type_id.clone(),
            },
            self.position.clone(),
        ));

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

            // Check if current type can be refined by the new type
            // This allows lambdas with Unknown parameter types to be updated
            if let Type::Function {
                params: current_params,
                return_value: current_return,
            } = current_type
            {
                // If current lambda has Unknown parameter types, allow refinement
                let can_refine = current_params.iter().any(|p| matches!(p, Type::Unknown))
                    || matches!(current_return.as_ref(), Type::Unknown);

                if can_refine {
                    // Allow the update to proceed - fall through to the update logic below
                } else {
                    // Types are concrete and don't match - this is an error
                    return Err(TypeCheckError::TypeMismatch(
                        TypeMismatch {
                            expected: current_type.clone(),
                            actual: type_id,
                        },
                        self.position.clone(),
                    ));
                }
            } else {
                // Current type is not a function - this is an error
                return Err(TypeCheckError::TypeMismatch(
                    TypeMismatch {
                        expected: current_type.clone(),
                        actual: type_id,
                    },
                    self.position.clone(),
                ));
            }
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
            let position = &self.parameters[i].name.position;

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
                        position: position.clone(),
                    }),
                    false,
                )
                .is_err()
            {
                return Err(TypeCheckError::RedefinedConstant(
                    RedefinedConstant {
                        constant_name: name.to_string(),
                    },
                    position.clone(),
                ));
            }
        }

        // check (the reverted) expression
        let expr =
            <Expression<()> as TypeCheckable>::revert(self.expression.as_ref()).check(&mut ctx)?;

        // check, if return types match
        if let Some(expr_type) = expr.get_info().type_id.borrow_mut().as_ref()
            && *expr_type != *return_value
        {
            return Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: expr_type.clone(),
                    actual: *return_value.clone(),
                },
                expr.position(),
            ));
        }

        // update types of parameters accordingly
        for (i, t) in params.iter().enumerate() {
            self.parameters[i].update_type(t.to_owned())?;
        }

        // update our expression as well
        self.expression = Box::new(expr);

        self.info.type_id = Rc::new(RefCell::new(Some(type_id)));

        Ok(())
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Lambda {
            parameters,
            expression,
            info,
            position,
        } = self;

        let mut validated_parameters = vec![];
        for param in parameters {
            validated_parameters.push(param.validate()?);
        }

        Ok(Lambda {
            parameters: validated_parameters,
            expression: Box::new(expression.validate()?),
            info: info.validate(&position)?,
            position,
        })
    }
}

impl TypeCheckable for LambdaParameter<()> {
    type Typed = LambdaParameter<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let LambdaParameter {
            name,
            position: param_position,
            ..
        } = self;

        let Id {
            name,
            position: id_position,
            ..
        } = name;

        let type_id = Rc::new(RefCell::new(None));

        let id = Id {
            name,
            info: TypeInformation {
                type_id: type_id.clone(),
                context: ctx.clone(),
            },
            position: id_position,
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
                param_position,
            ));
        }

        Ok(LambdaParameter {
            name: id,
            info: TypeInformation {
                type_id,
                context: ctx.clone(),
            },
            position: param_position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let LambdaParameter { name, position, .. } = this;

        LambdaParameter {
            name: TypeCheckable::revert(name),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for LambdaParameter<TypeInformation> {
    type Validated = LambdaParameter<ValidatedTypeInformation>;

    fn update_type(&mut self, type_id: Type) -> std::result::Result<(), TypeCheckError> {
        *self.info.type_id.borrow_mut() = Some(type_id);

        Ok(())
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let LambdaParameter {
            name,
            info,
            position,
        } = self;

        Ok(LambdaParameter {
            name: name.validate()?,
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
        parser::ast::{Expression, Id, Initialisation, Lambda, LambdaParameter, Num, TypeName},
        typechecker::{TypeCheckable, TypeInformation, context::Context, types::Type},
    };

    #[test]
    fn test_parameter_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let param = LambdaParameter {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            info: (),
            position: Span::default(),
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
                    },
                    position: Span::default(),
                },
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(None)),
                    context: Context::default(),
                },
                position: Span::default(),
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
                position: Span::default(),
            },
            info: (),
            position: Span::default(),
        };

        param.check(&mut ctx)?;

        assert_eq!(
            ctx.scope.resolve_name("foo"),
            Some(Rc::new(RefCell::new(None)))
        );

        Ok(())
    }

    #[test]
    fn test_lambda_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let lambda = Lambda {
            parameters: vec![],
            expression: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
            info: (),
            position: Span::default(),
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
                    },
                    Span::default()
                ))),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Function {
                        params: vec![],
                        return_value: Box::new(Type::Integer)
                    }))),
                    context: Context::default(),
                },
                position: Span::default(),
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
                position: Span::default(),
            },
            mutable: false,
            type_name: Some(crate::parser::ast::TypeName::Fn {
                params: vec![TypeName::Literal("i64".into(), Span::default())],
                return_type: Box::new(TypeName::Literal("i64".into(), Span::default())),
                position: Span::default(),
            }),
            value: Expression::Lambda(Lambda {
                parameters: vec![LambdaParameter {
                    name: Id {
                        name: "x".into(),
                        info: (),
                        position: Span::default(),
                    },
                    info: (),
                    position: Span::default(),
                }],
                expression: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default(),
                })),
                info: (),
                position: Span::default(),
            }),
            info: (),
            position: Span::default(),
        };

        init.check(&mut ctx)?;

        ctx.scope.update_variable(
            "foo",
            Type::Function {
                params: vec![Type::Integer],
                return_value: Box::new(Type::Integer),
            },
        )?;

        assert!(
            ctx.scope
                .update_variable(
                    "foo",
                    Type::Function {
                        params: vec![Type::FloatingPoint],
                        return_value: Box::new(Type::Integer),
                    },
                )
                .is_err()
        );
        Ok(())
    }
}
