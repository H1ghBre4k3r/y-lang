use pest::iterators::Pair;

use super::{Expression, Position, PrefixOp, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrefixExpr<T> {
    pub op: PrefixOp,
    pub rhs: Box<Expression<T>>,
    pub position: Position,
    pub info: T,
}

impl PrefixExpr<()> {
    pub fn from_op_rhs(op_pair: Pair<Rule>, rhs: Expression<()>, file: &str) -> PrefixExpr<()> {
        let (line, col) = op_pair.line_col();

        let op = PrefixOp::from(op_pair.as_rule());

        PrefixExpr {
            op,
            rhs: Box::new(rhs),
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
