//! # Identifier Type Checking: Lexical Scope Resolution
//!
//! Identifier resolution in Y implements lexical scoping that enables predictable
//! variable lookup without the complexity of dynamic scoping. This design supports:
//!
//! - Compile-time name resolution for zero-cost variable access
//! - Nested scope support for functions, blocks, and closures
//! - Shared type references that allow bidirectional type inference
//! - Prevention of variable shadowing ambiguities
//!
//! The scope chain traversal here is optimized for the common case where variables
//! are found in the immediate scope, reducing lookup costs for local variables.

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::Id,
    typechecker::{
        context::Context,
        error::{TypeCheckError, UndefinedVariable},
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Id<()> {
    type Typed = Id<TypeInformation>;

    /// Identifier type lookup leverages shared type references for bidirectional inference.
    ///
    /// The shared RefCell approach enables type information to flow both from
    /// declaration sites to usage sites and vice versa, supporting Y's type
    /// inference without requiring complex constraint solving algorithms.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Id { name, position, .. } = self;

        // Identifier type checking involves variable/constant resolution in the scope chain
        // We traverse the scope hierarchy from innermost to outermost to find the binding
        // The scope returns a shared reference to the type information established at declaration
        let Some(type_id) = ctx.scope.resolve_name(&name) else {
            // Identifier not found in any accessible scope - this is an undefined variable error
            return Err(TypeCheckError::UndefinedVariable(
                UndefinedVariable {
                    variable_name: name,
                },
                position,
            ));
        };

        // Use the type information from the variable's declaration site
        // The type_id is a shared reference that may be updated during type inference
        Ok(Id {
            name,
            info: TypeInformation {
                type_id,
                context: ctx.clone(),
            },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Id { name, position, .. } = this;

        Id {
            name: name.to_owned(),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Id<TypeInformation> {
    type Validated = Id<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Id {
            name,
            info,
            position,
        } = self;

        Ok(Id {
            name,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Expression, Id},
        typechecker::{
            context::Context,
            error::{TypeCheckError, UndefinedVariable},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_no_member_modification() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope
            .add_variable(
                "foo",
                Expression::Id(Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: Context::default(),
                    },
                    position: Span::default(),
                }),
                false,
            )
            .expect("something went wrong");

        let id = Id {
            name: "foo".into(),
            info: (),
            position: Span::default(),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(id.name, "foo".to_string());

        Ok(())
    }

    #[test]
    fn test_correct_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope
            .add_variable(
                "foo",
                Expression::Id(Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: Context::default(),
                    },
                    position: Span::default(),
                }),
                false,
            )
            .expect("something went wrong");

        let id = Id {
            name: "foo".into(),
            info: (),
            position: Span::default(),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(
            id.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_error_on_undefined() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let id = Id {
            name: "foo".into(),
            info: (),
            position: Span::default(),
        };

        let res = id.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::UndefinedVariable(
                UndefinedVariable {
                    variable_name: "foo".into()
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_retrival_of_constant() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope.add_constant("foo", Type::Integer)?;

        let id = Id {
            name: "foo".into(),
            info: (),
            position: Span::default(),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(id.info.type_id, Rc::new(RefCell::new(Some(Type::Integer))));

        Ok(())
    }
}
