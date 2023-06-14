use log::trace;
use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Boolean<T> {
    pub position: Position,
    pub value: bool,
    pub info: T,
}

impl Boolean<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Boolean<()> {
        assert_eq!(pair.as_rule(), Rule::boolean);
        trace!("creating Boolean from pair '{pair:?}'");

        let (line, col) = pair.line_col();
        Boolean {
            value: pair.as_str().parse::<bool>().unwrap(),
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
