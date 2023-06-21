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
use tracing::{error, metadata::LevelFilter};

pub static LIBRARY_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/lib");

fn main() {
    let args = Cli::init();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("something went wrong");

    if let Err(error) = match &args.command {
        Commands::Build(args) => build_executable(args),
        Commands::Setup => setup_library(),
    } {
        error!("{error}");
    }
}
