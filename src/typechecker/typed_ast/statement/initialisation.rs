use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Id, Initialisation},
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
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

        let mut value = value.check(ctx)?;

        let mut info = value.get_info();

        // check for annotated type
        if let Some(type_name) = type_name.clone() {
            // is it actually a valid type?
            if let Ok(type_id) = Type::try_from((type_name, ctx.borrow())) {
                // check of type of associated expression
                let inner = info.type_id.clone();
                let inner = inner.borrow_mut();

                match inner.as_ref() {
                    // we have a type...
                    Some(inner_type) => {
                        // check, if they are equal
                        if type_id != *inner_type {
                            return Err(TypeCheckError::TypeMismatch(TypeMismatch {
                                expected: type_id,
                                actual: inner_type.clone(),
                            }));
                        }
                    }
                    // oups - no value of associated expression
                    None => {
                        // update type of underlying expression
                        value.update_type(type_id.clone());

                        // ...and the type of enclosed in the information
                        info.type_id = Rc::new(RefCell::new(Some(type_id)));
                    }
                }
            } else if info.type_id.borrow_mut().is_none() {
                todo!()
            }
        }

        let type_id = info.type_id.clone();

        ctx.scope.add_variable(&name, value.clone());

        Ok(Initialisation {
            id: Id { name, info },
            mutable,
            type_name,
            value,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
            },
        })
    }
}

impl TypedConstruct for Initialisation<TypeInformation> {}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
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
                    type_id: Rc::new(RefCell::new(Some(Type::Integer)))
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
                type_id: Rc::new(RefCell::new(Some(Type::Void)))
            }
        );
        assert_eq!(
            init.id.info,
            TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer)))
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
            Err(TypeCheckError::TypeMismatch(TypeMismatch {
                expected: Type::FloatingPoint,
                actual: Type::Integer
            }))
        );
    }

    #[test]
    fn test_correct_type_propagation() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
            },
            mutable: false,
            type_name: None,
            value: Expression::Lambda(Lambda {
                parameters: vec![],
                expression: Box::new(Expression::Num(Num::Integer(42, ()))),
                info: (),
            }),
            info: (),
        };

        let init = init.check(&mut ctx)?;

        assert_eq!(
            init,
            Initialisation {
                id: Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(None))
                    },
                },
                mutable: false,
                type_name: None,
                value: Expression::Lambda(Lambda {
                    parameters: vec![],
                    expression: Box::new(Expression::Num(Num::Integer(
                        42,
                        TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer)))
                        }
                    ))),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(None))
                    },
                }),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void)))
                },
            }
        );

        let Some(type_id) = ctx.scope.get_variable("foo") else {
            unreachable!()
        };

        assert_eq!(type_id, Rc::new(RefCell::new(None)));

        ctx.scope.update_variable(
            "foo",
            Type::Function {
                params: vec![],
                return_value: Box::new(Type::Integer),
            },
        );

        assert_eq!(
            init,
            Initialisation {
                id: Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Integer),
                        })))
                    },
                },
                mutable: false,
                type_name: None,
                value: Expression::Lambda(Lambda {
                    parameters: vec![],
                    expression: Box::new(Expression::Num(Num::Integer(
                        42,
                        TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer)))
                        }
                    ))),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Integer),
                        })))
                    },
                }),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void)))
                },
            }
        );

        Ok(())
    }
}
