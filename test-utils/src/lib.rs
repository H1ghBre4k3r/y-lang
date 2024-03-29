use std::{
    error::Error,
    io,
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

fn run_type_checker(src_path: &Path) -> Result<Output, io::Error> {
    Command::new(WHY_PATH).arg(src_path).output()
}

fn run_compiler(src_path: &Path, out_path: &Path) -> Result<Output, io::Error> {
    Command::new(WHY_PATH)
        .arg("build")
        .arg("-o")
        .arg(out_path)
        .arg(src_path)
        .output()
}

pub fn check_compilation(src_path: &Path, expected: Expected) -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(OUTPUT_PATH).join(src_path.file_stem().unwrap());

    let compile_output = run_compiler(src_path, &out_path)?;
    let compile_stdout = std::str::from_utf8(&compile_output.stdout)?;
    let compile_stderr = std::str::from_utf8(&compile_output.stderr)?;

    println!("{compile_stdout}");
    assert!(compile_stderr.is_empty(), "{}", compile_stderr);
    assert!(
        compile_output.status.success(),
        "Why compiler exited with status {:?}",
        compile_output.status.code()
    );

    let output = Command::new(out_path).output()?;

    expected.assert_matches(&output)?;
    assert!(
        output.status.success(),
        "Compiled program exited with status {:?}",
        compile_output.status.code()
    );

    Ok(())
}

pub fn check_failing_type_checking(src_path: &Path) -> Result<(), Box<dyn Error>> {
    let type_check_output = run_type_checker(src_path)?;

    println!("{type_check_output:?}");
    assert!(
        !type_check_output.status.success(),
        "Why type checker should exit with status -1"
    );

    Ok(())
}
