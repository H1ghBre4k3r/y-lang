use crate::parser::ast::AstString;
use crate::typechecker::context::Context;
use crate::typechecker::types::Type;
use crate::typechecker::{TypeCheckable, TypeInformation, TypeResult};
use std::cell::RefCell;
use std::rc::Rc;

impl TypeCheckable for AstString<()> {
    type Output = AstString<TypeInformation>;
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let AstString {
            value, position, ..
        } = self;

        Ok(AstString {
            value,
            position,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::String))),
                context: ctx.clone(),
            },
        })
    }

    fn revert(this: &Self::Output) -> Self {
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
