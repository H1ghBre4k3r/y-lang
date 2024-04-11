use colored::Colorize;
use std::ops::Range;

use lex_derive::{LooseEq, Token as ParseToken};
use regex::{Match, Regex};

#[derive(Default, Debug, Clone, Eq)]
pub struct Span {
    pub line: usize,
    pub col: Range<usize>,
    pub source: String,
}

impl Span {
    pub fn to_string(&self, msg: impl ToString) -> String {
        let Span { line, col, source } = self;
        let lines = source.lines().collect::<Vec<_>>();
        let prev_line = if *line > 1 { lines[*line - 2] } else { "" };
        let line_str = lines[*line - 1];

        let left_margin = format!("{line}").len();
        let left_margin_fill = vec![' '; left_margin].iter().collect::<String>();

        let left_padding_fill = vec![' '; col.start - 1].iter().collect::<String>();

        let error_len = vec!['^'; col.end - col.start]
            .iter()
            .collect::<String>()
            .red();

        format!("{left_margin_fill} |{prev_line} \n{line} |{line_str} \n{left_margin_fill} |{left_padding_fill}{error_len}   {}", msg.to_string())
    }
}

impl PartialEq<Span> for Span {
    fn eq(&self, _other: &Span) -> bool {
        // TODO: maybe this should not be the case...
        true
    }
}

#[derive(Clone, ParseToken, LooseEq)]
pub enum Token {
    #[terminal("=")]
    Assign { position: Span },
    #[terminal("let")]
    Let { position: Span },
    #[terminal("const")]
    Const { position: Span },
    #[terminal("mut")]
    Mut { position: Span },
    #[literal("[a-zA-Z_][a-zA-Z0-9_]*")]
    Id { value: String, position: Span },
    #[literal("[0-9]+")]
    Integer { value: u64, position: Span },
    #[literal("[0-9]+\\.[0-9]+")]
    FloatingPoint { value: f64, position: Span },
    #[terminal(";")]
    Semicolon { position: Span },
    #[literal("//.*")]
    Comment { value: String, position: Span },
    #[terminal("+")]
    Plus { position: Span },
    #[terminal("-")]
    Minus { position: Span },
    #[terminal("*")]
    Times { position: Span },
    #[terminal("(")]
    LParen { position: Span },
    #[terminal(")")]
    RParen { position: Span },
    #[terminal("{")]
    LBrace { position: Span },
    #[terminal("}")]
    RBrace { position: Span },
    #[terminal("[")]
    LBracket { position: Span },
    #[terminal("]")]
    RBracket { position: Span },
    #[terminal("fn")]
    FnKeyword { position: Span },
    #[terminal("if")]
    IfKeyword { position: Span },
    #[terminal("else")]
    ElseKeyword { position: Span },
    #[terminal("while")]
    WhileKeyword { position: Span },
    #[terminal("return")]
    ReturnKeyword { position: Span },
    #[terminal(":")]
    Colon { position: Span },
    #[terminal(",")]
    Comma { position: Span },
    #[terminal(".")]
    Dot { position: Span },
    #[terminal("->")]
    SmallRightArrow { position: Span },
    #[terminal("=>")]
    BigRightArrow { position: Span },
    #[terminal("\\")]
    Backslash { position: Span },
    #[terminal("==")]
    Equal { position: Span },
    #[terminal(">")]
    GreaterThan { position: Span },
    #[terminal("<")]
    LessThan { position: Span },
    #[terminal(">=")]
    GreaterOrEqual { position: Span },
    #[terminal("<0")]
    LessOrEqual { position: Span },
    #[terminal("&")]
    Ampersand { position: Span },
    #[terminal("declare")]
    DeclareKeyword { position: Span },
    #[terminal("struct")]
    StructKeyword { position: Span },
    #[terminal("!")]
    ExclamationMark { position: Span },
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assign { .. } => f.debug_struct("Assign").finish(),
            Self::Let { .. } => f.debug_struct("Let").finish(),
            Self::Const { .. } => f.debug_struct("Const").finish(),
            Self::Mut { .. } => f.debug_struct("Mut").finish(),
            Self::Id { value, .. } => f.debug_struct("Id").field("value", value).finish(),
            Self::Integer { value, .. } => f.debug_struct("Integer").field("value", value).finish(),
            Self::FloatingPoint { value, .. } => f
                .debug_struct("FloatingPoint")
                .field("value", value)
                .finish(),
            Self::Semicolon { .. } => f.debug_struct("Semicolon").finish(),
            Self::Comment { value, .. } => f.debug_struct("Comment").field("value", value).finish(),
            Self::Plus { .. } => f.debug_struct("Plus").finish(),
            Self::Minus { .. } => f.debug_struct("Minus").finish(),
            Self::Times { .. } => f.debug_struct("Times").finish(),
            Self::LParen { .. } => f.debug_struct("LParen").finish(),
            Self::RParen { .. } => f.debug_struct("RParen").finish(),
            Self::LBrace { .. } => f.debug_struct("LBrace").finish(),
            Self::RBrace { .. } => f.debug_struct("RBrace").finish(),
            Self::LBracket { .. } => f.debug_struct("LBracket").finish(),
            Self::RBracket { .. } => f.debug_struct("RBracket").finish(),
            Self::FnKeyword { .. } => f.debug_struct("FnKeyword").finish(),
            Self::IfKeyword { .. } => f.debug_struct("IfKeyword").finish(),
            Self::ElseKeyword { .. } => f.debug_struct("ElseKeyword").finish(),
            Self::WhileKeyword { .. } => f.debug_struct("WhileKeyword").finish(),
            Self::ReturnKeyword { .. } => f.debug_struct("ReturnKeyword").finish(),
            Self::Colon { .. } => f.debug_struct("Colon").finish(),
            Self::Comma { .. } => f.debug_struct("Comma").finish(),
            Self::Dot { .. } => f.debug_struct("Dot").finish(),
            Self::SmallRightArrow { .. } => f.debug_struct("SmallRightArrow").finish(),
            Self::BigRightArrow { .. } => f.debug_struct("BigRightArrow").finish(),
            Self::Backslash { .. } => f.debug_struct("Backslash").finish(),
            Self::Equal { .. } => f.debug_struct("Equal").finish(),
            Self::GreaterThan { .. } => f.debug_struct("GreaterThan").finish(),
            Self::LessThan { .. } => f.debug_struct("LessThan").finish(),
            Self::GreaterOrEqual { .. } => f.debug_struct("GreaterOrEqual").finish(),
            Self::LessOrEqual { .. } => f.debug_struct("LessOrEqual").finish(),
            Self::Ampersand { .. } => f.debug_struct("Ampersand").finish(),
            Self::DeclareKeyword { .. } => f.debug_struct("DeclareKeyword").finish(),
            Self::StructKeyword { .. } => f.debug_struct("StructKeyword").finish(),
            Self::ExclamationMark { .. } => f.debug_struct("ExclamationMark").finish(),
        }
    }
}
