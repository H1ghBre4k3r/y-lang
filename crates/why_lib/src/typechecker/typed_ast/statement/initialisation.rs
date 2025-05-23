use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Id, Initialisation},
    typechecker::{
        context::Context,
        error::{
            MissingInitialisationType, RedefinedConstant, TypeCheckError, TypeMismatch,
            UndefinedType,
        },
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Initialisation<()> {
    type Typed = Initialisation<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Initialisation {
            id,
            mutable,
            type_name,
            value,
            position: init_position,
            ..
        } = self;

        let context = ctx.clone();

        let Id {
            name,
            position: id_position,
            ..
        } = id;

        let mut value = value.check(ctx)?;

        let info = value.get_info();

        // check for annotated type
        if let Some(type_name) = type_name.clone() {
            // is it actually a valid type?
            if let Ok(type_id) = Type::try_from((&type_name, &*ctx)) {
                // check of type of associated expression
                let inner = info.type_id.clone();
                let inner = inner.borrow_mut().clone();

                match inner.as_ref() {
                    // we have a type...
                    Some(inner_type) => {
                        // check, if they are equal
                        if type_id != *inner_type {
                            return Err(TypeCheckError::TypeMismatch(
                                TypeMismatch {
                                    expected: type_id,
                                    actual: inner_type.clone(),
                                },
                                value.position(),
                            ));
                        }
                    }
                    // oups - no value of associated expression
                    None => {
                        // update type of underlying expression
                        value.update_type(type_id.clone())?;

                        // ...and the type of enclosed in the information
                        *info.type_id.borrow_mut() = Some(type_id);
                    }
                }
            } else {
                let position = type_name.position();
                return Err(TypeCheckError::UndefinedType(
                    UndefinedType { type_name },
                    position,
                ));
            }
        } else if !info.has_type() {
            return Err(TypeCheckError::MissingInitialisationType(
                MissingInitialisationType,
                init_position,
            ));
        }

        if ctx
            .scope
            .add_variable(&name, value.clone(), mutable)
            .is_err()
        {
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: name.to_string(),
                },
                id_position,
            ));
        };

        Ok(Initialisation {
            id: Id {
                name,
                info,
                position: id_position,
            },
            mutable,
            type_name,
            value,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position: init_position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Initialisation {
            id,
            mutable,
            type_name,
            value,
            position,
            ..
        } = this;

        Initialisation {
            id: TypeCheckable::revert(id),
            mutable: *mutable,
            type_name: type_name.to_owned(),
            value: TypeCheckable::revert(value),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Initialisation<TypeInformation> {}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::typechecker::error::MissingInitialisationType;
    use crate::{
        lexer::Span,
        parser::ast::{Expression, Id, Initialisation, Lambda, Num, TypeName},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_not_manipulation_of_fields() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
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
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: Context::default(),
                },
                Span::default()
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
                position: Span::default(),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        init.check(&mut ctx)?;

        let var = ctx.scope.resolve_name("foo");

        assert_eq!(var, Some(Rc::new(RefCell::new(Some(Type::Integer)))));

        Ok(())
    }

    #[test]
    fn test_correct_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: None,
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let init = init.check(&mut ctx)?;

        assert_eq!(
            init.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context: Context::default(),
            }
        );
        assert_eq!(
            init.id.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
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
                position: Span::default(),
            },
            mutable: false,
            type_name: Some(TypeName::Literal("f64".into(), Span::default())),
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let init = init.check(&mut ctx);
        assert_eq!(
            init,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::FloatingPoint,
                    actual: Type::Integer
                },
                Span::default()
            ))
        );
    }

    #[test]
    fn test_error_on_missing_type() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: None,
            value: Expression::Lambda(Lambda {
                parameters: vec![],
                expression: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
                info: (),
                position: Span::default(),
            }),
            info: (),
            position: Span::default(),
        };

        let res = init.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::MissingInitialisationType(
                MissingInitialisationType,
                Span::default()
            ))
        );

        Ok(())
    }
}
