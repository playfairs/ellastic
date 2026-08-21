use rayon::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
  MediaProcessor,
  MediaType,
  ProcessingContext,
  ProcessingMode,
  ProcessingOptions,
  Transformation,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};

#[derive(Debug)]
pub struct ProcessingEngine {
  processors: Vec<Arc<dyn MediaProcessor + Send + Sync>>,
  transformations: Vec<Arc<dyn Transformation + Send + Sync>>,
}

impl ProcessingEngine {
  pub fn new() -> Self {
    Self {
      processors: Vec::new(),
      transformations: Vec::new(),
    }
  }

  pub fn register_processor<P>(&mut self, processor: P)
  where
    P: MediaProcessor + Send + Sync + 'static,
  {
    self.processors.push(Arc::new(processor));
  }

  pub fn register_transformation<T>(&mut self, transformation: T)
  where
    T: Transformation + Send + Sync + 'static,
  {
    self.transformations.push(Arc::new(transformation));
  }

  pub fn process_media(
    &self,
    data: &[u8],
    media_type: MediaType,
    context: &ProcessingContext,
    options: &ProcessingOptions,
  ) -> Result<Vec<u8>> {
    let processor = self.find_processor(media_type)?;
    let mut result = processor.process(data, context)?;

    if !context.parameters.is_empty() {
      result = self.apply_transformations(&result, context, options)?;
    }

    if options.validate_output {
      self.validate_output(&result)?;
    }

    Ok(result)
  }

  pub fn process_batch(
    &self,
    items: &[(Vec<u8>, MediaType, ProcessingContext)],
    options: &ProcessingOptions,
  ) -> Result<Vec<Result<Vec<u8>>>> {
    match options.mode {
      ProcessingMode::Parallel => Ok(
        items
          .par_iter()
          .map(|(data, media_type, context)| {
            self.process_media(data, *media_type, context, options)
          })
          .collect(),
      ),
      ProcessingMode::Sequential | ProcessingMode::Batch => {
        let mut results = Vec::new();
        for (data, media_type, context) in items {
          results.push(self.process_media(data, *media_type, context, options));
        }
        Ok(results)
      }
    }
  }

  fn find_processor(&self, media_type: MediaType) -> Result<Arc<dyn MediaProcessor + Send + Sync>> {
    self
      .processors
      .iter()
      .find(|p| p.can_process(media_type))
      .cloned()
      .ok_or_else(|| EllasticError::NoProcessorForMediaType(format!("{:?}", media_type)))
  }

  fn apply_transformations(
    &self,
    data: &[u8],
    context: &ProcessingContext,
    options: &ProcessingOptions,
  ) -> Result<Vec<u8>> {
    let mut result = data.to_vec();

    for transformation in &self.transformations {
      if context.parameters.contains_key(transformation.name()) {
        transformation.apply(&mut result, context)?;
      }
    }

    Ok(result)
  }

  fn validate_output(&self, data: &[u8]) -> Result<()> {
    if data.is_empty() {
      return Err(EllasticError::ProcessingError(
        "Output data is empty".to_string(),
      ));
    }
    Ok(())
  }

  pub fn list_available_transformations(&self) -> Vec<&str> {
    self.transformations.iter().map(|t| t.name()).collect()
  }

  pub fn get_transformation_info(&self, name: &str) -> Option<(String, &str)> {
    self
      .transformations
      .iter()
      .find(|t| t.name() == name)
      .map(|t| (t.name().to_string(), t.description()))
  }
}

impl Default for ProcessingEngine {
  fn default() -> Self {
    Self::new()
  }
}
