use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ident<T> {
    pub value: String,
    pub position: Position,
    pub info: T,
}

impl Ident<()> {
    pub fn from_pair(pair: Pair<Rule>) -> Ident<()> {
        assert_eq!(pair.as_rule(), Rule::ident);
        Ident {
            value: pair.as_str().to_owned(),
            position: pair.line_col(),
            info: (),
        }
    }
}
