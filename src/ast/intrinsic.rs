use super::{Declaration, Rule, WhileLoop};

use pest::iterators::Pair;

use super::{Assignment, Definition};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intrinsic<T> {
    Declaration(Declaration),
    Definition(Definition<T>),
    Assignment(Assignment<T>),
    WhileLoop(WhileLoop<T>),
}

impl Intrinsic<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Intrinsic<()> {
        match pair.as_rule() {
            Rule::declaration => Intrinsic::Declaration(Declaration::from_pair(pair, file)),
            Rule::definition => Intrinsic::Definition(Definition::from_pair(pair, file)),
            Rule::assignment => Intrinsic::Assignment(Assignment::from_pair(pair, file)),
            Rule::whileLoop => Intrinsic::WhileLoop(WhileLoop::from_pair(pair, file)),
            _ => panic!("Unexpected intrinsic '{pair:#?}'"),
        }
    }
}

impl<T> Intrinsic<T>
where
    T: Clone,
{
    pub fn info(&self) -> T {
        match self {
            Intrinsic::Definition(Definition { info, .. })
            | Intrinsic::Assignment(Assignment { info, .. }) => info.clone(),
            _ => unimplemented!(),
        }
    }
}
