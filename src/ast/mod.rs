mod node;
mod parser;

use pest::iterators::Pairs;

use self::parser::Rule;

pub use self::node::*;
pub use self::parser::*;

#[derive(Debug)]
pub struct Ast {
    nodes: Vec<AstNode>,
}

impl Ast {
    pub fn from_program(program: Pairs<Rule>) -> Ast {
        let mut ast = vec![];

        for statement in program {
            if statement.as_rule() != Rule::EOI {
                ast.push(AstNode::from_statement(statement));
            }
        }
        Self { nodes: ast }
    }

    pub fn nodes(&self) -> Vec<AstNode> {
        self.nodes.clone()
    }
}
