//! # Why
//!
//! This binary is the compiler of Y. It combines parser, type checker, and compiler into a single
//! application.
extern crate pest;
extern crate y_lang;

mod cli;

use cli::*;

use std::{collections::HashMap, error::Error, fs};

use log::{error, info};
use y_lang::{
    compiler::Compiler,
    loader::{load_module, load_modules, Module, Modules},
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::init();

    simple_logger::init_with_level(args.verbosity.into()).unwrap();

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