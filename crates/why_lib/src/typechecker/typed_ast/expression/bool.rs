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

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        Ok(Bool {
            value: self.value,
            position: self.position,
            info: TypeInformation {
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
        // Bool type is fixed, no need to update
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
