use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/functions.why";

#[test]
fn interpret_functions() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args([FILE_NAME, "-r"])
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "7 10 65");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_functions() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args([FILE_NAME, "-o", "./output/functions"])
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/functions").output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "7 10 65");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
