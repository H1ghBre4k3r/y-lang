use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Declaration, Id},
    typechecker::{
        context::Context, types::Type, TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
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

        let type_id = Rc::new(RefCell::new(Some(type_id)));

        ctx.scope.add_variable(&name, type_id.clone());

        Ok(Declaration {
            name: Id {
                name,
                info: TypeInformation {
                    type_id: type_id.clone(),
                },
            },
            type_name,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
            },
        })
    }
}

impl TypedConstruct for Declaration<TypeInformation> {}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

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

        assert_eq!(var, Some(Rc::new(RefCell::new(Some(Type::Integer)))));

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

        assert_eq!(
            dec.name.info.type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );
        assert_eq!(dec.info.type_id, Rc::new(RefCell::new(Some(Type::Void))));

        Ok(())
    }
}
