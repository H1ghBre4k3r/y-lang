use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/ackermann.why";

#[test]
fn interpret_ackermann() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-r"])
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "13");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_ackermann() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-c", "-o", "./output/ackermann"])
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/ackermann").output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "13");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
