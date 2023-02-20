use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/expressions.why";
const EXPECTED_OUTPUT: &str = "22 39 -201";

#[test]
fn interpret_expressions() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-r"])
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        EXPECTED_OUTPUT
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_expressions() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-o", "./output/expressions"])
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/expressions").output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        EXPECTED_OUTPUT
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
