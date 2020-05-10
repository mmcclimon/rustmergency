use std::{cell::Cell, env, ffi::OsStr, path::PathBuf, process::Command};

use crate::config::Config;
use crate::errors::{MergerError, MergerResult};

#[derive(Debug)]
pub struct Merger {
  pub config:      Config,
  pub interactive: Cell<bool>,
}

impl Merger {
  pub fn from_config_file(file: &str) -> MergerResult<Self> {
    let config = Config::new(file)?;

    Ok(Merger {
      config,
      interactive: Cell::new(true),
    })
  }

  pub fn run(&self, auto_mode: bool) -> MergerResult<()> {
    if auto_mode {
      self.interactive.set(false);
    }

    self.prepare_local_directory()?;

    for step in &self.config.steps {
      println!("would fetch mrs for step {}", step.config.name);
    }

    if self.interactive.get() {
      self.confirm_plan();
    }

    for step in &self.config.steps {
      println!("would merge step {}", step.config.name);
    }

    self.finalize();

    Ok(())
  }

  fn prepare_local_directory(&self) -> MergerResult<()> {
    let dir = PathBuf::from(&self.config.local.path);

    if !dir.is_dir() {
      if !self.config.local.should_clone {
        let err = format!(
          "path {} does not exist, and clone was not requested",
          dir.to_string_lossy()
        );

        return Err(MergerError::Local(err));
      }

      // TODO clone.
    }

    env::set_current_dir(&dir)?;

    // make sure we're actually in a gitdir
    run_git(&["rev-parse", "--show-toplevel"]).map_err(|_| {
      MergerError::Local(format!(
        "{} does not appear to be a git directory!",
        dir.to_string_lossy(),
      ))
    })?;

    Ok(())
  }

  fn confirm_plan(&self) {}

  fn finalize(&self) {}
}

pub fn run_git<I: Clone, S>(args: I) -> MergerResult<String>
where
  I: IntoIterator<Item = S>,
  S: AsRef<OsStr>,
{
  // We clone this here for the sole purpose of generating a pretty error if git
  // fails.
  let args_clone = args.clone();

  let output = Command::new("git")
    .args(args)
    .output()
    .map_err(|e| MergerError::Io(e))?;

  if !output.status.success() {
    let cmd = args_clone
      .into_iter()
      .map(|s| (*s.as_ref()).to_string_lossy().into_owned())
      .collect::<Vec<_>>()
      .join(" ");

    let stderr = String::from_utf8(output.stderr)?;
    return Err(MergerError::Git(cmd, stderr));
  }

  Ok(String::from_utf8(output.stdout)?.trim_end().to_string())
}
