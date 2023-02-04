use super::{Position, Rule};
use pest::iterators::Pair;
use unescape::unescape;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Str {
    pub value: String,
    pub position: Position,
}

impl Str {
    pub fn from_pair(pair: Pair<Rule>) -> Str {
        assert_eq!(pair.as_rule(), Rule::string);
        Str {
            value: unescape(pair.clone().into_inner().next().unwrap().as_str())
                .expect("Invalid character escaped"),
            position: pair.line_col(),
        }
    }
}
