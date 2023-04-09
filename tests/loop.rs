use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/loop.why";
const EXPECTED: Expected = Expected {
    stdout: "0123456789",
    stderr: "",
};

#[test]
fn compile_and_run_loop() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
