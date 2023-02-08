use super::Rule;

use pest::iterators::Pair;

use super::{Assignment, Definition};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intrinsic<T> {
    Definition(Definition<T>),
    Assignment(Assignment<T>),
}

impl Intrinsic<()> {
    pub fn from_pair(pair: Pair<Rule>) -> Intrinsic<()> {
        match pair.as_rule() {
            Rule::definition => Intrinsic::Definition(Definition::from_pair(pair)),
            Rule::assignment => Intrinsic::Assignment(Assignment::from_pair(pair)),
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
        }
    }
}
