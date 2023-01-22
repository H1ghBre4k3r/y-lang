mod ast;
mod interpreter;
mod typechecker;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use clap::Parser as CParser;
use interpreter::Interpreter;
use log::error;

use crate::{
    ast::{Ast, YParser},
    typechecker::check_ast,
};

#[derive(CParser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    file: std::path::PathBuf,

    #[arg(short, long)]
    run: bool,
}

fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let args = Cli::parse();

    let file_content = std::fs::read_to_string(&args.file).expect(&format!(
        "Could not read file: '{}'",
        args.file.to_string_lossy()
    ));

    let pairs = YParser::parse_program(&file_content);

    let ast = Ast::from_program(pairs);

    if let Err(type_error) = check_ast(&ast) {
        error!(
            "{} ({}:{})",
            type_error.message, type_error.position.0, type_error.position.1
        );
        std::process::exit(-1);
    }

    if args.run {
        let interpreter = Interpreter::from_ast(ast);

        interpreter.run();
    }
}
