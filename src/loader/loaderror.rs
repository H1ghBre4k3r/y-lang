use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct FileLoadError {
    pub message: String,
    pub position: (String, usize, usize),
}

impl Display for FileLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (file, line, col) = &self.position;
        f.write_str(&format!("{} ({}:{}:{})", self.message, file, line, col))
    }
}

impl Error for FileLoadError {}
