use std::{fmt::Display, str::FromStr};

use crate::loader::Module;

use super::TypeInfo;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum VariableType {
    #[default]
    Void,
    Bool,
    Str,
    Int,
    Char,
    // TODO: Maybe just dont use
    Any,
    Unknown,
    Func {
        params: Vec<VariableType>,
        return_type: Box<VariableType>,
        source: Option<Module<TypeInfo>>,
    },
    ArraySlice(Box<VariableType>),
    TupleArray {
        item_type: Box<VariableType>,
        size: usize,
    },
}

pub struct VariableParseError(String);

impl FromStr for VariableType {
    type Err = VariableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "void" => Ok(Self::Void),
            "bool" => Ok(Self::Bool),
            "str" => Ok(Self::Str),
            "int" => Ok(Self::Int),
            "any" => Ok(Self::Any),
            "char" => Ok(Self::Char),
            "unknown" => Ok(Self::Unknown),
            _ => Err(VariableParseError(format!("Invalid type '{s}'"))),
        }
    }
}

impl Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use VariableType::*;

        let value = &match self {
            Void => "void".to_owned(),
            Bool => "bool".to_owned(),
            Int => "int".to_owned(),
            Str => "str".to_owned(),
            Any => "any".to_owned(),
            Char => "char".to_owned(),
            Unknown => "unknown".to_owned(),
            Func {
                params,
                return_type: return_value,
                ..
            } => format!("{params:?} -> {return_value:?}"),
            ArraySlice(item_type) => format!("&[{item_type}]"),
            TupleArray { item_type, size } => format!("[{item_type}; {size}]"),
        };

        f.write_str(value)
    }
}

impl VariableType {
    pub fn size(&self) -> usize {
        match self {
            VariableType::Void => 0,
            VariableType::Bool => 1,
            VariableType::Str => 8,
            VariableType::Int => 8,
            VariableType::Char => 1,
            VariableType::Any => 8,
            VariableType::Unknown => 8,
            VariableType::Func { .. } => 8,
            VariableType::ArraySlice(_) => 8,
            VariableType::TupleArray { item_type, size } => 8,
        }
    }

    pub fn set_source(self, source: Module<TypeInfo>) -> Self {
        match self {
            VariableType::Func {
                params,
                return_type: return_value,
                ..
            } => VariableType::Func {
                params,
                return_type: return_value,
                source: Some(source),
            },
            _ => self,
        }
    }

    pub fn get_source(&self) -> Option<Module<TypeInfo>> {
        match self {
            VariableType::Func { source, .. } => source.clone(),
            _ => None,
        }
    }

    /// Try to convert this variable type to another. If the conversion is successful, it returns
    /// the new variable type. If it is not successful, it returns Err(()).
    ///
    /// Note the rules:
    ///     - `unknown` can be converted to anything
    ///     - nothing can be converted to `unknown` (except `unknown` itself)
    ///     - everything can be converted to `any`
    ///     - `any` can not be converted to anything else
    ///     - every basic type can be converted to itself
    pub fn convert_to(&self, to_convert_to: &Self) -> Result<Self, ()> {
        use VariableType::*;
        match (self, to_convert_to) {
            (Unknown, other) => Ok(other.clone()),
            (_, Any) => Ok(Any),
            (TupleArray { item_type, .. }, ArraySlice(other_item_type)) => {
                Ok(ArraySlice(Box::new(item_type.convert_to(other_item_type)?)))
            }
            (Str, ArraySlice(other_item_type)) => {
                if *other_item_type == Box::new(Char) {
                    Ok(ArraySlice(Box::new(Char)))
                } else {
                    Err(())
                }
            }
            (Char, Int) => Ok(Int),
            (Int, Char) => Ok(Char),
            (TupleArray { item_type, .. }, Str) => {
                if *item_type == Box::new(Char) {
                    Ok(Str)
                } else {
                    Err(())
                }
            }
            (left, right) => {
                if left == right {
                    Ok(right.clone())
                } else {
                    Err(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VariableType::*;

    #[test]
    fn test_convert_to_any() {
        assert_eq!(Void.convert_to(&Any), Ok(Any));
        assert_eq!(Int.convert_to(&Any), Ok(Any));
        assert_eq!(Any.convert_to(&Any), Ok(Any));
    }

    #[test]
    fn test_convert_from_any() {
        assert_eq!(Any.convert_to(&Void), Err(()));
        assert_eq!(Any.convert_to(&Int), Err(()));
        assert_eq!(Any.convert_to(&Str), Err(()));
    }

    #[test]
    fn test_convert_from_unknown() {
        assert_eq!(Unknown.convert_to(&Int), Ok(Int));
        assert_eq!(Unknown.convert_to(&Any), Ok(Any));
        assert_eq!(Unknown.convert_to(&Unknown), Ok(Unknown));
    }

    #[test]
    fn test_conver_to_unknown() {
        assert_eq!(Int.convert_to(&Unknown), Err(()));
        assert_eq!(Any.convert_to(&Unknown), Err(()));
        assert_eq!(Void.convert_to(&Unknown), Err(()));
    }
}
