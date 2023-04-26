extern crate pest;
extern crate y_lang;

use std::{collections::HashMap, error::Error, fs};

use clap::Parser as CParser;
use log::{error, info};
use y_lang::{
    compiler::Compiler,
    loader::{load_module, load_modules, Module, Modules},
    typechecker::Typechecker,
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
        let local_modules = module.convert_imports_to_local_names(&modules);

        let Module {
            name,
            file_path,
            exports,
            imports,
            ast,
        } = module;

        let typechecker = Typechecker::from_ast(ast.clone(), local_modules);
        let ast = match typechecker.check() {
            Ok(ast) => ast,
            Err(type_error) => {
                error!("{}", type_error);
                std::process::exit(-1);
            }
        };

        type_safe_modules.insert(
            key.to_owned(),
            Module {
                ast,
                name: name.clone(),
                exports: exports.clone(),
                imports: imports.clone(),
                file_path: file_path.clone(),
            },
        );
    }

    let local_modules = main_module.convert_imports_to_local_names(&modules);

    let typechecker = Typechecker::from_ast(main_module.ast, local_modules);

    let ast = match typechecker.check() {
        Ok(ast) => ast,
        Err(type_error) => {
            error!("{}", type_error);
            std::process::exit(-1);
        }
    };

    if args.dump_typed {
        info!("Typed AST:\n{:#?}", ast);
    }

    if let Some(output) = args.output {
        let mut compiler = Compiler::from_ast(ast, type_safe_modules.clone());

        compiler.compile_program(output)?;
    }

    Ok(())
}
