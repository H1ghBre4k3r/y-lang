//! # Instance Block Type Checking: Method Extension Mechanism
//!
//! Instance blocks in Y provide a mechanism for extending existing types with
//! methods, similar to Rust's impl blocks. This design enables modular code
//! organization while maintaining type safety and performance:
//!
//! - Methods are resolved at compile time for zero-cost dispatch
//! - The "this" variable provides explicit access to the instance being operated on
//! - Instance blocks can extend both built-in types and user-defined structs
//! - Method resolution follows lexical scoping to prevent ambiguous dispatch
//!
//! The scope isolation ensures that method implementations don't interfere with
//! each other while the explicit "this" binding maintains clarity about method context.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
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
    type Typed = Instance<TypeInformation>;

    /// Instance type checking creates method scopes with automatic "this" binding.
    ///
    /// The "this" variable injection enables methods to access instance data without
    /// explicit parameters, following object-oriented conventions. Scope isolation
    /// prevents method parameter conflicts while maintaining type safety.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();

        let Instance {
            name,
            functions,
            position,
            declarations,
            ..
        } = self;

        // Step 1: Resolve the type that this instance block is implementing methods for
        // Instance blocks add methods to existing types (built-in or user-defined structs)
        let type_id = Type::try_from((&name, &*ctx))?;

        // Step 2: Enter a new scope for method implementations with "this" variable
        // Methods have access to a "this" variable representing the instance of the type
        ctx.scope.enter_scope();
        if ctx.scope.add_constant("this", type_id).is_err() {
            // Failed to add "this" variable - scope collision (shouldn't happen in practice)
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: "this".into(),
                },
                position,
            ));
        };

        // Step 3: Type check all function implementations within the instance scope
        // Each function becomes a method available on instances of this type
        let mut checked_functions = vec![];

        for function in functions.into_iter() {
            checked_functions.push(function.check(ctx)?);
        }

        // Step 4: Type check all method declarations within the instance scope
        // Method declarations define method signatures without implementations
        let mut checked_declarations = vec![];

        for declaration in declarations.into_iter() {
            checked_declarations.push(declaration.check(ctx)?);
        }

        // Step 5: Exit the instance scope to return to the outer context
        // The "this" variable is no longer accessible outside the instance block
        ctx.scope.exit_scope();

        // Step 6: Return the typed instance block with void type
        // Instance blocks are statements and don't yield values
        Ok(Instance {
            name,
            functions: checked_functions,
            declarations: checked_declarations,
            info: TypeInformation {
                // Instance blocks always have Void type as they are statements
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Instance {
            name,
            functions,
            position,
            declarations,
            ..
        } = this;

        Instance {
            name: name.clone(),
            functions: functions.iter().map(TypeCheckable::revert).collect(),
            declarations: declarations.iter().map(TypeCheckable::revert).collect(),
            info: (),
            position: position.clone(),
        }
    }
}

impl ShallowCheck for Instance<()> {
    /// Shallow checking registers all methods before type checking their bodies.
    ///
    /// This two-phase approach enables methods to call each other recursively
    /// without forward declaration requirements. Method signature registration
    /// occurs before implementation checking to support complex method interactions.
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let Instance {
            name,
            functions,
            declarations,
            ..
        } = self;

        // Step 1: Resolve the type that this instance block will implement methods for
        // Instance blocks can only be defined for existing types
        let type_id = Type::try_from((name, &*ctx))?;

        // Step 2: Register all function implementations as methods on the target type
        // Each function becomes available as a method when called on instances of the type
        for function in functions.iter() {
            // Get the function's type signature for method registration
            let function_type = function.simple_shallow_check(ctx)?;
            if ctx
                .scope
                .add_method_to_type(type_id.clone(), &function.id.name, function_type)
                .is_err()
            {
                // Method with this name already exists for this type
                return Err(TypeCheckError::RedefinedMethod(
                    RedefinedMethod {
                        type_id,
                        function_name: function.id.name.clone(),
                    },
                    function.position.clone(),
                ));
            };
        }

        // Step 3: Register all method declarations as available methods on the target type
        // Method declarations define method signatures without implementations
        for declaration in declarations.iter() {
            // Get the declaration's type signature for method registration
            let declaration_type = declaration.simple_shallow_check(ctx)?;
            if ctx
                .scope
                .add_method_to_type(type_id.clone(), &declaration.id.name, declaration_type)
                .is_err()
            {
                // Method with this name already exists for this type
                return Err(TypeCheckError::RedefinedMethod(
                    RedefinedMethod {
                        type_id,
                        function_name: declaration.id.name.clone(),
                    },
                    declaration.position.clone(),
                ));
            }
        }

        Ok(())
    }
}

