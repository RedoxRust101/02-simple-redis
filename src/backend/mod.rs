use crate::RespFrame;
use dashmap::DashMap;
use std::{ops::Deref, sync::Arc};

#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug)]
pub struct BackendInner {
  pub(crate) map: DashMap<String, RespFrame>,
}

impl Deref for Backend {
  type Target = Arc<BackendInner>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Default for Backend {
  fn default() -> Self {
    Self(Arc::new(BackendInner::default()))
  }
}

impl Default for BackendInner {
  fn default() -> Self {
    Self { map: DashMap::new() }
  }
}

impl Backend {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn get(&self, key: &str) -> Option<RespFrame> {
    self.map.get(key).map(|v| v.value().clone())
  }

  pub fn set(&self, key: String, value: RespFrame) {
    self.map.insert(key, value);
  }
}
