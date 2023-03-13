use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/import.why";
const EXPECTED: Expected = Expected {
    stdout: "42 10",
    stderr: "",
};

#[test]
fn compile_and_run_import() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
