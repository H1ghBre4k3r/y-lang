use std::ops::{BitOr, Shr};

use crate::lexer::{Token, Tokens};

use super::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Matchable {
    Eq,
    Let,
    Id,
    Num,
    Semicolon,
}

impl PartialEq<Token> for Matchable {
    fn eq(&self, other: &Token) -> bool {
        matches!(
            (self, other),
            (Matchable::Eq, Token::Eq { .. })
                | (Matchable::Let, Token::Let { .. })
                | (Matchable::Id, Token::Id { .. })
                | (Matchable::Num, Token::Num { .. })
                | (Matchable::Semicolon, Token::Semicolon { .. })
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comb {
    Single {
        token: Matchable,
        should_yield: bool,
    },
    Sequence {
        current: Box<Comb>,
        next: Box<Comb>,
    },
    Either {
        left: Box<Comb>,
        right: Box<Comb>,
    },
}

impl Comb {
    pub const ID: Comb = Comb::Single {
        token: Matchable::Id,
        should_yield: true,
    };

    pub const LET: Comb = Comb::Single {
        token: Matchable::Let,
        should_yield: false,
    };

    pub const NUM: Comb = Comb::Single {
        token: Matchable::Num,
        should_yield: true,
    };

    pub const EQ: Comb = Comb::Single {
        token: Matchable::Eq,
        should_yield: false,
    };
    pub const SEMI: Comb = Comb::Single {
        token: Matchable::Semicolon,
        should_yield: false,
    };

    pub fn parse(&self, tokens: &mut Tokens) -> Result<Vec<Token>, ParseError> {
        let mut matched = vec![];
        match self {
            Comb::Single {
                token,
                should_yield,
            } => {
                let Some(t) = tokens.next() else {
                    return Err(ParseError {
                        message: "Reached EOF!".into(),
                        position: None,
                    });
                };

                if *token != t {
                    return Err(ParseError {
                        message: format!("encountered {:?} while trying to parse {:?}", t, token),
                        position: None,
                    });
                }

                if *should_yield {
                    matched.push(t);
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
        }

        Ok(matched)
    }
}

impl Shr for Comb {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Comb::Sequence {
            current: Box::new(self),
            next: Box::new(rhs),
        }
    }
}

impl BitOr for Comb {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Comb::Either {
            left: Box::new(self),
            right: Box::new(rhs),
        }
    }
}
