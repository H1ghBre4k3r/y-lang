use pesca_parser_derive::{LooseEq, Token as ParseToken};
use regex::{Match, Regex};

type Position = usize;

#[derive(Debug, Clone, ParseToken, LooseEq)]
pub enum Token {
    #[terminal("=")]
    Assign { position: Position },
    #[terminal("let")]
    Let { position: Position },
    #[terminal("const")]
    Const { position: Position },
    #[terminal("mut")]
    Mut { position: Position },
    #[literal("[a-zA-Z_][a-zA-Z0-9_]*")]
    Id { value: String, position: Position },
    #[literal("[0-9]+")]
    Integer { value: u64, position: Position },
    #[literal("[0-9]+\\.[0-9]+")]
    FloatingPoint { value: f64, position: Position },
    #[terminal(";")]
    Semicolon { position: Position },
    #[literal("//.*")]
    Comment { value: String, position: Position },
    #[terminal("+")]
    Plus { position: Position },
    #[terminal("-")]
    Minus { position: Position },
    #[terminal("*")]
    Times { position: Position },
    #[terminal("(")]
    LParen { position: Position },
    #[terminal(")")]
    RParen { position: Position },
    #[terminal("{")]
    LBrace { position: Position },
    #[terminal("}")]
    RBrace { position: Position },
    #[terminal("[")]
    LBracket { position: Position },
    #[terminal("]")]
    RBracket { position: Position },
    #[terminal("fn")]
    FnKeyword { position: Position },
    #[terminal("if")]
    IfKeyword { position: Position },
    #[terminal("else")]
    ElseKeyword { position: Position },
    #[terminal("while")]
    WhileKeyword { position: Position },
    #[terminal("return")]
    ReturnKeyword { position: Position },
    #[terminal(":")]
    Colon { position: Position },
    #[terminal(",")]
    Comma { position: Position },
    #[terminal(".")]
    Dot { position: Position },
    #[terminal("->")]
    SmallRightArrow { position: Position },
    #[terminal("=>")]
    BigRightArrow { position: Position },
    #[terminal("\\")]
    Backslash { position: Position },
    #[terminal("==")]
    Equal { position: Position },
    #[terminal(">")]
    GreaterThan { position: Position },
    #[terminal("<")]
    LessThan { position: Position },
    #[terminal(">=")]
    GreaterOrEqual { position: Position },
    #[terminal("<0")]
    LessOrEqual { position: Position },
    #[terminal("&")]
    Ampersand { position: Position },
    #[terminal("declare")]
    DeclareKeyword { position: Position },
    #[terminal("struct")]
    StructKeyword { position: Position },
    #[terminal("!")]
    ExclamationMark { position: Position },
}
