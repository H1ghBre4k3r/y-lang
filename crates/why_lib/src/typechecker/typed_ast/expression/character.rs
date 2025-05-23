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
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Character {
            character,
            position,
            ..
        } = self;

        Ok(Character {
            character,
            position,
            info: TypeInformation {
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
