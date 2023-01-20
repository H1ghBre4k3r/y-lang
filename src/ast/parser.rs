use pest::{iterators::Pairs, Parser};

#[derive(Parser)]
#[grammar = "y-lang.pest"]
pub struct YParser;

impl YParser {
    pub fn parse_program(program: &str) -> Pairs<Rule> {
        Self::parse(Rule::program, program).expect("failed to parse file")
    }
}
