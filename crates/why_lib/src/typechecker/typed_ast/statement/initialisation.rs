//! # Variable Initialization Type Checking: Inference with Optional Annotations
//!
//! Variable initialization in Y balances type safety with developer convenience
//! through optional explicit type annotations. This hybrid approach enables both
//! rapid prototyping and precise type control:
//!
//! - Type inference reduces boilerplate for obvious cases (let x = 42)
//! - Optional annotations provide type precision when needed (let x: i64 = 42)
//! - Mutability annotations enable selective optimization strategies
//! - LLVM can optimize immutable variables more aggressively than mutable ones
//!
//! The flexible annotation system adapts to developer preferences while maintaining
//! the type safety guarantees that enable zero-cost abstractions.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Id, Initialisation},
    typechecker::{
        context::Context,
        error::{
            MissingInitialisationType, RedefinedConstant, TypeCheckError, TypeMismatch,
            UndefinedType,
        },
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Initialisation<()> {
    type Typed = Initialisation<TypeInformation>;

    /// Initialization type checking supports both inference and explicit annotations.
    ///
    /// This flexibility accommodates different coding styles while maintaining type
    /// safety. The value is type-checked first to establish a baseline, then the
    /// annotation (if present) provides additional constraints or clarification.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Initialisation {
            id,
            mutable,
            type_name,
            value,
            position: init_position,
            ..
        } = self;

        let context = ctx.clone();

        let Id {
            name,
            position: id_position,
            ..
        } = id;

        // Step 1: Type check the initialization value expression
        // The value's type will be used for type inference if no explicit annotation is provided
        let mut value = value.check(ctx)?;

        let info = value.get_info();

        // Step 2: Handle optional explicit type annotation
        // Variables can be initialized with or without explicit type annotations
        if let Some(type_name) = type_name.clone() {
            // Explicit type annotation provided - validate and enforce it
            if let Ok(type_id) = Type::try_from((&type_name, &*ctx)) {
                // Step 2a: Verify type compatibility between annotation and inferred value type
                let inner = info.type_id.clone();
                let inner = inner.borrow_mut().clone();

                match inner.as_ref() {
                    // Value has a concrete type - must match the declared type exactly
                    Some(inner_type) => {
                        if type_id != *inner_type {
                            // Type mismatch between declared type and inferred value type
                            return Err(TypeCheckError::TypeMismatch(
                                TypeMismatch {
                                    expected: type_id,
                                    actual: inner_type.clone(),
                                },
                                value.position(),
                            ));
                        }
                    }
                    // Value has unknown type - propagate the declared type to the value
                    None => {
                        // Update the value expression to have the declared type
                        value.update_type(type_id.clone())?;

                        // Update the value's type information to match the declaration
                        *info.type_id.borrow_mut() = Some(type_id);
                    }
                }
            } else {
                // Type annotation references an undefined or invalid type
                let position = type_name.position();
                return Err(TypeCheckError::UndefinedType(
                    UndefinedType { type_name },
                    position,
                ));
            }
        } else if !info.has_type() {
            // Step 2b: No type annotation and value type cannot be inferred
            // This typically happens with complex expressions like lambdas that need explicit types
            return Err(TypeCheckError::MissingInitialisationType(
                MissingInitialisationType,
                init_position,
            ));
        }

        // Step 3: Register the variable in the scope with its mutability setting
        // Variables must have unique names within their scope - redefinition is an error
        if ctx
            .scope
            .add_variable(&name, value.clone(), mutable)
            .is_err()
        {
            // Variable with this name already exists in the current scope
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: name.to_string(),
                },
                id_position,
            ));
        };

        // Step 4: Return the typed initialization with void type for the statement itself
        // Variable initializations are statements and don't yield values
        Ok(Initialisation {
            id: Id {
                name,
                info,
                position: id_position,
            },
            mutable,
            type_name,
            value,
            info: TypeInformation {
                // Initialization statements always have Void type as they don't yield values
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position: init_position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Initialisation {
            id,
            mutable,
            type_name,
            value,
            position,
            ..
        } = this;

        Initialisation {
            id: TypeCheckable::revert(id),
            mutable: *mutable,
            type_name: type_name.to_owned(),
            value: TypeCheckable::revert(value),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Initialisation<TypeInformation> {
    type Validated = Initialisation<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Initialisation {
            id,
            mutable,
            type_name,
            value,
            info,
            position,
        } = self;

        Ok(Initialisation {
            id: id.validate()?,
            mutable,
            type_name,
            value: value.validate()?,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::parser::ast::LambdaParameter;
    use crate::typechecker::error::MissingInitialisationType;
    use crate::{
        lexer::Span,
        parser::ast::{Expression, Id, Initialisation, Lambda, Num, TypeName},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_not_manipulation_of_fields() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        }
        .check(&mut ctx)?;

        assert_eq!(init.id.name, "foo".to_string());
        assert!(!init.mutable);
        assert!(init.type_name.is_none());
        assert_eq!(
            init.value,
            Expression::Num(Num::Integer(
                42,
                TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: Context::default(),
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_add_variable() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        init.check(&mut ctx)?;

        let var = ctx.scope.resolve_name("foo");

        assert_eq!(var, Some(Rc::new(RefCell::new(Some(Type::Integer)))));

        Ok(())
    }

    #[test]
    fn test_correct_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let init = init.check(&mut ctx)?;

        assert_eq!(
            init.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context: Context::default(),
            }
        );
        assert_eq!(
            init.id.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_type_mismatch() {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: Some(TypeName::Literal("f64".into(), Span::default())),
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let init = init.check(&mut ctx);
        assert_eq!(
            init,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::FloatingPoint,
                    actual: Type::Integer
                },
                Span::default()
            ))
        );
    }

    #[test]
    fn test_error_on_missing_type() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: None,
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

        let res = init.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::MissingInitialisationType(
                MissingInitialisationType,
                Span::default()
            ))
        );

        Ok(())
    }
}
