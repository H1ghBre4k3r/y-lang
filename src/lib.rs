pub mod util;

use std::{fs, process};

use clap::{Parser, command};
use why_lib::{
    Module,
    formatter::{self},
};

use crate::util::convert_parse_error;

#[derive(Parser, Debug, serde::Serialize, serde::Deserialize)]
#[command(author, version, about)]
#[command(propagate_version = true)]
pub struct VCArgs {
    /// The path to the source file.
    #[arg(index = 1)]
    pub file: std::path::PathBuf,

    /// Print the lexed source tree.
    #[arg(short = 'l', long)]
    pub print_lexed: bool,

    /// Print the parsed AST.
    #[arg(short = 'p', long)]
    pub print_parsed: bool,

    /// Print the typechecked AST.
    #[arg(short = 'c', long)]
    pub print_checked: bool,

    /// Print the validated AST.
    #[arg(short = 'v', long)]
    pub print_validated: bool,

    /// Format the source code and print to stdout.
    #[arg(long)]
    pub format: bool,

    /// Format the source code and write to output file.
    #[arg(long)]
    pub format_output: Option<std::path::PathBuf>,

    #[arg(short, long, default_value = "a.out")]
    pub output: std::path::PathBuf,

    /// Emit LLVM IR (.ll files)
    #[arg(long)]
    pub emit_llvm: bool,

    /// Emit LLVM bitcode (.bc files)
    #[arg(long)]
    pub emit_bitcode: bool,

    /// Emit native assembly (.s files)
    #[arg(long)]
    pub emit_assembly: bool,

    /// Emit object files (.o files)
    #[arg(long)]
    pub emit_object: bool,
}

impl VCArgs {
    pub fn init() -> Self {
        VCArgs::parse()
    }
}

pub fn compile_file(args: VCArgs) -> anyhow::Result<()> {
    let module = Module::new(args.file.to_str().map(|path| path.to_string()).expect(""))?;

    // if !module.exists() || args.force {
    let module = match module.lex() {
        Ok(program) => program,
        Err(errors) => {
            let mut spans = vec![];
            for error in errors {
                convert_parse_error(error, &module.input, &mut spans);
            }
            for (msg, span) in spans {
                eprintln!("{}", span.to_string(msg));
            }
            process::exit(-1);
        }
    };

    if args.print_lexed {
        println!("{:#?}", module.inner);
    }

    let module = match module.parse() {
        Ok(module) => module,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    if args.print_parsed {
        println!("{:#?}", module.inner);
    }

    // Handle formatting requests
    if args.format || args.format_output.is_some() {
        let formatted = formatter::format_program(&module.inner)
            .map_err(|e| anyhow::anyhow!("Formatting error: {}", e))?;

        if args.format {
            println!("{formatted}");
        }

        let format_output_provided = args.format_output.is_some();
        if let Some(output_path) = args.format_output {
            fs::write(output_path, formatted)?;
        }

        // If only formatting was requested, return early
        if args.format && format_output_provided && !args.print_checked && !args.print_validated {
            return Ok(());
        }
    }

    let module = match module.check() {
        Ok(module) => module,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    if args.print_checked {
        println!("{:#?}", module.inner);
    }

    let module = match module.validate() {
        Ok(module) => module,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    if args.print_validated {
        println!("{module:#?}");
    }

    if let Err(e) = module.codegen(
        args.emit_llvm,
        args.emit_bitcode,
        args.emit_assembly,
        args.emit_object,
    ) {
        eprintln!("Codegen error: {e}");
        process::exit(-1);
    }

    module.compile(args.output.to_str().unwrap());

    Ok(())
}
