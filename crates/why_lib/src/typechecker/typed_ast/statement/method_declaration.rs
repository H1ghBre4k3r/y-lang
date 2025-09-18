//! # Method Declaration Type Checking: Interface Definition without Implementation
//!
//! Method declarations in Y define method signatures within instance blocks
//! without providing implementations, similar to abstract methods or trait
//! declarations. This design enables interface-oriented programming:
//!
//! - Method signatures establish contracts for external implementations
//! - Type checking validates parameter and return type annotations
//! - Forward method references enable complex method interaction patterns
//! - LLVM can generate efficient vtables for polymorphic dispatch when needed
//!
//! The separation between declaration and implementation enables incremental
//! development where interfaces are established before implementation details.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::{Id, MethodDeclaration, TypeName},
    typechecker::{
        context::Context, error::RedefinedFunction, types::Type, ShallowCheck, TypeCheckError,
        TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for MethodDeclaration<()> {
    type Typed = MethodDeclaration<TypeInformation>;

    /// Method declaration type checking validates signatures without requiring implementations.
    ///
    /// This approach enables interface-first development where method contracts
    /// are established before implementation details. The type validation ensures
    /// that all referenced types exist and parameter patterns are well-formed.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();

        // Step 1: Generate the method's function type from its signature
        // Method declarations define function signatures without implementations
        let type_id = self.simple_shallow_check(ctx)?;

        let MethodDeclaration {
            id,
            parameter_types,
            return_type,
            position,
            ..
        } = self;

        let Id {
            name,
            position: id_position,
            ..
        } = id;

        // Step 2: Create the typed identifier with the method's function type
        // The method identifier holds the complete function type information
        let id = Id {
            name,
            position: id_position,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(type_id))),
                context: context.clone(),
            },
        };

        // Step 3: Return the typed method declaration with void type for the statement itself
        // Method declarations are statements and don't yield values
        Ok(MethodDeclaration {
            id,
            parameter_types,
            return_type,
            position,
            info: TypeInformation {
                // Method declarations always have Void type as they are statements
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let MethodDeclaration {
            id,
            parameter_types,
            return_type,
            position,
            ..
        } = this;

        MethodDeclaration {
            id: TypeCheckable::revert(id),
            parameter_types: parameter_types.clone(),
            return_type: return_type.clone(),
            position: position.clone(),
            info: (),
        }
    }
}

impl MethodDeclaration<()> {
    /// Generate function type from method signature for early validation.
    ///
    /// This helper enables both shallow checking and full type checking to share
    /// the same signature validation logic, ensuring consistency between the
    /// two phases and reducing code duplication.
    pub fn simple_shallow_check(&self, ctx: &Context) -> TypeResult<Type> {
        let MethodDeclaration {
            parameter_types,
            return_type,
            position,
            ..
        } = self;

        // Construct a function type from the method's parameter and return type annotations
        // This creates the complete type signature for the method declaration
        let function_type = TypeName::Fn {
            params: parameter_types.clone(),
            return_type: Box::new(return_type.clone()),
            position: position.clone(),
        };

        // Parse the function type to ensure all referenced types are valid
        Type::try_from((&function_type, ctx))
    }
}

impl ShallowCheck for MethodDeclaration<()> {
    /// Shallow checking registers method signatures before implementation validation.
    ///
    /// This enables methods to reference each other without forward declaration
    /// ordering constraints. Early signature registration supports complex method
    /// interaction patterns while catching type annotation errors efficiently.
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let MethodDeclaration { id, position, .. } = self;

        let type_id = self.simple_shallow_check(&*ctx)?;

        if ctx.scope.add_constant(&id.name, type_id).is_err() {
            return Err(TypeCheckError::RedefinedFunction(
                RedefinedFunction {
                    function_name: id.name.clone(),
                },
                position.clone(),
            ));
        };
        Ok(())
    }
}

impl TypedConstruct for MethodDeclaration<TypeInformation> {
    type Validated = MethodDeclaration<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let MethodDeclaration {
            id,
            parameter_types,
            return_type,
            info,
            position,
        } = self;

        Ok(MethodDeclaration {
            id: id.validate()?,
            parameter_types,
            return_type,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Id, MethodDeclaration, TypeName},
        typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation},
    };

    #[test]
    fn test_simple_method_declaration() -> anyhow::Result<()> {
        let mut ctx = Context::default();

        let dec = MethodDeclaration {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            parameter_types: vec![],
            return_type: TypeName::Literal("void".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(
            dec,
            MethodDeclaration {
                id: Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Void)
                        }))),
                        context: ctx.clone()
                    },
                    position: Span::default(),
                },
                parameter_types: vec![],
                return_type: TypeName::Literal("void".into(), Span::default()),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void))),
                    context: ctx.clone()
                },
                position: Span::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_complex_method_declaration() -> anyhow::Result<()> {
        let mut ctx = Context::default();

        let dec = MethodDeclaration {
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
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(
            dec,
            MethodDeclaration {
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
            }
        );

        Ok(())
    }
}
