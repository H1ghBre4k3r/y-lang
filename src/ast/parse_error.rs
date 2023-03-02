use std::fmt::Display;

use pest::error::Error;

use super::{Position, Rule};

/// Struct representing an error which happened while parsing the code.
#[derive(Clone, Debug)]
pub struct ParseError {
    /// Error message of this parse error
    pub message: String,
    pub position: Position,
}

impl ParseError {
    pub fn from_file_and_parse_error(file: impl ToString, value: Error<Rule>) -> Self {
        match value.line_col {
            pest::error::LineColLocation::Pos((line, col)) => ParseError {
                message: format!("{value}"),
                position: (file.to_string(), line, col),
            },
            pest::error::LineColLocation::Span(_, _) => todo!(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}{}", self.position.0, self.message))
    }
}

impl std::error::Error for ParseError {}

/// The result of parsing a pair.
pub type ParseResult<T> = Result<T, ParseError>;
