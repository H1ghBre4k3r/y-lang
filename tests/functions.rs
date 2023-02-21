use std::{error::Error, path::Path};

use test_utils::{check_compilation, check_interpretation, Expected};

const SRC_PATH: &str = "./examples/functions.why";
const EXPECTED: Expected = Expected {
    stdout: "7 10 65",
    stderr: "",
};

#[test]
fn interpret_functions() -> Result<(), Box<dyn Error>> {
    check_interpretation(Path::new(SRC_PATH), EXPECTED)
}

#[test]
fn compile_and_run_functions() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
