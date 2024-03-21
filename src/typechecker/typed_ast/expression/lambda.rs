use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Expression, Id, Lambda, LambdaParameter},
    typechecker::{
        context::Context, error::TypeMismatch, types::Type, TypeCheckable, TypeInformation,
        TypeResult, TypedConstruct,
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
            },
        })
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
            },
        };

        ctx.scope.add_variable(&id.name, Expression::Id(id.clone()));

        Ok(LambdaParameter {
            name: id,
            info: TypeInformation { type_id },
        })
    }
}

impl TypedConstruct for LambdaParameter<TypeInformation> {
    fn update_type(&mut self, type_id: Type) -> Result<(), TypeMismatch> {
        // let Type::Function {
        //     params,
        //     return_value,
        // } = type_id.clone()
        // else {
        //     return Err(TypeMismatch {
        //         expected: Type::Function {
        //             params: vec![],
        //             return_value: Box::new(Type::Void),
        //         },
        //         actual: type_id,
        //     });
        // };

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        parser::ast::{Expression, Id, Lambda, LambdaParameter, Num},
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
                        type_id: Rc::new(RefCell::new(None))
                    }
                },
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(None))
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
                        type_id: Rc::new(RefCell::new(Some(Type::Integer)))
                    }
                ))),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(None))
                }
            }
        );

        Ok(())
    }
}
