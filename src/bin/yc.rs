use anyhow::Result;
use pesca_lang::{Cli, compile_file};

fn main() -> Result<()> {
    let args = Cli::init();

    compile_file(args)
}
