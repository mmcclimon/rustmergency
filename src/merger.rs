use std::{
  cell::Cell, collections::HashMap, env, ffi::OsStr, path::PathBuf,
  process::Command,
};

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
    let local_conf = &self.config.local;
    let dir = PathBuf::from(&local_conf.path);

    if !dir.is_dir() {
      let dir_string = dir.to_string_lossy();

      if !local_conf.should_clone {
        let err = format!(
          "path {} does not exist, and clone was not requested",
          dir_string,
        );

        return Err(MergerError::Local(err));
      }

      env::set_current_dir(dir.parent().unwrap())?;

      let remote = local_conf.upstream_remote(&self.config);
      let clone_url = remote.clone_url()?;

      println!("cloning into {}...", dir_string);

      run_git(&[
        "clone",
        "--recursive",
        "-o",
        remote.name(),
        &clone_url,
        &dir.file_name().unwrap().to_string_lossy(),
      ])?;
    }

    env::set_current_dir(&dir)?;

    // make sure we're actually in a gitdir
    run_git(&["rev-parse", "--show-toplevel"]).map_err(|_| {
      MergerError::Local(format!(
        "{} does not appear to be a git directory!",
        dir.to_string_lossy(),
      ))
    })?;

    self.ensure_remotes()
  }

  fn ensure_remotes(&self) -> MergerResult<()> {
    let output = run_git(&["remote", "-v"])?;

    use std::iter::FromIterator;
    let have_remotes: HashMap<&str, &str> = HashMap::from_iter(
      output
        .split("\n")
        .filter(|l| l.ends_with("(fetch)"))
        .map(|l| l.trim_end_matches(" (fetch)"))
        .map(|l| {
          let bits = l.split("\t").take(2).collect::<Vec<_>>();
          (bits[0], bits[1])
        }),
    );

    for r in self.config.all_remotes() {
      let name = r.name();
      let remote_url = &r.clone_url()?;

      if let Some(have_url) = have_remotes.get(name) {
        if have_url != remote_url {
          return Err(MergerError::Config(format!(
            "mismatched remote {}: have {}, want {}",
            name, have_url, remote_url
          )));
        }

        // nothing to do!
        continue;
      }

      println!("adding missing remote for {} at {}", name, remote_url);
      run_git(&["remote", "add", name, remote_url])?;
    }

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
