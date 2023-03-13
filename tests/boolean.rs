use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/boolean.why";
const EXPECTED: Expected = Expected {
    stdout: "456123 From Function  From Function 4",
    stderr: "",
};

#[test]
fn compile_and_run_boolean() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