impl TypedConstruct for Instance<TypeInformation> {
    type Validated = Instance<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Instance {
            name,
            functions,
            declarations,
            info,
            position,
        } = self;
        let mut validated_functions = vec![];
        for f in functions {
            validated_functions.push(f.validate()?);
        }

        let mut validated_declarations = vec![];
        for d in declarations {
            validated_declarations.push(d.validate()?);
        }

        Ok(Instance {
            name,
            functions: validated_functions,
            declarations: validated_declarations,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{
            Block, Expression, Function, Id, Instance, MethodDeclaration, Postfix, Statement,
            TypeName,
        },
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
            declarations: vec![],
            info: (),
            position: Span::default(),
        };

        let result = inst.check(&mut ctx)?;

        assert_eq!(
            result,
            Instance {
                name: TypeName::Literal("i64".into(), Span::default()),
                functions: vec![],
                declarations: vec![],
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
            declarations: vec![],
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
            declarations: vec![],
            info: (),
            position: Span::default(),
        };

        let result = inst.check(&mut ctx)?;

        assert_eq!(
            result,
            Instance {
                name: TypeName::Literal("Foo".into(), Span::default()),
                functions: vec![],
                declarations: vec![],
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
                body: Block {
                    statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                        name: "this".into(),
                        info: (),
                        position: Span::default(),
                    }))],
                    info: (),
                    position: Span::default(),
                },
                info: (),
                position: Span::default(),
            }],
            declarations: vec![],
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
                    body: Block {
                        statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                            name: "this".into(),
                            info: TypeInformation {
                                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                                context: Context::default()
                            },
                            position: Span::default(),
                        }))],
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: Context::default()
                        },
                        position: Span::default(),
                    },
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Integer)
                        }))),
                        context: Context::default()
                    },
                    position: Span::default(),
                }],
                declarations: vec![],
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
                body: Block {
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
                },
                info: (),
                position: Span::default(),
            }],
            declarations: vec![],
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
                    body: Block {
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
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: Context::default(),
                        },
                        position: Span::default(),
                    },
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Integer),
                        }))),
                        context: Context::default(),
                    },
                    position: Span::default(),
                }],
                declarations: vec![],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void))),
                    context: Context::default(),
                },
                position: Span::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_complex_instance() -> anyhow::Result<()> {
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
                body: Block {
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
                },
                info: (),
                position: Span::default(),
            }],
            declarations: vec![MethodDeclaration {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default(),
                },
                parameter_types: vec![
                    TypeName::Literal("i64".into(), Span::default()),
                    TypeName::Tuple(
                        vec![
                            TypeName::Literal("i64".into(), Span::default()),
                            TypeName::Literal("f64".into(), Span::default()),
                        ],
                        Span::default(),
                    ),
                ],
                return_type: TypeName::Literal("i64".into(), Span::default()),
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
                    body: Block {
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
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: Context::default(),
                        },
                        position: Span::default(),
                    },
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Integer),
                        }))),
                        context: Context::default(),
                    },
                    position: Span::default(),
                }],
                declarations: vec![MethodDeclaration {
                    id: Id {
                        name: "foo".into(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Function {
                                params: vec![
                                    Type::Integer,
                                    Type::Tuple(vec![Type::Integer, Type::FloatingPoint])
                                ],
                                return_value: Box::new(Type::Integer)
                            }))),
                            context: ctx.clone()
                        },
                        position: Span::default(),
                    },
                    parameter_types: vec![
                        TypeName::Literal("i64".into(), Span::default()),
                        TypeName::Tuple(
                            vec![
                                TypeName::Literal("i64".into(), Span::default()),
                                TypeName::Literal("f64".into(), Span::default()),
                            ],
                            Span::default(),
                        ),
                    ],
                    return_type: TypeName::Literal("i64".into(), Span::default()),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Void))),
                        context: ctx.clone()
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
