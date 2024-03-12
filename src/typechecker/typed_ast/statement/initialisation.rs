use std::borrow::Borrow;

use crate::{
    parser::ast::{Id, Initialisation},
    typechecker::{
        context::Context, error::TypeError, types::Type, TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for Initialisation<()> {
    type Output = Initialisation<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let Initialisation {
            id,
            mutable,
            type_name,
            value,
            ..
        } = self;

        let name = id.name;

        let value = value.check(ctx)?;

        let info = value.get_info();

        if let Some(type_name) = type_name.clone() {
            if let Ok(type_id) = Type::try_from((type_name, ctx.borrow())) {
                if type_id != info.type_id {
                    return Err(TypeError {
                        expected: type_id,
                        actual: info.type_id,
                    });
                }
            }
        }

        ctx.scope.add_variable(&name, info.type_id.clone());

        Ok(Initialisation {
            id: Id {
                name,
                info: info.clone(),
            },
            mutable,
            type_name,
            value,
            info,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        parser::ast::{Expression, Id, Initialisation, Num, TypeName},
        typechecker::{
            context::Context, error::TypeError, types::Type, TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_not_manipulation_of_fields() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, ())),
            info: (),
        }
        .check(&mut ctx)?;

        assert_eq!(init.id.name, "foo".to_string());
        assert!(!init.mutable);
        assert!(init.type_name.is_none());
        assert_eq!(
            init.value,
            Expression::Num(Num::Integer(
                42,
                TypeInformation {
                    type_id: Type::Integer
                }
            ))
        );

        Ok(())
    }

    #[test]
    fn test_add_variable() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, ())),
            info: (),
        };

        init.check(&mut ctx)?;

        let var = ctx.scope.get_variable("foo");

        assert_eq!(var, Some(Type::Integer));

        Ok(())
    }

    #[test]
    fn test_correct_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, ())),
            info: (),
        };

        let init = init.check(&mut ctx)?;

        assert_eq!(
            init.info,
            TypeInformation {
                type_id: Type::Integer
            }
        );
        assert_eq!(
            init.id.info,
            TypeInformation {
                type_id: Type::Integer
            }
        );

        Ok(())
    }

    #[test]
    fn test_type_mismatch() {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
            },
            mutable: false,
            type_name: Some(TypeName::Literal("f64".into())),
            value: Expression::Num(Num::Integer(42, ())),
            info: (),
        };

        let init = init.check(&mut ctx);
        assert_eq!(
            init,
            Err(TypeError {
                expected: Type::FloatingPoint,
                actual: Type::Integer
            })
        );
    }
}
