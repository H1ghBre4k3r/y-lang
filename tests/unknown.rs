use std::{error::Error, path::Path};

use test_utils::check_failing_type_checking;

const SRC_PATH: &str = "./examples/unknown.why";

#[test]
fn type_check_unknown() -> Result<(), Box<dyn Error>> {
    check_failing_type_checking(Path::new(SRC_PATH))
}
