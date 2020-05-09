use serde::Deserialize;
use std::fmt::Debug;

use crate::config::RemoteConfig;

#[derive(Debug, Deserialize, Clone)]
pub enum Impl {
  Github,
  GitLab,
}

pub trait Remote: Debug {
  fn config(&self) -> &RemoteConfig;
  fn obtain_clone_url(&self);
  fn get_mrs_for_label(&self);
  fn ua(&self);
}

// Github
// ------

#[derive(Debug)]
pub struct Github {
  name:   String,
  config: RemoteConfig,
}

impl Github {
  pub fn new(name: &str, cfg: &RemoteConfig) -> Self {
    Self {
      name:   name.to_string(),
      config: cfg.clone(),
    }
  }
}

impl Remote for Github {
  fn obtain_clone_url(&self) {}
  fn get_mrs_for_label(&self) {}
  fn ua(&self) {}

  fn config(&self) -> &RemoteConfig { &self.config }
}

// GitLab
// ------

#[derive(Debug)]
pub struct GitLab {
  name:   String,
  config: RemoteConfig,
}

impl GitLab {
  pub fn new(name: &str, cfg: &RemoteConfig) -> Self {
    Self {
      name:   name.to_string(),
      config: cfg.clone(),
    }
  }
}

impl Remote for GitLab {
  fn obtain_clone_url(&self) {}
  fn get_mrs_for_label(&self) {}
  fn ua(&self) {}

  fn config(&self) -> &RemoteConfig { &self.config }
}
