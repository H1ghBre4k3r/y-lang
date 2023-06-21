use pest::iterators::Pair;
use tracing::trace;

use super::{Block, Expression, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct If<T> {
    pub condition: Box<Expression<T>>,
    pub if_block: Block<T>,
    pub else_block: Option<Block<T>>,
    pub position: Position,
    pub info: T,
}

impl If<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> If<()> {
        assert_eq!(pair.as_rule(), Rule::ifStmt);
        trace!("creating If from pair '{pair}'");

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();
        let condition = Expression::from_pair(inner.next().unwrap(), file);
        let if_block = inner.next().unwrap();
        let else_block = inner.next().map(|block| Block::from_pair(block, file));

        If {
            condition: Box::new(condition),
            if_block: Block::from_pair(if_block, file),
            else_block,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
