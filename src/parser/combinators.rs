use std::ops::{BitOr, BitXor, Not, Shr};

use crate::lexer::{Token, Tokens};

use super::{
    ast::{AstNode, Expression, Id, Initialization, Num, Statement},
    FromTokens, ParseError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminal {
    Eq,
    Let,
    Semicolon,
    Plus,
    Times,
}

impl PartialEq<Token> for Terminal {
    fn eq(&self, other: &Token) -> bool {
        matches!(
            (self, other),
            (Terminal::Eq, Token::Eq { .. })
                | (Terminal::Let, Token::Let { .. })
                | (Terminal::Semicolon, Token::Semicolon { .. })
                | (Terminal::Plus, Token::Plus { .. })
                | (Terminal::Times, Token::Times { .. })
        )
    }
}

#[derive(Clone)]
pub enum Comb<'a> {
    /// Combinator for parsing a non terminal symbol. Therefore, we utilize the parsing function of
    /// this respective non-terminal.
    Node {
        parser: &'a dyn Fn(&mut Tokens) -> Result<AstNode, ParseError>,
    },
    /// Combinator for matching a terminal.
    Terminal { token: Terminal },
    /// Combinator for matching a sequence of two other combinators.
    ///
    /// Note: This will nest arbitrary deep
    Sequence {
        current: Box<Comb<'a>>,
        next: Box<Comb<'a>>,
    },
    /// Combinator for parsing either the left or the right combinator.
    ///
    /// Note: It will try to parse the left combinator FIRST.
    Either {
        left: Box<Comb<'a>>,
        right: Box<Comb<'a>>,
    },
    /// Combinator for optinally parsing another combinator. If the contained combinator does not
    /// match, it is just ignored (and the tokens are not touched).
    Optional { inner: Box<Comb<'a>> },
    /// Combinator for parsing an arbitrary repitition of another combinator. If amount is 0, the
    /// combinator will consume as many tokens as the inner combinator matches.
    Repitition {
        inner: Box<Comb<'a>>,
        amount: Option<usize>,
    },
}

impl<'a> PartialEq for Comb<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Node { .. }, Self::Node { .. }) => false,
            (Self::Terminal { token: l_token }, Self::Terminal { token: r_token }) => {
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
            (Self::Optional { inner: l_inner }, Self::Optional { inner: r_inner }) => {
                l_inner == r_inner
            }
            (
                Self::Repitition {
                    inner: l_inner,
                    amount: l_amount,
                },
                Self::Repitition {
                    inner: r_inner,
                    amount: r_amount,
                },
            ) => l_inner == r_inner && l_amount == r_amount,
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
            Self::Terminal { token } => f.debug_struct("Single").field("token", token).finish(),
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
            Self::Optional { inner } => f.debug_struct("Optional").field("inner", inner).finish(),
            Self::Repitition { inner, amount } => f
                .debug_struct("Repitition")
                .field("inner", inner)
                .field("amount", amount)
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

    pub const LET: Comb<'static> = Comb::Terminal {
        token: Terminal::Let,
    };

    pub const EQ: Comb<'static> = Comb::Terminal {
        token: Terminal::Eq,
    };

    pub const PLUS: Comb<'static> = Comb::Terminal {
        token: Terminal::Plus,
    };

    pub const TIMES: Comb<'static> = Comb::Terminal {
        token: Terminal::Times,
    };

    pub const SEMI: Comb<'static> = Comb::Terminal {
        token: Terminal::Semicolon,
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
            Comb::Terminal { token } => {
                let Some(t) = tokens.next() else {
                    return Err(ParseError {
                        message: "Reached EOF!".into(),
                        position: None,
                    });
                };

                // try to parse the terminal
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
            Comb::Optional { inner } => {
                let current_index = tokens.get_index();
                if let Ok(mut result) = inner.parse(tokens) {
                    matched.append(&mut result);
                } else {
                    tokens.set_index(current_index);
                }
            }
            Comb::Repitition { inner, amount } => {
                // make a case distinction on the amount
                if let Some(amount) = amount {
                    // match exactly the specified amount of tokens
                    for _ in 0..*amount {
                        let mut result = inner.parse(tokens)?;
                        matched.append(&mut result);
                    }
                } else {
                    // match an arbitrary amount of tokens
                    let mut current_index = tokens.get_index();
                    while let Ok(mut result) = inner.parse(tokens) {
                        matched.append(&mut result);
                        current_index = tokens.get_index();
                    }
                    tokens.set_index(current_index);
                }
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

impl<'a> Not for Comb<'a> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Comb::Optional {
            inner: Box::new(self),
        }
    }
}

