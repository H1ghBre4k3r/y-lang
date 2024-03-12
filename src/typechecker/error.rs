use std::{error::Error, fmt::Display};

use super::types::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeError {
    pub expected: Type,
    pub actual: Type,
}

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Expected type '{:?}' but got '{:?}'",
            self.expected, self.actual
        ))
    }
}

impl Error for TypeError {}
