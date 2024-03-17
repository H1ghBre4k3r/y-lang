use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::Num,
    typechecker::{
        context::Context, types::Type, TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Num<()> {
    type Output = Num<TypeInformation>;

    fn check(self, _context: &mut Context) -> TypeResult<Self::Output> {
        match self {
            Num::Integer(val, _) => Ok(Num::Integer(
                val,
                TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                },
            )),
            Num::FloatingPoint(val, _) => Ok(Num::FloatingPoint(
                val,
                TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::FloatingPoint))),
                },
            )),
        }
    }
}

impl TypedConstruct for Num<TypeInformation> {}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        parser::ast::Num,
        typechecker::{context::Context, types::Type, TypeCheckable},
    };

    #[test]
    fn test_check_integer() -> Result<(), Box<dyn Error>> {
        let Num::Integer(num, info) = Num::Integer(42, ()).check(&mut Context::default())? else {
            unreachable!()
        };

        assert_eq!(num, 42);
        assert_eq!(info.type_id, Rc::new(RefCell::new(Some(Type::Integer))));
        Ok(())
    }

    #[test]
    fn test_check_floatingpoint() -> Result<(), Box<dyn Error>> {
        let Num::FloatingPoint(num, info) =
            Num::FloatingPoint(42.0, ()).check(&mut Context::default())?
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
