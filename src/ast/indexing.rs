use log::trace;
use pest::iterators::Pair;

use super::{Expression, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Indexing<T> {
    pub index: Box<Expression<T>>,
    pub position: Position,
    pub info: T,
}

impl Indexing<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Indexing<()> {
        assert_eq!(pair.as_rule(), Rule::indexing);
        trace!("creating Indexing from pair '{pair}'");

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let index = inner.next().unwrap();
        let index = Expression::from_pair(index, file);

        Indexing {
            index: Box::new(index),
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
