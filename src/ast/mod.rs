//! Module for parsing Y programs.
//!
//! It contains all structs for the internal representation of Y (i.e., the AST).
mod array;
mod assignment;
mod binary_expr;
mod binary_op;
mod block;
mod boolean;
mod call;
mod character;
mod compiler_directive;
mod declaration;
mod definition;
mod expression;
mod fn_def;
mod ident;
mod if_statement;
mod import;
mod indexing;
mod inline_asm;
mod integer;
mod intrinsic;
mod param;
mod parse_error;
mod parser;
mod postfix_expr;
mod postfix_op;
mod prefix_expr;
mod prefix_op;
mod statement;
mod str;
mod type_annotation;
mod types;
mod while_loop;

pub use self::array::*;
pub use self::assignment::*;
pub use self::binary_expr::*;
pub use self::binary_op::*;
pub use self::block::*;
pub use self::boolean::*;
pub use self::call::*;
pub use self::character::*;
pub use self::compiler_directive::*;
pub use self::declaration::*;
pub use self::definition::*;
pub use self::expression::*;
pub use self::fn_def::*;
pub use self::ident::*;
pub use self::if_statement::*;
pub use self::import::*;
pub use self::indexing::*;
pub use self::inline_asm::*;
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
pub use self::while_loop::*;

use pest::iterators::Pair;
use tracing::trace;

pub use self::parser::Rule;

pub use self::parser::*;

/// A position within a file (i.e., line and column)
pub type Position = (String, usize, usize);

/// AST, representing a single Y program.
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ast<T> {
    /// Nodes within this AST.
    nodes: Vec<Statement<T>>,
}

impl Ast<()> {
    /// Create a new AST from a given pair of rules.
    /// Note: This AST is not type-correct by default.
    pub fn from_program(program: Vec<Pair<Rule>>, file: &str) -> Ast<()> {
        trace!("creating Ast from programm '{program:?}'");
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
