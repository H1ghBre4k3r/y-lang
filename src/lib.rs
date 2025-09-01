use std::{fs, process};

use clap::{Parser, command};
use why_lib::{lexer::Lexer, optimizer::optimize, parser::parse, typechecker::TypeChecker};

#[derive(Parser, Debug, serde::Serialize, serde::Deserialize)]
#[command(author, version, about)]
#[command(propagate_version = true)]
pub struct VCArgs {
    /// The path to the source file.
    #[arg(index = 1)]
    pub file: std::path::PathBuf,

    /// Print the lexed source tree.
    #[arg(short = 'l', long)]
    pub print_lexed: bool,

    /// Print the parsed AST.
    #[arg(short = 'p', long)]
    pub print_parsed: bool,

    /// Print the typechecked AST.
    #[arg(short = 'c', long)]
    pub print_checked: bool,

    /// Print the validated AST.
    #[arg(short = 'v', long)]
    pub print_validated: bool,

    #[arg(short, long, default_value = "a.out")]
    pub output: Option<std::path::PathBuf>,
}

impl VCArgs {
    pub fn init() -> Self {
        VCArgs::parse()
    }
}

pub fn compile_file(args: VCArgs) -> anyhow::Result<()> {
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

    if args.print_checked {
        println!("{checked:#?}");
    }

    let validated = match TypeChecker::validate(checked) {
        Ok(validated) => validated,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    if args.print_validated {
        println!("{validated:#?}");
    }

    let optimized = optimize(validated);

    println!("{optimized:#?}");

    Ok(())
}
