//! # Boolean Literal Type Checking: Compile-Time Type Certainty
//!
//! Boolean literals in Y demonstrate the language's preference for compile-time
//! type certainty over runtime flexibility. This design philosophy enables:
//!
//! - Zero-cost boolean operations with no runtime type checks
//! - LLVM can generate optimal conditional branch instructions
//! - Predictable memory layout (1 byte) for boolean values
//! - No boxing or dynamic dispatch overhead for primitive operations
//!
//! This strict typing prevents the ambiguity found in dynamic languages where
//! truthy/falsy values can lead to unexpected behavior and performance costs.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::Bool,
    typechecker::{
        context::Context, error::TypeCheckError, types::Type, TypeCheckable, TypeInformation,
        TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Bool<()> {
    type Typed = Bool<TypeInformation>;

    /// Boolean type checking is trivial because the type is always known at compile time.
    ///
    /// This simplicity is by design - Y avoids truthy/falsy semantics that require
    /// runtime type coercion, enabling LLVM to generate efficient conditional
    /// branches without any type checking overhead.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // Type checking for boolean literals is straightforward since the type is always known
        // Boolean values (true/false) have a fixed, concrete type that never changes
        // We assign the Boolean type immediately without any inference needed
        Ok(Bool {
            value: self.value,
            position: self.position,
            info: TypeInformation {
                // Boolean literals always have Boolean type - no ambiguity
                type_id: Rc::new(RefCell::new(Some(Type::Boolean))),
                context: ctx.clone(),
            },
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        Bool {
            value: this.value,
            position: this.position.clone(),
            info: (),
        }
    }
}

impl TypedConstruct for Bool<TypeInformation> {
    type Validated = Bool<ValidatedTypeInformation>;

    fn update_type(&mut self, _type_id: Type) -> Result<(), TypeCheckError> {
        // Boolean literals have a fixed type that cannot be changed
        // Since boolean values are always of type Boolean, attempting to update
        // their type to something else indicates a logic error in the type system
        // This should never be called in practice for boolean literals
        unreachable!()
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let type_id = self.info.type_id.borrow().clone().unwrap();
        Ok(Bool {
            value: self.value,
            position: self.position,
            info: ValidatedTypeInformation {
                type_id,
                context: self.info.context,
            },
        })
    }
}
