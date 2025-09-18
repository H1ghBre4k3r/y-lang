//! # Variable Declaration Type Checking: Forward Reference Support
//!
//! Variable declarations in Y establish type bindings without initialization,
//! enabling forward references and separation of concerns. This design supports
//! complex declaration patterns while maintaining type safety:
//!
//! - Forward declarations enable mutual recursion and complex data structures
//! - Explicit type annotations eliminate inference ambiguity in declaration context
//! - Two-phase checking allows circular references between declarations
//! - LLVM can allocate stack space with known types before initialization
//!
//! The separation between declaration and initialization enables developers to
//! establish variable contracts before providing implementations.

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Declaration, Id},
    typechecker::{
        context::Context,
        error::{RedefinedConstant, TypeCheckError, UndefinedType},
        types::Type,
        ShallowCheck, TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};
use std::{cell::RefCell, rc::Rc};

impl TypeCheckable for Declaration<()> {
    type Typed = Declaration<TypeInformation>;

    /// Declaration type checking validates type annotations without requiring values.
    ///
    /// This approach enables forward declarations and complex dependency patterns
    /// while ensuring type correctness. The explicit type requirement prevents
    /// inference ambiguity in contexts where variable purposes aren't immediately clear.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Declaration {
            name,
            type_name,
            position: dec_position,
            ..
        } = self;
        let context = ctx.clone();

        let Id {
            name,
            position: id_position,
            ..
        } = name;

        // Step 1: Parse and validate the type annotation
        // Declarations require explicit type annotations to define the variable's type
        let Ok(type_id) = Type::try_from((&type_name, &*ctx)) else {
            // Type annotation references an undefined or invalid type
            let position = type_name.position();
            return Err(TypeCheckError::UndefinedType(
                UndefinedType { type_name },
                position,
            ));
        };

        // Step 2: Create type information for the declared variable
        // The variable will have the explicitly declared type
        let type_id = Rc::new(RefCell::new(Some(type_id)));

        let id = Id {
            name,
            info: TypeInformation {
                type_id: type_id.clone(),
                context: context.clone(),
            },
            position: id_position,
        };

        // Step 3: Return the typed declaration with void type for the statement itself
        // Variable declarations are statements and don't yield values
        Ok(Declaration {
            name: id,
            type_name,
            info: TypeInformation {
                // Declaration statements always have Void type as they don't yield values
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position: dec_position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Declaration {
            name,
            type_name,
            position,
            ..
        } = this;

        Declaration {
            name: TypeCheckable::revert(name),
            type_name: type_name.to_owned(),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Declaration<TypeInformation> {
    type Validated = Declaration<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Declaration {
            name,
            type_name,
            info,
            position,
        } = self;

        Ok(Declaration {
            name: name.validate()?,
            type_name,
            info: info.validate(&position)?,
            position,
        })
    }
}

impl ShallowCheck for Declaration<()> {
    /// Shallow checking establishes declared names in scope before full validation.
    ///
    /// This enables complex declaration ordering where variables may reference
    /// each other's types. Early name registration prevents undefined variable
    /// errors during the full type checking phase for interdependent declarations.
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let Declaration {
            name, type_name, ..
        } = self;

        // Step 1: Parse the type annotation and validate it exists in the type scope
        // Shallow check ensures type names are resolved before full type checking
        let Ok(type_id) = Type::try_from((type_name, &*ctx)) else {
            // Type annotation references an undefined or invalid type
            let position = type_name.position();
            return Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: type_name.clone(),
                },
                position,
            ));
        };

        // Step 2: Register the declared variable in the scope for later references
        // Variable declarations must have unique names within their scope
        if ctx.scope.add_constant(&name.name, type_id).is_err() {
            // Variable with this name already exists in the current scope
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: name.name.clone(),
                },
                name.position.clone(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Declaration, Id, TypeName},
        typechecker::{context::Context, types::Type, ShallowCheck, TypeCheckable},
    };

    #[test]
    fn test_no_field_manipulation() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let dec = Declaration {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(dec.name.name, "foo".to_string());
        assert_eq!(
            dec.type_name,
            TypeName::Literal("i64".into(), Span::default())
        );

        Ok(())
    }

    #[test]
    fn test_add_variable() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let dec = Declaration {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        dec.shallow_check(&mut ctx)?;

        let var = ctx.scope.resolve_name("foo");

        assert_eq!(var, Some(Rc::new(RefCell::new(Some(Type::Integer)))));

        Ok(())
    }

    #[test]
    fn test_correct_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let dec = Declaration {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(
            dec.name.info.type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );
        assert_eq!(dec.info.type_id, Rc::new(RefCell::new(Some(Type::Void))));

        Ok(())
    }
}
