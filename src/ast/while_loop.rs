use pest::iterators::Pair;

use super::{Block, Expression, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WhileLoop<T> {
    pub condition: Expression<T>,
    pub block: Block<T>,
    pub position: Position,
    pub info: T,
}

impl WhileLoop<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> WhileLoop<()> {
        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let condition = Expression::from_pair(
            inner.next().unwrap_or_else(|| {
                panic!("Expected expression in while loop header at {line}:{col}")
            }),
            file,
        );

        let block = Block::from_pair(
            inner.next().unwrap_or_else(|| {
                panic!("Expected expression in while loop header at {line}:{col}")
            }),
            file,
        );

        WhileLoop {
            condition,
            block,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
