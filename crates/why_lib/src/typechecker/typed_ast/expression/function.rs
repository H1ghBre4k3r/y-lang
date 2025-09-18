//! # Function Type Checking: First-Class Citizens with Scope Management
//!
//! Functions in Y are first-class citizens that must integrate seamlessly with
//! the type system while maintaining efficient LLVM code generation. The design
//! priorities that drive this implementation:
//!
//! - Lexical scoping that prevents variable capture complexity
//! - Explicit parameter types to avoid inference overhead at call sites
//! - Return type verification to catch logic errors at compile time
//! - Zero-cost function calls through LLVM's function pointer optimization
//!
//! The scope management here is critical because Y supports nested functions
//! and lambdas, requiring careful isolation of parameter bindings.

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

    /// Function type checking balances first-class function support with performance.
    ///
    /// The scope isolation here is essential because Y supports closures and nested
    /// functions. Without proper parameter scoping, variable capture could introduce
    /// hidden allocations and complicate LLVM's optimization passes.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // Function type checking requires managing a new scope for parameters and local variables
        // Enter a new scope to isolate function parameters from the outer scope
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

        // Process each function parameter in order
        // Parameters create new variable bindings within the function scope
        for param in parameters.into_iter() {
            // Type check parameter: resolves its type annotation and adds variable to scope
            let param = param.check(ctx)?;

            // Extract the concrete parameter type for the function signature
            // Parameters must have explicit types (no inference allowed)
            let Some(param_type) = { param.info.type_id.borrow() }.clone() else {
                // This should not happen since parameters have explicit type annotations
                todo!()
            };
            checked_parameters.push(param);
            param_types.push(param_type);
        }

        // Resolve the function's declared return type to a concrete Type
        // The return type annotation must reference a valid, known type
        let Ok(return_type_id) = Type::try_from((&return_type, &*ctx)) else {
            let position = return_type.position();
            return Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: return_type,
                },
                position,
            ));
        };

        // Type check the function body within the parameter scope
        // The body is a block that may or may not produce a return value
        let checked_body = body.check(ctx)?;

        // Verify that the body's inferred type matches the declared return type
        // This is where we enforce return type correctness
        let mut checked_body = checked_body;
        let body_type = { checked_body.info.type_id.borrow().clone() };
        match body_type {
            // Body has a concrete inferred type - must match declared return type
            Some(inferred_type) => {
                if inferred_type != return_type_id {
                    return Err(TypeCheckError::TypeMismatch(
                        TypeMismatch {
                            expected: return_type_id,
                            actual: inferred_type,
                        },
                        checked_body.position.clone(),
                    ));
                }
            }
            // Body produces no value but function expects void - this is correct
            None if return_type_id == Type::Void => {
                // Body correctly produces no value for void function
            }
            // Body produces no value but function expects a specific type
            None => {
                // Try to propagate the expected return type down to the body's yielding expression
                // This allows type inference to work backwards from the return type
                if let Err(_) = checked_body.update_type(return_type_id.clone()) {
                    // Type propagation failed - body cannot produce the expected type
                    return Err(TypeCheckError::TypeMismatch(
                        TypeMismatch {
                            expected: return_type_id,
                            actual: Type::Void,
                        },
                        return_type.position(),
                    ));
                }
            }
        }

        // Exit the function parameter scope - parameters are no longer accessible
        ctx.scope.exit_scope();

        // Construct the function's type signature from its parameters and return type
        // This creates the Type::Function that represents this function's interface
        let function_type_id = Type::Function {
            params: param_types,
            return_value: Box::new(return_type_id),
        };

        // Create shared type information for both the function and its identifier
        let function_type = Rc::new(RefCell::new(Some(function_type_id.clone())));

        let info = TypeInformation {
            type_id: function_type.clone(),
            context: ctx.clone(),
        };

        // Type the function identifier with the complete function type
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

        // Initialize type information container for the parameter
        let info = TypeInformation {
            type_id: Rc::new(RefCell::new(None)),
            context: ctx.clone(),
        };

        // Resolve the parameter's type annotation to a concrete Type
        // Function parameters must have explicit type annotations
        match Type::try_from((&type_name, &*ctx)) {
            // Type annotation is valid - store the resolved type
            Ok(type_id) => *info.type_id.borrow_mut() = Some(type_id),
            // Type annotation references an undefined type - propagate the error
            Err(e) => {
                return Err(e);
            }
        };

        // Create the typed identifier for this parameter
        let id = Id {
            name: name.clone(),
            info: info.clone(),
            position: id_position.clone(),
        };

        // Add the parameter as an immutable variable in the current function scope
        // Parameters cannot be reassigned within the function body
        if ctx
            .scope
            .add_variable(&id.name, Expression::Id(id.clone()), false)
            .is_err()
        {
            // Parameter name conflicts with an existing binding in this scope
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
