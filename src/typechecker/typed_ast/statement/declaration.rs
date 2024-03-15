use std::borrow::Borrow;

use crate::{
    parser::ast::{Declaration, Id},
    typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for Declaration<()> {
    type Output = Declaration<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let Declaration {
            name, type_name, ..
        } = self;

        let name = name.name;

        let Ok(type_id) = Type::try_from((type_name.clone(), ctx.borrow())) else {
            todo!()
        };

        ctx.scope.add_variable(&name, type_id.clone());

        Ok(Declaration {
            name: Id {
                name,
                info: TypeInformation {
                    type_id: type_id.clone(),
                },
            },
            type_name,
            info: TypeInformation { type_id },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        parser::ast::{Declaration, Id, TypeName},
        typechecker::{context::Context, types::Type, TypeCheckable},
    };

    #[test]
    fn test_no_field_manipulation() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let dec = Declaration {
            name: Id {
                name: "foo".into(),
                info: (),
            },
            type_name: TypeName::Literal("i64".into()),
            info: (),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(dec.name.name, "foo".to_string());
        assert_eq!(dec.type_name, TypeName::Literal("i64".into()));

        Ok(())
    }

    #[test]
    fn test_add_variable() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let dec = Declaration {
            name: Id {
                name: "foo".into(),
                info: (),
            },
            type_name: TypeName::Literal("i64".into()),
            info: (),
        };

        dec.check(&mut ctx)?;

        let var = ctx.scope.get_variable("foo");

        assert_eq!(var, Some(Type::Integer));

        Ok(())
    }

    #[test]
    fn test_correct_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let dec = Declaration {
            name: Id {
                name: "foo".into(),
                info: (),
            },
            type_name: TypeName::Literal("i64".into()),
            info: (),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(dec.name.info.type_id, Type::Integer);
        assert_eq!(dec.info.type_id, Type::Integer);

        Ok(())
    }
}
