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
    // TODO: Maybe just dont use
    Any,
    Func {
        params: Vec<VariableType>,
        return_value: Box<VariableType>,
        source: Option<Module<TypeInfo>>,
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
            Func {
                params,
                return_value,
                ..
            } => format!("{params:?} -> {return_value:?}"),
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
            VariableType::Any => 8,
            VariableType::Func { .. } => 8,
        }
    }

    pub fn set_source(self, source: Module<TypeInfo>) -> Self {
        match self {
            VariableType::Func {
                params,
                return_value,
                ..
            } => VariableType::Func {
                params,
                return_value,
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
}
