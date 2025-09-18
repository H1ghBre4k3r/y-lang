//! # Constant Declaration Type Checking: Explicit Types for Clarity
//!
//! Constant declarations in Y require explicit type annotations to eliminate
//! ambiguity and support compile-time evaluation. This design enforces clarity
//! over convenience because constants are often used in performance-critical
//! contexts where type precision matters:
//!
//! - Explicit types prevent hidden type conversions in constant expressions
//! - LLVM can embed constants directly in machine code with known types
//! - Type annotations serve as documentation for API consumers
//! - Compiler can validate constant initialization without inference complexity
//!
//! The two-phase checking (shallow then full) enables forward references while
//! maintaining dependency ordering for type resolution.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Constant, Id},
    typechecker::{
        context::Context,
        error::{InvalidConstantType, RedefinedConstant, TypeCheckError, TypeMismatch},
        types::Type,
        ShallowCheck, TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Constant<()> {
    type Typed = Constant<TypeInformation>;

    /// Constant type checking validates explicit annotations against inferred types.
    ///
    /// This approach catches type errors early while allowing the value expression
    /// to benefit from type inference. The explicit annotation requirement prevents
    /// accidental type changes when constant values are modified during development.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Constant {
            id,
            type_name,
            value,
            position: const_position,
            ..
        } = self;

        let context = ctx.clone();

        let Id {
            name,
            position: id_position,
            ..
        } = id;

        // Step 1: Type check the constant's value expression
        // Constants must have their values fully resolved at compile time
        let mut value = value.check(ctx)?;

        let info = value.get_info();

        // Step 2: Parse and validate the explicit type annotation
        // Constants require explicit type annotations to ensure clarity and prevent ambiguity
        let Ok(type_id) = Type::try_from((&type_name, &*ctx)) else {
            // Type annotation is invalid or references an undefined type
            return Err(TypeCheckError::InvalidConstantType(
                InvalidConstantType {
                    constant_name: name,
                },
                type_name.position(),
            ));
        };

        // Step 3: Verify type compatibility between annotation and value
        // The value's inferred type must match the explicitly declared type
        {
            let inner = info.type_id.clone();
            let mut inner = inner.borrow_mut();

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
                    *inner = Some(type_id.clone());
                }
            }
        }

        Ok(Constant {
            id: Id {
                name,
                info,
                position: id_position,
            },
            type_name,
            value,
            info: TypeInformation {
                // Constant declarations always have Void type as statements
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position: const_position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Constant {
            id,
            type_name,
            value,
            position,
            ..
        } = this;

        Constant {
            id: TypeCheckable::revert(id),
            type_name: type_name.clone(),
            value: TypeCheckable::revert(value),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Constant<TypeInformation> {
    type Validated = Constant<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Constant {
            id,
            type_name,
            value,
            info,
            position,
        } = self;

        Ok(Constant {
            id: id.validate()?,
            type_name,
            value: value.validate()?,
            info: info.validate(&position)?,
            position,
        })
    }
}

impl ShallowCheck for Constant<()> {
    /// Shallow checking establishes constant names before validating values.
    ///
    /// This two-phase approach enables constants to reference each other without
    /// complex dependency ordering. The type annotation is validated early to
    /// catch invalid type references before expensive value type checking occurs.
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let Constant { id, type_name, .. } = self;

        let name = id.name.clone();

        // Step 1: Parse the type annotation and validate it exists in the type scope
        // Shallow check ensures type names are valid before full type checking begins
        let Ok(type_id) = Type::try_from((type_name, &*ctx)) else {
            // Type annotation references an undefined or invalid type
            return Err(TypeCheckError::InvalidConstantType(
                InvalidConstantType {
                    constant_name: name,
                },
                type_name.position(),
            ));
        };

        // Step 2: Register the constant in the scope to make it available for later references
        // Constants must have unique names within their scope - redefinition is an error
        if ctx.scope.add_constant(&name, type_id).is_err() {
            // Constant with this name already exists in the current scope
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: name,
                },
                id.position.clone(),
            ));
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Constant, Expression, Id, Num, TypeName},
        typechecker::{
            context::Context,
            error::{InvalidConstantType, TypeCheckError},
            types::Type,
            ShallowCheck, TypeCheckable,
        },
    };

    #[test]
    fn test_constant_simple() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let constant = Constant {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into(), Span::default()),
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        constant.shallow_check(&mut ctx)?;
        let constant = constant.check(&mut ctx)?;

        assert_eq!(
            constant.id.info.type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        assert_eq!(
            ctx.scope.resolve_name("foo"),
            Some(Rc::new(RefCell::new(Some(Type::Integer))))
        );

        Ok(())
    }

    #[test]
    fn test_error_on_missing_type_annotation() {
        let mut ctx = Context::default();

        let constant = Constant {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("".into(), Span::default()),
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let result = constant.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::InvalidConstantType(
                InvalidConstantType {
                    constant_name: "foo".into()
                },
                Span::default()
            ))
        );
    }

    #[test]
    fn test_error_on_redefinition() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope.add_constant("foo", Type::Integer)?;

        let constant = Constant {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("".into(), Span::default()),
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let result = constant.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::InvalidConstantType(
                InvalidConstantType {
                    constant_name: "foo".into()
                },
                Span::default()
            ))
        );

        Ok(())
    }
}
