mod assignment;
mod binary_expr;
mod binary_op;
mod block;
mod boolean;
mod call;
mod declaration;
mod definition;
mod expression;
mod fn_def;
mod ident;
mod if_statement;
mod import;
mod integer;
mod intrinsic;
mod param;
mod parser;
mod postfix_expr;
mod postfix_op;
mod prefix_expr;
mod prefix_op;
mod statement;
mod str;
mod type_annotation;
mod types;

use std::fmt::Display;

pub use self::assignment::*;
pub use self::binary_expr::*;
pub use self::binary_op::*;
pub use self::block::*;
pub use self::boolean::*;
pub use self::call::*;
pub use self::declaration::*;
pub use self::definition::*;
pub use self::expression::*;
pub use self::fn_def::*;
pub use self::ident::*;
pub use self::if_statement::*;
pub use self::import::*;
pub use self::integer::*;
pub use self::intrinsic::*;
pub use self::param::*;
pub use self::parser::*;
pub use self::postfix_expr::*;
pub use self::postfix_op::*;
pub use self::prefix_expr::*;
pub use self::prefix_op::*;
pub use self::statement::*;
pub use self::str::*;
pub use self::type_annotation::*;
pub use self::types::*;

use pest::iterators::Pair;

pub use self::parser::Rule;

pub use self::parser::*;

pub type Position = (String, usize, usize);

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ast<T> {
    nodes: Vec<Statement<T>>,
}

impl Ast<()> {
    pub fn from_program(program: Vec<Pair<Rule>>, file: &str) -> Ast<()> {
        let mut ast = vec![];

        for statement in program {
            if statement.as_rule() != Rule::EOI {
                ast.push(Statement::from_pair(statement, file));
            }
        }
        Self { nodes: ast }
    }
}

impl<T> Ast<T>
where
    T: Clone,
{
    pub fn from_nodes(nodes: Vec<Statement<T>>) -> Ast<T> {
        Self { nodes }
    }

    pub fn nodes(&self) -> Vec<Statement<T>> {
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
            "{} ({}:{}:{})",
            self.message, self.position.0, self.position.1, self.position.2
        ))
    }
}

/// The result of parsing a pair.
pub type ParseResult<T> = Result<T, ParseError>;
