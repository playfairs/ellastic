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

pub mod analysis;
pub mod cache;
pub mod chains;
pub mod effects;
pub mod export;
pub mod parameters;
pub mod plugins;
pub mod presets;
pub mod processors;
pub mod rendering;

pub use analysis::*;
pub use cache::*;
pub use chains::*;
pub use effects::*;
pub use export::*;
pub use parameters::*;
pub use plugins::*;
pub use presets::*;
pub use processors::*;
pub use rendering::*;

#[derive(Debug, Clone)]
pub enum EffectType {
  Image { image_effect: ImageEffect },
  Audio { audio_effect: AudioEffect },
  Video { video_effect: VideoEffect },
  Composite { composite_effect: CompositeEffect },
  Custom { custom_effect: CustomEffect },
}

#[derive(Debug, Clone)]
pub enum EffectCategory {
  Basic,
  Advanced,
  Glitch,
  Filter,
  Transform,
  Color,
  Audio,
  Video,
  Composite,
  Custom,
}

#[derive(Debug, Clone)]
pub struct EffectMetadata {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub category: EffectCategory,
  pub version: String,
  pub author: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub tags: Vec<String>,
  pub parameters: Vec<EffectParameter>,
  pub supported_media_types: Vec<MediaType>,
}

impl EffectMetadata {
  pub fn new(name: String, description: String, category: EffectCategory) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name,
      description,
      category,
      version: "1.0.0".to_string(),
      author: "Ellastic Team".to_string(),
      created_at: now,
      updated_at: now,
      tags: Vec::new(),
      parameters: Vec::new(),
      supported_media_types: Vec::new(),
    }
  }

  pub fn with_parameters(mut self, parameters: Vec<EffectParameter>) -> Self {
    self.parameters = parameters;
    self
  }

  pub fn with_tags(mut self, tags: Vec<String>) -> Self {
    self.tags = tags;
    self
  }

  pub fn with_supported_media_types(mut self, media_types: Vec<MediaType>) -> Self {
    self.supported_media_types = media_types;
    self
  }

  pub fn update_timestamp(&mut self) {
    self.updated_at = Utc::now();
  }

  pub fn clone(&self) -> EffectMetadata {
    EffectMetadata {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      category: self.category,
      version: self.version.clone(),
      author: self.author.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
      tags: self.tags.clone(),
      parameters: self.parameters.clone(),
      supported_media_types: self.supported_media_types.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct EffectParameter {
  pub name: String,
  pub parameter_type: ParameterType,
  pub default_value: ParameterValue,
  pub min_value: Option<ParameterValue>,
  pub max_value: Option<ParameterValue>,
  pub description: String,
  pub required: bool,
  pub advanced: bool,
}

impl EffectParameter {
  pub fn new(name: String, parameter_type: ParameterType, default_value: ParameterValue) -> Self {
    Self {
      name,
      parameter_type,
      default_value,
      min_value: None,
      max_value: None,
      description: String::new(),
      required: false,
      advanced: false,
    }
  }

  pub fn with_range(mut self, min: ParameterValue, max: ParameterValue) -> Self {
    self.min_value = Some(min);
    self.max_value = Some(max);
    self
  }

  pub fn with_description(mut self, description: String) -> Self {
    self.description = description;
    self
  }

  pub fn required(mut self) -> Self {
    self.required = true;
    self
  }

  pub fn advanced(mut self) -> Self {
    self.advanced = true;
    self
  }

  pub fn clone(&self) -> EffectParameter {
    EffectParameter {
      name: self.name.clone(),
      parameter_type: self.parameter_type,
      default_value: self.default_value.clone(),
      min_value: self.min_value.clone(),
      max_value: self.max_value.clone(),
      description: self.description.clone(),
      required: self.required,
      advanced: self.advanced,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
  Integer,
  Float,
  Boolean,
  String,
  Color,
  Vector2,
  Vector3,
  Vector4,
  Matrix3,
  Matrix4,
  Enum(Vec<String>),
  File,
  Directory,
  Image,
  Audio,
  Video,
  Custom(String),
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
  Integer(i64),
  Float(f64),
  Boolean(bool),
  String(String),
  Color([u8; 4]),
  Vector2([f32; 2]),
  Vector3([f32; 3]),
  Vector4([f32; 4]),
  Matrix3([[f32; 3]; 3]),
  Matrix4([[f32; 4]; 4]),
  Enum(String),
  File(String),
  Directory(String),
  Image(Vec<u8>),
  Audio(Vec<u8>),
  Video(Vec<u8>),
  Custom(String, String),
}

impl ParameterValue {
  pub fn as_integer(&self) -> Option<i64> {
    match self {
      ParameterValue::Integer(v) => Some(*v),
      _ => None,
    }
  }

  pub fn as_float(&self) -> Option<f64> {
    match self {
      ParameterValue::Float(v) => Some(*v),
      _ => None,
    }
  }

  pub fn as_boolean(&self) -> Option<bool> {
    match self {
      ParameterValue::Boolean(v) => Some(*v),
      _ => None,
    }
  }

  pub fn as_string(&self) -> Option<&String> {
    match self {
      ParameterValue::String(v) => Some(v),
      _ => None,
    }
  }

  pub fn as_color(&self) -> Option<[u8; 4]> {
    match self {
      ParameterValue::Color(v) => Some(*v),
      _ => None,
    }
  }

  pub fn as_vector2(&self) -> Option<[f32; 2]> {
    match self {
      ParameterValue::Vector2(v) => Some(*v),
      _ => None,
    }
  }

  pub fn as_vector3(&self) -> Option<[f32; 3]> {
    match self {
      ParameterValue::Vector3(v) => Some(*v),
      _ => None,
    }
  }

  pub fn as_vector4(&self) -> Option<[f32; 4]> {
    match self {
      ParameterValue::Vector4(v) => Some(*v),
      _ => None,
    }
  }

  pub fn clone(&self) -> ParameterValue {
    match self {
      ParameterValue::Integer(v) => ParameterValue::Integer(*v),
      ParameterValue::Float(v) => ParameterValue::Float(*v),
      ParameterValue::Boolean(v) => ParameterValue::Boolean(*v),
      ParameterValue::String(v) => ParameterValue::String(v.clone()),
      ParameterValue::Color(v) => ParameterValue::Color(*v),
      ParameterValue::Vector2(v) => ParameterValue::Vector2(*v),
      ParameterValue::Vector3(v) => ParameterValue::Vector3(*v),
      ParameterValue::Vector4(v) => ParameterValue::Vector4(*v),
      ParameterValue::Matrix3(v) => ParameterValue::Matrix3(*v),
      ParameterValue::Matrix4(v) => ParameterValue::Matrix4(*v),
      ParameterValue::Enum(v) => ParameterValue::Enum(v.clone()),
      ParameterValue::File(v) => ParameterValue::File(v.clone()),
      ParameterValue::Directory(v) => ParameterValue::Directory(v.clone()),
      ParameterValue::Image(v) => ParameterValue::Image(v.clone()),
      ParameterValue::Audio(v) => ParameterValue::Audio(v.clone()),
      ParameterValue::Video(v) => ParameterValue::Video(v.clone()),
      ParameterValue::Custom(t, v) => ParameterValue::Custom(t.clone(), v.clone()),
    }
  }
}

#[derive(Debug, Clone)]
pub struct EffectProcessor {
  metadata: EffectMetadata,
  parameters: HashMap<String, ParameterValue>,
  cache: Arc<RwLock<EffectCache>>,
  performance_stats: Arc<RwLock<PerformanceStats>>,
}

impl EffectProcessor {
  pub fn new(metadata: EffectMetadata) -> Self {
    Self {
      metadata,
      parameters: HashMap::new(),
      cache: Arc::new(RwLock::new(EffectCache::new())),
      performance_stats: Arc::new(RwLock::new(PerformanceStats::new())),
    }
  }

  pub fn metadata(&self) -> &EffectMetadata {
    &self.metadata
  }

  pub fn parameters(&self) -> &HashMap<String, ParameterValue> {
    &self.parameters
  }

  pub fn parameters_mut(&mut self) -> &mut HashMap<String, ParameterValue> {
    &mut self.parameters
  }

  pub fn cache(&self) -> Arc<RwLock<EffectCache>> {
    self.cache.clone()
  }

  pub fn performance_stats(&self) -> Arc<RwLock<PerformanceStats>> {
    self.performance_stats.clone()
  }

  pub fn set_parameter(&mut self, name: String, value: ParameterValue) -> Result<()> {
    if let Some(param) = self.metadata.parameters.iter().find(|p| p.name == name) {
      self.validate_parameter_value(param, &value)?;
      self.parameters.insert(name, value);
      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Unknown parameter: {}",
        name
      )))
    }
  }

  pub fn get_parameter(&self, name: &str) -> Option<&ParameterValue> {
    self.parameters.get(name)
  }

  pub fn reset_parameters(&mut self) {
    self.parameters.clear();
    for param in &self.metadata.parameters {
      self
        .parameters
        .insert(param.name.clone(), param.default_value.clone());
    }
  }

  pub fn apply_effect(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
    let start_time = std::time::Instant::now();

    self.validate_parameters()?;

    let cache_key = self.generate_cache_key(media_processor);

    if let Some(cached_result) = self.cache.read().get(&cache_key) {
      *media_processor = cached_result.clone();
      return Ok(());
    }

    self.process_media(media_processor)?;

    let processing_time = start_time.elapsed();
    self
      .performance_stats
      .write()
      .record_execution(processing_time);

    self.cache.write().put(cache_key, media_processor.clone());

    Ok(())
  }

  pub fn apply_effect_async(
    &mut self,
    media_processor: MediaProcessor,
  ) -> Result<tokio::task::JoinHandle<Result<MediaProcessor>>> {
    let metadata = self.metadata.clone();
    let parameters = self.parameters.clone();
    let cache = self.cache.clone();
    let performance_stats = self.performance_stats.clone();

    let handle = tokio::spawn(async move {
      let start_time = std::time::Instant::now();

      let mut processor = media_processor;

      let cache_key = Self::generate_cache_key_static(&metadata, &parameters, &processor);

      if let Some(cached_result) = cache.read().get(&cache_key) {
        return Ok(cached_result);
      }

      Self::process_media_static(&metadata, &parameters, &mut processor)?;

      let processing_time = start_time.elapsed();
      performance_stats.write().record_execution(processing_time);

      cache.write().put(cache_key, processor.clone());

      Ok(processor)
    });

    Ok(handle)
  }

  pub fn preview_effect(
    &mut self,
    media_processor: &mut MediaProcessor,
    preview_size: (u32, u32),
  ) -> Result<()> {
    let original_size = (media_processor.width(), media_processor.height());

    if original_size != preview_size {
      media_processor.resize(preview_size.0, preview_size.1)?;
    }

    self.apply_effect(media_processor)?;

    if original_size != preview_size {
      media_processor.resize(original_size.0, original_size.1)?;
    }

    Ok(())
  }

  pub fn batch_apply(
    &mut self,
    media_processors: &mut [MediaProcessor],
  ) -> Result<Vec<Result<()>>> {
    media_processors
      .par_iter_mut()
      .map(|processor| {
        let mut temp_processor = processor.clone();
        let mut temp_effect_processor = EffectProcessor::new(self.metadata.clone());
        temp_effect_processor.parameters = self.parameters.clone();

        match temp_effect_processor.apply_effect(&mut temp_processor) {
          Ok(()) => {
            *processor = temp_processor;
            Ok(())
          }
          Err(e) => Err(e),
        }
      })
      .collect()
  }

  pub fn batch_apply_async(
    &mut self,
    media_processors: Vec<MediaProcessor>,
  ) -> Result<Vec<tokio::task::JoinHandle<Result<MediaProcessor>>>> {
    let metadata = self.metadata.clone();
    let parameters = self.parameters.clone();
    let cache = self.cache.clone();
    let performance_stats = self.performance_stats.clone();

    let handles: Vec<_> = media_processors
      .into_iter()
      .map(|processor| {
        let metadata = metadata.clone();
        let parameters = parameters.clone();
        let cache = cache.clone();
        let performance_stats = performance_stats.clone();

        tokio::spawn(async move {
          let start_time = std::time::Instant::now();

          let mut temp_processor = processor;

          let cache_key = Self::generate_cache_key_static(&metadata, &parameters, &temp_processor);

          if let Some(cached_result) = cache.read().get(&cache_key) {
            return Ok(cached_result);
          }

          Self::process_media_static(&metadata, &parameters, &mut temp_processor)?;

          let processing_time = start_time.elapsed();
          performance_stats.write().record_execution(processing_time);

          cache.write().put(cache_key, temp_processor.clone());

          Ok(temp_processor)
        })
      })
      .collect();

    Ok(handles)
  }

  fn validate_parameters(&self) -> Result<()> {
    for param in &self.metadata.parameters {
      if param.required && !self.parameters.contains_key(&param.name) {
        return Err(EllasticError::InvalidParameter(format!(
          "Required parameter missing: {}",
          param.name
        )));
      }

      if let Some(value) = self.parameters.get(&param.name) {
        self.validate_parameter_value(param, value)?;
      }
    }
    Ok(())
  }

  fn validate_parameter_value(
    &self,
    param: &EffectParameter,
    value: &ParameterValue,
  ) -> Result<()> {
    match (&param.parameter_type, value) {
      (ParameterType::Integer, ParameterValue::Integer(_)) => Ok(()),
      (ParameterType::Float, ParameterValue::Float(_)) => Ok(()),
      (ParameterType::Boolean, ParameterValue::Boolean(_)) => Ok(()),
      (ParameterType::String, ParameterValue::String(_)) => Ok(()),
      (ParameterType::Color, ParameterValue::Color(_)) => Ok(()),
      (ParameterType::Vector2, ParameterValue::Vector2(_)) => Ok(()),
      (ParameterType::Vector3, ParameterValue::Vector3(_)) => Ok(()),
      (ParameterType::Vector4, ParameterValue::Vector4(_)) => Ok(()),
      (ParameterType::Matrix3, ParameterValue::Matrix3(_)) => Ok(()),
      (ParameterType::Matrix4, ParameterValue::Matrix4(_)) => Ok(()),
      (ParameterType::Enum(options), ParameterValue::Enum(value)) => {
        if options.contains(value) {
          Ok(())
        } else {
          Err(EllasticError::InvalidParameter(format!(
            "Invalid enum value: {}",
            value
          )))
        }
      }
      (ParameterType::File, ParameterValue::File(_)) => Ok(()),
      (ParameterType::Directory, ParameterValue::Directory(_)) => Ok(()),
      (ParameterType::Image, ParameterValue::Image(_)) => Ok(()),
      (ParameterType::Audio, ParameterValue::Audio(_)) => Ok(()),
      (ParameterType::Video, ParameterValue::Video(_)) => Ok(()),
      (ParameterType::Custom(_), ParameterValue::Custom(..)) => Ok(()),
      _ => Err(EllasticError::InvalidParameter(
        "Parameter type mismatch".to_string(),
      )),
    }
  }

  fn generate_cache_key(&self, media_processor: &MediaProcessor) -> String {
    Self::generate_cache_key_static(&self.metadata, &self.parameters, media_processor)
  }

  fn generate_cache_key_static(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &MediaProcessor,
  ) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{
      Hash,
      Hasher,
    };

    let mut hasher = DefaultHasher::new();

    metadata.id.hash(&mut hasher);

    let mut param_keys: Vec<_> = parameters.keys().collect();
    param_keys.sort();

    for key in param_keys {
      key.hash(&mut hasher);
      parameters[key].hash(&mut hasher);
    }

    media_processor.data().hash(&mut hasher);

    format!(
      "{}:{}:{}",
      metadata.id,
      hasher.finish(),
      media_processor.data().len()
    )
  }

  fn process_media(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
    Self::process_media_static(&self.metadata, &self.parameters, media_processor)
  }

  fn process_media_static(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.category {
      EffectCategory::Basic => Self::apply_basic_effect(metadata, parameters, media_processor),
      EffectCategory::Advanced => {
        Self::apply_advanced_effect(metadata, parameters, media_processor)
      }
      EffectCategory::Glitch => Self::apply_glitch_effect(metadata, parameters, media_processor),
      EffectCategory::Filter => Self::apply_filter_effect(metadata, parameters, media_processor),
      EffectCategory::Transform => {
        Self::apply_transform_effect(metadata, parameters, media_processor)
      }
      EffectCategory::Color => Self::apply_color_effect(metadata, parameters, media_processor),
      EffectCategory::Audio => Self::apply_audio_effect(metadata, parameters, media_processor),
      EffectCategory::Video => Self::apply_video_effect(metadata, parameters, media_processor),
      EffectCategory::Composite => {
        Self::apply_composite_effect(metadata, parameters, media_processor)
      }
      EffectCategory::Custom => Self::apply_custom_effect(metadata, parameters, media_processor),
    }
  }

  fn apply_basic_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.name.as_str() {
      "brightness" => {
        if let Some(ParameterValue::Float(brightness)) = parameters.get("brightness") {
          media_processor.adjust_brightness(*brightness as f32)?;
        }
      }
      "contrast" => {
        if let Some(ParameterValue::Float(contrast)) = parameters.get("contrast") {
          media_processor.adjust_contrast(*contrast as f32)?;
        }
      }
      "saturation" => {
        if let Some(ParameterValue::Float(saturation)) = parameters.get("saturation") {
          media_processor.adjust_saturation(*saturation as f32)?;
        }
      }
      "gamma" => {
        if let Some(ParameterValue::Float(gamma)) = parameters.get("gamma") {
          media_processor.adjust_gamma(*gamma as f32)?;
        }
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown basic effect: {}",
          metadata.name
        )));
      }
    }
    Ok(())
  }

  fn apply_advanced_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.name.as_str() {
      "blur" => {
        if let Some(ParameterValue::Float(radius)) = parameters.get("radius") {
          media_processor.blur(*radius as f32)?;
        }
      }
      "sharpen" => {
        if let Some(ParameterValue::Float(amount)) = parameters.get("amount") {
          media_processor.sharpen(*amount as f32)?;
        }
      }
      "edge_detection" => {
        media_processor.edge_detection()?;
      }
      "emboss" => {
        media_processor.emboss()?;
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown advanced effect: {}",
          metadata.name
        )));
      }
    }
    Ok(())
  }

  fn apply_glitch_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    let mut glitch_processor = GlitchProcessor::new(media_processor.clone());

    let glitch_effect = match metadata.name.as_str() {
      "pixel_sort" => {
        let threshold = parameters
          .get("threshold")
          .and_then(|v| v.as_float())
          .unwrap_or(0.5) as f32;
        let mode = parameters
          .get("mode")
          .and_then(|v| v.as_string())
          .map(|s| match s.as_str() {
            "brightness" => crate::effects::PixelSortMode::Brightness,
            "hue" => crate::effects::PixelSortMode::Hue,
            "saturation" => crate::effects::PixelSortMode::Saturation,
            "random" => crate::effects::PixelSortMode::Random,
            _ => crate::effects::PixelSortMode::Brightness,
          })
          .unwrap_or(crate::effects::PixelSortMode::Brightness);

        GlitchEffect::Image {
          image_glitch: crate::effects::ImageGlitch::PixelSort { threshold, mode },
        }
      }
      "data_mosh" => {
        let intensity = parameters
          .get("intensity")
          .and_then(|v| v.as_float())
          .unwrap_or(0.5) as f32;
        let preserve_size = parameters
          .get("preserve_size")
          .and_then(|v| v.as_boolean())
          .unwrap_or(true);

        GlitchEffect::Image {
          image_glitch: crate::effects::ImageGlitch::DataMosh {
            intensity,
            preserve_size,
          },
        }
      }
      "bit_crush" => {
        let bit_depth = parameters
          .get("bit_depth")
          .and_then(|v| v.as_integer())
          .unwrap_or(8) as u8;
        let sample_rate_reduction = parameters
          .get("sample_rate_reduction")
          .and_then(|v| v.as_integer())
          .unwrap_or(1) as u32;

        GlitchEffect::Audio {
          audio_glitch: crate::effects::AudioGlitch::BitCrush {
            bit_depth,
            sample_rate_reduction,
          },
        }
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown glitch effect: {}",
          metadata.name
        )));
      }
    };

    glitch_processor.apply_glitch(&glitch_effect)?;
    *media_processor = glitch_processor.into_media_processor();

    Ok(())
  }

  fn apply_filter_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.name.as_str() {
      "low_pass" => {
        if let Some(ParameterValue::Float(cutoff)) = parameters.get("cutoff") {
          media_processor.low_pass_filter(*cutoff as f32)?;
        }
      }
      "high_pass" => {
        if let Some(ParameterValue::Float(cutoff)) = parameters.get("cutoff") {
          media_processor.high_pass_filter(*cutoff as f32)?;
        }
      }
      "band_pass" => {
        if let (Some(ParameterValue::Float(low_cutoff)), Some(ParameterValue::Float(high_cutoff))) =
          (parameters.get("low_cutoff"), parameters.get("high_cutoff"))
        {
          media_processor.band_pass_filter(*low_cutoff as f32, *high_cutoff as f32)?;
        }
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown filter effect: {}",
          metadata.name
        )));
      }
    }
    Ok(())
  }

  fn apply_transform_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.name.as_str() {
      "rotate" => {
        if let Some(ParameterValue::Float(angle)) = parameters.get("angle") {
          media_processor.rotate(*angle as f32)?;
        }
      }
      "scale" => {
        if let (Some(ParameterValue::Float(scale_x)), Some(ParameterValue::Float(scale_y))) =
          (parameters.get("scale_x"), parameters.get("scale_y"))
        {
          media_processor.scale(*scale_x as f32, *scale_y as f32)?;
        }
      }
      "flip" => {
        if let Some(ParameterValue::Enum(direction)) = parameters.get("direction") {
          match direction.as_str() {
            "horizontal" => media_processor.flip_horizontal()?,
            "vertical" => media_processor.flip_vertical()?,
            "both" => {
              media_processor.flip_horizontal()?;
              media_processor.flip_vertical()?;
            }
            _ => {
              return Err(EllasticError::InvalidParameter(
                "Invalid flip direction".to_string(),
              ));
            }
          }
        }
      }
      "crop" => {
        if let (
          Some(ParameterValue::Integer(x)),
          Some(ParameterValue::Integer(y)),
          Some(ParameterValue::Integer(width)),
          Some(ParameterValue::Integer(height)),
        ) = (
          parameters.get("x"),
          parameters.get("y"),
          parameters.get("width"),
          parameters.get("height"),
        ) {
          media_processor.crop(*x as u32, *y as u32, *width as u32, *height as u32)?;
        }
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown transform effect: {}",
          metadata.name
        )));
      }
    }
    Ok(())
  }

  fn apply_color_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.name.as_str() {
      "grayscale" => {
        media_processor.grayscale()?;
      }
      "sepia" => {
        media_processor.sepia()?;
      }
      "invert" => {
        media_processor.invert()?;
      }
      "hue_rotate" => {
        if let Some(ParameterValue::Float(angle)) = parameters.get("angle") {
          media_processor.hue_rotate(*angle as f32)?;
        }
      }
      "colorize" => {
        if let Some(ParameterValue::Color(color)) = parameters.get("color") {
          media_processor.colorize(color[0], color[1], color[2], color[3])?;
        }
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown color effect: {}",
          metadata.name
        )));
      }
    }
    Ok(())
  }

  fn apply_audio_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.name.as_str() {
      "reverb" => {
        if let Some(ParameterValue::Float(room_size)) = parameters.get("room_size") {
          media_processor.reverb(*room_size as f32)?;
        }
      }
      "echo" => {
        if let Some(ParameterValue::Float(delay)) = parameters.get("delay") {
          media_processor.echo(*delay as f32)?;
        }
      }
      "delay" => {
        if let Some(ParameterValue::Float(delay)) = parameters.get("delay") {
          media_processor.delay(*delay as f32)?;
        }
      }
      "distortion" => {
        if let Some(ParameterValue::Float(amount)) = parameters.get("amount") {
          media_processor.distortion(*amount as f32)?;
        }
      }
      "compressor" => {
        if let Some(ParameterValue::Float(ratio)) = parameters.get("ratio") {
          media_processor.compressor(*ratio as f32)?;
        }
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown audio effect: {}",
          metadata.name
        )));
      }
    }
    Ok(())
  }

  fn apply_video_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.name.as_str() {
      "frame_duplication" => {
        if let Some(ParameterValue::Integer(count)) = parameters.get("count") {
          media_processor.duplicate_frames(*count as u32)?;
        }
      }
      "frame_dropping" => {
        if let Some(ParameterValue::Integer(count)) = parameters.get("count") {
          media_processor.drop_frames(*count as u32)?;
        }
      }
      "time_stretch" => {
        if let Some(ParameterValue::Float(ratio)) = parameters.get("ratio") {
          media_processor.time_stretch(*ratio as f32)?;
        }
      }
      "reverse_playback" => {
        media_processor.reverse_playback()?;
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown video effect: {}",
          metadata.name
        )));
      }
    }
    Ok(())
  }

  fn apply_composite_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    match metadata.name.as_str() {
      "blend" => {
        if let (Some(ParameterValue::Enum(mode)), Some(ParameterValue::Float(mix_ratio))) =
          (parameters.get("mode"), parameters.get("mix_ratio"))
        {
          let blend_mode = match mode.as_str() {
            "add" => ellastic_media::BlendMode::Add,
            "multiply" => ellastic_media::BlendMode::Multiply,
            "screen" => ellastic_media::BlendMode::Screen,
            "overlay" => ellastic_media::BlendMode::Overlay,
            "difference" => ellastic_media::BlendMode::Difference,
            _ => {
              return Err(EllasticError::InvalidParameter(
                "Invalid blend mode".to_string(),
              ));
            }
          };

          if let Some(ParameterValue::File(overlay_path)) = parameters.get("overlay") {
            let overlay_processor = MediaProcessor::from_file(overlay_path)?;
            media_processor.blend(&overlay_processor, blend_mode, *mix_ratio as f32)?;
          }
        }
      }
      "composite" => {
        if let Some(ParameterValue::File(composite_path)) = parameters.get("composite") {
          let composite_processor = MediaProcessor::from_file(composite_path)?;
          media_processor.composite(&composite_processor)?;
        }
      }
      "mask" => {
        if let Some(ParameterValue::File(mask_path)) = parameters.get("mask") {
          let mask_processor = MediaProcessor::from_file(mask_path)?;
          media_processor.apply_mask(&mask_processor)?;
        }
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown composite effect: {}",
          metadata.name
        )));
      }
    }
    Ok(())
  }

  fn apply_custom_effect(
    metadata: &EffectMetadata,
    parameters: &HashMap<String, ParameterValue>,
    media_processor: &mut MediaProcessor,
  ) -> Result<()> {
    Err(EllasticError::UnsupportedOperation(format!(
      "Custom effect '{}' requires custom implementation",
      metadata.name
    )))
  }

  pub fn clone(&self) -> EffectProcessor {
    EffectProcessor {
      metadata: self.metadata.clone(),
      parameters: self.parameters.clone(),
      cache: self.cache.clone(),
      performance_stats: self.performance_stats.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct EffectRegistry {
  effects: HashMap<Uuid, EffectMetadata>,
  effects_by_name: HashMap<String, Uuid>,
  effects_by_category: HashMap<EffectCategory, Vec<Uuid>>,
}

impl EffectRegistry {
  pub fn new() -> Self {
    Self {
      effects: HashMap::new(),
      effects_by_name: HashMap::new(),
      effects_by_category: HashMap::new(),
    }
  }

  pub fn register_effect(&mut self, metadata: EffectMetadata) -> Result<()> {
    if self.effects_by_name.contains_key(&metadata.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Effect '{}' already registered",
        metadata.name
      )));
    }

    let id = metadata.id;
    self.effects_by_name.insert(metadata.name.clone(), id);

    self
      .effects_by_category
      .entry(metadata.category.clone())
      .or_insert_with(Vec::new)
      .push(id);

    self.effects.insert(id, metadata);

    Ok(())
  }

  pub fn unregister_effect(&mut self, id: Uuid) -> Option<EffectMetadata> {
    if let Some(metadata) = self.effects.remove(&id) {
      self.effects_by_name.remove(&metadata.name);

      if let Some(effects) = self.effects_by_category.get_mut(&metadata.category) {
        effects.retain(|&effect_id| effect_id != id);
      }

      Some(metadata)
    } else {
      None
    }
  }

  pub fn get_effect(&self, id: Uuid) -> Option<&EffectMetadata> {
    self.effects.get(&id)
  }

  pub fn get_effect_by_name(&self, name: &str) -> Option<&EffectMetadata> {
    self
      .effects_by_name
      .get(name)
      .and_then(|id| self.effects.get(id))
  }

  pub fn list_effects(&self) -> Vec<&EffectMetadata> {
    self.effects.values().collect()
  }

  pub fn list_effects_by_category(&self, category: EffectCategory) -> Vec<&EffectMetadata> {
    self
      .effects_by_category
      .get(&category)
      .map(|ids| ids.iter().filter_map(|id| self.effects.get(id)).collect())
      .unwrap_or_default()
  }

  pub fn search_effects(&self, query: &str) -> Vec<&EffectMetadata> {
    let query = query.to_lowercase();
    self
      .effects
      .values()
      .filter(|metadata| {
        metadata.name.to_lowercase().contains(&query)
          || metadata.description.to_lowercase().contains(&query)
          || metadata
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query))
      })
      .collect()
  }

  pub fn clear(&mut self) {
    self.effects.clear();
    self.effects_by_name.clear();
    self.effects_by_category.clear();
  }

  pub fn len(&self) -> usize {
    self.effects.len()
  }

  pub fn is_empty(&self) -> bool {
    self.effects.is_empty()
  }

  pub fn clone(&self) -> EffectRegistry {
    EffectRegistry {
      effects: self.effects.clone(),
      effects_by_name: self.effects_by_name.clone(),
      effects_by_category: self.effects_by_category.clone(),
    }
  }

  pub fn load_default_effects(&mut self) -> Result<()> {
    self.register_basic_effects()?;
    self.register_advanced_effects()?;
    self.register_glitch_effects()?;
    self.register_filter_effects()?;
    self.register_transform_effects()?;
    self.register_color_effects()?;
    self.register_audio_effects()?;
    self.register_video_effects()?;
    self.register_composite_effects()?;

    Ok(())
  }

  fn register_basic_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "brightness".to_string(),
        "Adjust image brightness".to_string(),
        EffectCategory::Basic,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "brightness".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.0),
        )
        .with_range(ParameterValue::Float(-1.0), ParameterValue::Float(1.0))
        .with_description("Brightness adjustment (-1.0 to 1.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "contrast".to_string(),
        "Adjust image contrast".to_string(),
        EffectCategory::Basic,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "contrast".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.0),
        )
        .with_range(ParameterValue::Float(-1.0), ParameterValue::Float(1.0))
        .with_description("Contrast adjustment (-1.0 to 1.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "saturation".to_string(),
        "Adjust image saturation".to_string(),
        EffectCategory::Basic,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "saturation".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.0),
        )
        .with_range(ParameterValue::Float(-1.0), ParameterValue::Float(1.0))
        .with_description("Saturation adjustment (-1.0 to 1.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "gamma".to_string(),
        "Adjust image gamma".to_string(),
        EffectCategory::Basic,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "gamma".to_string(),
          ParameterType::Float,
          ParameterValue::Float(1.0),
        )
        .with_range(ParameterValue::Float(0.1), ParameterValue::Float(3.0))
        .with_description("Gamma correction (0.1 to 3.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }

  fn register_advanced_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "blur".to_string(),
        "Apply blur filter".to_string(),
        EffectCategory::Advanced,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "radius".to_string(),
          ParameterType::Float,
          ParameterValue::Float(1.0),
        )
        .with_range(ParameterValue::Float(0.1), ParameterValue::Float(10.0))
        .with_description("Blur radius (0.1 to 10.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "sharpen".to_string(),
        "Apply sharpen filter".to_string(),
        EffectCategory::Advanced,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "amount".to_string(),
          ParameterType::Float,
          ParameterValue::Float(1.0),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(5.0))
        .with_description("Sharpen amount (0.0 to 5.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "edge_detection".to_string(),
        "Detect edges in image".to_string(),
        EffectCategory::Advanced,
      )
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "emboss".to_string(),
        "Apply emboss effect".to_string(),
        EffectCategory::Advanced,
      )
      .with_supported_media_types(vec![MediaType::Image]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }

  fn register_glitch_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "pixel_sort".to_string(),
        "Sort pixels by brightness".to_string(),
        EffectCategory::Glitch,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "threshold".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.5),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
        .with_description("Sorting threshold (0.0 to 1.0)".to_string()),
        EffectParameter::new(
          "mode".to_string(),
          ParameterType::Enum(vec![
            "brightness".to_string(),
            "hue".to_string(),
            "saturation".to_string(),
            "random".to_string(),
          ]),
          ParameterValue::Enum("brightness".to_string()),
        )
        .with_description("Sorting mode".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "data_mosh".to_string(),
        "Apply data moshing effect".to_string(),
        EffectCategory::Glitch,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "intensity".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.5),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
        .with_description("Mosh intensity (0.0 to 1.0)".to_string()),
        EffectParameter::new(
          "preserve_size".to_string(),
          ParameterType::Boolean,
          ParameterValue::Boolean(true),
        )
        .with_description("Preserve original size".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image, MediaType::Video]),
      EffectMetadata::new(
        "bit_crush".to_string(),
        "Apply bit crushing effect".to_string(),
        EffectCategory::Glitch,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "bit_depth".to_string(),
          ParameterType::Integer,
          ParameterValue::Integer(8),
        )
        .with_range(ParameterValue::Integer(1), ParameterValue::Integer(32))
        .with_description("Target bit depth (1 to 32)".to_string()),
        EffectParameter::new(
          "sample_rate_reduction".to_string(),
          ParameterType::Integer,
          ParameterValue::Integer(1),
        )
        .with_range(ParameterValue::Integer(1), ParameterValue::Integer(100))
        .with_description("Sample rate reduction factor (1 to 100)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }

  fn register_filter_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "low_pass".to_string(),
        "Apply low-pass filter".to_string(),
        EffectCategory::Filter,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "cutoff".to_string(),
          ParameterType::Float,
          ParameterValue::Float(1000.0),
        )
        .with_range(ParameterValue::Float(20.0), ParameterValue::Float(20000.0))
        .with_description("Cutoff frequency (20.0 to 20000.0 Hz)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
      EffectMetadata::new(
        "high_pass".to_string(),
        "Apply high-pass filter".to_string(),
        EffectCategory::Filter,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "cutoff".to_string(),
          ParameterType::Float,
          ParameterValue::Float(1000.0),
        )
        .with_range(ParameterValue::Float(20.0), ParameterValue::Float(20000.0))
        .with_description("Cutoff frequency (20.0 to 20000.0 Hz)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
      EffectMetadata::new(
        "band_pass".to_string(),
        "Apply band-pass filter".to_string(),
        EffectCategory::Filter,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "low_cutoff".to_string(),
          ParameterType::Float,
          ParameterValue::Float(500.0),
        )
        .with_range(ParameterValue::Float(20.0), ParameterValue::Float(20000.0))
        .with_description("Low cutoff frequency (20.0 to 20000.0 Hz)".to_string()),
        EffectParameter::new(
          "high_cutoff".to_string(),
          ParameterType::Float,
          ParameterValue::Float(2000.0),
        )
        .with_range(ParameterValue::Float(20.0), ParameterValue::Float(20000.0))
        .with_description("High cutoff frequency (20.0 to 20000.0 Hz)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }

  fn register_transform_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "rotate".to_string(),
        "Rotate image".to_string(),
        EffectCategory::Transform,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "angle".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.0),
        )
        .with_range(ParameterValue::Float(-360.0), ParameterValue::Float(360.0))
        .with_description("Rotation angle in degrees (-360 to 360)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "scale".to_string(),
        "Scale image".to_string(),
        EffectCategory::Transform,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "scale_x".to_string(),
          ParameterType::Float,
          ParameterValue::Float(1.0),
        )
        .with_range(ParameterValue::Float(0.1), ParameterValue::Float(10.0))
        .with_description("X scale factor (0.1 to 10.0)".to_string()),
        EffectParameter::new(
          "scale_y".to_string(),
          ParameterType::Float,
          ParameterValue::Float(1.0),
        )
        .with_range(ParameterValue::Float(0.1), ParameterValue::Float(10.0))
        .with_description("Y scale factor (0.1 to 10.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "flip".to_string(),
        "Flip image".to_string(),
        EffectCategory::Transform,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "direction".to_string(),
          ParameterType::Enum(vec![
            "horizontal".to_string(),
            "vertical".to_string(),
            "both".to_string(),
          ]),
          ParameterValue::Enum("horizontal".to_string()),
        )
        .with_description("Flip direction".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "crop".to_string(),
        "Crop image".to_string(),
        EffectCategory::Transform,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "x".to_string(),
          ParameterType::Integer,
          ParameterValue::Integer(0),
        )
        .with_range(ParameterValue::Integer(0), ParameterValue::Integer(10000))
        .with_description("X coordinate".to_string()),
        EffectParameter::new(
          "y".to_string(),
          ParameterType::Integer,
          ParameterValue::Integer(0),
        )
        .with_range(ParameterValue::Integer(0), ParameterValue::Integer(10000))
        .with_description("Y coordinate".to_string()),
        EffectParameter::new(
          "width".to_string(),
          ParameterType::Integer,
          ParameterValue::Integer(100),
        )
        .with_range(ParameterValue::Integer(1), ParameterValue::Integer(10000))
        .with_description("Width".to_string()),
        EffectParameter::new(
          "height".to_string(),
          ParameterType::Integer,
          ParameterValue::Integer(100),
        )
        .with_range(ParameterValue::Integer(1), ParameterValue::Integer(10000))
        .with_description("Height".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }

  fn register_color_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "grayscale".to_string(),
        "Convert to grayscale".to_string(),
        EffectCategory::Color,
      )
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "sepia".to_string(),
        "Apply sepia tone".to_string(),
        EffectCategory::Color,
      )
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "invert".to_string(),
        "Invert colors".to_string(),
        EffectCategory::Color,
      )
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "hue_rotate".to_string(),
        "Rotate hue".to_string(),
        EffectCategory::Color,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "angle".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.0),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(360.0))
        .with_description("Hue rotation angle in degrees (0 to 360)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "colorize".to_string(),
        "Colorize image".to_string(),
        EffectCategory::Color,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "color".to_string(),
          ParameterType::Color,
          ParameterValue::Color([255, 0, 0, 255]),
        )
        .with_description("Color to apply (RGBA)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }

  fn register_audio_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "reverb".to_string(),
        "Apply reverb effect".to_string(),
        EffectCategory::Audio,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "room_size".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.5),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
        .with_description("Room size (0.0 to 1.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
      EffectMetadata::new(
        "echo".to_string(),
        "Apply echo effect".to_string(),
        EffectCategory::Audio,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "delay".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.3),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(2.0))
        .with_description("Echo delay in seconds (0.0 to 2.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
      EffectMetadata::new(
        "delay".to_string(),
        "Apply delay effect".to_string(),
        EffectCategory::Audio,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "delay".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.3),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(2.0))
        .with_description("Delay time in seconds (0.0 to 2.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
      EffectMetadata::new(
        "distortion".to_string(),
        "Apply distortion effect".to_string(),
        EffectCategory::Audio,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "amount".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.5),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
        .with_description("Distortion amount (0.0 to 1.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
      EffectMetadata::new(
        "compressor".to_string(),
        "Apply compressor effect".to_string(),
        EffectCategory::Audio,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "ratio".to_string(),
          ParameterType::Float,
          ParameterValue::Float(4.0),
        )
        .with_range(ParameterValue::Float(1.0), ParameterValue::Float(20.0))
        .with_description("Compression ratio (1.0 to 20.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }

  fn register_video_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "frame_duplication".to_string(),
        "Duplicate frames".to_string(),
        EffectCategory::Video,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "count".to_string(),
          ParameterType::Integer,
          ParameterValue::Integer(1),
        )
        .with_range(ParameterValue::Integer(1), ParameterValue::Integer(10))
        .with_description("Number of duplicates (1 to 10)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Video]),
      EffectMetadata::new(
        "frame_dropping".to_string(),
        "Drop frames".to_string(),
        EffectCategory::Video,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "count".to_string(),
          ParameterType::Integer,
          ParameterValue::Integer(1),
        )
        .with_range(ParameterValue::Integer(1), ParameterValue::Integer(10))
        .with_description("Number of frames to drop (1 to 10)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Video]),
      EffectMetadata::new(
        "time_stretch".to_string(),
        "Stretch video time".to_string(),
        EffectCategory::Video,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "ratio".to_string(),
          ParameterType::Float,
          ParameterValue::Float(1.0),
        )
        .with_range(ParameterValue::Float(0.1), ParameterValue::Float(10.0))
        .with_description("Time stretch ratio (0.1 to 10.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Video]),
      EffectMetadata::new(
        "reverse_playback".to_string(),
        "Reverse video playback".to_string(),
        EffectCategory::Video,
      )
      .with_supported_media_types(vec![MediaType::Video]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }

  fn register_composite_effects(&mut self) -> Result<()> {
    let effects = vec![
      EffectMetadata::new(
        "blend".to_string(),
        "Blend with overlay".to_string(),
        EffectCategory::Composite,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "overlay".to_string(),
          ParameterType::File,
          ParameterValue::File("".to_string()),
        )
        .with_description("Overlay image file path".to_string()),
        EffectParameter::new(
          "mode".to_string(),
          ParameterType::Enum(vec![
            "add".to_string(),
            "multiply".to_string(),
            "screen".to_string(),
            "overlay".to_string(),
            "difference".to_string(),
          ]),
          ParameterValue::Enum("overlay".to_string()),
        )
        .with_description("Blend mode".to_string()),
        EffectParameter::new(
          "mix_ratio".to_string(),
          ParameterType::Float,
          ParameterValue::Float(0.5),
        )
        .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
        .with_description("Mix ratio (0.0 to 1.0)".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "composite".to_string(),
        "Composite with another image".to_string(),
        EffectCategory::Composite,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "composite".to_string(),
          ParameterType::File,
          ParameterValue::File("".to_string()),
        )
        .with_description("Composite image file path".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectMetadata::new(
        "mask".to_string(),
        "Apply mask".to_string(),
        EffectCategory::Composite,
      )
      .with_parameters(vec![
        EffectParameter::new(
          "mask".to_string(),
          ParameterType::File,
          ParameterValue::File("".to_string()),
        )
        .with_description("Mask image file path".to_string()),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
    ];

    for effect in effects {
      self.register_effect(effect)?;
    }

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub enum ImageEffect {
  Brightness {
    amount: f32,
  },
  Contrast {
    amount: f32,
  },
  Saturation {
    amount: f32,
  },
  Gamma {
    gamma: f32,
  },
  Blur {
    radius: f32,
  },
  Sharpen {
    amount: f32,
  },
  EdgeDetection,
  Emboss,
  PixelSort {
    threshold: f32,
    mode: crate::effects::PixelSortMode,
  },
  DataMosh {
    intensity: f32,
    preserve_size: bool,
  },
  Custom {
    custom_function: Box<dyn Fn(&mut ImageProcessor) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone)]
pub enum AudioEffect {
  Reverb {
    room_size: f32,
  },
  Echo {
    delay: f32,
  },
  Delay {
    delay: f32,
  },
  Distortion {
    amount: f32,
  },
  Compressor {
    ratio: f32,
  },
  BitCrush {
    bit_depth: u8,
    sample_rate_reduction: u32,
  },
  Custom {
    custom_function: Box<dyn Fn(&mut AudioProcessor) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone)]
pub enum VideoEffect {
  FrameDuplication {
    count: u32,
  },
  FrameDropping {
    count: u32,
  },
  TimeStretch {
    ratio: f32,
  },
  ReversePlayback,
  DataMosh {
    intensity: f32,
    preserve_duration: bool,
  },
  Custom {
    custom_function: Box<dyn Fn(&mut ellastic_media::VideoProcessor) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone)]
pub enum CompositeEffect {
  Blend {
    overlay: MediaProcessor,
    mode: ellastic_media::BlendMode,
    mix_ratio: f32,
  },
  Composite {
    composite: MediaProcessor,
  },
  Mask {
    mask: MediaProcessor,
  },
  Custom {
    custom_function: Box<dyn Fn(&mut MediaProcessor) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone)]
pub struct CustomEffect {
  pub name: String,
  pub description: String,
  pub parameters: Vec<EffectParameter>,
  pub apply_function:
    Box<dyn Fn(&mut MediaProcessor, &HashMap<String, ParameterValue>) -> Result<()> + Send + Sync>,
}

impl CustomEffect {
  pub fn new(
    name: String,
    description: String,
    parameters: Vec<EffectParameter>,
    apply_function: Box<
      dyn Fn(&mut MediaProcessor, &HashMap<String, ParameterValue>) -> Result<()> + Send + Sync,
    >,
  ) -> Self {
    Self {
      name,
      description,
      parameters,
      apply_function,
    }
  }

  pub fn apply(
    &self,
    media_processor: &mut MediaProcessor,
    parameters: &HashMap<String, ParameterValue>,
  ) -> Result<()> {
    (self.apply_function)(media_processor, parameters)
  }

  pub fn clone(&self) -> CustomEffect {
    CustomEffect {
      name: self.name.clone(),
      description: self.description.clone(),
      parameters: self.parameters.clone(),
      apply_function: Box::new(|_, _| {
        Err(EllasticError::UnsupportedOperation(
          "Custom effect cloning not supported".to_string(),
        ))
      }),
    }
  }
}

pub fn create_effect_processor(metadata: EffectMetadata) -> EffectProcessor {
  EffectProcessor::new(metadata)
}

pub fn create_effect_registry() -> EffectRegistry {
  EffectRegistry::new()
}

pub fn create_effect_metadata(
  name: String,
  description: String,
  category: EffectCategory,
) -> EffectMetadata {
  EffectMetadata::new(name, description, category)
}

pub fn create_effect_parameter(
  name: String,
  parameter_type: ParameterType,
  default_value: ParameterValue,
) -> EffectParameter {
  EffectParameter::new(name, parameter_type, default_value)
}
