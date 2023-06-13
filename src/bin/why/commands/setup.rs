use std::{error::Error, fmt::Display, io::Write};

use include_dir::{Dir, File};
use log::{debug, trace};

use crate::LIBRARY_DIR;

#[derive(Debug, Clone)]
enum SetupError {
    DirectoryError(String),
    FileError(String),
}

impl Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            SetupError::DirectoryError(err) => err.to_owned(),
            SetupError::FileError(err) => err.to_owned(),
        };
        f.write_str(&err)
    }
}

impl Error for SetupError {}

pub fn setup_library() -> Result<(), Box<dyn Error>> {
    debug!("starting setup bundelled library");
    let why_directory = format!(
        "{}/.why/lib",
        home::home_dir().unwrap_or(".".into()).to_string_lossy()
    );

    // first, remove the library directory
    if std::fs::remove_dir_all(&why_directory).is_err() {
        trace!("directory '{why_directory}' did not exist");
    }

    // now, create the library directory shipped with this compiler
    create_directory(&why_directory, &LIBRARY_DIR)?;

    debug!("finished setup of bundelled library");

    Ok(())
}

fn create_directory(parent: &str, directory: &Dir) -> Result<(), SetupError> {
    let path = format!("{parent}/{}", directory.path().to_string_lossy());

    println!("[SETUP] Creating '{path}'");

    if std::fs::create_dir_all(&path).is_err() {
        return Err(SetupError::DirectoryError(format!(
            "Failed to create directory '{path}'"
        )));
    };

    for entry in directory.entries() {
        match entry {
            include_dir::DirEntry::Dir(dir) => create_directory(parent, dir)?,
            include_dir::DirEntry::File(file) => create_file(parent, file)?,
        }
    }

    Ok(())
}

fn create_file(base: &str, file: &File) -> Result<(), SetupError> {
    let path = format!("{base}/{}", file.path().to_string_lossy());

    debug!("creating file '{path}'");

    let Ok(mut file_to_write) = std::fs::File::create(&path) else {
        return Err(SetupError::FileError(format!("Failed to create file '{path}'")));
    };

    if file_to_write.write_all(file.contents()).is_err() {
        return Err(SetupError::FileError(format!(
            "Failed to write contents of file '{path}'"
        )));
    };
    Ok(())
}
