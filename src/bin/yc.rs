use std::{error::Error, fs};

use clap::{command, Parser};
use pesca_lang::{lexer::Lexer, parser::parse, typechecker::TypeChecker};

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The path to the source file.
    #[arg(index = 1)]
    pub file: std::path::PathBuf,

    #[arg(short = 'l', long)]
    pub print_lexed: bool,

    #[arg(short = 'p', long)]
    pub print_parsed: bool,

    #[arg(short, long, default_value = "a.out")]
    pub output: Option<std::path::PathBuf>,
}

impl Cli {
    pub fn init() -> Self {
        Cli::parse()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::init();

    let input = fs::read_to_string(args.file)?;

    let lexer = Lexer::new(&input);
    let tokens = lexer.lex()?;

    if args.print_lexed {
        println!("{tokens:#?}");
    }

    let statements = parse(&mut tokens.into())?;

    if args.print_parsed {
        println!("{statements:#?}");
    }

    let checked = TypeChecker::new().check(statements)?;

    println!("{checked:#?}");
    Ok(())
}
