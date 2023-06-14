use log::trace;
use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Integer<T> {
    pub value: i64,
    pub position: Position,
    pub info: T,
}

impl Integer<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Integer<()> {
        trace!("creating Integer from pair '{pair:?}'");

        let (line, col) = pair.line_col();

        match pair.as_rule() {
            Rule::decimalNumber => Integer {
                value: pair.as_str().parse::<i64>().unwrap(),
                position: (file.to_owned(), line, col),
                info: (),
            },
            Rule::hexNumber => {
                let value = pair.as_str();
                let without_prefix = value.trim_start_matches("0x");
                Integer {
                    value: i64::from_str_radix(without_prefix, 16).unwrap(),
                    position: (file.to_owned(), line, col),
                    info: (),
                }
            }
            _ => unreachable!(),
        }
    }
}
