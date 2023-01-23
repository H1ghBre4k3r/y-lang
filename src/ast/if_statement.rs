use pest::iterators::Pair;

use super::{Block, Expression, Position, Rule};

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Expression>,
    pub if_block: Block,
    pub else_block: Option<Block>,
    pub position: Position,
}

impl If {
    pub fn from_pair(pair: Pair<Rule>) -> If {
        assert_eq!(pair.as_rule(), Rule::ifStmt);

        let position = pair.line_col();

        let mut inner = pair.into_inner();
        let condition = Expression::from_pair(inner.next().unwrap());
        let if_block = inner.next().unwrap();
        let else_block = inner.next().map(|block| Block::from_pair(block));

        If {
            condition: Box::new(condition),
            if_block: Block::from_pair(if_block),
            else_block,
            position,
        }
    }
}
