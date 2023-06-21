use pest::iterators::Pair;
use tracing::trace;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ident<T> {
    pub value: String,
    pub position: Position,
    pub info: T,
}

impl Ident<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Ident<()> {
        trace!("creating Ident from pair '{pair}'");

        let (line, col) = pair.line_col();
        Ident {
            value: pair.as_str().to_owned(),
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
