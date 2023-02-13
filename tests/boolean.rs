use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/boolean.why";

#[test]
fn interpret_boolean() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-r"])
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "456123 From Function  From Function 4"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_boolean() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-c", "-o", "./output/boolean"])
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/boolean").output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "456123 From Function  From Function 4"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
