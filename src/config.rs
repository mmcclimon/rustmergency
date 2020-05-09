use std::{collections::HashMap, fs::File, io::Read};

use serde::Deserialize;
use toml;

use crate::errors::{MergerError, MergerResult};
use crate::remote::{self, GitLab, Github, Remote};

#[derive(Debug)]
pub struct Config {
  remotes: HashMap<String, Box<dyn Remote>>,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
  local:       LocalConfig,
  meta:        MetaConfig,
  #[serde(rename = "remote")]
  remotes:     HashMap<String, RemoteConfig>,
  build_steps: Vec<StepConfig>,
}

#[derive(Debug, Deserialize)]
struct MetaConfig {
  #[serde(default = "Config::default_committer_name")]
  committer_name:  String,
  committer_email: String,
}

#[derive(Debug, Deserialize)]
struct LocalConfig {
  path:          String,
  target_branch: String,
  upstream_base: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RemoteConfig {
  interface:   remote::Impl,
  pub api_url: String,
  pub api_key: String,
  pub repo:    String,
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

    let cfg: RawConfig = toml::from_str(&s)
      .map_err(|e| MergerError::Config(filename.to_string(), e))?;

    let remotes = cfg.assemble_remotes();

    Ok(Config { remotes })
  }

  fn default_committer_name() -> String { "Mergeotron".to_string() }
}

impl RawConfig {
  fn assemble_remotes(&self) -> HashMap<String, Box<dyn Remote>> {
    let mut ret = HashMap::new();

    for (name, cfg) in &self.remotes {
      use remote::Impl;

      let remote: Box<dyn Remote> = match cfg.interface {
        Impl::Github => Box::new(Github::new(name, cfg)),
        Impl::GitLab => Box::new(GitLab::new(name, cfg)),
      };

      ret.insert(name.to_string(), remote);
    }

    ret
  }
}
