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
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> BinaryExpr<()> {
        assert_eq!(pair.as_rule(), Rule::binaryExpr);

        let (line, col) = pair.line_col();

        let mut inner = pair.clone().into_inner();

        let lhs = Expression::from_pair(inner.next().unwrap(), file);

        let op = inner.next().unwrap_or_else(|| {
            panic!(
                "Expected op in binary expression '{}' at {}:{}",
                pair.as_str(),
                pair.line_col().0,
                pair.line_col().1
            )
        });

        let op = op.as_str().parse::<BinaryOp>().expect("Invalid binary op");

        let rhs = Expression::from_pair(inner.next().unwrap(), file);
        BinaryExpr {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            op,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
