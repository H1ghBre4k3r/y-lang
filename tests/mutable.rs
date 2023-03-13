use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/mutable.why";
const EXPECTED: Expected = Expected {
    stdout: "3 4",
    stderr: "",
};

#[test]
fn compile_and_run_mutable() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
