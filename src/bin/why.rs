extern crate pest;
extern crate y_lang;

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

fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let args = Cli::parse();

    let file_content = std::fs::read_to_string(&args.file).expect(&format!(
        "Could not read file: '{}'",
        args.file.to_string_lossy()
    ));

    let pairs = YParser::parse_program(&file_content);

    let ast = Ast::from_program(pairs);

    let typechecker = Typechecker::from_ast(ast.clone());

    if let Err(type_error) = typechecker.check() {
        error!(
            "{} ({}:{})",
            type_error.message, type_error.position.0, type_error.position.1
        );
        std::process::exit(-1);
    }

    if args.run {
        let interpreter = Interpreter::from_ast(ast.clone());

        interpreter.run();
    }

    if args.compile {
        let mut compiler = Compiler::from_ast(ast.clone());

        compiler.compile(args.output.unwrap_or("a".to_owned()));
    }
}
