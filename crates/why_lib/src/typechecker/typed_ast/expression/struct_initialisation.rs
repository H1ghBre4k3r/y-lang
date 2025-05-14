use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    lexer::Span,
    parser::ast::{Id, StructFieldInitialisation, StructInitialisation, TypeName},
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch, UndefinedType, UndefinedVariable},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for StructInitialisation<()> {
    type Output = StructInitialisation<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let context = ctx.clone();

        let StructInitialisation {
            id,
            fields,
            position: struct_position,
            ..
        } = self;

        let Id {
            name,
            position: id_position,
            ..
        } = id;

        let Some(Type::Struct(struct_type_name, struct_type_fields)) = ctx.scope.get_type(&name)
        else {
            return Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: TypeName::Literal(name, Span::default()),
                },
                struct_position,
            ));
        };

        let mut checked_fields = vec![];

        for field in fields.into_iter() {
            checked_fields.push(field.check(ctx)?);
        }

        let mut checked_fields_map = checked_fields
            .iter()
            .map(|dec| (dec.name.name.clone(), dec.clone()))
            .collect::<HashMap<_, _>>();

        let mut checked_fields = vec![];

        for (struct_field_name, struct_field_type) in struct_type_fields.iter() {
            let Some(mut initialised_field) =
                checked_fields_map.get_mut(struct_field_name).cloned()
            else {
                // TODO: use different error for this
                return Err(TypeCheckError::UndefinedVariable(
                    UndefinedVariable {
                        variable_name: format!("{name}.{struct_field_name}"),
                    },
                    struct_position,
                ));
            };

            let field_type = initialised_field.info.type_id.clone();

            let initialised_field_type = {
                let inner = field_type.borrow_mut();
                inner.as_ref().cloned()
            };

            match initialised_field_type {
                Some(field_type) => {
                    if field_type != *struct_field_type {
                        return Err(TypeCheckError::TypeMismatch(
                            TypeMismatch {
                                expected: struct_field_type.clone(),
                                actual: field_type,
                            },
                            initialised_field.position,
                        ));
                    }
                }
                None => {
                    initialised_field
                        .value
                        .update_type(struct_field_type.clone())?;

                    *field_type.borrow_mut() = Some(struct_field_type.clone());
                }
            }

            checked_fields.push(initialised_field);
        }

        let info = TypeInformation {
            type_id: Rc::new(RefCell::new(Some(Type::Struct(
                struct_type_name,
                struct_type_fields,
            )))),
            context,
        };

        Ok(StructInitialisation {
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
        let StructInitialisation {
            id,
            fields,
            position,
            ..
        } = this;

        StructInitialisation {
            id: Id {
                name: id.name.clone(),
                info: (),
                position: id.position.clone(),
            },
            fields: fields.iter().map(TypeCheckable::revert).collect(),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypeCheckable for StructFieldInitialisation<()> {
    type Output = StructFieldInitialisation<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let context = ctx.clone();

        let StructFieldInitialisation {
            name,
            value,
            position: struct_position,
            ..
        } = self;

        let Id {
            name,
            position: id_position,
            ..
        } = name;

        let value = value.check(ctx)?;

        let type_id = value.get_info().type_id;

        let info = TypeInformation { type_id, context };

        Ok(StructFieldInitialisation {
            name: Id {
                name,
                info: info.clone(),
                position: id_position,
            },
            value,
            info,
            position: struct_position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let StructFieldInitialisation {
            name,
            value,
            position,
            ..
        } = this;

        StructFieldInitialisation {
            name: Id {
                name: name.name.clone(),
                info: (),
                position: name.position.clone(),
            },
            value: TypeCheckable::revert(value),
            info: (),
            position: position.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::{Ok, Result};

    use crate::{
        lexer::Span,
        parser::ast::{
            Expression, Id, Num, StructDeclaration, StructFieldDeclaration,
            StructFieldInitialisation, StructInitialisation, TypeName,
        },
        typechecker::{context::Context, types::Type, ShallowCheck, TypeCheckable},
    };

    #[test]
    fn test_empty_struct_initialisation() -> Result<()> {
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

        let init = StructInitialisation {
            id: Id {
                name: "BarStruct".into(),
                info: (),
                position: Span::default(),
            },
            fields: vec![],
            info: (),
            position: Span::default(),
        };

        let init = init.check(&mut ctx)?;

        assert_eq!(
            init.info.type_id,
            Rc::new(RefCell::new(Some(Type::Struct("BarStruct".into(), vec![]))))
        );

        Ok(())
    }

    #[test]
    fn test_filled_struct_initialisation() -> Result<()> {
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
        dec.check(&mut ctx)?;

        let init = StructInitialisation {
            id: Id {
                name: "Foo".into(),
                info: (),
                position: Span::default(),
            },
            fields: vec![
                StructFieldInitialisation {
                    name: Id {
                        name: "bar".into(),
                        info: (),
                        position: Span::default(),
                    },
                    value: Expression::Num(Num::Integer(42, (), Span::default())),
                    info: (),
                    position: Span::default(),
                },
                StructFieldInitialisation {
                    name: Id {
                        name: "baz".into(),
                        info: (),
                        position: Span::default(),
                    },
                    value: Expression::Num(Num::FloatingPoint(133.7, (), Span::default())),
                    info: (),
                    position: Span::default(),
                },
            ],
            info: (),
            position: Span::default(),
        };

        let init = init.check(&mut ctx)?;

        assert_eq!(
            init.info.type_id,
            Rc::new(RefCell::new(Some(Type::Struct(
                "Foo".into(),
                vec![
                    ("bar".into(), Type::Integer),
                    ("baz".into(), Type::FloatingPoint)
                ]
            ))))
        );

        assert_eq!(
            init.fields[0].info.type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        assert_eq!(
            init.fields[1].info.type_id,
            Rc::new(RefCell::new(Some(Type::FloatingPoint)))
        );

        Ok(())
    }

    #[test]
    fn test_filled_struct_initialisation_swapped_fields() -> Result<()> {
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
        dec.check(&mut ctx)?;

        let init = StructInitialisation {
            id: Id {
                name: "Foo".into(),
                info: (),
                position: Span::default(),
            },
            fields: vec![
                StructFieldInitialisation {
                    name: Id {
                        name: "baz".into(),
                        info: (),
                        position: Span::default(),
                    },
                    value: Expression::Num(Num::FloatingPoint(133.7, (), Span::default())),
                    info: (),
                    position: Span::default(),
                },
                StructFieldInitialisation {
                    name: Id {
                        name: "bar".into(),
                        info: (),
                        position: Span::default(),
                    },
                    value: Expression::Num(Num::Integer(42, (), Span::default())),
                    info: (),
                    position: Span::default(),
                },
            ],
            info: (),
            position: Span::default(),
        };

        let init = init.check(&mut ctx)?;

        assert_eq!(
            init.info.type_id,
            Rc::new(RefCell::new(Some(Type::Struct(
                "Foo".into(),
                vec![
                    ("bar".into(), Type::Integer),
                    ("baz".into(), Type::FloatingPoint)
                ]
            ))))
        );

        assert_eq!(
            init.fields[0].info.type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        assert_eq!(
            init.fields[1].info.type_id,
            Rc::new(RefCell::new(Some(Type::FloatingPoint)))
        );

        Ok(())
    }
}
