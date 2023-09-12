mod lexer;
mod parser;

use std::error::Error;

use crate::parser::{combinators::Comb, parse};

use self::lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    let input = r#"
        let a = 3;
        let b = 5 + 2;
        let c = a + 4;
        let d = c + b * 4;
    "#;

    let lexed = lex(input)?;

    println!("{lexed:#?}");

    let statements = parse(&mut lexed.into())?;

    println!("{statements:#?}");

    let input = lex("let some = 42;")?;

    let matcher = Comb::LET >> Comb::ID >> Comb::EQ >> (Comb::NUM | Comb::ID) >> Comb::SEMI;

    let result = matcher.parse(&mut input.into())?;
    println!("{result:#?}");

    Ok(())
}
