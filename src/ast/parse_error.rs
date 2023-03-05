use std::fmt::Display;

use pest::error::Error;

use super::{Position, Rule};

/// Struct representing an error which happened while parsing the code.
#[derive(Clone, Debug)]
pub struct ParseError {
    /// Error message of this parse error
    pub message: String,
    /// Position of this error
    pub position: Position,
    /// The "inner error" which caused this parse error. It is only used when trying to pretty
    /// print a ParseError
    error: Error<Rule>,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}{}", self.position.0, self.error))
    }
}

impl std::error::Error for ParseError {}

impl<T> From<(Error<Rule>, T)> for ParseError
where
    T: ToString,
{
    fn from((value, file): (Error<Rule>, T)) -> Self {
        match value.line_col {
            pest::error::LineColLocation::Pos((line, col)) => ParseError {
                message: value.variant.message().to_string(),
                position: (file.to_string(), line, col),
                error: value,
            },
            pest::error::LineColLocation::Span(_, _) => todo!(),
        }
    }
}

/// The result of parsing a pair.
pub type ParseResult<T> = Result<T, ParseError>;
