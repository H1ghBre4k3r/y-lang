use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::Array,
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for Array<()> {
    type Typed = Array<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();
        match self {
            Array::Literal {
                values, position, ..
            } => {
                let mut checked_values = vec![];

                for value in values.into_iter() {
                    checked_values.push(value.check(ctx)?);
                }

                let type_id = checked_values.first().map(|val| val.get_info().type_id);

                if let Some(type_id) = &type_id {
                    let type_id = { type_id.borrow() }.clone();
                    for value in checked_values.iter() {
                        let value_type = { value.get_info().type_id.borrow() }.clone();
                        if let (Some(type_id), Some(value_type)) = (&type_id, value_type) {
                            if *type_id != value_type {
                                return Err(TypeCheckError::TypeMismatch(
                                    TypeMismatch {
                                        expected: type_id.clone(),
                                        actual: value_type,
                                    },
                                    value.position(),
                                ));
                            }
                        }
                    }
                }

                Ok(Array::Literal {
                    values: checked_values,
                    info: TypeInformation {
                        type_id: type_id.map_or(
                            // TODO: which one?
                            Rc::new(RefCell::new(None)),
                            // Rc::new(RefCell::new(Some(Type::Array(Box::new(Type::Unknown))))),
                            |type_id| {
                                Rc::new(RefCell::new(
                                    type_id
                                        .borrow()
                                        .clone()
                                        .map(|type_id| Type::Array(Box::new(type_id))),
                                ))
                            },
                        ),
                        context,
                    },
                    position,
                })
            }
            Array::Default {
                initial_value,
                length,
                position,
                ..
            } => {
                let initial_value = initial_value.check(ctx)?;

                // TODO: This should be an expression which evaluates to an integer
                // FIXME: This currently allows for FloatingPoint lengths
                let length = length.check(ctx)?;

                let type_id = { initial_value.get_info().type_id.borrow() }.clone();

                Ok(Array::Default {
                    initial_value: Box::new(initial_value),
                    length,
                    info: TypeInformation {
                        type_id: type_id.map_or(
                            // TODO: which one?
                            Rc::new(RefCell::new(None)),
                            // Rc::new(RefCell::new(Some(Type::Array(Box::new(Type::Unknown))))),
                            |type_id| Rc::new(RefCell::new(Some(Type::Array(Box::new(type_id))))),
                        ),
                        context,
                    },
                    position,
                })
            }
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            Array::Literal {
                values, position, ..
            } => Array::Literal {
                values: values.iter().map(TypeCheckable::revert).collect(),
                info: (),
                position: position.clone(),
            },
            Array::Default {
                initial_value,
                length,
                position,
                ..
            } => Array::Default {
                initial_value: Box::new(TypeCheckable::revert(initial_value.as_ref())),
                length: TypeCheckable::revert(length),
                info: (),
                position: position.clone(),
            },
        }
    }
}

impl TypedConstruct for Array<TypeInformation> {
    type Validated = Array<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            Array::Default {
                initial_value,
                length,
                info,
                position,
            } => Ok(Array::Default {
                initial_value: Box::new(initial_value.validate()?),
                length: length.validate()?,
                info: info.validate(&position)?,
                position,
            }),
            Array::Literal {
                values,
                info,
                position,
            } => {
                let mut validated_values = vec![];
                for value in values {
                    validated_values.push(value.validate()?);
                }

                Ok(Array::Literal {
                    values: validated_values,
                    info: info.validate(&position)?,
                    position,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{Array, Expression, Num},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            TypeCheckable,
        },
    };

    #[test]
    fn test_empty_array_literal() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Literal {
            values: vec![],
            info: (),
            position: Span::default(),
        };

        let arr = arr.check(&mut ctx)?;

        assert_eq!(arr.get_info().type_id, Rc::new(RefCell::new(None)));

        Ok(())
    }

    #[test]
    fn test_single_element_array_literal() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Literal {
            values: vec![Expression::Num(Num::Integer(42, (), Span::default()))],
            info: (),
            position: Span::default(),
        };

        let arr = arr.check(&mut ctx)?;

        assert_eq!(
            arr.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Array(Box::new(Type::Integer)))))
        );
        Ok(())
    }

    #[test]
    fn test_multiple_element_array_literal_match() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Literal {
            values: vec![
                Expression::Num(Num::FloatingPoint(42.0, (), Span::default())),
                Expression::Num(Num::FloatingPoint(1337.0, (), Span::default())),
            ],
            info: (),
            position: Span::default(),
        };

        let arr = arr.check(&mut ctx)?;

        assert_eq!(
            arr.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Array(Box::new(
                Type::FloatingPoint
            )))))
        );
        Ok(())
    }

    #[test]
    fn test_multiple_element_array_literal_mismatch() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Literal {
            values: vec![
                Expression::Num(Num::Integer(42, (), Span::default())),
                Expression::Num(Num::FloatingPoint(1337.0, (), Span::default())),
            ],
            info: (),
            position: Span::default(),
        };

        let res = arr.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::Integer,
                    actual: Type::FloatingPoint
                },
                Span::default()
            ))
        );
        Ok(())
    }

    #[test]
    fn test_simple_default_array() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Default {
            initial_value: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
            length: Num::Integer(10, (), Span::default()),
            info: (),
            position: Span::default(),
        };

        let arr = arr.check(&mut ctx)?;

        assert_eq!(
            arr.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Array(Box::new(Type::Integer)))))
        );

        Ok(())
    }
}
