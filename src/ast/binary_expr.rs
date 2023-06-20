use log::trace;
use pest::iterators::Pair;

use super::{BinaryOp, Expression, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryExpr<T> {
    pub op: BinaryOp,
    pub lhs: Box<Expression<T>>,
    pub rhs: Box<Expression<T>>,
    pub position: Position,
    pub info: T,
}

impl BinaryExpr<()> {
    pub fn from_lhs_op_rhs(
        lhs: Expression<()>,
        op_pair: Pair<Rule>,
        rhs: Expression<()>,
        file: &str,
    ) -> BinaryExpr<()> {
        trace!("creating BinaryExpr from lhs '{lhs:?}', op_pair '{op_pair}' and rhs '{rhs:?}'");

        let (line, col) = op_pair.line_col();

        let op = BinaryOp::from(op_pair.as_rule());

        BinaryExpr {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            op,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
