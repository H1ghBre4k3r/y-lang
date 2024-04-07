use anyhow::Result;
use pesca_lang::{compile_file, Cli};

fn main() -> Result<()> {
    let args = Cli::init();

    compile_file(args)
}
