extern crate pest;
extern crate y_lang;

use std::error::Error;

use clap::Parser as CParser;
use log::error;
use y_lang::{
    ast::{Ast, YParser},
    compiler::Compiler,
    interpreter::Interpreter,
    typechecker::Typechecker,
};

#[derive(CParser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    file: std::path::PathBuf,

    #[arg(short, long)]
    run: bool,

    #[arg(short, long)]
    compile: bool,

    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = Cli::parse();

    let file_content = std::fs::read_to_string(&args.file)
        .unwrap_or_else(|_| panic!("Could not read file: '{}'", args.file.to_string_lossy()));

    let pairs = YParser::parse_program(&file_content);

    let ast = Ast::from_program(pairs);

    let typechecker = Typechecker::from_ast(ast);

    let ast = match typechecker.check() {
        Ok(ast) => ast,
        Err(type_error) => {
            error!(
                "{} ({}:{})",
                type_error.message, type_error.position.0, type_error.position.1
            );
            std::process::exit(-1);
        }
    };

    if args.run {
        let interpreter = Interpreter::from_ast(ast.clone());

        interpreter.run();
    }

    if args.compile {
        let mut compiler = Compiler::from_ast(ast);

        compiler.compile(args.output.unwrap_or("a".to_owned()))?;
    }

    Ok(())
}
