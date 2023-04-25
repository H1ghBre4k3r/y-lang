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

    let Module { ast, imports, .. } = load_module(file.clone())?;

    if args.dump_parsed {
        info!("Parsed AST:\n{:#?}", ast);
    }

    let modules = match load_modules(&ast, file, Modules::default()) {
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
            imports,
            file_path,
        },
    ) in &modules
    {
        let local_modules = convert_imports_to_local_names(imports, &modules);

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

    let local_modules = convert_imports_to_local_names(&imports, &modules);

    let typechecker = Typechecker::from_ast(ast, local_modules);

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

fn convert_imports_to_local_names(
    imports: &[(String, String)],
    modules: &Modules<()>,
) -> Modules<()> {
    let mut local_modules = Modules::default();

    for (import_path, real_path) in imports {
        local_modules.insert(
            import_path.to_owned(),
            modules.get(real_path).unwrap().to_owned(),
        );
    }
    local_modules
}
