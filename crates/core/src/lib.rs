use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

pub mod engine;
pub mod media;
pub mod processing;
pub mod task;

pub use engine::*;
pub use media::*;
pub use processing::*;
pub use task::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaType {
  Image,
  Audio,
  Video,
  Binary,
}

#[derive(Debug, Clone)]
pub struct MediaMetadata {
  pub id: Uuid,
  pub media_type: MediaType,
  pub format: String,
  pub path: Option<PathBuf>,
  pub size: u64,
  pub properties: HashMap<String, String>,
}

impl MediaMetadata {
  pub fn new(media_type: MediaType, format: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      media_type,
      format,
      path: None,
      size: 0,
      properties: HashMap::new(),
    }
  }

  pub fn with_path(mut self, path: PathBuf) -> Self {
    self.path = Some(path);
    self
  }

  pub fn with_size(mut self, size: u64) -> Self {
    self.size = size;
    self
  }

  pub fn with_property(mut self, key: String, value: String) -> Self {
    self.properties.insert(key, value);
    self
  }
}

#[derive(Debug, Clone)]
pub struct ProcessingContext {
  pub media_id: Uuid,
  pub parameters: HashMap<String, serde_json::Value>,
  pub seed: Option<u64>,
}

impl ProcessingContext {
  pub fn new(media_id: Uuid) -> Self {
    Self {
      media_id,
      parameters: HashMap::new(),
      seed: None,
    }
  }

  pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
    self.parameters.insert(key, value);
    self
  }

  pub fn with_seed(mut self, seed: u64) -> Self {
    self.seed = Some(seed);
    self
  }
}

pub trait MediaProcessor {
  fn process(&self, data: &[u8], context: &ProcessingContext) -> ellastic_errors::Result<Vec<u8>>;
  fn can_process(&self, media_type: MediaType) -> bool;
}

pub trait Transformation {
  fn apply(&self, data: &mut [u8], context: &ProcessingContext) -> ellastic_errors::Result<()>;
  fn name(&self) -> &str;
  fn description(&self) -> &str;
}

#[derive(Debug, Clone)]
pub enum ProcessingMode {
  Sequential,
  Parallel,
  Batch,
}

#[derive(Debug, Clone)]
pub struct ProcessingOptions {
  pub mode: ProcessingMode,
  pub preserve_original: bool,
  pub validate_output: bool,
}

impl Default for ProcessingOptions {
  fn default() -> Self {
    Self {
      mode: ProcessingMode::Sequential,
      preserve_original: true,
      validate_output: true,
    }
  }
}
