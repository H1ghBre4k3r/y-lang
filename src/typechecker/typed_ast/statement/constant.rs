use std::{cell::RefCell, rc::Rc};

use crate::{
    parser::ast::{Constant, Id},
    typechecker::{
        context::Context,
        error::{InvalidConstantType, RedefinedConstant, TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Constant<()> {
    type Output = Constant<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        let Constant {
            id,
            type_name,
            value,
            position: const_position,
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

        let Ok(type_id) = Type::try_from((&type_name, &*ctx)) else {
            return Err(TypeCheckError::InvalidConstantType(InvalidConstantType {
                constant_name: name,
            }));
        };

        {
            let inner = info.type_id.clone();
            let mut inner = inner.borrow_mut();

            match inner.as_ref() {
                Some(inner_type) => {
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
                    value.update_type(type_id.clone())?;

                    // ...and the type of enclosed in the information
                    *inner = Some(type_id.clone());
                }
            }
        }

        if ctx.scope.add_constant(&name, type_id).is_err() {
            return Err(TypeCheckError::RedefinedConstant(RedefinedConstant {
                constant_name: name,
            }));
        };

        Ok(Constant {
            id: Id {
                name,
                info,
                position: id_position,
            },
            type_name,
            value,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position: const_position,
        })
    }

    fn revert(this: &Self::Output) -> Self {
        let Constant {
            id,
            type_name,
            value,
            position,
            ..
        } = this;

        Constant {
            id: TypeCheckable::revert(id),
            type_name: type_name.clone(),
            value: TypeCheckable::revert(value),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Constant<TypeInformation> {}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Constant, Expression, Id, Num, TypeName},
        typechecker::{
            context::Context,
            error::{InvalidConstantType, TypeCheckError},
            types::Type,
            TypeCheckable,
        },
    };

    #[test]
    fn test_constant_simple() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let constant = Constant {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("i64".into()),
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let constant = constant.check(&mut ctx)?;

        assert_eq!(
            constant.id.info.type_id,
            Rc::new(RefCell::new(Some(Type::Integer)))
        );

        assert_eq!(ctx.scope.get_constant("foo"), Some(Type::Integer));

        Ok(())
    }

    #[test]
    fn test_error_on_missing_type_annotation() {
        let mut ctx = Context::default();

        let constant = Constant {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("".into()),
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let result = constant.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::InvalidConstantType(InvalidConstantType {
                constant_name: "foo".into()
            }))
        );
    }

    #[test]
    fn test_error_on_redefinition() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope.add_constant("foo", Type::Integer)?;

        let constant = Constant {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            type_name: TypeName::Literal("".into()),
            value: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let result = constant.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::InvalidConstantType(InvalidConstantType {
                constant_name: "foo".into()
            }))
        );

        Ok(())
    }
}
