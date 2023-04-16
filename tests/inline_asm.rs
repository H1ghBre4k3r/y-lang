use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/inline_asm.why";
const EXPECTED: Expected = Expected {
    stdout: "11",
    stderr: "",
};

#[test]
fn compile_and_run_inline_asm() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
