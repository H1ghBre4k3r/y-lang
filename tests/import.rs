use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/import.why";

#[test]
fn interpret_import() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args([FILE_NAME, "-r"])
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "42 10");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_import() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args([FILE_NAME, "-o", "./output/import"])
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/import").output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "42 10");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
