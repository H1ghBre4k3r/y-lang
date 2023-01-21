mod ast;
mod checker;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use clap::Parser as CParser;

use crate::{
    ast::{Ast, YParser},
    checker::check_ast,
};

#[derive(CParser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    file: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    let file_content = std::fs::read_to_string(&args.file).expect(&format!(
        "Could not read file: '{}'",
        args.file.to_string_lossy()
    ));

    let pairs = YParser::parse_program(&file_content);

    println!("{:#?}", pairs);
    let ast = Ast::from_program(pairs);

    check_ast(&ast);
}
