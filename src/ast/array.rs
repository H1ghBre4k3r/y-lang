use log::trace;
use pest::iterators::Pair;

use super::{Expression, Integer, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Array<T> {
    pub initializer: Box<Expression<T>>,
    pub size: Integer<()>,
    pub position: Position,
    pub info: T,
}

impl Array<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Array<()> {
        assert_eq!(pair.as_rule(), Rule::array);
        trace!("creating Array from pair '{pair}'");

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let initializer = inner.next().unwrap();
        let initializer = Expression::from_pair(initializer, file);

        let size = inner.next().unwrap();
        let size = Integer::from_pair(size, file);

        Array {
            initializer: Box::new(initializer),
            size,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
