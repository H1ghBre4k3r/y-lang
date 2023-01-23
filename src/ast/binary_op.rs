use pest::iterators::Pair;

use super::{BinaryVerb, Expression, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryOp {
    pub verb: BinaryVerb,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
    pub position: Position,
}

impl BinaryOp {
    pub fn from_pair(pair: Pair<Rule>) -> BinaryOp {
        assert_eq!(pair.as_rule(), Rule::binaryExpr);

        let mut inner = pair.clone().into_inner();

        let lhs = Expression::from_pair(inner.next().unwrap());

        let verb = inner.next().expect(&format!(
            "Expected verb in binary expression '{}' at {}:{}",
            pair.as_str(),
            pair.line_col().0,
            pair.line_col().1,
        ));

        let verb = verb
            .as_str()
            .parse::<BinaryVerb>()
            .expect("Invalid binary verb");

        let rhs = Expression::from_pair(inner.next().unwrap());
        BinaryOp {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            verb,
            position: pair.line_col(),
        }
    }
}
