use std::ops::{BitOr, Shr};

use crate::lexer::{Token, Tokens};

use super::{
    ast::{AstNode, Expression, Id, Initialization, Num, Statement},
    FromTokens, ParseError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Matchable {
    Eq,
    Let,
    Semicolon,
}

impl PartialEq<Token> for Matchable {
    fn eq(&self, other: &Token) -> bool {
        matches!(
            (self, other),
            (Matchable::Eq, Token::Eq { .. })
                | (Matchable::Let, Token::Let { .. })
                | (Matchable::Semicolon, Token::Semicolon { .. })
        )
    }
}

#[derive(Clone)]
pub enum Comb<'a> {
    Node {
        parser: &'a dyn Fn(&mut Tokens) -> Result<AstNode, ParseError>,
    },
    Single {
        token: Matchable,
    },
    Sequence {
        current: Box<Comb<'a>>,
        next: Box<Comb<'a>>,
    },
    Either {
        left: Box<Comb<'a>>,
        right: Box<Comb<'a>>,
    },
}

impl<'a> Comb<'a> {
    pub const ID: Comb<'static> = Comb::Node { parser: &Id::parse };

    pub const NUM: Comb<'static> = Comb::Node {
        parser: &Num::parse,
    };

    pub const EXPR: Comb<'static> = Comb::Node {
        parser: &Expression::parse,
    };

    pub const LET: Comb<'static> = Comb::Single {
        token: Matchable::Let,
    };

    pub const EQ: Comb<'static> = Comb::Single {
        token: Matchable::Eq,
    };

    pub const SEMI: Comb<'static> = Comb::Single {
        token: Matchable::Semicolon,
    };

    pub const STATEMENT: Comb<'static> = Comb::Node {
        parser: &Statement::parse,
    };

    pub const INITIALIZATION: Comb<'static> = Comb::Node {
        parser: &Initialization::parse,
    };

    pub fn parse(&self, tokens: &mut Tokens) -> Result<Vec<AstNode>, ParseError> {
        let mut matched = vec![];
        match self {
            Comb::Single { token } => {
                let Some(t) = tokens.next() else {
                    return Err(ParseError {
                        message: "Reached EOF!".into(),
                        position: None,
                    });
                };

                if *token != t {
                    return Err(ParseError {
                        message: format!("Unexpected {:?} while trying to parse {:?}", t, token),
                        position: None,
                    });
                }
            }
            Comb::Sequence { current, next } => {
                let mut current_matches = current.parse(tokens)?;
                matched.append(&mut current_matches);

                let mut next_matches = next.parse(tokens)?;
                matched.append(&mut next_matches);
            }
            Comb::Either { left, right } => {
                let current_index = tokens.get_index();

                if let Ok(mut left_matches) = left.parse(tokens) {
                    matched.append(&mut left_matches);
                } else {
                    tokens.set_index(current_index);
                    let mut right_matches = right.parse(tokens)?;
                    matched.append(&mut right_matches);
                }
            }
            Comb::Node { parser } => {
                let matches = parser(tokens)?;
                matched.push(matches);
            }
        }

        Ok(matched)
    }
}

impl<'a> Shr for Comb<'a> {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Comb::Sequence {
            current: Box::new(self),
            next: Box::new(rhs),
        }
    }
}

impl<'a> BitOr for Comb<'a> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Comb::Either {
            left: Box::new(self),
            right: Box::new(rhs),
        }
    }
}
