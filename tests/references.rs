use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/references.why";
const EXPECTED: Expected = Expected {
    stdout: "99",
    stderr: "",
};

#[test]
fn compile_and_run_references() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
