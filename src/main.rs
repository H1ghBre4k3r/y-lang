mod lexer;
mod parser;

use std::error::Error;

use self::lexer::*;
use self::parser::*;

fn main() -> Result<(), Box<dyn Error>> {
    let input = r#"
        let a = 3;
        "#;

    let lexed = lex(input)?;

    parse(lexed);
    Ok(())
}
