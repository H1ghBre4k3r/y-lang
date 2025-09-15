use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Expression, Id, Postfix},
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch, UndefinedVariable},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Postfix<()> {
    type Typed = Postfix<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();
        match self {
            Postfix::Call {
                expr,
                args,
                position,
                ..
            } => {
                let expr = expr.check(ctx)?;

                let expr_type_id = { expr.get_info().type_id.borrow() }.clone();

                // Check if this is a function call that might need closure type resolution
                if let Some(Type::Function { .. }) = &expr_type_id {
                    // If the expression is an ID, check if it's a function that might return closures
                    if let Expression::Id(id_expr) = &expr {
                        // Check if this function has been processed for closures yet
                        // Force a scope refresh to get the latest type information
                        if let Some(updated_type) = ctx.scope.resolve_name(&id_expr.name) {
                            let updated_type = updated_type.borrow().clone();
                            if let Some(updated_type) = updated_type {
                                // Update the expression type if the scope has newer information
                                *expr.get_info().type_id.borrow_mut() = Some(updated_type.clone());
                                let expr_type_id = { expr.get_info().type_id.borrow() }.clone();
                                // eprintln!("DEBUG: Refreshed function type for {}: {:?}", id_expr.name, expr_type_id);

                                // If it's a function type, check what the return value looks like
                                if let Type::Function {
                                    params,
                                    return_value,
                                } = &updated_type
                                {
                                    // eprintln!("DEBUG: Function has params: {:?}, return_value: {:?}", params, return_value);
                                }
                            }
                        }
                    }
                }

                let mut checked_args = vec![];
                for arg in args.into_iter() {
                    checked_args.push(arg.check(ctx)?);
                }

                let arg_types = checked_args
                    .iter()
                    .map(|a| {
                        { a.get_info().type_id.borrow() }
                            .clone()
                            .unwrap_or(Type::Unknown)
                    })
                    .collect::<Vec<_>>();

                let expected_type = Type::Function {
                    params: arg_types.clone(),
                    return_value: Box::new(Type::Unknown),
                };

                let type_id = match &expr_type_id {
                    Some(Type::Function {
                        params,
                        return_value,
                    }) => {
                        // param length did not match
                        if params.len() != checked_args.len() {
                            return Err(TypeCheckError::TypeMismatch(
                                TypeMismatch {
                                    expected: expected_type,
                                    actual: Type::Function {
                                        params: params.clone(),
                                        return_value: return_value.clone(),
                                    },
                                },
                                position,
                            ));
                        }

                        // check if types of parameters and arguments match
                        for (i, arg) in checked_args.iter_mut().enumerate() {
                            let expected = params[i].clone();
                            let actual = arg_types[i].clone();

                            if actual != expected {
                                if actual == Type::Unknown {
                                    arg.update_type(expected)?;
                                } else {
                                    return Err(TypeCheckError::TypeMismatch(
                                        TypeMismatch { expected, actual },
                                        arg.position(),
                                    ));
                                }
                            }
                        }

                        eprintln!("DEBUG: Function call return_value: {:?}", return_value);

                        // CRITICAL FIX: Check if this is a call to a function that might have been updated
                        // with closure type information after initial processing
                        let final_return_type = if let Expression::Id(id_expr) = &expr {
                            // Get the most recent type information from scope
                            if let Some(updated_function_type) = ctx.scope.resolve_name(&id_expr.name) {
                                let borrowed = updated_function_type.borrow();
                                let full_type = borrowed.as_ref();
                                eprintln!("DEBUG: Full function type from scope: {:?}", full_type);

                                if let Some(Type::Function { return_value: updated_return, .. }) = full_type {
                                    eprintln!("DEBUG: Extracted return value from function type: {:?}", updated_return);

                                    // INTELLIGENT TYPE SELECTION: Prefer closure types over function types
                                    let current_return = return_value.as_ref();
                                    let scope_return = updated_return.as_ref();

                                    let final_type = match (current_return, scope_return) {
                                        // If scope has closure and current doesn't, use scope
                                        (Type::Function { .. }, Type::Closure { .. }) => {
                                            eprintln!("DEBUG: Scope has closure, current has function - using scope closure");
                                            scope_return.clone()
                                        }
                                        // If current has closure and scope doesn't, use current
                                        (Type::Closure { .. }, Type::Function { .. }) => {
                                            eprintln!("DEBUG: Current has closure, scope has function - using current closure");
                                            current_return.clone()
                                        }
                                        // If both are closures, use scope (more recent)
                                        (Type::Closure { .. }, Type::Closure { .. }) => {
                                            eprintln!("DEBUG: Both are closures - using scope (more recent)");
                                            scope_return.clone()
                                        }
                                        // Otherwise use scope
                                        _ => {
                                            eprintln!("DEBUG: Using scope return value");
                                            scope_return.clone()
                                        }
                                    };

                                    eprintln!("DEBUG: Final selected type: {:?}", final_type);
                                    final_type
                                } else {
                                    eprintln!("DEBUG: Scope doesn't contain function type, using original");
                                    return_value.as_ref().clone()
                                }
                            } else {
                                eprintln!("DEBUG: Function not found in scope, using original");
                                return_value.as_ref().clone()
                            }
                        } else {
                            eprintln!("DEBUG: Not an ID expression, using original");
                            return_value.as_ref().clone()
                        };

                        eprintln!("DEBUG: Function call final return_type: {:?}", final_return_type);
                        Rc::new(RefCell::new(Some(final_return_type)))
                    }
                    Some(Type::Closure {
                        params,
                        return_value,
                        captures,
                    }) => {
                        // Handle closure calls - return the closure type that was returned
                        if params.len() != checked_args.len() {
                            return Err(TypeCheckError::TypeMismatch(
                                TypeMismatch {
                                    expected: expected_type,
                                    actual: Type::Function {
                                        params: params.clone(),
                                        return_value: return_value.clone(),
                                    },
                                },
                                position,
                            ));
                        }

                        // check if types of parameters and arguments match
                        for (i, arg) in checked_args.iter_mut().enumerate() {
                            let expected = params[i].clone();
                            let actual = arg_types[i].clone();

                            if actual != expected {
                                if actual == Type::Unknown {
                                    arg.update_type(expected)?;
                                } else {
                                    return Err(TypeCheckError::TypeMismatch(
                                        TypeMismatch { expected, actual },
                                        arg.position(),
                                    ));
                                }
                            }
                        }

                        // Simply use the return value as-is for closure calls too
                        let return_type = return_value.as_ref().clone();
                        Rc::new(RefCell::new(Some(return_type)))
                    }
                    Some(t) => {
                        return Err(TypeCheckError::TypeMismatch(
                            TypeMismatch {
                                expected: expected_type,
                                actual: t.clone(),
                            },
                            position,
                        ));
                    }
                    _ => Rc::new(RefCell::new(None)),
                };

                // eprintln!("DEBUG: Final call type_id: {:?}", type_id);
                Ok(Postfix::Call {
                    expr: Box::new(expr),
                    args: checked_args,
                    info: TypeInformation { type_id, context },
                    position,
                })
            }
            Postfix::Index {
                expr,
                index,
                position,
                ..
            } => {
                let expr = expr.check(ctx)?;
                let index = index.check(ctx)?;

                let expr_type = { expr.get_info().type_id.borrow() }.clone();
                let index_type = { index.get_info().type_id.borrow() }.clone();

                // check, if expr is callable and if index is an integer
                let type_id = match (expr_type, index_type) {
                    // all good
                    (Some(Type::Array(inner)), Some(Type::Integer)) => {
                        Rc::new(RefCell::new(Some(*inner)))
                    }
                    // Nope - not callable
                    (Some(expr_type), Some(Type::Integer)) => {
                        return Err(TypeCheckError::TypeMismatch(
                            TypeMismatch {
                                expected: Type::Array(Box::new(Type::Unknown)),
                                actual: expr_type,
                            },
                            expr.position(),
                        ));
                    }
                    // Not index with an integer
                    (Some(_), Some(index_type)) => {
                        return Err(TypeCheckError::TypeMismatch(
                            TypeMismatch {
                                expected: Type::Integer,
                                actual: index_type,
                            },
                            index.position(),
                        ));
                    }
                    // We somehow have no valuable information about this
                    _ => Rc::new(RefCell::new(None)),
                };

                Ok(Postfix::Index {
                    expr: Box::new(expr),
                    index: Box::new(index),
                    info: TypeInformation { type_id, context },
                    position,
                })
            }
            Postfix::PropertyAccess {
                expr,
                property,
                position,
                ..
            } => {
                let expr = expr.check(ctx)?;

                let Id {
                    name: property_name,
                    position: property_position,
                    ..
                } = property;

                let expr_type = { expr.get_info().type_id.borrow() }.clone();

                let type_id = match expr_type {
                    Some(type_id) => {
                        match ctx.scope.resolve_property_for_type(type_id, &property_name) {
                            Some(type_id) => Some(type_id),
                            None => {
                                return Err(TypeCheckError::UndefinedVariable(
                                    UndefinedVariable {
                                        variable_name: property_name.clone(),
                                    },
                                    property_position,
                                ));
                            }
                        }
                    }
                    None => None,
                };

                let type_id = Rc::new(RefCell::new(type_id));

                Ok(Postfix::PropertyAccess {
                    expr: Box::new(expr),
                    property: Id {
                        name: property_name,
                        position: property_position,
                        info: TypeInformation {
                            type_id: type_id.clone(),
                            context: context.clone(),
                        },
                    },
                    info: TypeInformation { type_id, context },
                    position,
                })
            }
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            Postfix::Call {
                expr,
                args,
                position,
                ..
            } => Postfix::Call {
                expr: Box::new(TypeCheckable::revert(expr.as_ref())),
                args: args.iter().map(TypeCheckable::revert).collect(),
                info: (),
                position: position.clone(),
            },
            Postfix::Index {
                expr,
                index,
                position,
                ..
            } => Postfix::Index {
                expr: Box::new(TypeCheckable::revert(expr.as_ref())),
                index: Box::new(TypeCheckable::revert(index.as_ref())),
                info: (),
                position: position.clone(),
            },
            Postfix::PropertyAccess {
                expr,
                property,
                position,
                ..
            } => Postfix::PropertyAccess {
                expr: Box::new(TypeCheckable::revert(expr.as_ref())),
                property: TypeCheckable::revert(property),
                info: (),
                position: position.clone(),
            },
        }
    }
}

