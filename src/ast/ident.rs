use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone)]
pub struct Ident {
    pub value: String,
    pub position: Position,
}

impl Ident {
    pub fn from_pair(pair: Pair<Rule>) -> Ident {
        assert_eq!(pair.as_rule(), Rule::ident);
        Ident {
            value: pair.as_str().to_owned(),
            position: pair.line_col(),
        }
    }
}
