mod lexer;
mod parser;

use std::error::Error;

use crate::parser::parse;

use self::lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    let input = r#"
let a = 3;"#;

    let lexed = lex(input)?;

    println!("{lexed:#?}");

    let statements = parse(&mut lexed.into())?;

    println!("{statements:#?}");
    Ok(())
}
