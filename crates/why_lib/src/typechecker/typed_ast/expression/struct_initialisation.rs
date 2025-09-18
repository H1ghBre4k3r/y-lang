use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
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
    type Typed = StructInitialisation<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
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

        // Step 1: Resolve the struct type from the type scope
        // Struct initializations require that the named struct type exists and is accessible
        let Some(Type::Struct(struct_type_name, struct_type_fields)) = ctx.scope.get_type(&name)
        else {
            // Struct type not found in scope - report undefined type error
            return Err(TypeCheckError::UndefinedType(
                UndefinedType {
                    type_name: TypeName::Literal(name, Span::default()),
                },
                struct_position,
            ));
        };

        // Step 2: Type check all field initializations provided in the source
        // Each field initialization is checked independently for type correctness
        let mut checked_fields = vec![];
        for field in fields.into_iter() {
            checked_fields.push(field.check(ctx)?);
        }

        // Step 3: Create a lookup map for efficient field resolution
        // This allows us to match provided fields against the struct's declared fields
        let mut checked_fields_map = checked_fields
            .iter()
            .map(|dec| (dec.name.name.clone(), dec.clone()))
            .collect::<HashMap<_, _>>();

        // Step 4: Validate that all required struct fields are initialized with correct types
        let mut checked_fields = vec![];

        // Iterate through each field declared in the struct type definition
        // Every declared field must be initialized with a value of the correct type
        for (struct_field_name, struct_field_type) in struct_type_fields.iter() {
            // Look up the initialization for this struct field
            let Some(mut initialised_field) =
                checked_fields_map.get_mut(struct_field_name).cloned()
            else {
                // Required struct field was not provided in the initialization - this is an error
                // TODO: use different error for this (should be "missing field" not "undefined variable")
                return Err(TypeCheckError::UndefinedVariable(
                    UndefinedVariable {
                        variable_name: format!("{name}.{struct_field_name}"),
                    },
                    struct_position,
                ));
            };

            // Extract the type of the initialized field value
            let field_type = initialised_field.info.type_id.clone();
            let initialised_field_type = {
                let inner = field_type.borrow_mut();
                inner.as_ref().cloned()
            };

            // Verify type compatibility between initialized value and struct field declaration
            match initialised_field_type {
                // Field has a concrete type - must match the struct's declared field type
                Some(field_type) => {
                    if field_type != *struct_field_type {
                        // Type mismatch between initialized value and struct field declaration
                        return Err(TypeCheckError::TypeMismatch(
                            TypeMismatch {
                                expected: struct_field_type.clone(),
                                actual: field_type,
                            },
                            initialised_field.position,
                        ));
                    }
                }
                // Field has unknown type - propagate the expected type from struct declaration
                None => {
                    // Update the field value with the expected type from struct definition
                    initialised_field
                        .value
                        .update_type(struct_field_type.clone())?;

                    // Update the field's type information to match the struct declaration
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

    fn revert(this: &Self::Typed) -> Self {
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

impl TypedConstruct for StructInitialisation<TypeInformation> {
    type Validated = StructInitialisation<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let StructInitialisation {
            id,
            fields,
            info,
            position,
        } = self;

        let mut validated_fields = vec![];
        for field in fields {
            validated_fields.push(field.validate()?);
        }

        Ok(StructInitialisation {
            id: id.validate()?,
            fields: validated_fields,
            info: info.validate(&position)?,
            position,
        })
    }
}

impl TypeCheckable for StructFieldInitialisation<()> {
    type Typed = StructFieldInitialisation<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
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

        // Type check the value expression being assigned to this struct field
        // The value's type will be verified against the struct field's declared type later
        let value = value.check(ctx)?;

        // Extract the value's inferred type to use for this field initialization
        // This type will be compared against the struct's field type during struct initialization
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

    fn revert(this: &Self::Typed) -> Self {
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

impl TypedConstruct for StructFieldInitialisation<TypeInformation> {
    type Validated = StructFieldInitialisation<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let StructFieldInitialisation {
            name,
            value,
            info,
            position,
        } = self;

        Ok(StructFieldInitialisation {
            name: name.validate()?,
            value: value.validate()?,
            info: info.validate(&position)?,
            position,
        })
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
