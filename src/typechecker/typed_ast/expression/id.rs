use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::Id,
    typechecker::{
        context::Context,
        error::{TypeCheckError, UndefinedVariable},
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Id<()> {
    type Output = Id<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let Id { name, .. } = self;

        let constant = ctx.scope.get_constant(&name);
        let variable = ctx.scope.get_variable(&name);

        if constant.is_some() && variable.is_some() {
            todo!("same identifier is defined as variable and as constant")
        }

        let type_id = if let Some(type_id) = constant {
            Rc::new(RefCell::new(Some(type_id)))
        } else if let Some(type_id) = variable {
            type_id
        } else {
            return Err(TypeCheckError::UndefinedVariable(UndefinedVariable {
                variable_name: name,
            }));
        };

        Ok(Id {
            name,
            info: TypeInformation {
                type_id,
                context: ctx.clone(),
            },
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let Id { name, .. } = this;

        Id {
            name: name.to_owned(),
            info: (),
        }
    }
}

impl TypedConstruct for Id<TypeInformation> {}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        parser::ast::{Expression, Id},
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
        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: Context::default(),
                },
            }),
        );

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
        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: Context::default(),
                },
            }),
        );

        let id = Id {
            name: "foo".into(),
            info: (),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(
            id.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
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

    #[test]
    fn test_retrival_of_constant() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope.add_constant("foo", Type::Integer)?;

        let id = Id {
            name: "foo".into(),
            info: (),
        };

        let id = id.check(&mut ctx)?;

        assert_eq!(id.info.type_id, Rc::new(RefCell::new(Some(Type::Integer))));

        Ok(())
    }
}
