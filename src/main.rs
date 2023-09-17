mod lexer;
mod parser;

use std::error::Error;

use crate::parser::parse;

use self::lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    let input = r#"
let a = 3;
let b = 5 + 2;
let c = a + 4;
let d = c + (b * 4);

let add = fn (x: i32, y: i32): i32 {
    return x + y;
};
    "#;

    let lexed = lex(input)?;

    println!("{lexed:#?}");

    let statements = parse(&mut lexed.into())?;

    println!("{statements:#?}");
    Ok(())
}
