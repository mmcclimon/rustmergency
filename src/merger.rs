use std::cell::Cell;

use crate::config::Config;

#[derive(Debug)]
pub struct Merger {
  pub config:      Config,
  pub interactive: Cell<bool>,
}

impl Merger {
  pub fn from_config_file(file: &str) -> Self {
    let config = Config::new(file);

    if let Err(e) = config {
      println!("{}", e);
      std::process::exit(1);
    }

    Merger {
      config:      config.unwrap(),
      interactive: Cell::new(true),
    }
  }

  pub fn run(&self, auto_mode: bool) {
    if auto_mode {
      self.interactive.set(false);
    }

    self.prepare_local_directory();

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
  }

  fn prepare_local_directory(&self) {}

  fn confirm_plan(&self) {}

  fn finalize(&self) {}
}
