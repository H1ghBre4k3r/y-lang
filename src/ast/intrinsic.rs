use super::Rule;

use pest::iterators::Pair;

use super::{Assignment, Declaration};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intrinsic {
    Declaration(Declaration),
    Assignment(Assignment),
}

impl Intrinsic {
    pub fn from_pair(pair: Pair<Rule>) -> Intrinsic {
        match pair.as_rule() {
            Rule::declaration => Intrinsic::Declaration(Declaration::from_pair(pair)),
            Rule::assignment => Intrinsic::Assignment(Assignment::from_pair(pair)),
            _ => panic!("Unexpected intrinsic '{pair:#?}'"),
        }
    }
}
