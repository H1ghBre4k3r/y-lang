use crate::{
    parser::ast::Id,
    typechecker::{
        context::Context,
        error::{TypeCheckError, UndefinedVariable},
        TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for Id<()> {
    type Output = Id<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let Id { name, .. } = self;

        let Some(type_id) = ctx.scope.get_variable(&name) else {
            return Err(TypeCheckError::UndefinedVariable(UndefinedVariable {
                variable_name: name,
            }));
        };

        Ok(Id {
            name,
            info: TypeInformation { type_id },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        parser::ast::Id,
        typechecker::{
            context::Context,
            error::{TypeCheckError, UndefinedVariable},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_no_member_modification() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope.add_variable("foo", Type::Integer);

        let id = Id {
            name: "foo".into(),
            info: (),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(id.name, "foo".to_string());

        Ok(())
    }

    #[test]
    fn test_correct_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope.add_variable("foo", Type::Integer);

        let id = Id {
            name: "foo".into(),
            info: (),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(
            id.info,
            TypeInformation {
                type_id: Type::Integer
            }
        );

        Ok(())
    }

    #[test]
    fn test_error_on_undefined() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let id = Id {
            name: "foo".into(),
            info: (),
        };

        let res = id.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::UndefinedVariable(UndefinedVariable {
                variable_name: "foo".into()
            }))
        );

        Ok(())
    }
}
