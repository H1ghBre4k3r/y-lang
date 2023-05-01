extern crate pest;
extern crate y_lang;

use std::{collections::HashMap, error::Error, fs};

use clap::Parser as CParser;
use log::{error, info};
use y_lang::{
    compiler::Compiler,
    loader::{load_module, load_modules, Module, Modules},
};

#[derive(CParser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// The path to the why source file.
    #[arg(index = 1)]
    file: std::path::PathBuf,

    /// Whether to dump the parsed AST (for debugging).
    #[arg(long)]
    dump_parsed: bool,

    /// Whether to dump the type-checked AST (for debugging).
    #[arg(long)]
    dump_typed: bool,

    /// The path to the output binary.
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = Cli::parse();

    let file = fs::canonicalize(&args.file)?;

    let main_module = load_module(file.clone())?;

    if args.dump_parsed {
        info!("Parsed AST:\n{:#?}", main_module.ast);
    }

    let modules = match load_modules(&main_module.ast, file, Modules::default()) {
        Err(load_error) => {
            error!("{}", load_error);
            std::process::exit(-1);
        }
        Ok(modules) => modules,
    };

    let mut type_safe_modules = HashMap::default();

    for (key, module) in &modules {
        type_safe_modules.insert(key.to_owned(), module.type_check(&modules)?);
    }

    let Module { ast, .. } = main_module.type_check(&modules)?;

    if args.dump_typed {
        info!("Typed AST:\n{:#?}", ast);
    }

    if let Some(output) = args.output {
        let mut compiler = Compiler::from_ast(ast, type_safe_modules.clone());

        compiler.compile_program(output)?;
    }

    Ok(())
}
