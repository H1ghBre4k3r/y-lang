use std::fs;
use std::path::Path;

use clap::Parser;
use why_lib::{formatter, lexer::Lexer, parser::parse};

#[derive(clap::Parser, Debug, serde::Serialize, serde::Deserialize)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct YFmtArgs {
    /// The path to the source file or directory to format.
    #[arg(index = 1)]
    pub path: std::path::PathBuf,

    /// Whether the edit should be done in place.
    #[arg(short = 'i', long)]
    pub in_place: bool,
}

fn main() -> anyhow::Result<()> {
    let args = YFmtArgs::parse();

    if args.path.is_file() {
        format_file(&args.path, args.in_place)?;
    } else if args.path.is_dir() {
        format_directory(&args.path, args.in_place)?;
    } else {
        eprintln!(
            "Error: {} is neither a file nor a directory",
            args.path.display()
        );
        std::process::exit(-1);
    }

    Ok(())
}

fn format_file(file_path: &Path, in_place: bool) -> anyhow::Result<()> {
    let input = fs::read_to_string(file_path)?;

    let lexer = Lexer::new(&input);
    let tokens = lexer.lex()?;

    let statements = match parse(&mut tokens.into()) {
        Ok(stms) => stms,
        Err(e) => {
            eprintln!("Parse error in {}: {}", file_path.display(), e);
            return Ok(()); // Continue processing other files
        }
    };

    let formatted = formatter::format_program(&statements)
        .map_err(|e| anyhow::anyhow!("Formatting error in {}: {}", file_path.display(), e))?;

    if in_place {
        fs::write(file_path, formatted)?;
        println!("Formatted: {}", file_path.display());
    } else {
        println!("{formatted}");
    }

    Ok(())
}

fn format_directory(dir_path: &Path, in_place: bool) -> anyhow::Result<()> {
    let why_files = collect_why_files(dir_path)?;

    if why_files.is_empty() {
        println!("No .why files found in {}", dir_path.display());
        return Ok(());
    }

    for file_path in why_files {
        if let Err(e) = format_file(&file_path, in_place) {
            eprintln!("Error formatting {}: {}", file_path.display(), e);
        }
    }

    Ok(())
}

fn collect_why_files(dir_path: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut why_files = Vec::new();
    collect_why_files_recursive(dir_path, &mut why_files)?;
    Ok(why_files)
}

fn collect_why_files_recursive(
    dir_path: &Path,
    why_files: &mut Vec<std::path::PathBuf>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_why_files_recursive(&path, why_files)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("why") {
            why_files.push(path);
        }
    }
    Ok(())
}
