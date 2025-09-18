use std::{borrow::Borrow, error::Error, fmt::Display};

use crate::{lexer::Span, parser::ast::TypeName};

use super::{
    context::Context,
    error::{TypeCheckError, UndefinedType},
};

#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
/// Canonical semantic type representation used throughout inference,
/// validation and code generation. Variants are intentionally minimal;
/// additional semantic concepts (e.g., generics) would extend this.
pub enum Type {
    /// 64-bit signed integer primitive
    Integer,
    /// 64-bit IEEE 754 floating point primitive
    FloatingPoint,
    /// Boolean truth value primitive
    Boolean,
    /// Unicode scalar value (character) primitive
    Character,
    /// Immutable UTF-8 string value
    String,
    /// Unit / no-value type used for statements or functions that do not return a value
    Void,
    /// Placeholder used during inference when a concrete type has not yet been determined
    Unknown,
    /// Borrow-style reference to an underlying type (treated transparently for equality)
    Reference(Box<Type>),
    /// Fixed-length heterogeneous ordered collection
    Tuple(Vec<Type>),
    /// Homogeneous sequential collection of elements with a single element type
    Array(Box<Type>),
    /// User defined struct with name and ordered list of (field_name, field_type) pairs
    Struct(String, Vec<(String, Type)>),
    /// Function signature with parameter types and return type
    Function {
        /// Ordered parameter types
        params: Vec<Type>,
        /// Return value type
        return_value: Box<Type>,
    },
}

impl Type {
    /// Structural semantic equality with lightweight reference collapsing.
    ///
    /// Behaviour summary:
    /// - `Reference(T)` equals `T` (one level transparent indirection)
    /// - Function, tuple, array and struct variants compare their contained types structurally
    /// - All other primitives compare by discriminant only
    /// - Does not perform recursive dereferencing of nested `Reference(Reference(T))` chains
    ///   (only a single layer is collapsed) which is sufficient for current language semantics.
    pub fn does_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Reference(l0), r0) => l0.as_ref() == r0,
            (l0, Self::Reference(r0)) => l0 == r0.as_ref(),
            (Self::Tuple(l0), Self::Tuple(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Struct(l0, l1), Self::Struct(r0, r1)) => l0 == r0 && l1 == r1,
            (
                Self::Function {
                    params: l_params,
                    return_value: l_return_value,
                },
                Self::Function {
                    params: r_params,
                    return_value: r_return_value,
                },
            ) => l_params == r_params && l_return_value == r_return_value,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer => write!(f, "Integer"),
            Self::FloatingPoint => write!(f, "FloatingPoint"),
            Self::Boolean => write!(f, "Boolean"),
            Self::Void => write!(f, "Void"),
            Self::Character => write!(f, "Character"),
            Self::String => write!(f, "String"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Reference(arg0) => f.debug_tuple("Reference").field(arg0).finish(),
            Self::Tuple(arg0) => f.debug_tuple("Tuple").field(arg0).finish(),
            Self::Array(arg0) => f.debug_tuple("Array").field(arg0).finish(),
            Self::Struct(arg0, _) => f.write_fmt(format_args!("struct {arg0}")),
            Self::Function {
                params,
                return_value,
            } => f.write_fmt(format_args!(
                "({}) -> {return_value:?}",
                params
                    .iter()
                    .map(|i| format!("{i:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Error converting a parsed `TypeName` AST node into a concrete `Type`.
/// Usually indicates an undefined user type or unsupported literal.
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

/// Convert a parsed `TypeName` (or convertible) plus current `Context` into
/// a fully resolved `Type`, performing recursive resolution of nested types
/// and validating user defined type references.
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
                "bool" => Ok(Type::Boolean),
                "char" => Ok(Type::Character),
                "str" => Ok(Type::String),
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
