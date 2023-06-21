use pest::iterators::Pair;
use tracing::trace;

use super::{Call, Indexing, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PostfixOp<T> {
    Call(Call<T>),
    Indexing(Indexing<T>),
}

impl PostfixOp<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> PostfixOp<()> {
        trace!("creating PostfixOp from pair '{pair}'");
        match pair.as_rule() {
            Rule::call => PostfixOp::Call(Call::from_pair(pair, file)),
            Rule::indexing => PostfixOp::Indexing(Indexing::from_pair(pair, file)),
            rule => unreachable!("Unexpected rule {:?} while parsing postfix op", rule),
        }
    }
}
