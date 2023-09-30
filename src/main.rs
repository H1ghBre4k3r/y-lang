mod lexer;
mod parser;

use std::error::Error;

use crate::parser::parse;

use self::lexer::*;

fn main() -> Result<(), Box<dyn Error>> {
    let input = r"
        let foo: (i32) -> i32 = \(x) => x * 2;
        let bar: (i32, i32) -> i32 = \(x, y) => x + y;

        let fizz: (i32) -> i32 = \(x) => {
            return bar(x);
        };

        let main = fn (): i32 {
            baz(42)
        };
    ";

    println!("{input}");

    let lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    println!("{tokens:#?}");

    let statements = parse(&mut tokens.into())?;

    println!("{statements:#?}");
    Ok(())
}
