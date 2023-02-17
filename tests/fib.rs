use std::{error::Error, process::Command};

const FILE_NAME: &str = "./examples/fib.why";

#[test]
fn interpret_fib() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-r"])
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "6765");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    Ok(())
}

#[test]
fn compile_and_run_fib() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./target/debug/why")
        .args(["-f", FILE_NAME, "-o", "./output/fib"])
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");

    let output = Command::new("./output/fib").output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "6765");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    Ok(())
}
