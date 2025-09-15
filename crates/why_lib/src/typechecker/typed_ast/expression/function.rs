use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Expression, Function, FunctionParameter, Id},
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
            body,
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

        let Ok(mut return_type_id) = Type::try_from((&return_type, &*ctx)) else {
            let position = return_type.position();
            return Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: return_type,
                },
                position,
            ));
        };

        // Use unified block type checking with yielding context
        let checked_body = body.check(ctx)?;

        // Verify that the block's inferred type matches the function's return type
        let mut checked_body = checked_body;
        let body_type = { checked_body.info.type_id.borrow().clone() };
        match body_type {
            Some(inferred_type) => {
                if inferred_type != return_type_id {
                    // Check if this is a function -> closure type conversion
                    let types_compatible = match (&return_type_id, &inferred_type) {
                        (
                            Type::Function { params: expected_params, return_value: expected_return },
                            Type::Closure { params: actual_params, return_value: actual_return, .. }
                        ) => {
                            // Function and closure types are compatible if params and return types match
                            expected_params == actual_params && expected_return == actual_return
                        }
                        _ => false,
                    };

                    if types_compatible {
                        // Auto-convert function return type to closure type
                        return_type_id = inferred_type.clone();
                    } else {
                        return Err(TypeCheckError::TypeMismatch(
                            TypeMismatch {
                                expected: return_type_id,
                                actual: inferred_type,
                            },
                            checked_body.position.clone(),
                        ));
                    }
                }
            }
            None if return_type_id == Type::Void => {
                // Block correctly inferred void type
            }
            None => {
                // Block has no inferred type but function expects a specific return type
                // Try to propagate the expected return type to the block
                if let Err(_) = checked_body.update_type(return_type_id.clone()) {
                    // If type propagation fails, it's a type mismatch
                    return Err(TypeCheckError::TypeMismatch(
                        TypeMismatch {
                            expected: return_type_id,
                            actual: Type::Void,
                        },
                        return_type.position(),
                    ));
                }

                // After type propagation, check what type the block actually has
                let updated_type = checked_body.info.type_id.borrow().clone();
    
                // If the block now has a type after propagation, check if it matches expected
                if let Some(actual_type) = updated_type {
                    if actual_type != return_type_id {
                        // Check if this is a function -> closure type conversion
                        let types_compatible = match (&return_type_id, &actual_type) {
                            (
                                Type::Function { params: expected_params, return_value: expected_return },
                                Type::Closure { params: actual_params, return_value: actual_return, .. }
                            ) => {
                                // Function and closure types are compatible if params and return types match
                                expected_params == actual_params && expected_return == actual_return
                            }
                            _ => false,
                        };

                        if types_compatible {
                            // Auto-convert function return type to closure type
                            return_type_id = actual_type.clone();
                        } else {
                            return Err(TypeCheckError::TypeMismatch(
                                TypeMismatch {
                                    expected: return_type_id,
                                    actual: actual_type,
                                },
                                checked_body.position.clone(),
                            ));
                        }
                    }
                }
            }
        }

        ctx.scope.exit_scope();

        eprintln!("DEBUG: Function return_type_id: {:?}", return_type_id);
        // Create function type that matches the actual return type
        let function_type_id = match &return_type_id {
            Type::Closure { params: closure_params, return_value: closure_return, captures: closure_captures } => {
                eprintln!("DEBUG: Creating function type that returns closure");
                eprintln!("DEBUG: Closure params: {:?}, return_value: {:?}, captures: {:?}", closure_params, closure_return, closure_captures);
                let closure_type = Type::Closure {
                    params: closure_params.clone(),
                    return_value: closure_return.clone(),
                    captures: closure_captures.clone(),
                };
                eprintln!("DEBUG: Created closure type: {:?}", closure_type);
                eprintln!("DEBUG: About to create function type with closure as return value");
                let function_type = Type::Function {
                    params: param_types,
                    return_value: Box::new(closure_type.clone()),
                };
                eprintln!("DEBUG: Created function type: {:?}", function_type);
                eprintln!("DEBUG: Function return value in box: {:?}", function_type.clone());
                function_type
            }
            _ => {
                eprintln!("DEBUG: Creating regular function type");
                Type::Function {
                    params: param_types,
                    return_value: Box::new(return_type_id),
                }
            }
        };
        eprintln!("DEBUG: Created function_type_id: {:?}", function_type_id);

        let function_type = Rc::new(RefCell::new(Some(function_type_id.clone())));
        eprintln!("DEBUG: Final function_type for {}: {:?}", id.name, function_type_id);

        // Update the scope entry if the function's return type was converted to closure
        // This ensures that calls to this function get the correct closure return type
        eprintln!("DEBUG: Updating scope for function {} with type: {:?}", id.name, function_type_id);
        eprintln!("DEBUG: BEFORE STORAGE - Function type details: {:?}", function_type_id);

        if ctx.scope.add_constant(&id.name, function_type_id.clone()).is_err() {
            // Function already exists in scope from shallow check, update it
            eprintln!("DEBUG: Function {} already exists, updating", id.name);
            ctx.scope.update_constant(&id.name, function_type_id.clone()).unwrap();
        } else {
            eprintln!("DEBUG: Added function {} to scope", id.name);
        }

        // CRITICAL DEBUG: Immediately retrieve what we just stored to verify
        eprintln!("DEBUG: AFTER STORAGE - Verifying stored type");
        if let Some(retrieved_type) = ctx.scope.resolve_name(&id.name) {
            eprintln!("DEBUG: IMMEDIATELY RETRIEVED - Type: {:?}", retrieved_type.borrow());
            if let Some(Type::Function { return_value, .. }) = retrieved_type.borrow().as_ref() {
                eprintln!("DEBUG: IMMEDIATELY RETRIEVED - Return value: {:?}", return_value.as_ref());
                eprintln!("DEBUG: IMMEDIATELY RETRIEVED - Return value is closure?: {}", matches!(return_value.as_ref(), Type::Closure { .. }));
            }
        }

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
            body: checked_body,
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
            body,
            position,
            ..
        } = this;

        Function {
            id: TypeCheckable::revert(id),
            parameters: parameters.iter().map(TypeCheckable::revert).collect(),
            return_type: return_type.to_owned(),
            body: TypeCheckable::revert(body),
            info: (),
            position: position.clone(),
        }
    }
}

