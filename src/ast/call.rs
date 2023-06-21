use pest::iterators::Pair;
use tracing::trace;

use super::{Expression, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Call<T> {
    pub params: Vec<Expression<T>>,
    pub position: Position,
    pub info: T,
}

impl Call<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Call<()> {
        assert_eq!(pair.as_rule(), Rule::call);
        trace!("creating Call from pair '{pair}");

        let (line, col) = pair.line_col();

        let inner = pair.into_inner();

        let mut params = vec![];

        for param in inner {
            params.push(Expression::from_pair(param, file));
        }

        Call {
            params,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
