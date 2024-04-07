use std::{borrow::Borrow, error::Error, fmt::Display};

use crate::{lexer::Span, parser::ast::TypeName};

use super::{
    context::Context,
    error::{TypeCheckError, UndefinedType},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    FloatingPoint,
    Boolean,
    Void,
    Unknown,
    Reference(Box<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Struct(String, Vec<(String, Type)>),
    Function {
        params: Vec<Type>,
        return_value: Box<Type>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeFromTypeNameError {
    source: TypeName,
}

impl Display for TypeFromTypeNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Failed to convert '{:?}' to a qualified type",
            self.source
        ))
    }
}

impl Error for TypeFromTypeNameError {}

impl From<TypeFromTypeNameError> for TypeCheckError {
    fn from(value: TypeFromTypeNameError) -> Self {
        TypeCheckError::UndefinedType(
            UndefinedType {
                type_name: value.source,
            },
            Span::default(),
        )
    }
}

impl<T> TryFrom<(T, &Context)> for Type
where
    T: Into<TypeName>,
{
    type Error = TypeCheckError;

    fn try_from((value, ctx): (T, &Context)) -> Result<Self, Self::Error> {
        let value = value.into();
        match &value {
            TypeName::Literal(lit, span) => match lit.as_str() {
                "i64" => Ok(Type::Integer),
                "f64" => Ok(Type::FloatingPoint),
                "void" => Ok(Type::Void),
                literal => match ctx.scope.get_type(literal) {
                    Some(type_id) => Ok(type_id),
                    None => Err(TypeCheckError::UndefinedType(
                        UndefinedType {
                            type_name: value.clone(),
                        },
                        span.clone(),
                    )),
                },
            },
            TypeName::Fn {
                params,
                return_type,
                ..
            } => {
                let mut new_params = vec![];

                for p in params.iter() {
                    new_params.push((p, ctx).try_into()?)
                }

                Ok(Type::Function {
                    params: new_params,
                    return_value: Box::new((return_type.borrow(), ctx).try_into()?),
                })
            }
            TypeName::Tuple(inner, _) => {
                let mut elements = vec![];

                for el in inner.iter() {
                    elements.push((el, ctx).try_into()?);
                }

                Ok(Type::Tuple(elements))
            }
            TypeName::Array(inner, _) => {
                Ok(Type::Array(Box::new((inner.borrow(), ctx).try_into()?)))
            }
            TypeName::Reference(inner, _) => {
                Ok(Type::Reference(Box::new((inner.borrow(), ctx).try_into()?)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        lexer::Span,
        parser::ast::TypeName,
        typechecker::{context::Context, types::Type},
    };

    #[test]
    fn test_primitive_literals() {
        let ctx = Context::default();

        assert_eq!(
            Type::try_from((TypeName::Literal("i64".into(), Span::default()), &ctx)),
            Ok(Type::Integer)
        );

        assert_eq!(
            Type::try_from((TypeName::Literal("f64".into(), Span::default()), &ctx)),
            Ok(Type::FloatingPoint)
        );
    }

    #[test]
    fn test_invalid_literal() {
        let ctx = Context::default();
        assert!(Type::try_from((TypeName::Literal("f32".into(), Span::default()), &ctx)).is_err());
        assert!(Type::try_from((TypeName::Literal("i32".into(), Span::default()), &ctx)).is_err());
        assert!(Type::try_from((TypeName::Literal("foo".into(), Span::default()), &ctx)).is_err());
    }

    #[test]
    fn test_custom_type() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope
            .add_type("Foo", Type::Array(Box::new(Type::Integer)))?;

        assert_eq!(
            Type::try_from((TypeName::Literal("Foo".into(), Span::default()), &ctx)),
            Ok(Type::Array(Box::new(Type::Integer)))
        );

        Ok(())
    }

    #[test]
    fn test_reference() {
        let ctx = Context::default();

        assert_eq!(
            Type::try_from((
                TypeName::Reference(
                    Box::new(TypeName::Literal("i64".into(), Span::default())),
                    Span::default()
                ),
                &ctx
            )),
            Ok(Type::Reference(Box::new(Type::Integer)))
        );
    }

    #[test]
    fn test_tuple() {
        let ctx = Context::default();

        assert_eq!(
            Type::try_from((
                TypeName::Tuple(
                    vec![
                        TypeName::Literal("i64".into(), Span::default()),
                        TypeName::Literal("f64".into(), Span::default())
                    ],
                    Span::default()
                ),
                &ctx
            )),
            Ok(Type::Tuple(vec![Type::Integer, Type::FloatingPoint]))
        )
    }

    #[test]
    fn test_array() {
        let ctx = Context::default();

        assert_eq!(
            Type::try_from((
                TypeName::Array(
                    Box::new(TypeName::Literal("i64".into(), Span::default())),
                    Span::default()
                ),
                &ctx
            )),
            Ok(Type::Array(Box::new(Type::Integer)))
        )
    }

    #[test]
    fn test_function() {
        let ctx = Context::default();

        let func = TypeName::Fn {
            params: vec![
                TypeName::Literal("i64".into(), Span::default()),
                TypeName::Literal("f64".into(), Span::default()),
            ],
            return_type: Box::new(TypeName::Literal("f64".into(), Span::default())),
            position: Span::default(),
        };

        assert_eq!(
            Type::try_from((func, &ctx)),
            Ok(Type::Function {
                params: vec![Type::Integer, Type::FloatingPoint],
                return_value: Box::new(Type::FloatingPoint)
            })
        )
    }
}
