use std::{collections::HashMap, fs::File, io::Read};

use serde::Deserialize;
use toml;

use crate::errors::{MergerError, MergerResult};
use crate::remote::{self, GitLab, Github, Remote};
use crate::step::BuildStep;

type RemoteCollection = HashMap<String, Box<dyn Remote>>;

#[derive(Debug)]
pub struct Config {
  remotes:     RemoteCollection,
  meta:        MetaConfig,
  local:       LocalConfig,
  build_steps: Vec<BuildStep>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct RemoteConfig {
  interface:   remote::Impl,
  pub api_url: String,
  pub api_key: String,
  pub repo:    String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StepConfig {
  name:        String,
  remote:      String,
  label:       String,
  trusted_org: Option<String>,
  tag_format:  Option<String>,
  push_tag_to: Option<String>,
}

impl Config {
  pub fn new(filename: &str) -> MergerResult<Self> {
    let mut file = File::open(filename)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    let cfg: RawConfig = toml::from_str(&s)
      .map_err(|e| MergerError::De(filename.to_string(), e))?;

    let remotes = cfg.assemble_remotes();
    let build_steps = cfg.assemble_steps(&remotes)?;

    let meta = cfg.meta;
    let local = cfg.local;

    Ok(Config {
      remotes,
      meta,
      local,
      build_steps,
    })
  }

  fn default_committer_name() -> String { "Mergeotron".to_string() }
}

impl RawConfig {
  fn assemble_remotes(&self) -> RemoteCollection {
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

  fn assemble_steps(
    &self,
    remotes: &RemoteCollection,
  ) -> MergerResult<Vec<BuildStep>> {
    let mut ret = Vec::with_capacity(remotes.len());

    for step in &self.build_steps {
      if !remotes.contains_key(&step.remote) {
        let err = format!(
          "step {} wants a remote named {}, but corresponding remote not found",
          step.name, step.remote
        );

        return Err(MergerError::Config(err));
      }

      ret.push(BuildStep::new(step));
    }

    Ok(ret)
  }
}
