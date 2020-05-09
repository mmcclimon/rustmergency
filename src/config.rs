use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    path: PathBuf,
}

impl Config {
    pub fn new(filename: &str) -> Self {
        Config {
            path: PathBuf::from(filename),
        }
    }
}
