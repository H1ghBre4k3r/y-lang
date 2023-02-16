use pest::iterators::Pair;

use super::{Position, Rule, Statement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block<T> {
    pub block: Vec<Statement<T>>,
    pub position: Position,
    pub info: T,
}

impl Block<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Block<()> {
        assert_eq!(pair.as_rule(), Rule::block);

        let (line, col) = pair.line_col();

        let block = pair.into_inner();

        let mut block_ast = vec![];

        for statement in block {
            block_ast.push(Statement::from_pair(statement, file));
        }

        Block {
            block: block_ast,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
