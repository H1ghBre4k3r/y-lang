use std::{error::Error, path::Path};

use test_utils::{check_compilation, check_interpretation, Expected};

const SRC_PATH: &str = "./examples/print.why";
const EXPECTED: Expected = Expected {
    stdout: "literal variable function block if else ",
    stderr: "",
};

#[test]
fn interpret_print() -> Result<(), Box<dyn Error>> {
    check_interpretation(Path::new(SRC_PATH), EXPECTED)
}

#[test]
fn compile_and_run_print() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}