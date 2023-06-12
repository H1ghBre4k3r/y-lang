use std::{collections::HashMap, error::Error, fs};

use log::{error, info};
use y_lang::{
    compiler::Compiler,
    loader::{load_module, load_modules, Module, Modules},
};

use crate::cli::BuildArgs;

pub fn build_executable(args: &BuildArgs) -> Result<(), Box<dyn Error>> {
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

    if let Some(output) = &args.output {
        let mut compiler = Compiler::from_ast(ast, type_safe_modules.clone());

        compiler.compile_program(output.clone())?;
    }

    Ok(())
}
