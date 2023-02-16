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
        assert_eq!(pair.as_rule(), Rule::integer);
        let (line, col) = pair.line_col();
        Integer {
            value: pair.as_str().parse::<i64>().unwrap(),
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
