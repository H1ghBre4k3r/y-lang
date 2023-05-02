use std::{error::Error, path::Path};

use test_utils::{check_compilation, Expected};

const SRC_PATH: &str = "./examples/arrays.why";
const EXPECTED: Expected = Expected {
    stdout: "42 42 17 17 13 1337 13 Hehl HOhl Hello World! 5 10 5 13 1337 ",
    stderr: "",
};

#[test]
fn compile_and_run_arrays() -> Result<(), Box<dyn Error>> {
    check_compilation(Path::new(SRC_PATH), EXPECTED)
}
