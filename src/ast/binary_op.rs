use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    GreaterThan,
    LessThan,
    Equal,
    Plus,
    Minus,
    Times,
    DividedBy,
}

#[derive(Debug)]
pub struct UndefinedOpError(String);

impl FromStr for BinaryOp {
    type Err = UndefinedOpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">" => Ok(BinaryOp::GreaterThan),
            "<" => Ok(BinaryOp::LessThan),
            "==" => Ok(BinaryOp::Equal),
            "+" => Ok(BinaryOp::Plus),
            "-" => Ok(BinaryOp::Minus),
            "*" => Ok(BinaryOp::Times),
            "/" => Ok(BinaryOp::DividedBy),
            _ => Err(UndefinedOpError(format!("Unexpected binary op '{s}'"))),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BinaryOp::GreaterThan => ">",
            BinaryOp::LessThan => "<",
            BinaryOp::Equal => "==",
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Times => "*",
            BinaryOp::DividedBy => "/",
        })
    }
}
