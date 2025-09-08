use std::{fs, process};

use clap::{Parser, command};
use why_lib::{
    formatter::{self},
    grammar,
    parser::parse_program,
    typechecker::TypeChecker,
};

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

    /// Format the source code and print to stdout.
    #[arg(short = 'f', long)]
    pub format: bool,

    /// Format the source code and write to output file.
    #[arg(long)]
    pub format_output: Option<std::path::PathBuf>,

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

    let program = grammar::parse(&input).unwrap();
    // println!("{program:#?}");

    // let lexer = Lexer::new(&input);
    // let tokens = lexer.lex()?;

    // if args.print_lexed {
    //     println!("{tokens:#?}");
    // }
    //
    let statements = parse_program(program, &input);

    // let statements = match parse(&mut tokens.into()) {
    //     Ok(stms) => stms,
    //     Err(e) => {
    //         eprintln!("{e}");
    //         process::exit(-1);
    //     }
    // };

    if args.print_parsed {
        println!("{statements:#?}");
    }

    // Handle formatting requests
    if args.format || args.format_output.is_some() {
        let formatted = formatter::format_program(&statements)
            .map_err(|e| anyhow::anyhow!("Formatting error: {}", e))?;

        if args.format {
            println!("{formatted}");
        }

        let format_output_provided = args.format_output.is_some();
        if let Some(output_path) = args.format_output {
            fs::write(output_path, formatted)?;
        }

        // If only formatting was requested, return early
        if args.format && format_output_provided && !args.print_checked && !args.print_validated {
            return Ok(());
        }
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

    Ok(())
}
