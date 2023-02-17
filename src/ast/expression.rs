use log::error;
use pest::iterators::Pair;

use super::{BinaryExpr, Block, Boolean, FnCall, FnDef, Ident, If, Integer, Position, Rule, Str};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression<T> {
    If(If<T>),
    Binary(BinaryExpr<T>),
    FnCall(FnCall<T>),
    Integer(Integer<T>),
    Ident(Ident<T>),
    Str(Str<T>),
    FnDef(FnDef<T>),
    Block(Block<T>),
    Boolean(Boolean<T>),
}

impl Expression<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Expression<()> {
        match pair.as_rule() {
            Rule::integer => Expression::Integer(Integer::from_pair(pair, file)),
            Rule::ident => Expression::Ident(Ident::from_pair(pair, file)),
            Rule::fnCall => Expression::FnCall(FnCall::from_pair(pair, file)),
            Rule::string => Expression::Str(Str::from_pair(pair, file)),
            Rule::binaryExpr => Expression::Binary(BinaryExpr::from_pair(pair, file)),
            Rule::fnDef => Expression::FnDef(FnDef::from_pair(pair, file)),
            Rule::ifStmt => Expression::If(If::from_pair(pair, file)),
            Rule::block => Expression::Block(Block::from_pair(pair, file)),
            Rule::boolean => Expression::Boolean(Boolean::from_pair(pair, file)),
            _ => {
                error!(
                    "Unexpected expression '{}' at {}:{}",
                    pair.as_str(),
                    pair.line_col().0,
                    pair.line_col().1
                );
                std::process::exit(-1)
            }
        }
    }
}

impl<T> Expression<T>
where
    T: Clone,
{
    pub fn position(&self) -> Position {
        match self {
            Expression::If(If { position, .. })
            | Expression::Binary(BinaryExpr { position, .. })
            | Expression::FnCall(FnCall { position, .. })
            | Expression::Integer(Integer { position, .. })
            | Expression::Ident(Ident { position, .. })
            | Expression::Str(Str { position, .. })
            | Expression::FnDef(FnDef { position, .. })
            | Expression::Block(Block { position, .. })
            | Expression::Boolean(Boolean { position, .. }) => position.to_owned(),
        }
    }

    pub fn info(&self) -> T {
        match self {
            Expression::If(If { info, .. })
            | Expression::Binary(BinaryExpr { info, .. })
            | Expression::FnCall(FnCall { info, .. })
            | Expression::Integer(Integer { info, .. })
            | Expression::Ident(Ident { info, .. })
            | Expression::Str(Str { info, .. })
            | Expression::FnDef(FnDef { info, .. })
            | Expression::Block(Block { info, .. })
            | Expression::Boolean(Boolean { info, .. }) => info.clone(),
        }
    }
}
