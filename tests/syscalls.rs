use std::{error::Error, path::Path};

use test_utils::{check_compilation, check_interpretation, Expected};

const SRC_PATH: &str = "./examples/syscall.why";
const EXPECTED: Expected = Expected {
    stdout: "Hello, World! This thing is supposed to be very long :) 13",
    stderr: "",
};

#[test]
fn interpret_syscall() -> Result<(), Box<dyn Error>> {
    check_interpretation(Path::new(SRC_PATH), EXPECTED)
}

#[test]
fn compile_and_run_syscall() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
