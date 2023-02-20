use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/assignment.why";

#[test]
fn interpret_assignment() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args([FILE_NAME, "-r"])
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "13 17 42 1337");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_assignment() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args([FILE_NAME, "-o", "./output/assignment"])
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/assignment").output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "13 17 42 1337");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
