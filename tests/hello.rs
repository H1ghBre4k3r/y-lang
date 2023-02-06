use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/hello.why";

#[test]
fn interpret_hello() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-r"])
        .output()?;

    assert_eq!(output.stdout, "Hello, World!".to_owned().into_bytes());
    assert_eq!(output.stderr, vec![]);

    Ok(())
}

#[test]
fn compile_and_run_hello() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-c", "-o", "./output/hello"])
        .output()?;

    assert_eq!(output.stderr, vec![]);

    let output = Command::new("./output/hello").output()?;

    assert_eq!(output.stdout, "Hello, World!".to_owned().into_bytes());
    assert_eq!(output.stderr, vec![]);
    Ok(())
}
