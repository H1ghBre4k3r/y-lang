use std::{fmt::Display, str::FromStr};

use super::Rule;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrefixOp {
    UnaryMinus,
    Not,
}

#[derive(Debug)]
pub struct UndefinedPrefixOpError(String);

impl FromStr for PrefixOp {
    type Err = UndefinedPrefixOpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(PrefixOp::UnaryMinus),
            "!" => Ok(PrefixOp::Not),
            _ => Err(UndefinedPrefixOpError(format!(
                "Unexpected prefix op '{s}'"
            ))),
        }
    }
}

impl Display for PrefixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PrefixOp::UnaryMinus => "-",
            PrefixOp::Not => "!",
        })
    }
}

impl From<Rule> for PrefixOp {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::unaryMinus => PrefixOp::UnaryMinus,
            Rule::not => PrefixOp::Not,
            _ => unreachable!("Unexpected rule {:?}", rule),
        }
    }
}
