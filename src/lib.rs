use std::{fs, process};

use clap::{Parser, command};
use why_lib::{lexer::Lexer, parser::parse, typechecker::TypeChecker};

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

pub fn compile_file(args: Cli) -> anyhow::Result<()> {
    let input = fs::read_to_string(args.file)?;

    let lexer = Lexer::new(&input);
    let tokens = lexer.lex()?;

    if args.print_lexed {
        println!("{tokens:#?}");
    }

    let statements = match parse(&mut tokens.into()) {
        Ok(stms) => stms,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    if args.print_parsed {
        println!("{statements:#?}");
    }

    let typechecker = TypeChecker::new(statements);
    let checked = match typechecker.check() {
        Ok(checked) => checked,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    let validated = match TypeChecker::validate(checked) {
        Ok(validated) => validated,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    println!("{validated:#?}");

    Ok(())
}
