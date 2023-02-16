use pest::iterators::Pair;

use super::{BinaryVerb, Expression, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryOp<T> {
    pub verb: BinaryVerb,
    pub lhs: Box<Expression<T>>,
    pub rhs: Box<Expression<T>>,
    pub position: Position,
    pub info: T,
}

impl BinaryOp<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> BinaryOp<()> {
        assert_eq!(pair.as_rule(), Rule::binaryExpr);

        let (line, col) = pair.line_col();

        let mut inner = pair.clone().into_inner();

        let lhs = Expression::from_pair(inner.next().unwrap(), file);

        let verb = inner.next().unwrap_or_else(|| {
            panic!(
                "Expected verb in binary expression '{}' at {}:{}",
                pair.as_str(),
                pair.line_col().0,
                pair.line_col().1
            )
        });

        let verb = verb
            .as_str()
            .parse::<BinaryVerb>()
            .expect("Invalid binary verb");

        let rhs = Expression::from_pair(inner.next().unwrap(), file);
        BinaryOp {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            verb,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
