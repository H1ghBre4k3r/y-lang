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

impl<'a> PartialEq for Comb<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Node { .. }, Self::Node { .. }) => false,
            (Self::Single { token: l_token }, Self::Single { token: r_token }) => {
                l_token == r_token
            }
            (
                Self::Sequence {
                    current: l_current,
                    next: l_next,
                },
                Self::Sequence {
                    current: r_current,
                    next: r_next,
                },
            ) => l_current == r_current && l_next == r_next,
            (
                Self::Either {
                    left: l_left,
                    right: l_right,
                },
                Self::Either {
                    left: r_left,
                    right: r_right,
                },
            ) => l_left == r_left && l_right == r_right,
            _ => false,
        }
    }
}

impl<'a> std::fmt::Debug for Comb<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node { .. } => f
                .debug_struct("Node")
                .field("parser", &"() -> {}".to_string())
                .finish(),
            Self::Single { token } => f.debug_struct("Single").field("token", token).finish(),
            Self::Sequence { current, next } => f
                .debug_struct("Sequence")
                .field("current", current)
                .field("next", next)
                .finish(),
            Self::Either { left, right } => f
                .debug_struct("Either")
                .field("left", left)
                .field("right", right)
                .finish(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shr_simple() {
        let left = Comb::LET;
        let right = Comb::EQ;
        let new = left >> right;

        assert_eq!(
            Comb::Sequence {
                current: Box::new(Comb::LET),
                next: Box::new(Comb::EQ)
            },
            new
        );
    }

    #[test]
    fn test_shr_complex() {
        let a = Comb::LET;
        let b = Comb::EQ;
        let c = Comb::SEMI;
        let new = a >> b >> c;

        assert_eq!(
            Comb::Sequence {
                current: Box::new(Comb::Sequence {
                    current: Box::new(Comb::LET),
                    next: Box::new(Comb::EQ),
                }),
                next: Box::new(Comb::SEMI)
            },
            new
        );
    }

    #[test]
    fn test_bitor_simple() {
        let left = Comb::LET;
        let right = Comb::EQ;
        let new = left | right;

        assert_eq!(
            Comb::Either {
                left: Box::new(Comb::LET),
                right: Box::new(Comb::EQ)
            },
            new
        );
    }

    #[test]
    fn test_bitor_complex() {
        let a = Comb::LET;
        let b = Comb::EQ;
        let c = Comb::SEMI;
        let new = a | b | c;

        assert_eq!(
            Comb::Either {
                left: Box::new(Comb::Either {
                    left: Box::new(Comb::LET),
                    right: Box::new(Comb::EQ),
                }),
                right: Box::new(Comb::SEMI)
            },
            new
        );
    }

    #[test]
    fn test_parse_single_simple() {
        let a = Comb::LET;
        let mut tokens = vec![Token::Let { position: (0, 0) }].into();
        let result = a.parse(&mut tokens);

        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_node_simple() {
        let a = Comb::NUM;
        let mut tokens = vec![Token::Num {
            value: 42,
            position: (0, 0),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(Ok(vec![AstNode::Num(Num(42))]), result);
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_shr() {
        let matcher = Comb::LET >> Comb::NUM;
        let mut tokens = vec![
            Token::Let { position: (0, 0) },
            Token::Num {
                value: 42,
                position: (0, 0),
            },
        ]
        .into();
        let result = matcher.parse(&mut tokens);
        assert_eq!(Ok(vec![AstNode::Num(Num(42))]), result);
        assert_eq!(tokens.get_index(), 2);
    }

    #[test]
    fn test_parse_bitor() {
        let matcher = Comb::ID | Comb::NUM;
        let mut tokens = vec![Token::Num {
            value: 42,
            position: (0, 0),
        }]
        .into();
        let result = matcher.parse(&mut tokens);

        assert_eq!(Ok(vec![AstNode::Num(Num(42))]), result);
        assert_eq!(tokens.get_index(), 1);

        let mut tokens = vec![Token::Id {
            value: "some_id".into(),
            position: (0, 0),
        }]
        .into();
        let result = matcher.parse(&mut tokens);
        assert_eq!(Ok(vec![AstNode::Id(Id("some_id".into()))]), result);
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_simple_error() {
        let a = Comb::LET;
        let mut tokens = vec![Token::Num {
            value: 42,
            position: (0, 0),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert!(result.is_err());
        assert_eq!(tokens.get_index(), 1);
    }
}
