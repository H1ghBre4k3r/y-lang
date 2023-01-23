use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryVerb {
    GreaterThan,
    LessThan,
    Equal,
    Plus,
    Minus,
    Times,
}

#[derive(Debug)]
pub struct UndefinedVerbError(String);

impl FromStr for BinaryVerb {
    type Err = UndefinedVerbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">" => Ok(BinaryVerb::GreaterThan),
            "<" => Ok(BinaryVerb::LessThan),
            "==" => Ok(BinaryVerb::Equal),
            "+" => Ok(BinaryVerb::Plus),
            "-" => Ok(BinaryVerb::Minus),
            "*" => Ok(BinaryVerb::Times),
            _ => Err(UndefinedVerbError(format!(
                "Unexpected binary verb '{}'",
                s
            ))),
        }
    }
}
