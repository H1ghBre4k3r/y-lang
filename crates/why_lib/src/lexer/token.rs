use colored::Colorize;

use lex_derive::{LooseEq, Token as ParseToken};
use regex::{Match, Regex};

#[derive(Default, Debug, Clone, Eq)]
pub struct Span {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub source: String,
}

impl Span {
    pub fn to_string(&self, msg: impl ToString) -> String {
        let Span { start, end, source } = self;
        let line = start.0;
        let lines = source.lines().collect::<Vec<_>>();
        let prev_line = if line > 0 { lines[line - 1] } else { "" };
        let line_str = lines[line];

        // margin _before_ left border
        let left_margin = format!("{}", end.0).len();
        let left_margin_fill = vec![' '; left_margin].iter().collect::<String>();

        // split right at the start of the error in the first line
        let (left, right) = line_str.split_at(start.1);

        // some case magic
        let (left, right) = if start.0 != end.0 {
            // if the error ranges over more than a single line, we can just mark rest of the line
            // as an error
            (left.to_string(), right.to_string().red().to_string())
        } else {
            // however, if the lines does not range beyond this line, we need to split at the end
            // again
            let (err_str, after_err) = right.split_at(end.1 - start.1);

            // now, just color the error part red
            (
                left.to_string(),
                format!("{err_str}{after_err}", err_str = err_str.to_string().red()),
            )
        };

        // and concatentate both together
        let line_str = format!("{left}{right}");

        // padding between border and squiggles
        let left_padding_fill = vec![' '; end.1 - 1].iter().collect::<String>();

        // the error with the first line
        let mut error_string = format!(
            "{left_margin_fill} |\n{left_margin_fill} |{prev_line} \n{line} |{line_str}",
            line = line + 1
        );

        // iterate over all lines of the error and make them shine red
        ((start.0 + 1)..(end.0 + 1)).for_each(|line_number| {
            error_string = format!(
                "{error_string}\n{left_margin_fill} |{}",
                lines[line_number].to_string().red()
            );
        });

        // actually add error message at bottom
        error_string = format!(
            "{error_string}\n{} |{left_padding_fill}^--- {}\n{left_margin_fill} |",
            end.0 + 2,
            msg.to_string()
        );

        error_string
    }

    pub fn merge(&self, other: &Span) -> Span {
        let Span { start, source, .. } = self.clone();
        let Span { end, .. } = other.clone();

        Span { start, end, source }
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
    #[terminal("class")]
    ClassKeyword { position: Span },
    #[terminal("instance")]
    InstanceKeyword { position: Span },
    #[terminal("!")]
    ExclamationMark { position: Span },
    #[terminal("#")]
    Hash { position: Span },
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
            Self::ClassKeyword { .. } => f.debug_struct("ClassKeyword").finish(),
            Self::InstanceKeyword { .. } => f.debug_struct("InstanceKeyword").finish(),
            Self::ExclamationMark { .. } => f.debug_struct("ExclamationMark").finish(),
            Self::Hash { .. } => f.debug_struct("Hash").finish(),
        }
    }
}
