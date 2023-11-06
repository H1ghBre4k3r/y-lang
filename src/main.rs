use std::{error::Error, fs};

use pesca_parser::{lexer::Lexer, parser::parse};

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("examples/main.why")?;

    println!("{input}");

    let lexer = Lexer::new(&input);
    let tokens = lexer.lex()?;

    println!("{tokens:#?}");

    let statements = parse(&mut tokens.into())?;

    println!("{statements:#?}");
    Ok(())
}
