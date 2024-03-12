use crate::{
    parser::ast::Num,
    typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for Num<()> {
    type Output = Num<TypeInformation>;

    fn check(self, _context: &mut Context) -> TypeResult<Self::Output> {
        match self {
            Num::Integer(val, _) => Ok(Num::Integer(
                val,
                TypeInformation {
                    type_id: Some(Type::Integer),
                },
            )),
            Num::FloatingPoint(val, _) => Ok(Num::FloatingPoint(
                val,
                TypeInformation {
                    type_id: Some(Type::Float),
                },
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

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
        assert_eq!(info.type_id, Some(Type::Integer));
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
        assert_eq!(info.type_id, Some(Type::Float));
        Ok(())
    }
}
