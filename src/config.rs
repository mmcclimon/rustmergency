use std::{collections::HashMap, fs::File, io::Read};

use serde::Deserialize;
use toml;

use crate::errors::{MergerError, MergerResult};
use crate::remote::{self, GitLab, Github, Remote};
use crate::step::BuildStep;

type RemoteCollection = HashMap<String, Box<dyn Remote>>;

#[derive(Debug)]
pub struct Config {
  pub remotes: RemoteCollection,
  pub meta:    MetaConfig,
  pub local:   LocalConfig,
  pub steps:   Vec<BuildStep>,
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
pub struct MetaConfig {
  #[serde(default = "Config::default_committer_name")]
  pub committer_name:  String,
  pub committer_email: String,
}

#[derive(Debug, Deserialize)]
pub struct LocalConfig {
  pub path:          String,
  pub target_branch: String,
  pub upstream_base: String,
  #[serde(default, rename = "clone")]
  pub should_clone:  bool,
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
  pub name:        String,
  pub remote:      String,
  pub label:       String,
  pub trusted_org: Option<String>,
  pub tag_format:  Option<String>,
  pub push_tag_to: Option<String>,
}

impl Config {
  pub fn new(filename: &str) -> MergerResult<Self> {
    let mut file = File::open(filename)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    let mut cfg: RawConfig = toml::from_str(&s)
      .map_err(|e| MergerError::De(filename.to_string(), e))?;

    let remotes = cfg.assemble_remotes()?;
    let steps = cfg.assemble_steps(&remotes)?;

    let meta = cfg.meta;
    let local = cfg.local;
    local.assert_ok(&remotes)?;

    Ok(Config {
      remotes,
      meta,
      local,
      steps,
    })
  }

  #[rustfmt::skip]
  fn default_committer_name() -> String { "Mergeotron".to_string() }

  pub fn remote_named(&self, name: &str) -> Option<&Box<dyn Remote>> {
    self.remotes.get(name)
  }

  pub fn all_remotes(&self) -> impl Iterator<Item = &Box<dyn Remote>> {
    self.remotes.values()
  }
}

impl RawConfig {
  fn assemble_remotes(&mut self) -> MergerResult<RemoteCollection> {
    let mut ret = HashMap::new();

    for (name, cfg) in &mut self.remotes {
      use remote::Impl;

      cfg.assert_ok(name)?;

      let remote: Box<dyn Remote> = match cfg.interface {
        Impl::Github => Box::new(Github::new(name, cfg)),
        Impl::GitLab => Box::new(GitLab::new(name, cfg)),
      };

      ret.insert(name.to_string(), remote);
    }

    Ok(ret)
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

impl LocalConfig {
  // This is here so that later we can safely unwrap the result of splitting the
  // strings.
  fn assert_ok(&self, remotes: &RemoteCollection) -> MergerResult<()> {
    if self.upstream_base.contains(" ") {
      return Err(MergerError::Config(
        "local.upstream_base contains spaces".to_string(),
      ));
    }

    if self.target_branch.contains(" ") {
      return Err(MergerError::Config(
        "target_branch contains spaces".to_string(),
      ));
    }

    if self.upstream_base.matches("/").count() != 1 {
      return Err(MergerError::Config(
        "local.upstream_base must have exactly one slash".to_string(),
      ));
    }

    let upstream = self.upstream_remote_name();

    if remotes.get(upstream).is_none() {
      return Err(MergerError::Config(format!(
        "local.upstream_base wants remote named {}, which was not found",
        upstream
      )));
    }

    Ok(())
  }

  pub fn upstream_remote<'a>(&self, config: &'a Config) -> &'a Box<dyn Remote> {
    config.remote_named(self.upstream_remote_name()).unwrap()
  }

  pub fn upstream_remote_name(&self) -> &str {
    self.upstream_base.split("/").next().unwrap()
  }

  pub fn _upstream_branch_name(&self) -> &str {
    self.upstream_base.split("/").nth(1).unwrap()
  }
}

impl RemoteConfig {
  fn assert_ok(&mut self, name: &str) -> MergerResult<()> {
    let key = &self.api_key;

    // make ENV: API keys work
    if key.starts_with("ENV:") {
      let want = key.trim_start_matches("ENV:");

      let from_env = std::env::var(want).map_err(|_| {
        let err = format!(
          "remote {} wants an environment variable named {}, which was not found",
          name, want
        );
        MergerError::Config(err)
      })?;

      self.api_key = from_env;
    }

    Ok(())
  }
}
