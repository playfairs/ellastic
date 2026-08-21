use crate::graph::{
  DataType,
  PipelineNode,
  PipelineNodeType,
  Port,
  PortType,
};
use chrono::{
  DateTime,
  Utc,
};
use ellastic_audio::{
  AudioData,
  AudioProcessor,
};
use ellastic_core::{
  MediaData,
  MediaType,
};
use ellastic_effects::{
  EffectProcessor,
  EffectType,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_glitch::{
  GlitchEffect,
  GlitchProcessor,
};
use ellastic_image::{
  ImageData,
  ImageProcessor,
};
use ellastic_media::MediaProcessor;
use ellastic_utils::create_random_generator;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NodeProcessor {
  pub node: PipelineNode,
  pub processor: Box<dyn NodeProcessorTrait + Send + Sync>,
  pub cache: Arc<RwLock<NodeCache>>,
  pub performance_stats: Arc<RwLock<NodePerformanceStats>>,
}

#[derive(Debug, Clone)]
pub struct NodeCache {
  cache: HashMap<String, MediaProcessor>,
  max_size: usize,
}

#[derive(Debug, Clone)]
pub struct NodePerformanceStats {
  execution_times: Vec<std::time::Duration>,
  total_executions: u64,
  total_time: std::time::Duration,
  min_time: std::time::Duration,
  max_time: std::time::Duration,
}

pub trait NodeProcessorTrait: Send + Sync {
  fn process(
    &mut self,
    input: &MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor>;
  fn validate_input(&self, input: &MediaProcessor) -> Result<()>;
  fn get_supported_media_types(&self) -> Vec<MediaType>;
  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync>;
}

impl NodeProcessor {
  pub fn new(node: PipelineNode, processor: Box<dyn NodeProcessorTrait + Send + Sync>) -> Self {
    Self {
      node,
      processor,
      cache: Arc::new(RwLock::new(NodeCache::new())),
      performance_stats: Arc::new(RwLock::new(NodePerformanceStats::new())),
    }
  }

  pub fn node(&self) -> &PipelineNode {
    &self.node
  }

  pub fn node_mut(&mut self) -> &mut PipelineNode {
    &mut self.node
  }

  pub fn cache(&self) -> Arc<RwLock<NodeCache>> {
    self.cache.clone()
  }

  pub fn performance_stats(&self) -> Arc<RwLock<NodePerformanceStats>> {
    self.performance_stats.clone()
  }

  pub fn process(
    &mut self,
    input: MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    let start_time = std::time::Instant::now();

    self.processor.validate_input(&input)?;

    let cache_key = self.generate_cache_key(&input, parameters);
    if let Some(cached_result) = self.cache.read().get(&cache_key) {
      return Ok(cached_result.clone());
    }

    let result = self.processor.process(&input, parameters)?;

    self.cache.write().put(cache_key, result.clone());

    let processing_time = start_time.elapsed();
    self
      .performance_stats
      .write()
      .record_execution(processing_time);

    Ok(result)
  }

  fn generate_cache_key(
    &self,
    input: &MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{
      Hash,
      Hasher,
    };

    let mut hasher = DefaultHasher::new();

    self.node.id.hash(&mut hasher);
    input.data().hash(&mut hasher);

    let mut param_keys: Vec<_> = parameters.keys().collect();
    param_keys.sort();

    for key in param_keys {
      key.hash(&mut hasher);
      parameters[key].hash(&mut hasher);
    }

    format!(
      "{}:{}:{}",
      self.node.id,
      hasher.finish(),
      input.data().len()
    )
  }

  pub fn clone(&self) -> NodeProcessor {
    NodeProcessor {
      node: self.node.clone(),
      processor: self.processor.clone_box(),
      cache: self.cache.clone(),
      performance_stats: self.performance_stats.clone(),
    }
  }
}

impl NodeCache {
  pub fn new() -> Self {
    Self {
      cache: HashMap::new(),
      max_size: 100,
    }
  }

  pub fn with_max_size(max_size: usize) -> Self {
    Self {
      cache: HashMap::new(),
      max_size,
    }
  }

  pub fn get(&self, key: &str) -> Option<&MediaProcessor> {
    self.cache.get(key)
  }

  pub fn put(&mut self, key: String, processor: MediaProcessor) {
    if self.cache.len() >= self.max_size {
      if let Some(oldest_key) = self.cache.keys().next().cloned() {
        self.cache.remove(&oldest_key);
      }
    }
    self.cache.insert(key, processor);
  }

  pub fn remove(&mut self, key: &str) -> Option<MediaProcessor> {
    self.cache.remove(key)
  }

  pub fn clear(&mut self) {
    self.cache.clear();
  }

  pub fn len(&self) -> usize {
    self.cache.len()
  }

  pub fn is_empty(&self) -> bool {
    self.cache.is_empty()
  }

  pub fn clone(&self) -> NodeCache {
    NodeCache {
      cache: self.cache.clone(),
      max_size: self.max_size,
    }
  }
}

impl NodePerformanceStats {
  pub fn new() -> Self {
    Self {
      execution_times: Vec::new(),
      total_executions: 0,
      total_time: std::time::Duration::ZERO,
      min_time: std::time::Duration::MAX,
      max_time: std::time::Duration::ZERO,
    }
  }

  pub fn record_execution(&mut self, execution_time: std::time::Duration) {
    self.execution_times.push(execution_time);
    self.total_executions += 1;
    self.total_time += execution_time;

    if execution_time < self.min_time {
      self.min_time = execution_time;
    }

    if execution_time > self.max_time {
      self.max_time = execution_time;
    }

    if self.execution_times.len() > 50 {
      self.execution_times.drain(0..25);
    }
  }

  pub fn average_time(&self) -> std::time::Duration {
    if self.total_executions == 0 {
      std::time::Duration::ZERO
    } else {
      self.total_time / self.total_executions as u32
    }
  }

  pub fn min_time(&self) -> std::time::Duration {
    self.min_time
  }

  pub fn max_time(&self) -> std::time::Duration {
    self.max_time
  }

  pub fn total_executions(&self) -> u64 {
    self.total_executions
  }

  pub fn total_time(&self) -> std::time::Duration {
    self.total_time
  }

  pub fn recent_average_time(&self, count: usize) -> std::time::Duration {
    let recent_times: Vec<_> = self.execution_times.iter().rev().take(count).collect();
    if recent_times.is_empty() {
      std::time::Duration::ZERO
    } else {
      let sum: std::time::Duration = recent_times.iter().sum();
      sum / recent_times.len() as u32
    }
  }

  pub fn reset(&mut self) {
    self.execution_times.clear();
    self.total_executions = 0;
    self.total_time = std::time::Duration::ZERO;
    self.min_time = std::time::Duration::MAX;
    self.max_time = std::time::Duration::ZERO;
  }

  pub fn clone(&self) -> NodePerformanceStats {
    NodePerformanceStats {
      execution_times: self.execution_times.clone(),
      total_executions: self.total_executions,
      total_time: self.total_time,
      min_time: self.min_time,
      max_time: self.max_time,
    }
  }
}

#[derive(Debug, Clone)]
pub struct InputNodeProcessor {
  media_type: MediaType,
}

impl InputNodeProcessor {
  pub fn new(media_type: MediaType) -> Self {
    Self { media_type }
  }
}

impl NodeProcessorTrait for InputNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    _parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    Ok(input.clone())
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    if input.media_type() != self.media_type {
      return Err(EllasticError::InvalidParameter(format!(
        "Input node expects {:?}, got {:?}",
        self.media_type,
        input.media_type()
      )));
    }
    Ok(())
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    vec![self.media_type]
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct OutputNodeProcessor {
  media_type: MediaType,
}

impl OutputNodeProcessor {
  pub fn new(media_type: MediaType) -> Self {
    Self { media_type }
  }
}

impl NodeProcessorTrait for OutputNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    _parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    Ok(input.clone())
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    if input.media_type() != self.media_type {
      return Err(EllasticError::InvalidParameter(format!(
        "Output node expects {:?}, got {:?}",
        self.media_type,
        input.media_type()
      )));
    }
    Ok(())
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    vec![self.media_type]
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct EffectNodeProcessor {
  effect_type: EffectType,
}

impl EffectNodeProcessor {
  pub fn new(effect_type: EffectType) -> Self {
    Self { effect_type }
  }
}

impl NodeProcessorTrait for EffectNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    let mut effect_processor = EffectProcessor::new(self.effect_type.clone());

    for (key, value) in parameters {
      effect_processor.set_parameter(key.clone(), value.clone())?;
    }

    effect_processor.apply_effect(input)
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    match &self.effect_type {
      EffectType::Image { .. } => {
        if input.media_type() != MediaType::Image {
          return Err(EllasticError::InvalidParameter(
            "Image effect requires image input".to_string(),
          ));
        }
      }
      EffectType::Audio { .. } => {
        if input.media_type() != MediaType::Audio {
          return Err(EllasticError::InvalidParameter(
            "Audio effect requires audio input".to_string(),
          ));
        }
      }
      EffectType::Video { .. } => {
        if input.media_type() != MediaType::Video {
          return Err(EllasticError::InvalidParameter(
            "Video effect requires video input".to_string(),
          ));
        }
      }
      EffectType::Composite { .. } => {
        if !matches!(input.media_type(), MediaType::Image | MediaType::Video) {
          return Err(EllasticError::InvalidParameter(
            "Composite effect requires image or video input".to_string(),
          ));
        }
      }
      EffectType::Custom { .. } => {}
    }

    Ok(())
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    match &self.effect_type {
      EffectType::Image { .. } => vec![MediaType::Image],
      EffectType::Audio { .. } => vec![MediaType::Audio],
      EffectType::Video { .. } => vec![MediaType::Video],
      EffectType::Composite { .. } => vec![MediaType::Image, MediaType::Video],
      EffectType::Custom {
        supported_media_types,
        ..
      } => supported_media_types.clone(),
    }
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct FilterNodeProcessor {
  filter_type: crate::graph::FilterType,
}

impl FilterNodeProcessor {
  pub fn new(filter_type: crate::graph::FilterType) -> Self {
    Self { filter_type }
  }
}

impl NodeProcessorTrait for FilterNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    match self.filter_type {
      crate::graph::FilterType::Gaussian => {
        let radius = parameters
          .get("radius")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(1.0);
        input.blur(radius)
      }
      crate::graph::FilterType::Median => {
        let kernel_size = parameters
          .get("kernel_size")
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(3);
        input.median_filter(kernel_size)
      }
      crate::graph::FilterType::Bilateral => {
        let sigma = parameters
          .get("sigma")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(1.0);
        input.bilateral_filter(sigma)
      }
      crate::graph::FilterType::LowPass => {
        let cutoff = parameters
          .get("cutoff")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(1000.0);
        input.low_pass_filter(cutoff)
      }
      crate::graph::FilterType::HighPass => {
        let cutoff = parameters
          .get("cutoff")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(1000.0);
        input.high_pass_filter(cutoff)
      }
      crate::graph::FilterType::BandPass => {
        let low_cutoff = parameters
          .get("low_cutoff")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(500.0);
        let high_cutoff = parameters
          .get("high_cutoff")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(2000.0);
        input.band_pass_filter(low_cutoff, high_cutoff)
      }
      crate::graph::FilterType::Notch => {
        let center = parameters
          .get("center")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(1000.0);
        let bandwidth = parameters
          .get("bandwidth")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(100.0);
        input.notch_filter(center, bandwidth)
      }
      crate::graph::FilterType::Custom => Err(EllasticError::UnsupportedOperation(
        "Custom filter not implemented".to_string(),
      )),
    }
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    match self.filter_type {
      crate::graph::FilterType::LowPass
      | crate::graph::FilterType::HighPass
      | crate::graph::FilterType::BandPass
      | crate::graph::FilterType::Notch => {
        if input.media_type() != MediaType::Audio {
          return Err(EllasticError::InvalidParameter(
            "Audio filter requires audio input".to_string(),
          ));
        }
      }
      crate::graph::FilterType::Gaussian
      | crate::graph::FilterType::Median
      | crate::graph::FilterType::Bilateral => {
        if input.media_type() != MediaType::Image {
          return Err(EllasticError::InvalidParameter(
            "Image filter requires image input".to_string(),
          ));
        }
      }
      crate::graph::FilterType::Custom => {}
    }

    Ok(())
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    match self.filter_type {
      crate::graph::FilterType::LowPass
      | crate::graph::FilterType::HighPass
      | crate::graph::FilterType::BandPass
      | crate::graph::FilterType::Notch => {
        vec![MediaType::Audio]
      }
      crate::graph::FilterType::Gaussian
      | crate::graph::FilterType::Median
      | crate::graph::FilterType::Bilateral => {
        vec![MediaType::Image]
      }
      crate::graph::FilterType::Custom => {
        vec![MediaType::Image, MediaType::Audio, MediaType::Video]
      }
    }
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct TransformNodeProcessor {
  transform_type: crate::graph::TransformType,
}

impl TransformNodeProcessor {
  pub fn new(transform_type: crate::graph::TransformType) -> Self {
    Self { transform_type }
  }
}

impl NodeProcessorTrait for TransformNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    match self.transform_type {
      crate::graph::TransformType::Scale => {
        let scale_x = parameters
          .get("scale_x")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(1.0);
        let scale_y = parameters
          .get("scale_y")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(1.0);
        input.scale(scale_x, scale_y)
      }
      crate::graph::TransformType::Rotate => {
        let angle = parameters
          .get("angle")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(0.0);
        input.rotate(angle)
      }
      crate::graph::TransformType::Translate => {
        let translate_x = parameters
          .get("translate_x")
          .and_then(|s| s.parse::<i32>().ok())
          .unwrap_or(0);
        let translate_y = parameters
          .get("translate_y")
          .and_then(|s| s.parse::<i32>().ok())
          .unwrap_or(0);
        input.translate(translate_x, translate_y)
      }
      crate::graph::TransformType::Crop => {
        let x = parameters
          .get("x")
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(0);
        let y = parameters
          .get("y")
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(0);
        let width = parameters
          .get("width")
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(input.width());
        let height = parameters
          .get("height")
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(input.height());
        input.crop(x, y, width, height)
      }
      crate::graph::TransformType::Flip => {
        let direction = parameters
          .get("direction")
          .unwrap_or(&"horizontal".to_string());
        match direction.as_str() {
          "horizontal" => input.flip_horizontal(),
          "vertical" => input.flip_vertical(),
          "both" => {
            input.flip_horizontal()?;
            input.flip_vertical()
          }
          _ => Err(EllasticError::InvalidParameter(
            "Invalid flip direction".to_string(),
          )),
        }
      }
      crate::graph::TransformType::Resize => {
        let width = parameters
          .get("width")
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(input.width());
        let height = parameters
          .get("height")
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(input.height());
        input.resize(width, height)
      }
      crate::graph::TransformType::ColorSpace => {
        let color_space = parameters.get("color_space").unwrap_or(&"RGB".to_string());
        match color_space.as_str() {
          "RGB" => input.convert_to_rgb(),
          "RGBA" => input.convert_to_rgba(),
          "HSV" => input.convert_to_hsv(),
          "HSL" => input.convert_to_hsl(),
          "GRAY" => input.grayscale(),
          _ => Err(EllasticError::InvalidParameter(
            "Invalid color space".to_string(),
          )),
        }
      }
      crate::graph::TransformType::Custom => Err(EllasticError::UnsupportedOperation(
        "Custom transform not implemented".to_string(),
      )),
    }
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    match self.transform_type {
      crate::graph::TransformType::Scale
      | crate::graph::TransformType::Rotate
      | crate::graph::TransformType::Translate
      | crate::graph::TransformType::Crop
      | crate::graph::TransformType::Flip
      | crate::graph::TransformType::Resize => {
        if !matches!(input.media_type(), MediaType::Image | MediaType::Video) {
          return Err(EllasticError::InvalidParameter(
            "Transform requires image or video input".to_string(),
          ));
        }
      }
      crate::graph::TransformType::ColorSpace => {
        if input.media_type() != MediaType::Image {
          return Err(EllasticError::InvalidParameter(
            "Color space transform requires image input".to_string(),
          ));
        }
      }
      crate::graph::TransformType::Custom => {}
    }

    Ok(())
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    match self.transform_type {
      crate::graph::TransformType::Scale
      | crate::graph::TransformType::Rotate
      | crate::graph::TransformType::Translate
      | crate::graph::TransformType::Crop
      | crate::graph::TransformType::Flip
      | crate::graph::TransformType::Resize => {
        vec![MediaType::Image, MediaType::Video]
      }
      crate::graph::TransformType::ColorSpace => {
        vec![MediaType::Image]
      }
      crate::graph::TransformType::Custom => {
        vec![MediaType::Image, MediaType::Audio, MediaType::Video]
      }
    }
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct BranchNodeProcessor {
  condition: crate::graph::BranchCondition,
}

impl BranchNodeProcessor {
  pub fn new(condition: crate::graph::BranchCondition) -> Self {
    Self { condition }
  }
}

impl NodeProcessorTrait for BranchNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    _parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    Ok(input.clone())
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    match &self.condition {
      crate::graph::BranchCondition::MediaType(media_type) => {
        if input.media_type() != *media_type {
          return Err(EllasticError::InvalidParameter(format!(
            "Branch condition requires {:?}, got {:?}",
            media_type,
            input.media_type()
          )));
        }
      }
      crate::graph::BranchCondition::Parameter { .. } => {}
      crate::graph::BranchCondition::Custom { .. } => {}
    }

    Ok(())
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    match &self.condition {
      crate::graph::BranchCondition::MediaType(media_type) => {
        vec![*media_type]
      }
      crate::graph::BranchCondition::Parameter { .. }
      | crate::graph::BranchCondition::Custom { .. } => {
        vec![MediaType::Image, MediaType::Audio, MediaType::Video]
      }
    }
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct MergeNodeProcessor {
  merge_type: crate::graph::MergeType,
}

impl MergeNodeProcessor {
  pub fn new(merge_type: crate::graph::MergeType) -> Self {
    Self { merge_type }
  }
}

impl NodeProcessorTrait for MergeNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    Ok(input.clone())
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    Ok(())
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    vec![MediaType::Image, MediaType::Audio, MediaType::Video]
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct TemplateNodeProcessor {
  template_node_name: String,
}

impl TemplateNodeProcessor {
  pub fn new(template_node_name: String) -> Self {
    Self { template_node_name }
  }
}

impl NodeProcessorTrait for TemplateNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    Ok(input.clone())
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    Ok(())
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    vec![MediaType::Image, MediaType::Audio, MediaType::Video]
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct CustomNodeProcessor {
  custom_type: String,
  process_function:
    Box<dyn Fn(&MediaProcessor, &HashMap<String, String>) -> Result<MediaProcessor> + Send + Sync>,
  validate_function: Box<dyn Fn(&MediaProcessor) -> Result<()> + Send + Sync>,
  supported_media_types: Vec<MediaType>,
}

impl CustomNodeProcessor {
  pub fn new(
    custom_type: String,
    process_function: Box<
      dyn Fn(&MediaProcessor, &HashMap<String, String>) -> Result<MediaProcessor> + Send + Sync,
    >,
    validate_function: Box<dyn Fn(&MediaProcessor) -> Result<()> + Send + Sync>,
    supported_media_types: Vec<MediaType>,
  ) -> Self {
    Self {
      custom_type,
      process_function,
      validate_function,
      supported_media_types,
    }
  }
}

impl NodeProcessorTrait for CustomNodeProcessor {
  fn process(
    &mut self,
    input: &MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<MediaProcessor> {
    (self.process_function)(input, parameters)
  }

  fn validate_input(&self, input: &MediaProcessor) -> Result<()> {
    (self.validate_function)(input)
  }

  fn get_supported_media_types(&self) -> Vec<MediaType> {
    self.supported_media_types.clone()
  }

  fn clone_box(&self) -> Box<dyn NodeProcessorTrait + Send + Sync> {
    Box::new(self.clone())
  }
}

pub fn create_node_processor(node: PipelineNode) -> Result<NodeProcessor> {
  let processor: Box<dyn NodeProcessorTrait + Send + Sync> = match node.node_type {
    PipelineNodeType::Input { media_type } => Box::new(InputNodeProcessor::new(media_type)),
    PipelineNodeType::Output { media_type } => Box::new(OutputNodeProcessor::new(media_type)),
    PipelineNodeType::Effect { effect_type } => Box::new(EffectNodeProcessor::new(effect_type)),
    PipelineNodeType::Filter { filter_type } => Box::new(FilterNodeProcessor::new(filter_type)),
    PipelineNodeType::Transform { transform_type } => {
      Box::new(TransformNodeProcessor::new(transform_type))
    }
    PipelineNodeType::Branch { condition } => Box::new(BranchNodeProcessor::new(condition)),
    PipelineNodeType::Merge { merge_type } => Box::new(MergeNodeProcessor::new(merge_type)),
    PipelineNodeType::TemplateNode { template_node_name } => {
      Box::new(TemplateNodeProcessor::new(template_node_name))
    }
    PipelineNodeType::Custom { custom_type } => Box::new(CustomNodeProcessor::new(
      custom_type,
      Box::new(|input, _| Ok(input.clone())),
      Box::new(|_| Ok(())),
      vec![MediaType::Image, MediaType::Audio, MediaType::Video],
    )),
  };

  Ok(NodeProcessor::new(node, processor))
}

pub fn create_input_node_processor(media_type: MediaType) -> InputNodeProcessor {
  InputNodeProcessor::new(media_type)
}

pub fn create_output_node_processor(media_type: MediaType) -> OutputNodeProcessor {
  OutputNodeProcessor::new(media_type)
}

pub fn create_effect_node_processor(effect_type: EffectType) -> EffectNodeProcessor {
  EffectNodeProcessor::new(effect_type)
}

pub fn create_filter_node_processor(filter_type: crate::graph::FilterType) -> FilterNodeProcessor {
  FilterNodeProcessor::new(filter_type)
}

pub fn create_transform_node_processor(
  transform_type: crate::graph::TransformType,
) -> TransformNodeProcessor {
  TransformNodeProcessor::new(transform_type)
}

pub fn create_branch_node_processor(
  condition: crate::graph::BranchCondition,
) -> BranchNodeProcessor {
  BranchNodeProcessor::new(condition)
}

pub fn create_merge_node_processor(merge_type: crate::graph::MergeType) -> MergeNodeProcessor {
  MergeNodeProcessor::new(merge_type)
}

pub fn create_template_node_processor(template_node_name: String) -> TemplateNodeProcessor {
  TemplateNodeProcessor::new(template_node_name)
}

pub fn create_custom_node_processor(
  custom_type: String,
  process_function: Box<
    dyn Fn(&MediaProcessor, &HashMap<String, String>) -> Result<MediaProcessor> + Send + Sync,
  >,
  validate_function: Box<dyn Fn(&MediaProcessor) -> Result<()> + Send + Sync>,
  supported_media_types: Vec<MediaType>,
) -> CustomNodeProcessor {
  CustomNodeProcessor::new(
    custom_type,
    process_function,
    validate_function,
    supported_media_types,
  )
}
