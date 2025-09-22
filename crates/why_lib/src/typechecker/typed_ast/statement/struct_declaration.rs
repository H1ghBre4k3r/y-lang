use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Id, StructDeclaration, StructFieldDeclaration},
    typechecker::{
        ShallowCheck, TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
        context::Context,
        error::{TypeCheckError, UndefinedType},
        types::Type,
    },
};

impl TypeCheckable for StructDeclaration<()> {
    type Typed = StructDeclaration<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
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

    fn revert(this: &Self::Typed) -> Self {
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

impl TypedConstruct for StructDeclaration<TypeInformation> {
    type Validated = StructDeclaration<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let StructDeclaration {
            id,
            fields,
            info,
            position,
        } = self;

        let mut validated_fields = vec![];
        for field in fields {
            validated_fields.push(field.validate()?);
        }

        Ok(StructDeclaration {
            id: id.validate()?,
            fields: validated_fields,
            info: info.validate(&position)?,
            position,
        })
    }
}

impl ShallowCheck for StructDeclaration<()> {
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let StructDeclaration { id, fields, .. } = self;

        let mut field_types = vec![];

        for StructFieldDeclaration {
            name, type_name, ..
        } in fields.iter()
        {
            let Ok(type_id) = Type::try_from((type_name, &*ctx)) else {
                return Err(TypeCheckError::UndefinedType(
                    UndefinedType {
                        type_name: type_name.clone(),
                    },
                    type_name.position(),
                ));
            };

            field_types.push((name.name.clone(), type_id));
        }

        let type_id = Type::Struct(id.name.clone(), field_types);

        if let Err(e) = ctx.scope.add_type(&id.name, type_id) {
            eprintln!("{e}")
        };

        Ok(())
    }
}

impl TypeCheckable for StructFieldDeclaration<()> {
    type Typed = StructFieldDeclaration<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
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
                ));
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

    fn revert(this: &Self::Typed) -> Self {
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

impl TypedConstruct for StructFieldDeclaration<TypeInformation> {
    type Validated = StructFieldDeclaration<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let StructFieldDeclaration {
            name,
            type_name,
            info,
            position,
        } = self;

        Ok(StructFieldDeclaration {
            name: name.validate()?,
            type_name,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Id, StructDeclaration, StructFieldDeclaration, TypeName},
        typechecker::{ShallowCheck, TypeCheckable, context::Context, types::Type},
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

        dec.shallow_check(&mut ctx)?;
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

        dec.shallow_check(&mut ctx)?;
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

        dec.shallow_check(&mut ctx)?;
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

        dec.shallow_check(&mut ctx)?;
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
