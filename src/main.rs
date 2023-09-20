mod lexer;
mod parser;

use std::error::Error;

use crate::parser::parse;

use self::lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    let input = r#"
let fnoo = fn(x:void,y:i32):i32{};"#;

    let lexer = Lexer::new(input);
    let lexed = lexer.lex()?;

    println!("{lexed:#?}");

    let statements = parse(&mut lexed.into())?;

    println!("{statements:#?}");
    Ok(())
}
