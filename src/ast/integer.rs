use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone)]
pub struct Integer {
    pub value: i64,
    pub position: Position,
}

impl Integer {
    pub fn from_pair(pair: Pair<Rule>) -> Integer {
        assert_eq!(pair.as_rule(), Rule::integer);
        Integer {
            value: pair.as_str().parse::<i64>().unwrap(),
            position: pair.line_col(),
        }
    }
}
