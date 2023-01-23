use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone)]
pub struct Str {
    pub value: String,
    pub position: Position,
}

impl Str {
    pub fn from_pair(pair: Pair<Rule>) -> Str {
        assert_eq!(pair.as_rule(), Rule::string);
        Str {
            value: pair
                .clone()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .to_owned(),
            position: pair.line_col(),
        }
    }
}
