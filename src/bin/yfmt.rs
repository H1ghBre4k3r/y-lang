use std::fs;

use clap::Parser;
use why_lib::{formatter, lexer::Lexer, parser::parse};

#[derive(clap::Parser, Debug, serde::Serialize, serde::Deserialize)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct YFmtArgs {
    /// The path to the source file.
    #[arg(index = 1)]
    pub file: std::path::PathBuf,

    /// Whether the edit should be done in place.
    #[arg(short = 'i', long)]
    pub in_place: bool,
}

fn main() -> anyhow::Result<()> {
    let args = YFmtArgs::parse();

    let input = fs::read_to_string(&args.file)?;

    let lexer = Lexer::new(&input);
    let tokens = lexer.lex()?;

    let statements = match parse(&mut tokens.into()) {
        Ok(stms) => stms,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(-1);
        }
    };

    let formatted = formatter::format_program(&statements)
        .map_err(|e| anyhow::anyhow!("Formatting error: {}", e))?;

    if args.in_place {
        fs::write(&args.file, formatted)?;
    } else {
        println!("{formatted}");
    }

    Ok(())
}
