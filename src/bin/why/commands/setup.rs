use std::{error::Error, io::Write};

use include_dir::{Dir, File};

use crate::LIBRARY_DIR;

pub fn setup_library() -> Result<(), Box<dyn Error>> {
    let why_directory = format!(
        "{}/.why/lib",
        home::home_dir().unwrap_or(".".into()).to_string_lossy()
    );

    std::fs::remove_dir_all(&why_directory)?;

    std::fs::create_dir_all(&why_directory)?;

    create_directory(&why_directory, &LIBRARY_DIR)?;

    Ok(())
}

fn create_directory(parent: &str, directory: &Dir) -> Result<(), Box<dyn Error>> {
    let path = format!("{parent}/{}", directory.path().to_string_lossy());

    std::fs::create_dir_all(&path)?;

    for entry in directory.entries() {
        println!("{entry:#?}");
        match entry {
            include_dir::DirEntry::Dir(dir) => create_directory(parent, dir)?,
            include_dir::DirEntry::File(file) => create_file(parent, file)?,
        }
    }

    Ok(())
}

fn create_file(base: &str, file: &File) -> Result<(), Box<dyn Error>> {
    let path = format!("{base}/{}", file.path().to_string_lossy());

    let mut file_to_write = std::fs::File::create(path)?;

    file_to_write.write_all(file.contents())?;
    Ok(())
}
