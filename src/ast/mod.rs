mod node;
mod parser;

use pest::iterators::Pairs;

use self::{node::AstNode, parser::Rule};

pub use self::parser::*;

#[derive(Debug)]
pub struct Ast {
    inner: Vec<AstNode>,
}

impl Ast {
    pub fn from_program(program: Pairs<Rule>) -> Ast {
        let mut ast = vec![];

        for statement in program {
            if statement.as_rule() != Rule::EOI {
                ast.push(AstNode::from_statement(statement));
            }
        }
        Self { inner: ast }
    }

    pub fn inner(&self) -> Vec<AstNode> {
        self.inner.clone()
    }
}
