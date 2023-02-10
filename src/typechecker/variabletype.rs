use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableType {
    Void,
    Bool,
    Str,
    Int,
    // TODO: Maybe just dont use
    Any,
    Func {
        params: Vec<VariableType>,
        return_value: Box<VariableType>,
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
            VariableType::Str => 4,
            VariableType::Int => 8,
            VariableType::Any => 4,
            VariableType::Func { .. } => 4,
        }
    }
}
