use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Id, StructDeclaration, StructFieldDeclaration},
    typechecker::{
        context::Context,
        error::{TypeCheckError, UndefinedType},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for StructDeclaration<()> {
    type Output = StructDeclaration<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let StructDeclaration {
            id,
            fields,
            position: struct_position,
            ..
        } = self;

        let context = ctx.clone();

        let Id {
            name,
            position: id_position,
            ..
        } = id;

        let mut checked_fields = vec![];

        for field in fields.into_iter() {
            checked_fields.push(field.check(ctx)?);
        }

        let struct_fields = checked_fields
            .iter()
            .map(
                |StructFieldDeclaration {
                     name: Id { name, .. },
                     info: TypeInformation { type_id, .. },
                     ..
                 }| {
                    let inner = type_id.borrow();
                    let inner = inner.as_ref().cloned();
                    (name.clone(), inner.expect("something went wrong"))
                },
            )
            .collect::<Vec<_>>();

        let type_id = Type::Struct(name.clone(), struct_fields);

        if let Err(e) = ctx.scope.add_type(&name, type_id) {
            eprintln!("{e}")
        };

        let info = TypeInformation {
            type_id: Rc::new(RefCell::new(Some(Type::Void))),
            context,
        };

        Ok(StructDeclaration {
            id: Id {
                name,
                info: info.clone(),
                position: id_position,
            },
            fields: checked_fields,
            info,
            position: struct_position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let StructDeclaration {
            id,
            fields,
            position,
            ..
        } = this;

        StructDeclaration {
            id: Id {
                name: id.name.clone(),
                info: (),
                position: id.position.clone(),
            },
            fields: fields.iter().map(TypeCheckable::revert).collect::<Vec<_>>(),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for StructDeclaration<TypeInformation> {}

impl TypeCheckable for StructFieldDeclaration<()> {
    type Output = StructFieldDeclaration<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let StructFieldDeclaration {
            name,
            type_name,
            position,
            ..
        } = self;

        let type_id = match Type::try_from((&type_name, &*ctx)) {
            Ok(type_id) => type_id,
            Err(_) => {
                return Err(TypeCheckError::UndefinedType(
                    UndefinedType { type_name },
                    position,
                ))
            }
        };

        let info = TypeInformation {
            type_id: Rc::new(RefCell::new(Some(type_id))),
            context: ctx.clone(),
        };

        Ok(StructFieldDeclaration {
            name: Id {
                name: name.name,
                info: info.clone(),
                position: name.position,
            },
            type_name,
            info,
            position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let StructFieldDeclaration {
            name,
            type_name,
            position,
            ..
        } = this;

        StructFieldDeclaration {
            name: Id {
                name: name.name.clone(),
                info: (),
                position: name.position.clone(),
            },
            type_name: type_name.clone(),
            info: (),
            position: position.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Id, StructDeclaration, StructFieldDeclaration, TypeName},
        typechecker::{context::Context, types::Type, TypeCheckable},
    };

    #[test]
    fn test_empty_struct_declaration() -> Result<()> {
        let mut ctx = Context::default();

        let dec = StructDeclaration {
            id: Id {
                name: "Foo".into(),
                info: (),
                position: Span::default(),
            },
            fields: vec![],
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(dec.info.type_id, Rc::new(RefCell::new(Some(Type::Void))));

        assert_eq!(
            ctx.scope.get_type("Foo"),
            Some(Type::Struct("Foo".into(), vec![]))
        );

        Ok(())
    }

    #[test]
    fn test_filled_struct_declaration() -> Result<()> {
        let mut ctx = Context::default();

        let dec = StructDeclaration {
            id: Id {
                name: "Foo".into(),
                info: (),
                position: Span::default(),
            },
            fields: vec![
                StructFieldDeclaration {
                    name: Id {
                        name: "bar".into(),
                        info: (),
                        position: Span::default(),
                    },
                    type_name: TypeName::Literal("i64".into(), Span::default()),
                    info: (),
                    position: Span::default(),
                },
                StructFieldDeclaration {
                    name: Id {
                        name: "baz".into(),
                        info: (),
                        position: Span::default(),
                    },
                    type_name: TypeName::Literal("f64".into(), Span::default()),
                    info: (),
                    position: Span::default(),
                },
            ],
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(dec.info.type_id, Rc::new(RefCell::new(Some(Type::Void))));

        assert_eq!(
            ctx.scope.get_type("Foo"),
            Some(Type::Struct(
                "Foo".into(),
                vec![
                    ("bar".into(), Type::Integer),
                    ("baz".into(), Type::FloatingPoint)
                ]
            ))
        );

        Ok(())
    }

    #[test]
    fn test_nested_struct() -> Result<()> {
        let mut ctx = Context::default();

        let dec = StructDeclaration {
            id: Id {
                name: "BarStruct".into(),
                info: (),
                position: Span::default(),
            },
            fields: vec![],
            info: (),
            position: Span::default(),
        };

        dec.check(&mut ctx)?;

        let dec = StructDeclaration {
            id: Id {
                name: "Foo".into(),
                info: (),
                position: Span::default(),
            },
            fields: vec![
                StructFieldDeclaration {
                    name: Id {
                        name: "bar".into(),
                        info: (),
                        position: Span::default(),
                    },
                    type_name: TypeName::Literal("BarStruct".into(), Span::default()),
                    info: (),
                    position: Span::default(),
                },
                StructFieldDeclaration {
                    name: Id {
                        name: "baz".into(),
                        info: (),
                        position: Span::default(),
                    },
                    type_name: TypeName::Literal("f64".into(), Span::default()),
                    info: (),
                    position: Span::default(),
                },
            ],
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(dec.info.type_id, Rc::new(RefCell::new(Some(Type::Void))));

        assert_eq!(
            ctx.scope.get_type("Foo"),
            Some(Type::Struct(
                "Foo".into(),
                vec![
                    ("bar".into(), Type::Struct("BarStruct".into(), vec![])),
                    ("baz".into(), Type::FloatingPoint)
                ]
            ))
        );

        Ok(())
    }
}
