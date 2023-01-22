use log::error;
use pest::{iterators::Pairs, Parser};

#[derive(Parser)]
#[grammar = "y-lang.pest"]
pub struct YParser;

impl YParser {
    pub fn parse_program(program: &str) -> Pairs<Rule> {
        match Self::parse(Rule::program, program) {
            Ok(pairs) => pairs,
            Err(err) => {
                error!("Failed to parse file ({})", err);
                std::process::exit(-1);
            }
        }
    }
}
