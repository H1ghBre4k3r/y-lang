//! # Why
//!
//! This binary is the compiler of Y. It combines parser, type checker, and compiler into a single
//! application.
extern crate pest;
extern crate y_lang;

mod cli;
mod commands;

use cli::*;
use commands::*;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::init();

    simple_logger::init_with_level((&args.verbosity).into()).unwrap();

    match &args.command {
        Commands::Build(args) => build_executable(args),
    }
}
