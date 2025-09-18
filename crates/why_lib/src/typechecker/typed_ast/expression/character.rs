//! # Character Literal Type Checking: Fixed-Size Primitive Design
//!
//! Character literals in Y represent a single Unicode scalar value with known
//! size and alignment properties. This design reflects Y's commitment to:
//!
//! - Predictable memory layout for character data (4 bytes for Unicode scalar)
//! - No string interning or dynamic allocation for single characters
//! - Direct mapping to LLVM's i32 representation for efficient operations
//! - Clear distinction between single characters and string literals
//!
//! Unlike languages that blur the line between characters and strings, Y maintains
//! this distinction to enable specialized optimizations for each use case.

use crate::parser::ast::Character;
use crate::typechecker::context::Context;
use crate::typechecker::types::Type;
use crate::typechecker::{
    TypeCheckable, TypeInformation, TypeResult, TypeValidationError, TypedConstruct,
    ValidatedTypeInformation,
};
use std::cell::RefCell;
use std::rc::Rc;

impl TypeCheckable for Character<()> {
    type Typed = Character<TypeInformation>;
    /// Character type assignment is immediate because Unicode scalar values have
    /// a fixed, known type representation.
    ///
    /// This deterministic typing enables LLVM to generate optimal code without
    /// runtime type checks or dynamic dispatch, supporting Y's zero-cost abstraction goals.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Character {
            character,
            position,
            ..
        } = self;

        // Type checking for character literals is simple since the type is always deterministic
        // Character values ('a', 'b', etc.) have a fixed Character type with no ambiguity
        // No type inference or validation is needed - we can assign the type immediately
        Ok(Character {
            character,
            position,
            info: TypeInformation {
                // All character literals get the Character type unconditionally
                type_id: Rc::new(RefCell::new(Some(Type::Character))),
                context: ctx.clone(),
            },
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Character {
            character,
            position,
            ..
        } = this;

        Character {
            character: *character,
            position: position.clone(),
            info: (),
        }
    }
}

impl TypedConstruct for Character<TypeInformation> {
    type Validated = Character<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Character {
            character,
            info,
            position,
        } = self;

        Ok(Character {
            character,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Span;

    #[test]
    fn test_character_type_check() {
        let mut ctx = Context::default();

        let character = Character {
            character: 'a',
            info: (),
            position: Span::default(),
        };

        let result = character.check(&mut ctx);

        assert!(result.is_ok());
        assert_eq!(
            result,
            Ok(Character {
                character: 'a',
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Character))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            })
        )
    }
}
