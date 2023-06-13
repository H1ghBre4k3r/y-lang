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
use include_dir::{include_dir, Dir};
use log::error;

pub static LIBRARY_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/lib");

fn main() {
    let args = Cli::init();

    simple_logger::init_with_level((&args.verbosity).into()).unwrap();

    if let Err(error) = match &args.command {
        Commands::Build(args) => build_executable(args),
        Commands::Setup => setup_library(),
    } {
        error!("{error}");
    }
}
