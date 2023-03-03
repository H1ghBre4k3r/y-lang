use pest::{iterators::Pairs, Parser};

use super::parse_error::{ParseError, ParseResult};

#[derive(Parser)]
#[grammar = "y-lang.pest"]
pub struct YParser;

impl YParser {
    pub fn parse_program(file: impl ToString, program: &str) -> ParseResult<Pairs<Rule>> {
        Self::parse(Rule::program, program).map_err(|error| ParseError::from((error, file)))
    }
}
