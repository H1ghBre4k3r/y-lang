use std::{collections::HashMap, error::Error, fmt::Display};

use crate::parser::ast::TypeName;

use super::context::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    FloatingPoint,
    Boolean,
    Void,
    Reference(Box<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Struct(String, HashMap<String, Type>),
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

impl TryFrom<(TypeName, &Context)> for Type {
    type Error = TypeFromTypeNameError;

    fn try_from((value, ctx): (TypeName, &Context)) -> Result<Self, Self::Error> {
        match value.clone() {
            TypeName::Literal(lit) => match lit.as_str() {
                "i64" => Ok(Type::Integer),
                "f64" => Ok(Type::FloatingPoint),
                literal => match ctx.scope.get_type(literal) {
                    Some(type_id) => Ok(type_id),
                    None => Err(TypeFromTypeNameError { source: value }),
                },
            },
            TypeName::Fn {
                params,
                return_type,
            } => {
                let mut new_params = vec![];

                for p in params.into_iter() {
                    new_params.push((p, ctx).try_into()?)
                }

                Ok(Type::Function {
                    params: new_params,
                    return_value: Box::new((*return_type, ctx).try_into()?),
                })
            }
            TypeName::Tuple(inner) => {
                let mut elements = vec![];

                for el in inner.into_iter() {
                    elements.push((el, ctx).try_into()?);
                }

                Ok(Type::Tuple(elements))
            }
            TypeName::Array(inner) => Ok(Type::Array(Box::new((*inner, ctx).try_into()?))),
            TypeName::Reference(inner) => Ok(Type::Reference(Box::new((*inner, ctx).try_into()?))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        parser::ast::TypeName,
        typechecker::{context::Context, types::Type},
    };

    #[test]
    fn test_primitive_literals() {
        let ctx = Context::default();

        assert_eq!(
            Type::try_from((TypeName::Literal("i64".into()), &ctx)),
            Ok(Type::Integer)
        );

        assert_eq!(
            Type::try_from((TypeName::Literal("f64".into()), &ctx)),
            Ok(Type::FloatingPoint)
        );
    }

    #[test]
    fn test_invalid_literal() {
        let ctx = Context::default();
        assert!(Type::try_from((TypeName::Literal("f32".into()), &ctx)).is_err());
        assert!(Type::try_from((TypeName::Literal("i32".into()), &ctx)).is_err());
        assert!(Type::try_from((TypeName::Literal("foo".into()), &ctx)).is_err());
    }

    #[test]
    fn test_custom_type() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        ctx.scope
            .add_type("Foo", Type::Array(Box::new(Type::Integer)))?;

        assert_eq!(
            Type::try_from((TypeName::Literal("Foo".into()), &ctx)),
            Ok(Type::Array(Box::new(Type::Integer)))
        );

        Ok(())
    }

    #[test]
    fn test_reference() {
        let ctx = Context::default();

        assert_eq!(
            Type::try_from((
                TypeName::Reference(Box::new(TypeName::Literal("i64".into()))),
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
                TypeName::Tuple(vec![
                    TypeName::Literal("i64".into()),
                    TypeName::Literal("f64".into())
                ]),
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
                TypeName::Array(Box::new(TypeName::Literal("i64".into()))),
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
                TypeName::Literal("i64".into()),
                TypeName::Literal("f64".into()),
            ],
            return_type: Box::new(TypeName::Literal("f64".into())),
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