impl<'a> BitXor<()> for Comb<'a> {
    type Output = Self;

    fn bitxor(self, _rhs: ()) -> Self::Output {
        Comb::Repitition {
            inner: Box::new(self),
            amount: None,
        }
    }
}

impl<'a> BitXor<usize> for Comb<'a> {
    type Output = Self;

    fn bitxor(self, rhs: usize) -> Self::Output {
        Comb::Repitition {
            inner: Box::new(self),
            amount: Some(rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_simple() {
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
    fn test_sequence_complex() {
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
    fn test_either_simple() {
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
    fn test_either_complex() {
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
    fn test_optional_simple() {
        let a = !Comb::LET;

        assert_eq!(
            Comb::Optional {
                inner: Box::new(Comb::LET)
            },
            a
        )
    }

    #[test]
    fn test_parse_optional_matching_terminal() {
        let a = !Comb::LET;
        let mut tokens = vec![Token::Let { position: (0, 0) }].into();
        let result = a.parse(&mut tokens);

        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_optional_not_matching_terminal() {
        let a = !Comb::LET;
        let mut tokens = vec![Token::Eq { position: (0, 0) }].into();
        let result = a.parse(&mut tokens);

        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 0);
    }

    #[test]
    fn test_parse_optional_matching_node() {
        let a = !Comb::NUM;
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
    fn test_parse_optional_not_matching_node() {
        let a = !Comb::NUM;
        let mut tokens = vec![Token::Id {
            value: "some_id".into(),
            position: (0, 0),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 0);
    }

    #[test]
    fn test_repition_simple() {
        assert_eq!(
            Comb::Repitition {
                inner: Box::new(Comb::LET),
                amount: Some(5)
            },
            Comb::LET ^ 5
        );
        assert_eq!(
            Comb::Repitition {
                inner: Box::new(Comb::LET),
                amount: None
            },
            Comb::LET ^ ()
        );
    }

    #[test]
    fn test_parse_repition_simple_matching() {
        let a = Comb::LET ^ 5;
        let mut tokens = vec![
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
        ]
        .into();
        let result = a.parse(&mut tokens);
        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 5);
    }

    #[test]
    fn test_parse_repition_simple_not_matching() {
        let a = Comb::LET ^ 5;
        let mut tokens = vec![
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
            Token::Eq { position: (0, 0) },
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
        ]
        .into();
        let result = a.parse(&mut tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repition_simple_wildcard() {
        let a = Comb::LET ^ ();
        let mut tokens = vec![
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
            Token::Eq { position: (0, 0) },
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
        ]
        .into();
        let result = a.parse(&mut tokens);
        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 2);
    }

    #[test]
    fn test_parse_repition_node() {
        let a = Comb::NUM ^ 3;
        let mut tokens = vec![
            Token::Num {
                value: 42,
                position: (0, 0),
            },
            Token::Num {
                value: 1337,
                position: (0, 0),
            },
            Token::Num {
                value: 17,
                position: (0, 0),
            },
        ]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(
            Ok(vec![
                AstNode::Num(Num(42)),
                AstNode::Num(Num(1337)),
                AstNode::Num(Num(17))
            ]),
            result
        );
        assert_eq!(tokens.get_index(), 3);
    }

    #[test]
    fn test_parse_repition_node_wildcard() {
        let a = Comb::NUM ^ ();
        let mut tokens = vec![
            Token::Num {
                value: 42,
                position: (0, 0),
            },
            Token::Num {
                value: 1337,
                position: (0, 0),
            },
            Token::Num {
                value: 17,
                position: (0, 0),
            },
            Token::Let { position: (0, 0) },
            Token::Let { position: (0, 0) },
        ]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(
            Ok(vec![
                AstNode::Num(Num(42)),
                AstNode::Num(Num(1337)),
                AstNode::Num(Num(17))
            ]),
            result
        );
        assert_eq!(tokens.get_index(), 3);
    }

    #[test]
    fn test_parse_terminal_simple() {
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
