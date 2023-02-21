use std::{error::Error, path::Path};

use test_utils::{check_compilation, check_interpretation, Expected};

const SRC_PATH: &str = "./examples/printi.why";
const EXPECTED: Expected = Expected {
    stdout: "3 17 42 13 69 4",
    stderr: "",
};

#[test]
fn interpret_printi() -> Result<(), Box<dyn Error>> {
    check_interpretation(Path::new(SRC_PATH), EXPECTED)
}

#[test]
fn compile_and_run_printi() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
