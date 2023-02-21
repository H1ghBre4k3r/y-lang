use std::{
    error::Error,
    path::Path,
    process::{Command, Output},
    str,
};

const WHY_PATH: &str = "./target/debug/why";
const OUTPUT_PATH: &str = "./output";

pub struct Expected<'a> {
    pub stdout: &'a str,
    pub stderr: &'a str,
}

impl<'a> Expected<'a> {
    fn assert_matches(self, output: &Output) -> Result<(), Box<dyn Error>> {
        assert_eq!(str::from_utf8(&output.stdout)?, self.stdout);
        assert_eq!(str::from_utf8(&output.stderr)?, self.stderr);
        Ok(())
    }
}

pub fn check_interpretation(src_path: &Path, expected: Expected) -> Result<(), Box<dyn Error>> {
    let output = Command::new(WHY_PATH.clone())
        .arg("-r")
        .arg(src_path)
        .output()?;

    expected.assert_matches(&output)?;

    Ok(())
}

pub fn check_compilation(src_path: &Path, expected: Expected) -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(OUTPUT_PATH).join(src_path.file_stem().unwrap());
    let compile_output = Command::new(WHY_PATH.clone())
        .arg("-o")
        .arg(&out_path)
        .arg(src_path)
        .output()?;

    println!("{}", std::str::from_utf8(&compile_output.stdout)?);
    assert!(compile_output.stderr.is_empty());

    let output = Command::new(out_path).output()?;
    expected.assert_matches(&output)?;

    Ok(())
}
