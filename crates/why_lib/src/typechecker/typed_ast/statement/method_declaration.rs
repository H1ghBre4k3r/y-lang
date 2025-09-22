use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::{Id, MethodDeclaration, TypeName},
    typechecker::{
        ShallowCheck, TypeCheckError, TypeCheckable, TypeInformation, TypeResult, context::Context,
        error::RedefinedFunction, types::Type,
    },
};

impl TypeCheckable for MethodDeclaration<()> {
    type Typed = MethodDeclaration<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();

        let type_id = self.simple_shallow_check(ctx)?;

        let MethodDeclaration {
            id,
            parameter_types,
            return_type,
            position,
            ..
        } = self;

        let Id {
            name,
            position: id_position,
            ..
        } = id;

        let id = Id {
            name,
            position: id_position,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(type_id))),
                context: context.clone(),
            },
        };

        Ok(MethodDeclaration {
            id,
            parameter_types,
            return_type,
            position,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let MethodDeclaration {
            id,
            parameter_types,
            return_type,
            position,
            ..
        } = this;

        MethodDeclaration {
            id: TypeCheckable::revert(id),
            parameter_types: parameter_types.clone(),
            return_type: return_type.clone(),
            position: position.clone(),
            info: (),
        }
    }
}

impl MethodDeclaration<()> {
    pub fn simple_shallow_check(&self, ctx: &Context) -> TypeResult<Type> {
        let MethodDeclaration {
            parameter_types,
            return_type,
            position,
            ..
        } = self;

        let function_type = TypeName::Fn {
            params: parameter_types.clone(),
            return_type: Box::new(return_type.clone()),
            position: position.clone(),
        };

        Type::try_from((&function_type, ctx))
    }
}

impl ShallowCheck for MethodDeclaration<()> {
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        let MethodDeclaration { id, position, .. } = self;

        let type_id = self.simple_shallow_check(&*ctx)?;

        if ctx.scope.add_constant(&id.name, type_id).is_err() {
            return Err(TypeCheckError::RedefinedFunction(
                RedefinedFunction {
                    function_name: id.name.clone(),
                },
                position.clone(),
            ));
        };
        Ok(())
    }
}

impl TypedConstruct for MethodDeclaration<TypeInformation> {
    type Validated = MethodDeclaration<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let MethodDeclaration {
            id,
            parameter_types,
            return_type,
            info,
            position,
        } = self;

        Ok(MethodDeclaration {
            id: id.validate()?,
            parameter_types,
            return_type,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Id, MethodDeclaration, TypeName},
        typechecker::{TypeCheckable, TypeInformation, context::Context, types::Type},
    };

    #[test]
    fn test_simple_method_declaration() -> anyhow::Result<()> {
        let mut ctx = Context::default();

        let dec = MethodDeclaration {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            parameter_types: vec![],
            return_type: TypeName::Literal("void".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(
            dec,
            MethodDeclaration {
                id: Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![],
                            return_value: Box::new(Type::Void)
                        }))),
                        context: ctx.clone()
                    },
                    position: Span::default(),
                },
                parameter_types: vec![],
                return_type: TypeName::Literal("void".into(), Span::default()),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void))),
                    context: ctx.clone()
                },
                position: Span::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_complex_method_declaration() -> anyhow::Result<()> {
        let mut ctx = Context::default();

        let dec = MethodDeclaration {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            parameter_types: vec![
                TypeName::Literal("i64".into(), Span::default()),
                TypeName::Tuple(
                    vec![
                        TypeName::Literal("i64".into(), Span::default()),
                        TypeName::Literal("f64".into(), Span::default()),
                    ],
                    Span::default(),
                ),
            ],
            return_type: TypeName::Literal("i64".into(), Span::default()),
            info: (),
            position: Span::default(),
        };

        let dec = dec.check(&mut ctx)?;

        assert_eq!(
            dec,
            MethodDeclaration {
                id: Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Function {
                            params: vec![
                                Type::Integer,
                                Type::Tuple(vec![Type::Integer, Type::FloatingPoint])
                            ],
                            return_value: Box::new(Type::Integer)
                        }))),
                        context: ctx.clone()
                    },
                    position: Span::default(),
                },
                parameter_types: vec![
                    TypeName::Literal("i64".into(), Span::default()),
                    TypeName::Tuple(
                        vec![
                            TypeName::Literal("i64".into(), Span::default()),
                            TypeName::Literal("f64".into(), Span::default()),
                        ],
                        Span::default(),
                    ),
                ],
                return_type: TypeName::Literal("i64".into(), Span::default()),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Void))),
                    context: ctx.clone()
                },
                position: Span::default(),
            }
        );

        Ok(())
    }
}
