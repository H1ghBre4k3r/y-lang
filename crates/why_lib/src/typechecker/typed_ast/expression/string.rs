use crate::parser::ast::AstString;
use crate::typechecker::context::Context;
use crate::typechecker::types::Type;
use crate::typechecker::{
    TypeCheckable, TypeInformation, TypeResult, TypeValidationError, TypedConstruct,
    ValidatedTypeInformation,
};
use std::cell::RefCell;
use std::rc::Rc;

impl TypeCheckable for AstString<()> {
    type Typed = AstString<TypeInformation>;
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let AstString {
            value, position, ..
        } = self;

        // String literal type checking is straightforward - all string literals get String type
        // String literals ("hello", "world") have a deterministic type with no inference needed
        // Similar to numeric literals, the type is immediately known from the syntax
        Ok(AstString {
            value,
            position,
            info: TypeInformation {
                // Assign concrete String type immediately - no Unknown phase needed
                type_id: Rc::new(RefCell::new(Some(Type::String))),
                context: ctx.clone(),
            },
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let AstString {
            value, position, ..
        } = this;

        AstString {
            value: value.clone(),
            position: position.clone(),
            info: (),
        }
    }
}

impl TypedConstruct for AstString<TypeInformation> {
    type Validated = AstString<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let AstString {
            value,
            info,
            position,
        } = self;

        Ok(AstString {
            value,
            info: info.validate(&position)?,
            position,
        })
    }
}