impl TypedConstruct for Postfix<TypeInformation> {
    type Validated = Postfix<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            Postfix::Call {
                expr,
                args,
                info,
                position,
            } => {
                let mut validated_args = vec![];
                for arg in args {
                    validated_args.push(arg.validate()?);
                }

                Ok(Postfix::Call {
                    expr: Box::new(expr.validate()?),
                    args: validated_args,
                    info: info.validate(&position)?,
                    position,
                })
            }
            Postfix::Index {
                expr,
                index,
                info,
                position,
            } => Ok(Postfix::Index {
                expr: Box::new(expr.validate()?),
                index: Box::new(index.validate()?),
                info: info.validate(&position)?,
                position,
            }),
            Postfix::PropertyAccess {
                expr,
                property,
                info,
                position,
            } => Ok(Postfix::PropertyAccess {
                expr: Box::new(expr.validate()?),
                property: property.validate()?,
                info: info.validate(&position)?,
                position,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{Expression, Id, Num, Postfix},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch, UndefinedVariable},
            types::Type,
            TypeCheckable,
        },
    };

    #[test]
    fn test_simple_call() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_constant(
            "foo",
            Type::Function {
                params: vec![],
                return_value: Box::new(Type::Integer),
            },
        )?;

        let call = Postfix::Call {
            expr: Box::new(Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            })),
            args: vec![],
            info: (),
            position: Span::default(),
        };

        let call = call.check(&mut ctx)?;

        assert_eq!(
            call.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        Ok(())
    }

    #[test]
    fn test_complex_call() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_constant(
            "foo",
            Type::Function {
                params: vec![Type::FloatingPoint, Type::Integer],
                return_value: Box::new(Type::Integer),
            },
        )?;

        ctx.scope.add_constant("bar", Type::FloatingPoint)?;

        let call = Postfix::Call {
            expr: Box::new(Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            })),
            args: vec![
                Expression::Id(Id {
                    name: "bar".into(),
                    position: Span::default(),
                    info: (),
                }),
                Expression::Num(Num::Integer(42, (), Span::default())),
            ],
            info: (),
            position: Span::default(),
        };

        let call = call.check(&mut ctx)?;

        assert_eq!(
            call.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        Ok(())
    }

    #[test]
    fn test_call_mismatch() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_constant(
            "foo",
            Type::Function {
                params: vec![Type::Integer, Type::FloatingPoint],
                return_value: Box::new(Type::Integer),
            },
        )?;

        ctx.scope.add_constant("bar", Type::FloatingPoint)?;

        let call = Postfix::Call {
            expr: Box::new(Expression::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            })),
            args: vec![
                Expression::Id(Id {
                    name: "bar".into(),
                    position: Span::default(),
                    info: (),
                }),
                Expression::Num(Num::Integer(42, (), Span::default())),
            ],
            info: (),
            position: Span::default(),
        };

        let result = call.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::Integer,
                    actual: Type::FloatingPoint
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_simple_index() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope
            .add_constant("foo", Type::Array(Box::new(Type::Integer)))?;

        let index = Postfix::Index {
            expr: Box::new(Expression::Id(Id {
                name: "foo".into(),
                position: Span::default(),
                info: (),
            })),
            index: Box::new(Expression::Num(Num::Integer(10, (), Span::default()))),
            info: (),
            position: Span::default(),
        };

        let index = index.check(&mut ctx)?;

        assert_eq!(
            index.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        Ok(())
    }

    #[test]
    fn test_call_type_mismatch() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope
            .add_constant("foo", Type::Array(Box::new(Type::Integer)))?;

        ctx.scope
            .add_constant("bar", Type::Array(Box::new(Type::FloatingPoint)))?;

        let index = Postfix::Index {
            expr: Box::new(Expression::Id(Id {
                name: "foo".into(),
                position: Span::default(),
                info: (),
            })),
            index: Box::new(Expression::Id(Id {
                name: "bar".into(),
                position: Span::default(),
                info: (),
            })),
            info: (),
            position: Span::default(),
        };

        let res = index.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::Integer,
                    actual: Type::Array(Box::new(Type::FloatingPoint))
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_simple_property_access() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_constant(
            "foo",
            Type::Struct("Foo".into(), vec![("bar".into(), Type::Integer)]),
        )?;

        let access = Postfix::PropertyAccess {
            expr: Box::new(Expression::Id(Id {
                name: "foo".into(),
                position: Span::default(),
                info: (),
            })),
            property: Id {
                name: "bar".into(),
                position: Span::default(),
                info: (),
            },
            info: (),
            position: Span::default(),
        };

        let access = access.check(&mut ctx)?;

        assert_eq!(
            access.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        Ok(())
    }

    #[test]
    fn test_complex_property_access() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_constant(
            "foo",
            Type::Struct(
                "Foo".into(),
                vec![(
                    "bar".into(),
                    Type::Struct("Bar".into(), vec![("baz".into(), Type::FloatingPoint)]),
                )],
            ),
        )?;

        let access = Postfix::PropertyAccess {
            expr: Box::new(Expression::Postfix(Postfix::PropertyAccess {
                expr: Box::new(Expression::Id(Id {
                    name: "foo".into(),
                    position: Span::default(),
                    info: (),
                })),
                property: Id {
                    name: "bar".into(),
                    position: Span::default(),
                    info: (),
                },
                info: (),
                position: Span::default(),
            })),
            property: Id {
                name: "baz".into(),
                position: Span::default(),
                info: (),
            },
            info: (),
            position: Span::default(),
        };

        let access = access.check(&mut ctx)?;

        assert_eq!(
            access.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::FloatingPoint)))
        );

        Ok(())
    }

    #[test]
    fn test_missing_property_access() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_constant(
            "foo",
            Type::Struct("Foo".into(), vec![("bar".into(), Type::Integer)]),
        )?;

        let access = Postfix::PropertyAccess {
            expr: Box::new(Expression::Id(Id {
                name: "foo".into(),
                position: Span::default(),
                info: (),
            })),
            property: Id {
                name: "baz".into(),
                position: Span::default(),
                info: (),
            },
            info: (),
            position: Span::default(),
        };

        let res = access.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::UndefinedVariable(
                UndefinedVariable {
                    variable_name: "baz".into(),
                },
                Span::default()
            ))
        );

        Ok(())
    }
}
