use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/printi.why";

#[test]
fn interpret_printi() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-r"])
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "3 17 42 13 69 4");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_printi() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-c", "-o", "./output/printi"])
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/printi").output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "3 17 42 13 69 4");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
