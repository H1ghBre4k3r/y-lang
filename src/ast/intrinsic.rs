use super::Rule;

use pest::iterators::Pair;

use super::{Assignment, Definition};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intrinsic {
    Definition(Definition),
    Assignment(Assignment),
}

impl Intrinsic {
    pub fn from_pair(pair: Pair<Rule>) -> Intrinsic {
        match pair.as_rule() {
            Rule::definition => Intrinsic::Definition(Definition::from_pair(pair)),
            Rule::assignment => Intrinsic::Assignment(Assignment::from_pair(pair)),
            _ => panic!("Unexpected intrinsic '{pair:#?}'"),
        }
    }
}
