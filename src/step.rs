use crate::config::StepConfig;

#[derive(Debug)]
pub struct BuildStep {
  config: StepConfig,
}

impl BuildStep {
  pub fn new(cfg: &StepConfig) -> Self {
    Self {
      config: cfg.clone(),
    }
  }
}
