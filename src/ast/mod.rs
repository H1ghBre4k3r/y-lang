mod assignment;
mod binary_op;
mod binary_verb;
mod block;
mod declaration;
mod expression;
mod fn_call;
mod fn_def;
mod ident;
mod if_statement;
mod integer;
mod intrinsic;
mod param;
mod parser;
mod statement;
mod str;
mod type_annotation;

use std::fmt::Display;

pub use self::assignment::*;
pub use self::binary_op::*;
pub use self::binary_verb::*;
pub use self::block::*;
pub use self::declaration::*;
pub use self::expression::*;
pub use self::fn_call::*;
pub use self::fn_def::*;
pub use self::ident::*;
pub use self::if_statement::*;
pub use self::integer::*;
pub use self::intrinsic::*;
pub use self::param::*;
pub use self::parser::*;
pub use self::statement::*;
pub use self::str::*;
pub use self::type_annotation::*;

use pest::iterators::Pairs;

use self::parser::Rule;

pub use self::parser::*;

pub type Position = (usize, usize);

#[derive(Debug)]
pub struct Ast {
    nodes: Vec<Statement>,
}

impl Ast {
    pub fn from_program(program: Pairs<Rule>) -> Ast {
        let mut ast = vec![];

        for statement in program {
            if statement.as_rule() != Rule::EOI {
                ast.push(Statement::from_pair(statement));
            }
        }
        Self { nodes: ast }
    }

    pub fn nodes(&self) -> Vec<Statement> {
        self.nodes.clone()
    }
}

/// Struct representing an error which happened while parsing the code.
#[derive(Clone, Debug)]
pub struct ParseError {
    /// Error message of this parse error
    pub message: String,
    /// Position where this error occured
    pub position: Position,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} ({}:{})",
            self.message, self.position.0, self.position.1
        ))
    }
}

/// The result of parsing a pair.
pub type ParseResult<T> = Result<T, ParseError>;
