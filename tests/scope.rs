use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/scope.why";
const EXPECTED: Expected = Expected {
    stdout: "foo 42 13 foo 42 13 foo 42 13 foo foo 13 foo 42 13 13 42",
    stderr: "",
};

#[test]
fn compile_and_run_scope() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
