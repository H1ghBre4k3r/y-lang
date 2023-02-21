extern crate pest;
extern crate y_lang;

use std::{collections::HashMap, error::Error, fs};

use clap::Parser as CParser;
use log::error;
use y_lang::{
    ast::{Ast, YParser},
    compiler::Compiler,
    interpreter::Interpreter,
    loader::{load_modules, Module},
    typechecker::Typechecker,
};

#[derive(CParser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// The path to the why source file.
    #[arg(index = 1)]
    file: std::path::PathBuf,

    /// Whether to interpret instead of compiling.
    #[arg(short, long)]
    run: bool,

    /// The path to the output binary.
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
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

    let mut type_safe_modules = HashMap::default();

    for (
        key,
        Module {
            ast,
            name,
            exports,
            is_wildcard,
        },
    ) in modules
    {
        let typechecker = Typechecker::from_ast(ast.clone(), HashMap::default());
        let ast = match typechecker.check() {
            Ok(ast) => ast,
            Err(type_error) => {
                error!("{}", type_error);
                std::process::exit(-1);
            }
        };

        type_safe_modules.insert(
            key,
            Module {
                ast,
                name,
                exports,
                is_wildcard,
            },
        );
    }

    let typechecker = Typechecker::from_ast(ast, type_safe_modules.clone());

    let ast = match typechecker.check() {
        Ok(ast) => ast,
        Err(type_error) => {
            error!("{}", type_error);
            std::process::exit(-1);
        }
    };

    if args.run && args.output.is_none() {
        let interpreter = Interpreter::from_ast(ast.clone(), type_safe_modules.clone());

        interpreter.run();
    }

    if let Some(output) = args.output {
        let mut compiler = Compiler::from_ast(ast, type_safe_modules.clone());

        compiler.compile_program(output)?;
    }

    Ok(())
}
