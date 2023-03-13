use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/expressions.why";
const EXPECTED: Expected = Expected {
    stdout: "22 39 -201",
    stderr: "",
};

#[test]
fn compile_and_run_expressions() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
