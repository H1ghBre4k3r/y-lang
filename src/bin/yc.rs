use anyhow::Result;
use pesca_lang::{VCArgs, compile_file};

fn main() -> Result<()> {
    let args = VCArgs::init();

    compile_file(args)
}
