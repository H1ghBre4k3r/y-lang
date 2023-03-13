use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/ackermann.why";
const EXPECTED: Expected = Expected {
    stdout: "13",
    stderr: "",
};

#[test]
fn compile_and_run_ackermann() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
