use pest::iterators::Pair;

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
    pub fn from_pair(pair: Pair<Rule>) -> If<()> {
        assert_eq!(pair.as_rule(), Rule::ifStmt);

        let position = pair.line_col();

        let mut inner = pair.into_inner();
        let condition = Expression::from_pair(inner.next().unwrap());
        let if_block = inner.next().unwrap();
        let else_block = inner.next().map(Block::from_pair);

        If {
            condition: Box::new(condition),
            if_block: Block::from_pair(if_block),
            else_block,
            position,
            info: (),
        }
    }
}
