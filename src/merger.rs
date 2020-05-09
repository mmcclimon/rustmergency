use crate::config::Config;

#[derive(Debug)]
pub struct Merger {
    pub config: Config,
}

impl Merger {
    pub fn from_config_file(file: &str) -> Self {
        let config = Config::new(file).unwrap();
        Merger { config }
    }
}
