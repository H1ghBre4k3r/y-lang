use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Boolean<T> {
    pub position: Position,
    pub value: bool,
    pub info: T,
}

impl Boolean<()> {
    pub fn from_pair(pair: Pair<Rule>) -> Boolean<()> {
        assert_eq!(pair.as_rule(), Rule::boolean);
        Boolean {
            value: pair.as_str().parse::<bool>().unwrap(),
            position: pair.line_col(),
            info: (),
        }
    }
}
