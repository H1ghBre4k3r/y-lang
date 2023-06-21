use pest::iterators::Pair;
use tracing::trace;

use super::{Expression, Position, PostfixOp, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostfixExpr<T> {
    pub op: PostfixOp<T>,
    pub lhs: Box<Expression<T>>,
    pub position: Position,
    pub info: T,
}

impl PostfixExpr<()> {
    pub fn from_lhs_op(lhs: Expression<()>, op_pair: Pair<Rule>, file: &str) -> PostfixExpr<()> {
        trace!("creating PostfixExpr from lhs '{lhs:?}' and op_pair '{op_pair}");

        let (line, col) = op_pair.line_col();

        let op = PostfixOp::from_pair(op_pair, file);

        PostfixExpr {
            op,
            lhs: Box::new(lhs),
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
