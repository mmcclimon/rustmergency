use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{
  blocking::Client as HttpClient, blocking::Response as HttpResponse, header,
};
use serde::Deserialize;
use std::cell::RefCell;
use std::fmt::Debug;

use crate::config::RemoteConfig;
use crate::errors::MergerResult;

#[derive(Debug, Deserialize, Clone)]
pub enum Impl {
  Github,
  GitLab,
}

pub trait Remote: Debug {
  fn config(&self) -> &RemoteConfig;
  fn name(&self) -> &str;
  fn clone_url(&self) -> MergerResult<String>;
  fn get_mrs_for_label(&self);
  fn ua(&self) -> &HttpClient;

  fn http_get(&self, url: &str) -> MergerResult<HttpResponse> {
    Ok(self.ua().get(url).send()?)
  }
}

// Github
// ------

#[derive(Debug)]
pub struct Github {
  name:        String,
  config:      RemoteConfig,
  http_client: HttpClient,
  clone_url:   RefCell<Option<String>>,
}

impl Github {
  pub fn new(name: &str, cfg: &RemoteConfig) -> Self {
    let mut headers = header::HeaderMap::new();
    headers.insert(
      header::AUTHORIZATION,
      header::HeaderValue::from_str(&format!("token {}", cfg.api_key)).unwrap(),
    );

    headers.insert(
      header::ACCEPT,
      header::HeaderValue::from_str("application/vnd.github.v3+json").unwrap(),
    );

    let client = HttpClient::builder()
      .default_headers(headers)
      .build()
      .unwrap();

    Self {
      name:        name.to_string(),
      config:      cfg.clone(),
      http_client: client,
      clone_url:   RefCell::new(None),
    }
  }

  fn uri_for(&self, part: &str) -> String {
    format!("{}/repos/{}{}", self.config.api_url, self.config.repo, part)
  }
}

impl Remote for Github {
  fn get_mrs_for_label(&self) {}

  fn ua(&self) -> &HttpClient {
    &self.http_client
  }

  fn config(&self) -> &RemoteConfig {
    &self.config
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn clone_url(&self) -> MergerResult<String> {
    if self.clone_url.borrow().is_none() {
      let res = self.http_get(&self.uri_for(""))?;

      #[derive(Debug, Deserialize)]
      struct Proj {
        ssh_url: String, // the only thing we care about
      }

      let url = res.json::<Proj>()?.ssh_url;
      self.clone_url.replace(Some(url));
    }

    Ok(self.clone_url.borrow().as_ref().unwrap().clone())
  }
}

// GitLab
// ------

#[derive(Debug)]
pub struct GitLab {
  name:        String,
  config:      RemoteConfig,
  http_client: HttpClient,
  clone_url:   RefCell<Option<String>>,
}

impl GitLab {
  pub fn new(name: &str, cfg: &RemoteConfig) -> Self {
    let mut headers = header::HeaderMap::new();
    headers.insert(
      "Private-Token",
      header::HeaderValue::from_str(&cfg.api_key).unwrap(),
    );

    let client = HttpClient::builder()
      .default_headers(headers)
      .build()
      .unwrap();

    Self {
      name:        name.to_string(),
      config:      cfg.clone(),
      http_client: client,
      clone_url:   RefCell::new(None),
    }
  }

  fn uri_for(&self, part: &str) -> String {
    format!(
      "{}/projects/{}{}",
      self.config.api_url,
      utf8_percent_encode(&self.config.repo, NON_ALPHANUMERIC),
      part
    )
  }
}

impl Remote for GitLab {
  fn ua(&self) -> &HttpClient {
    &self.http_client
  }

  fn config(&self) -> &RemoteConfig {
    &self.config
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn get_mrs_for_label(&self) {}

  fn clone_url(&self) -> MergerResult<String> {
    if self.clone_url.borrow().is_none() {
      let res = self.http_get(&self.uri_for(""))?;

      #[derive(Debug, Deserialize)]
      struct Proj {
        ssh_url_to_repo: String, // the only thing we care about
      }

      let url = res.json::<Proj>()?.ssh_url_to_repo;
      self.clone_url.replace(Some(url));
    }

    Ok(self.clone_url.borrow().as_ref().unwrap().clone())
  }
}
