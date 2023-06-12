//! # Cli
//!
//! This module contains everything needed for parsing the CLI arguments for Why.

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Struct containing the CLI configuration for Why.
#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Specify the log level of the compiler.
    #[arg(value_enum, short, long, default_value_t = LogLevel::default())]
    pub verbosity: LogLevel,
}

impl Cli {
    pub fn init() -> Self {
        Cli::parse()
    }
}

/// Enum for specifying the log level of Why.
#[derive(ValueEnum, Clone, Default, Debug)]
pub enum LogLevel {
    /// The default log level. Only critical errors will be logged.
    #[default]
    #[value(alias("0"))]
    Error,

    /// A log level, where also warning (like unused variables) are logged.
    #[value(alias("1"))]
    Warn,

    /// Also log information about the general state of the compiler, e.g., which files are
    /// compiled, etc.
    #[value(alias("2"))]
    Info,

    /// Log everything compiler internal.
    /// Note: This output can be quite clunky, since _very much_ will be logged.
    #[value(alias("3"))]
    Debug,
}

impl From<&LogLevel> for log::Level {
    fn from(value: &LogLevel) -> Self {
        match value {
            LogLevel::Error => log::Level::Error,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Info => log::Level::Info,
            LogLevel::Debug => log::Level::Debug,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Build a Y executable from source files.
    Build(BuildArgs),

    /// Setup the buildin library (i.e., std and core) on your machine
    Setup,
}

#[derive(Args, Debug, Clone)]
pub struct BuildArgs {
    /// The path to the why source file.
    #[arg(index = 1)]
    pub file: std::path::PathBuf,

    /// Whether to dump the parsed AST (for debugging).
    #[arg(long)]
    pub dump_parsed: bool,

    /// Whether to dump the type-checked AST (for debugging).
    #[arg(long)]
    pub dump_typed: bool,

    /// The path to the output binary.
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}
