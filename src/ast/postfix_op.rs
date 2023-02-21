use pest::iterators::Pair;

use super::{Rule, Call};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PostfixOp<T> {
    Call(Call<T>),
}

impl PostfixOp<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> PostfixOp<()> {
        match pair.as_rule() {
            Rule::call => PostfixOp::Call(Call::from_pair(pair, file)),
            rule => unreachable!("Unexpected rule {:?} while parsing postfix op", rule),
        }
    }
}
