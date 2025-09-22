use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::Num,
    typechecker::{
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct, context::Context, types::Type,
    },
};

impl TypeCheckable for Num<()> {
    type Typed = Num<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        match self {
            Num::Integer(val, _, position) => Ok(Num::Integer(
                val,
                TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: ctx.clone(),
                },
                position,
            )),
            Num::FloatingPoint(val, _, position) => Ok(Num::FloatingPoint(
                val,
                TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::FloatingPoint))),
                    context: ctx.clone(),
                },
                position,
            )),
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            Num::Integer(val, _, pos) => Num::Integer(*val, (), pos.clone()),
            Num::FloatingPoint(val, _, pos) => Num::FloatingPoint(*val, (), pos.clone()),
        }
    }
}

impl TypedConstruct for Num<TypeInformation> {
    type Validated = Num<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            Num::Integer(val, info, position) => {
                Ok(Num::Integer(val, info.validate(&position)?, position))
            }
            Num::FloatingPoint(val, info, position) => {
                Ok(Num::FloatingPoint(val, info.validate(&position)?, position))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::Num,
        typechecker::{TypeCheckable, context::Context, types::Type},
    };

    #[test]
    fn test_check_integer() -> Result<(), Box<dyn Error>> {
        let Num::Integer(num, info, ..) =
            Num::Integer(42, (), Span::default()).check(&mut Context::default())?
        else {
            unreachable!()
        };

        assert_eq!(num, 42);
        assert_eq!(info.type_id, Rc::new(RefCell::new(Some(Type::Integer))));
        Ok(())
    }

    #[test]
    fn test_check_floatingpoint() -> Result<(), Box<dyn Error>> {
        let Num::FloatingPoint(num, info, ..) =
            Num::FloatingPoint(42.0, (), Span::default()).check(&mut Context::default())?
        else {
            unreachable!()
        };

        assert_eq!(num, 42.0);
        assert_eq!(
            info.type_id,
            Rc::new(RefCell::new(Some(Type::FloatingPoint)))
        );
        Ok(())
    }
}
