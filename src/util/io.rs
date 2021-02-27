use std::io::{Result, Error, ErrorKind};
use std::fs::File;
use std::path::PathBuf;

pub fn make_io_error(message: String) -> Error {
    Error::new(ErrorKind::Other, message)
}

pub fn open_file(path: &PathBuf) -> Result<File> {
    File::open(path)
        .map_err(|why| make_io_error(format!("Failed to open file at path [{:?}]: {}", path, why)))
}