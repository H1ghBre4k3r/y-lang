use pest::iterators::Pair;

use super::{Position, Rule, Statement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block {
    pub block: Vec<Statement>,
    pub position: Position,
}

impl Block {
    pub fn from_pair(pair: Pair<Rule>) -> Block {
        assert_eq!(pair.as_rule(), Rule::block);

        let position = pair.line_col();

        let block = pair.into_inner();

        let mut block_ast = vec![];

        for statement in block {
            block_ast.push(Statement::from_pair(statement));
        }

        Block {
            block: block_ast,
            position,
        }
    }
}
