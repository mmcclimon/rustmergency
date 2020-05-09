use std::{collections::HashMap, fs::File, io::Read};

use serde::Deserialize;
use toml;

use crate::errors::MergerResult;

#[derive(Debug)]
pub struct Config {}

#[derive(Debug, Deserialize)]
struct RawConfig {
  local:       LocalConfig,
  meta:        MetaConfig,
  remote:      HashMap<String, RemoteConfig>,
  build_steps: Vec<StepConfig>,
}

#[derive(Debug, Deserialize)]
struct MetaConfig {
  #[serde(default = "default_committer_name")]
  committer_name:  String,
  committer_email: String,
}

#[derive(Debug, Deserialize)]
struct LocalConfig {
  path:          String,
  target_branch: String,
  upstream_base: String,
}

#[derive(Debug, Deserialize)]
struct RemoteConfig {
  interface_class: String,
  api_url:         String,
  api_key:         String,
  repo:            String,
}

#[derive(Debug, Deserialize)]
struct StepConfig {
  name:        String,
  remote:      String,
  label:       String,
  tag_format:  Option<String>,
  push_tag_to: Option<String>,
}

impl Config {
  pub fn new(filename: &str) -> MergerResult<Self> {
    let mut file = File::open(filename)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    let cfg: RawConfig = toml::from_str(&s).expect("Invalid config file");
    // let cfg = s.parse::<toml::Value>().unwrap();
    println!("{:#?}", cfg);

    Ok(Config {})
  }
}

fn default_committer_name() -> String { "Mergotron".to_string() }
