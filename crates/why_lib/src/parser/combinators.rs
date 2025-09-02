use std::ops::{BitOr, BitXor, Not, Rem, Shr};

use super::{
    ast::{
        Array, Assignment, AstNode, Block, Constant, Declaration, Expression, Function,
        FunctionParameter, Id, If, Initialisation, Instance, Lambda, LambdaParameter,
        MethodDeclaration, Num, Statement, StructDeclaration, StructFieldDeclaration,
        StructFieldInitialisation, StructInitialisation, TypeName, WhileLoop,
    },
    FromTokens, ParseError, ParseState,
};
use crate::lexer::{GetPosition, Terminal, Token};
use crate::parser::ast::{AstString, Character};

#[derive(Clone)]
pub enum Comb<'a, Tok, Term, Node> {
    /// Combinator for parsing a non terminal symbol. Therefore, we utilize the parsing function of
    /// this respective non-terminal.
    Node {
        parser: &'a dyn Fn(&mut ParseState<Tok>) -> Result<Node, ParseError>,
    },
    /// Combinator for matching a terminal.
    Terminal { token: Term },
    /// Combinator for matching a sequence of two other combinators.
    ///
    /// Note: This will nest arbitrary deep
    Sequence {
        current: Box<Comb<'a, Tok, Term, Node>>,
        next: Box<Comb<'a, Tok, Term, Node>>,
    },
    /// Combinator for parsing either the left or the right combinator.
    ///
    /// Note: It will try to parse the left combinator FIRST.
    Either {
        left: Box<Comb<'a, Tok, Term, Node>>,
        right: Box<Comb<'a, Tok, Term, Node>>,
    },
    /// Combinator for optinally parsing another combinator. If the contained combinator does not
    /// match, it is just ignored (and the tokens are not touched).
    Optional {
        inner: Box<Comb<'a, Tok, Term, Node>>,
    },
    /// Combinator for parsing an arbitrary repitition of another combinator. If amount is 0, the
    /// combinator will consume as many tokens as the inner combinator matches.
    Repitition {
        inner: Box<Comb<'a, Tok, Term, Node>>,
        amount: Option<usize>,
    },
    /// Combinator for parsing an repititions of another combinator until "closing" matches.
    RepeatUntil {
        repeated: Box<Comb<'a, Tok, Term, Node>>,
        closing: Box<Comb<'a, Tok, Term, Node>>,
    },
}

impl<Tok, Term, Node> PartialEq for Comb<'_, Tok, Term, Node>
where
    Term: PartialEq,
{
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

impl<Tok, Term, Node> std::fmt::Debug for Comb<'_, Tok, Term, Node>
where
    Term: std::fmt::Debug,
{
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
            Self::RepeatUntil { repeated, closing } => f
                .debug_struct("RepeatUntil")
                .field("repeated", repeated)
                .field("closing", closing)
                .finish(),
        }
    }
}

/// Create a combinator for a specified terminal symbol.
#[macro_export]
macro_rules! terminal_comb {
    ($name:ident, $terminal:ident) => {
        pub const $name: Comb<'static, Token, Terminal, AstNode> = Comb::Terminal {
            token: Terminal::$terminal,
        };
    };
}

