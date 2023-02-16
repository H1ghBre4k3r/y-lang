extern crate pest;
extern crate y_lang;

use std::{error::Error, fs};

use clap::Parser as CParser;
use log::error;
use y_lang::{
    ast::{Ast, YParser},
    compiler::Compiler,
    interpreter::Interpreter,
    loader::load_modules,
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

    let file = fs::canonicalize(&args.file)?;

    let file_content = std::fs::read_to_string(&file)
        .unwrap_or_else(|_| panic!("Could not read file: '{}'", file.to_string_lossy()));

    let pairs = YParser::parse_program(&file_content);

    let ast = Ast::from_program(pairs.collect(), &file.to_string_lossy());

    let modules = match load_modules(&ast, file) {
        Err(load_error) => {
            error!("{}", load_error);
            std::process::exit(-1);
        }
        Ok(modules) => modules,
    };

    let typechecker = Typechecker::from_ast(ast, modules);

    let ast = match typechecker.check() {
        Ok(ast) => ast,
        Err(type_error) => {
            error!("{}", type_error);
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