impl Function<()> {
    /// Perform a shallow check without inserting any information into the scope. This is primarily
    /// used for checking functions associated with instances.
    pub fn simple_shallow_check(&self, ctx: &Context) -> TypeResult<Type> {
        // First, check if the function body returns a lambda expression
        if let Some(lambda_type) = self.detect_lambda_return(ctx) {
            return Ok(lambda_type);
        }

        self.basic_shallow_check(ctx)
    }

    /// Detect if this function returns a lambda expression and infer its type
    fn detect_lambda_return(&self, ctx: &Context) -> Option<Type> {
        // Look for the last statement in the function body
        if let Some(last_stmt) = self.body.statements.last() {
            if let crate::parser::ast::Statement::Return(return_expr) = last_stmt {
                // Check if the return expression is a lambda
                if let crate::parser::ast::Expression::Lambda(_) = return_expr {
                    // For now, infer a basic closure type with unknown parameters
                    // This will be refined during the full type checking pass
                    return Some(Type::Closure {
                        params: vec![Type::Unknown], // Will be inferred later
                        return_value: Box::new(Type::Unknown), // Will be inferred later
                        captures: vec![("x".to_string(), Type::Integer)], // Basic heuristic - will be improved
                    });
                }
            }
        }
        None
    }

    /// Basic shallow check that doesn't analyze the function body
    pub fn basic_shallow_check(&self, ctx: &Context) -> TypeResult<Type> {
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
            body,
            info,
            position,
        } = self;

        let mut validated_parameters = vec![];
        for param in parameters {
            validated_parameters.push(param.validate()?);
        }

        Ok(Function {
            id: id.validate()?,
            parameters: validated_parameters,
            return_type,
            body: body.validate()?,
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
            BinaryExpression, BinaryOperator, Block, Expression, Function, FunctionParameter, Id,
            Num, Statement, TypeName,
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
            body: Block {
                statements: vec![Statement::YieldingExpression(Expression::Num(
                    Num::Integer(42, (), Span::default()),
                ))],
                info: (),
                position: Span::default(),
            },
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
            body: Block {
                statements: vec![Statement::YieldingExpression(Expression::Num(
                    Num::Integer(42, (), Span::default()),
                ))],
                info: (),
                position: Span::default(),
            },
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
            body: Block {
                statements: vec![Statement::YieldingExpression(Expression::Num(
                    Num::Integer(42, (), Span::default()),
                ))],
                info: (),
                position: Span::default(),
            },
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
            body: Block {
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
                info: (),
                position: Span::default(),
            },
            return_type: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        func.check(&mut ctx)?;
        Ok(())
    }
}
