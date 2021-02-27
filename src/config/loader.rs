use std::fs::read_to_string;
use std::path::PathBuf;
use serde::Deserialize;
use crate::config::LoadingError;

pub fn deserialize_config<'de, T>(raw_file: &'de str) -> Result<T, LoadingError>
    where T: Deserialize<'de>
{
    toml::from_str::<T>(raw_file)
        .map_err(|why| LoadingError::DeserializationError(format!("{}", why)))
}

pub fn read_config(path: &PathBuf) -> Result<String, LoadingError> {
    read_to_string(path)
        .map_err(LoadingError::IoError)
}