/// Create a combinator for a specified non-terminal symbol.
#[macro_export]
macro_rules! node_comb {
    ($name:ident, $struct:ident) => {
        pub const $name: Comb<'static, Token, Terminal, AstNode> = Comb::Node {
            parser: &$struct::parse,
        };
    };
}
impl Comb<'_, Token, Terminal, AstNode> {
    terminal_comb!(LET, Let);

    terminal_comb!(CONST_KEYWORD, Const);

    terminal_comb!(MUT, Mut);

    terminal_comb!(ASSIGN, Assign);

    terminal_comb!(LPAREN, LParen);

    terminal_comb!(RPAREN, RParen);

    terminal_comb!(LBRACE, LBrace);

    terminal_comb!(RBRACE, RBrace);

    terminal_comb!(LBRACKET, LBracket);

    terminal_comb!(RBRACKET, RBracket);

    terminal_comb!(FN_KEYWORD, FnKeyword);

    terminal_comb!(IF_KEYWORD, IfKeyword);

    terminal_comb!(ELSE_KEYWORD, ElseKeyword);

    terminal_comb!(WHILE_KEYWORD, WhileKeyword);

    terminal_comb!(RETURN_KEYWORD, ReturnKeyword);

    terminal_comb!(MINUS, Minus);

    terminal_comb!(EXCLAMATION_MARK, ExclamationMark);

    terminal_comb!(COLON, Colon);

    terminal_comb!(COMMA, Comma);

    terminal_comb!(DOT, Dot);

    terminal_comb!(SEMI, Semicolon);

    terminal_comb!(SMALL_RIGHT_ARROW, SmallRightArrow);

    terminal_comb!(BIG_RIGHT_ARROW, BigRightArrow);

    terminal_comb!(BACKSLASH, Backslash);

    terminal_comb!(AMPERSAND, Ampersand);

    terminal_comb!(DECLARE_KEYWORD, DeclareKeyword);

    terminal_comb!(STRUCT_KEYWORD, StructKeyword);

    terminal_comb!(INSTANCE_KEYWORD, InstanceKeyword);

    node_comb!(ID, Id);

    node_comb!(NUM, Num);

    node_comb!(CHARACTER, Character);

    node_comb!(STRING, AstString);

    node_comb!(EXPR, Expression);

    node_comb!(STATEMENT, Statement);

    node_comb!(INITIALISATION, Initialisation);

    node_comb!(ASSIGNMENT, Assignment);

    node_comb!(FUNCTION, Function);

    node_comb!(LAMBDA, Lambda);

    node_comb!(LAMBDA_PARAMETER, LambdaParameter);

    node_comb!(IF, If);

    node_comb!(WHILE_LOOP, WhileLoop);

    node_comb!(BLOCK, Block);

    node_comb!(ARRAY, Array);

    node_comb!(PARAMETER, FunctionParameter);

    node_comb!(TYPE_NAME, TypeName);

    node_comb!(DECLARATION, Declaration);

    node_comb!(CONSTANT, Constant);

    node_comb!(STRUCT_DECLARATION, StructDeclaration);

    node_comb!(STRUCT_FIELD_DECLARATION, StructFieldDeclaration);

    node_comb!(STRUCT_INITILISATION, StructInitialisation);

    node_comb!(STRUCT_FIELD_INITIALISATION, StructFieldInitialisation);

    node_comb!(INSTANCE, Instance);

    node_comb!(METHOD_DECLARATION, MethodDeclaration);
}

impl<Tok, Term, Node> Comb<'_, Tok, Term, Node>
where
    Tok: Clone + std::fmt::Debug + GetPosition,
    Term: PartialEq<Tok> + std::fmt::Debug,
    Node: std::fmt::Debug,
{
    pub fn parse(&self, tokens: &mut ParseState<Tok>) -> Result<Vec<Node>, ParseError> {
        let mut matched = vec![];
        match self {
            Comb::Terminal { token } => {
                let Some(t) = tokens.next() else {
                    return Err(ParseError {
                        message: "Unexpected EOF!".into(),
                        position: tokens.last_token().map(|token| token.position()),
                    });
                };

                // try to parse the terminal
                if *token != t {
                    return Err(ParseError {
                        message: format!("Unexpected {t:?} while trying to parse {token:?}"),
                        position: Some(t.position()),
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
                    // let mut current_index = tokens.get_index();
                    // loop {
                    //     match inner.parse(tokens) {
                    //         Ok(mut result) => {
                    //             matched.append(&mut result);
                    //             current_index = tokens.get_index();
                    //         }
                    //         Err(e) => {
                    //             tokens.add_error(e);
                    //             break;
                    //         }
                    //     }
                    // }
                    // tokens.set_index(current_index);

                    // match an arbitrary amount of tokens
                    let mut current_index = tokens.get_index();
                    while let Ok(mut result) = inner.parse(tokens) {
                        matched.append(&mut result);
                        current_index = tokens.get_index();
                    }
                    tokens.set_index(current_index);
                }
            }
            Comb::RepeatUntil { repeated, closing } => {
                let mut current_index = tokens.get_index();
                // TODO: this should check for matches of `closing` before.
                while let Ok(mut result) = repeated.parse(tokens) {
                    matched.append(&mut result);
                    current_index = tokens.get_index();
                }
                tokens.set_index(current_index);

                let mut result = closing.parse(tokens).inspect_err(|e| {
                    tokens.add_error(e.clone());
                })?;
                matched.append(&mut result);
            }
        }

        Ok(matched)
    }
}

impl<Tok, Term, Node> Shr for Comb<'_, Tok, Term, Node> {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Comb::Sequence {
            current: Box::new(self),
            next: Box::new(rhs),
        }
    }
}

