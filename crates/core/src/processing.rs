use std::collections::HashMap;
use uuid::Uuid;

use crate::{
  AudioData,
  BinaryData,
  ImageData,
  MediaData,
  ProcessingContext,
  Transformation,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};

#[derive(Debug, Clone)]
pub struct ProcessingPipeline {
  pub id: Uuid,
  pub name: String,
  pub transformations: Vec<PipelineStep>,
  pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct PipelineStep {
  pub transformation_id: String,
  pub enabled: bool,
  pub parameters: HashMap<String, serde_json::Value>,
}

impl ProcessingPipeline {
  pub fn new(name: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      name,
      transformations: Vec::new(),
      parameters: HashMap::new(),
    }
  }

  pub fn add_step(
    &mut self,
    transformation_id: String,
    parameters: HashMap<String, serde_json::Value>,
  ) {
    self.transformations.push(PipelineStep {
      transformation_id,
      enabled: true,
      parameters,
    });
  }

  pub fn remove_step(&mut self, index: usize) -> Result<()> {
    if index >= self.transformations.len() {
      return Err(EllasticError::InvalidPipelineStep(index));
    }
    self.transformations.remove(index);
    Ok(())
  }

  pub fn enable_step(&mut self, index: usize) -> Result<()> {
    if index >= self.transformations.len() {
      return Err(EllasticError::InvalidPipelineStep(index));
    }
    self.transformations[index].enabled = true;
    Ok(())
  }

  pub fn disable_step(&mut self, index: usize) -> Result<()> {
    if index >= self.transformations.len() {
      return Err(EllasticError::InvalidPipelineStep(index));
    }
    self.transformations[index].enabled = false;
    Ok(())
  }

  pub fn step_count(&self) -> usize {
    self.transformations.len()
  }

  pub fn is_empty(&self) -> bool {
    self.transformations.is_empty()
  }
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
  pub pipeline_id: Uuid,
  pub media_id: Uuid,
  pub success: bool,
  pub output_data: Option<MediaData>,
  pub error: Option<String>,
  pub processing_time_ms: u64,
}

impl ProcessingResult {
  pub fn success(
    pipeline_id: Uuid,
    media_id: Uuid,
    output_data: MediaData,
    processing_time_ms: u64,
  ) -> Self {
    Self {
      pipeline_id,
      media_id,
      success: true,
      output_data: Some(output_data),
      error: None,
      processing_time_ms,
    }
  }

  pub fn failure(
    pipeline_id: Uuid,
    media_id: Uuid,
    error: String,
    processing_time_ms: u64,
  ) -> Self {
    Self {
      pipeline_id,
      media_id,
      success: false,
      output_data: None,
      error: Some(error),
      processing_time_ms,
    }
  }
}

pub trait DataProcessor {
  fn process_image(&self, data: &mut ImageData, context: &ProcessingContext) -> Result<()>;
  fn process_audio(&self, data: &mut AudioData, context: &ProcessingContext) -> Result<()>;
  fn process_binary(&self, data: &mut BinaryData, context: &ProcessingContext) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum ProcessingUnit {
  ImageProcessor(Box<dyn DataProcessor + Send + Sync>),
  AudioProcessor(Box<dyn DataProcessor + Send + Sync>),
  BinaryProcessor(Box<dyn DataProcessor + Send + Sync>),
}

impl ProcessingUnit {
  pub fn process(&self, data: &mut MediaData, context: &ProcessingContext) -> Result<()> {
    match (self, data) {
      (ProcessingUnit::ImageProcessor(processor), MediaData::Image(img_data)) => {
        processor.process_image(img_data, context)
      }
      (ProcessingUnit::AudioProcessor(processor), MediaData::Audio(audio_data)) => {
        processor.process_audio(audio_data, context)
      }
      (ProcessingUnit::BinaryProcessor(processor), MediaData::Binary(binary_data)) => {
        processor.process_binary(binary_data, context)
      }
      _ => Err(EllasticError::MismatchedProcessingUnit),
    }
  }
}

#[derive(Debug, Clone)]
pub struct BatchProcessor {
  pub batch_size: usize,
  pub parallel: bool,
}

impl BatchProcessor {
  pub fn new(batch_size: usize, parallel: bool) -> Self {
    Self {
      batch_size,
      parallel,
    }
  }

  pub fn process_batch<T>(
    &self,
    items: Vec<T>,
    processor: impl Fn(T) -> Result<()> + Send + Sync,
  ) -> Result<Vec<Result<()>>> {
    if self.parallel {
      use rayon::prelude::*;
      Ok(items.into_par_iter().map(processor).collect())
    } else {
      let mut results = Vec::new();
      for item in items {
        results.push(processor(item));
      }
      Ok(results)
    }
  }
}

impl Default for BatchProcessor {
  fn default() -> Self {
    Self {
      batch_size: 10,
      parallel: true,
    }
  }
}
