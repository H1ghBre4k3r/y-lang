use super::Rule;

use pest::iterators::Pair;

use super::{Assignment, Declaration, If};

#[derive(Debug, Clone)]
pub enum Intrinsic {
    If(If),
    Declaration(Declaration),
    Assignment(Assignment),
}

impl Intrinsic {
    pub fn from_pair(pair: Pair<Rule>) -> Intrinsic {
        match pair.as_rule() {
            Rule::ifStmt => Intrinsic::If(If::from_pair(pair)),
            Rule::declaration => Intrinsic::Declaration(Declaration::from_pair(pair)),
            Rule::assignment => Intrinsic::Assignment(Assignment::from_pair(pair)),
            _ => panic!("Unexpected intrinsic '{:#?}'", pair),
        }
    }
}