impl<Tok, Term, Node> BitOr for Comb<'_, Tok, Term, Node> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Comb::Either {
            left: Box::new(self),
            right: Box::new(rhs),
        }
    }
}

impl<Tok, Term, Node> Not for Comb<'_, Tok, Term, Node> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Comb::Optional {
            inner: Box::new(self),
        }
    }
}

impl<Tok, Term, Node> BitXor<()> for Comb<'_, Tok, Term, Node> {
    type Output = Self;

    fn bitxor(self, _rhs: ()) -> Self::Output {
        Comb::Repitition {
            inner: Box::new(self),
            amount: None,
        }
    }
}

impl<Tok, Term, Node> BitXor<usize> for Comb<'_, Tok, Term, Node> {
    type Output = Self;

    fn bitxor(self, rhs: usize) -> Self::Output {
        Comb::Repitition {
            inner: Box::new(self),
            amount: Some(rhs),
        }
    }
}

impl<'a, Tok, Term, Node> BitXor<Comb<'a, Tok, Term, Node>> for Comb<'a, Tok, Term, Node> {
    type Output = Self;

    fn bitxor(self, rhs: Comb<'a, Tok, Term, Node>) -> Self::Output {
        Comb::RepeatUntil {
            repeated: Box::new(self),
            closing: Box::new(rhs),
        }
    }
}

