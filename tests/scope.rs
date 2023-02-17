use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/scope.why";

#[test]
fn interpret_scope() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-r"])
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "foo 42 13 foo 42 13 foo 42 13 foo foo 13 foo 42 13 13 42"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_scope() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-c", "-o", "./output/scope"])
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/scope").output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "foo 42 13 foo 42 13 foo 42 13 foo foo 13 foo 42 13 13 42"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
