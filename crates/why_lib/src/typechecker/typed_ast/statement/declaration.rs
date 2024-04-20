use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Declaration, Id},
    typechecker::{
        context::Context,
        error::{RedefinedConstant, TypeCheckError, UndefinedType},
        types::Type,
        ShallowCheck, TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Declaration<()> {
    type Output = Declaration<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let Declaration {
            name,
            type_name,
            position: dec_position,
            ..
        } = self;
        let context = ctx.clone();

        let Id {
            name,
            position: id_position,
            ..
        } = name;

        let Ok(type_id) = Type::try_from((&type_name, &*ctx)) else {
            let position = type_name.position();
            return Err(TypeCheckError::UndefinedType(
                UndefinedType { type_name },
                position,
            ));
        };

        let type_id = Rc::new(RefCell::new(Some(type_id)));

        let id = Id {
            name,
            info: TypeInformation {
                type_id: type_id.clone(),
                context: context.clone(),
            },
            position: id_position,
        };

        Ok(Declaration {
            name: id,
            type_name,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position: dec_position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let Declaration {
            name,
            type_name,
            position,
            ..
        } = this;

        Declaration {
            name: TypeCheckable::revert(name),
            type_name: type_name.to_owned(),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Declaration<TypeInformation> {}

impl ShallowCheck for Declaration<()> {
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let Declaration {
            name, type_name, ..
        } = self;

        let Ok(type_id) = Type::try_from((type_name, &*ctx)) else {
            let position = type_name.position();
            return Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: type_name.clone(),
                },
                position,
            ));
        };

        if ctx.scope.add_constant(&name.name, type_id).is_err() {
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: name.name.clone(),
                },
                name.position.clone(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Declaration, Id, TypeName},
        typechecker::{context::Context, types::Type, ShallowCheck, TypeCheckable},
    };

    #[test]
    fn test_no_field_manipulation() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let dec = Declaration {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(dec.name.name, "foo".to_string());
        assert_eq!(
            dec.type_name,
            TypeName::Literal("i64".into(), Span::default())
        );

        Ok(())
    }

    #[test]
    fn test_add_variable() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let dec = Declaration {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        dec.shallow_check(&mut ctx)?;

        let var = ctx.scope.resolve_name("foo");

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
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
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