impl<'a, Tok, Term, Node> Rem for Comb<'a, Tok, Term, Node>
where
    Comb<'a, Tok, Term, Node>: Clone,
{
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        !(self.clone() >> ((rhs >> self) ^ ()))
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Span;

    use super::*;

    #[test]
    fn test_sequence_simple() {
        let left = Comb::LET;
        let right = Comb::ASSIGN;
        let new = left >> right;

        assert_eq!(
            Comb::Sequence {
                current: Box::new(Comb::LET),
                next: Box::new(Comb::ASSIGN)
            },
            new
        );
    }

    #[test]
    fn test_sequence_complex() {
        let a = Comb::LET;
        let b = Comb::ASSIGN;
        let c = Comb::SEMI;
        let new = a >> b >> c;

        assert_eq!(
            Comb::Sequence {
                current: Box::new(Comb::Sequence {
                    current: Box::new(Comb::LET),
                    next: Box::new(Comb::ASSIGN),
                }),
                next: Box::new(Comb::SEMI)
            },
            new
        );
    }

    #[test]
    fn test_either_simple() {
        let left = Comb::LET;
        let right = Comb::ASSIGN;
        let new = left | right;

        assert_eq!(
            Comb::Either {
                left: Box::new(Comb::LET),
                right: Box::new(Comb::ASSIGN)
            },
            new
        );
    }

    #[test]
    fn test_either_complex() {
        let a = Comb::LET;
        let b = Comb::ASSIGN;
        let c = Comb::SEMI;
        let new = a | b | c;

        assert_eq!(
            Comb::Either {
                left: Box::new(Comb::Either {
                    left: Box::new(Comb::LET),
                    right: Box::new(Comb::ASSIGN),
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
        let mut tokens = vec![Token::Let {
            position: Span::default(),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_optional_not_matching_terminal() {
        let a = !Comb::LET;
        let mut tokens = vec![Token::Assign {
            position: Span::default(),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 0);
    }

    #[test]
    fn test_parse_optional_matching_node() {
        let a = !Comb::NUM;
        let mut tokens = vec![Token::Integer {
            value: 42,
            position: Span::default(),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(
            Ok(vec![AstNode::Num(Num::Integer(42, (), Span::default()))]),
            result
        );
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_optional_not_matching_node() {
        let a = !Comb::NUM;
        let mut tokens = vec![Token::Id {
            value: "some_id".into(),
            position: Span::default(),
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
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
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
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
            Token::Assign {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
        ]
        .into();
        let result = a.parse(&mut tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repition_simple_wildcard() {
        let a = Comb::LET ^ ();
        let mut tokens = vec![
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
            Token::Assign {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
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
            Token::Integer {
                value: 42,
                position: Span::default(),
            },
            Token::Integer {
                value: 1337,
                position: Span::default(),
            },
            Token::Integer {
                value: 17,
                position: Span::default(),
            },
        ]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(
            Ok(vec![
                AstNode::Num(Num::Integer(42, (), Span::default())),
                AstNode::Num(Num::Integer(1337, (), Span::default())),
                AstNode::Num(Num::Integer(17, (), Span::default()))
            ]),
            result
        );
        assert_eq!(tokens.get_index(), 3);
    }

    #[test]
    fn test_parse_repition_node_wildcard() {
        let a = Comb::NUM ^ ();
        let mut tokens = vec![
            Token::Integer {
                value: 42,
                position: Span::default(),
            },
            Token::Integer {
                value: 1337,
                position: Span::default(),
            },
            Token::Integer {
                value: 17,
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
            Token::Let {
                position: Span::default(),
            },
        ]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(
            Ok(vec![
                AstNode::Num(Num::Integer(42, (), Span::default())),
                AstNode::Num(Num::Integer(1337, (), Span::default())),
                AstNode::Num(Num::Integer(17, (), Span::default()))
            ]),
            result
        );
        assert_eq!(tokens.get_index(), 3);
    }

    #[test]
    fn test_parse_terminal_simple() {
        let a = Comb::LET;
        let mut tokens = vec![Token::Let {
            position: Span::default(),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(Ok(vec![]), result);
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_node_simple() {
        let a = Comb::NUM;
        let mut tokens = vec![Token::Integer {
            value: 42,
            position: Span::default(),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert_eq!(
            Ok(vec![AstNode::Num(Num::Integer(42, (), Span::default()))]),
            result
        );
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_shr() {
        let matcher = Comb::LET >> Comb::NUM;
        let mut tokens = vec![
            Token::Let {
                position: Span::default(),
            },
            Token::Integer {
                value: 42,
                position: Span::default(),
            },
        ]
        .into();
        let result = matcher.parse(&mut tokens);
        assert_eq!(
            Ok(vec![AstNode::Num(Num::Integer(42, (), Span::default()))]),
            result
        );
        assert_eq!(tokens.get_index(), 2);
    }

    #[test]
    fn test_parse_bitor() {
        let matcher = Comb::ID | Comb::NUM;
        let mut tokens = vec![Token::Integer {
            value: 42,
            position: Span::default(),
        }]
        .into();
        let result = matcher.parse(&mut tokens);

        assert_eq!(
            Ok(vec![AstNode::Num(Num::Integer(42, (), Span::default()))]),
            result
        );
        assert_eq!(tokens.get_index(), 1);

        let mut tokens = vec![Token::Id {
            value: "some_id".into(),
            position: Span::default(),
        }]
        .into();
        let result = matcher.parse(&mut tokens);
        assert_eq!(
            Ok(vec![AstNode::Id(Id {
                name: "some_id".into(),
                info: (),
                position: Span::default(),
            })]),
            result
        );
        assert_eq!(tokens.get_index(), 1);
    }

    #[test]
    fn test_parse_simple_error() {
        let a = Comb::LET;
        let mut tokens = vec![Token::Integer {
            value: 42,
            position: Span::default(),
        }]
        .into();
        let result = a.parse(&mut tokens);

        assert!(result.is_err());
        assert_eq!(tokens.get_index(), 1);
    }
}
