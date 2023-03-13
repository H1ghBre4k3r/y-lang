use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/hello.why";
const EXPECTED: Expected = Expected {
    stdout: "Hello, World!",
    stderr: "",
};

#[test]
fn compile_and_run_hello() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
