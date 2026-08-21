use crate::{
  audio_codecs::*,
  image_codecs::*,
  video_codecs::*,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use std::collections::HashMap;
use std::sync::{
  Arc,
  RwLock,
};

#[derive(Debug, Clone)]
pub struct CodecRegistry {
  codecs: Arc<RwLock<HashMap<String, CodecInfo>>>,
  default_codecs: HashMap<String, CodecInfo>,
}

impl CodecRegistry {
  pub fn new() -> Self {
    let mut registry = Self {
      codecs: Arc::new(RwLock::new(HashMap::new())),
      default_codecs: HashMap::new(),
    };

    registry.register_default_codecs();
    registry
  }

  fn register_default_codecs(&mut self) {
    let image_codecs = vec![
      CodecInfo {
        name: "PNG".to_string(),
        media_type: ellastic_core::MediaType::Image,
        codec_type: CodecType::Image,
        supported_formats: vec!["png".to_string()],
        is_lossless: true,
        is_lossy: false,
        max_quality: 100,
        supports_streaming: false,
        supports_metadata: true,
        supports_animation: true,
        supports_transparency: true,
        max_resolution: Some((65536, 65536)),
        max_bit_depth: Some(16),
        max_channels: Some(4),
        max_sample_rate: None,
        max_bitrate: None,
        default_quality: Some(6),
        default_compression: Some(6),
        presets: vec![
          "default".to_string(),
          "fast".to_string(),
          "best".to_string(),
        ],
        parameters: vec![CodecParameter {
          name: "compression".to_string(),
          parameter_type: ParameterType::Integer,
          default_value: ParameterValue::Integer(6),
          min_value: Some(ParameterValue::Integer(0)),
          max_value: Some(ParameterValue::Integer(9)),
          description: "PNG compression level (0-9)".to_string(),
        }],
      },
      CodecInfo {
        name: "JPEG".to_string(),
        media_type: ellastic_core::MediaType::Image,
        codec_type: CodecType::Image,
        supported_formats: vec!["jpeg".to_string(), "jpg".to_string()],
        is_lossless: false,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: Some((65536, 65536)),
        max_bit_depth: Some(8),
        max_channels: Some(3),
        max_sample_rate: None,
        max_bitrate: None,
        default_quality: Some(85),
        default_compression: None,
        presets: vec![
          "default".to_string(),
          "high".to_string(),
          "medium".to_string(),
          "low".to_string(),
        ],
        parameters: vec![CodecParameter {
          name: "quality".to_string(),
          parameter_type: ParameterType::Integer,
          default_value: ParameterValue::Integer(85),
          min_value: Some(ParameterValue::Integer(1)),
          max_value: Some(ParameterValue::Integer(100)),
          description: "JPEG quality level (1-100)".to_string(),
        }],
      },
      CodecInfo {
        name: "WebP".to_string(),
        media_type: ellastic_core::MediaType::Image,
        codec_type: CodecType::Image,
        supported_formats: vec!["webp".to_string()],
        is_lossless: true,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: false,
        supports_metadata: true,
        supports_animation: true,
        supports_transparency: true,
        max_resolution: Some((16384, 16384)),
        max_bit_depth: Some(8),
        max_channels: Some(4),
        max_sample_rate: None,
        max_bitrate: None,
        default_quality: Some(80),
        default_compression: None,
        presets: vec![
          "default".to_string(),
          "lossless".to_string(),
          "lossy".to_string(),
        ],
        parameters: vec![CodecParameter {
          name: "quality".to_string(),
          parameter_type: ParameterType::Integer,
          default_value: ParameterValue::Integer(80),
          min_value: Some(ParameterValue::Integer(0)),
          max_value: Some(ParameterValue::Integer(100)),
          description: "WebP quality level (0-100)".to_string(),
        }],
      },
    ];

    let audio_codecs = vec![
      CodecInfo {
        name: "WAV".to_string(),
        media_type: ellastic_core::MediaType::Audio,
        codec_type: CodecType::Audio,
        supported_formats: vec!["wav".to_string()],
        is_lossless: true,
        is_lossy: false,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: None,
        max_bit_depth: Some(32),
        max_channels: Some(65535),
        max_sample_rate: Some(192000),
        max_bitrate: None,
        default_quality: None,
        default_compression: None,
        presets: vec!["default".to_string()],
        parameters: vec![CodecParameter {
          name: "bit_depth".to_string(),
          parameter_type: ParameterType::Integer,
          default_value: ParameterValue::Integer(16),
          min_value: Some(ParameterValue::Integer(8)),
          max_value: Some(ParameterValue::Integer(32)),
          description: "Bit depth (8, 16, 24, 32)".to_string(),
        }],
      },
      CodecInfo {
        name: "MP3".to_string(),
        media_type: ellastic_core::MediaType::Audio,
        codec_type: CodecType::Audio,
        supported_formats: vec!["mp3".to_string()],
        is_lossless: false,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: None,
        max_bit_depth: Some(16),
        max_channels: Some(2),
        max_sample_rate: Some(48000),
        max_bitrate: Some(320000),
        default_quality: Some(128),
        default_compression: None,
        presets: vec![
          "default".to_string(),
          "high".to_string(),
          "medium".to_string(),
          "low".to_string(),
        ],
        parameters: vec![CodecParameter {
          name: "bitrate".to_string(),
          parameter_type: ParameterType::Integer,
          default_value: ParameterValue::Integer(128000),
          min_value: Some(ParameterValue::Integer(32000)),
          max_value: Some(ParameterValue::Integer(320000)),
          description: "MP3 bitrate in bps".to_string(),
        }],
      },
      CodecInfo {
        name: "FLAC".to_string(),
        media_type: ellastic_core::MediaType::Audio,
        codec_type: CodecType::Audio,
        supported_formats: vec!["flac".to_string()],
        is_lossless: true,
        is_lossy: false,
        max_quality: 100,
        supports_streaming: false,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: None,
        max_bit_depth: Some(24),
        max_channels: Some(8),
        max_sample_rate: Some(192000),
        max_bitrate: None,
        default_quality: Some(5),
        default_compression: Some(5),
        presets: vec![
          "default".to_string(),
          "fast".to_string(),
          "best".to_string(),
        ],
        parameters: vec![CodecParameter {
          name: "compression".to_string(),
          parameter_type: ParameterType::Integer,
          default_value: ParameterValue::Integer(5),
          min_value: Some(ParameterValue::Integer(0)),
          max_value: Some(ParameterValue::Integer(8)),
          description: "FLAC compression level (0-8)".to_string(),
        }],
      },
    ];

    let video_codecs = vec![
      CodecInfo {
        name: "H.264".to_string(),
        media_type: ellastic_core::MediaType::Video,
        codec_type: CodecType::Video,
        supported_formats: vec!["mp4".to_string(), "mov".to_string()],
        is_lossless: false,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: Some((7680, 4320)),
        max_bit_depth: Some(10),
        max_channels: Some(8),
        max_sample_rate: Some(192000),
        max_bitrate: Some(10000000),
        default_quality: Some(23),
        default_compression: None,
        presets: vec![
          "ultrafast".to_string(),
          "superfast".to_string(),
          "veryfast".to_string(),
          "faster".to_string(),
          "fast".to_string(),
          "medium".to_string(),
          "slow".to_string(),
          "slower".to_string(),
          "veryslow".to_string(),
        ],
        parameters: vec![CodecParameter {
          name: "crf".to_string(),
          parameter_type: ParameterType::Integer,
          default_value: ParameterValue::Integer(23),
          min_value: Some(ParameterValue::Integer(0)),
          max_value: Some(ParameterValue::Integer(51)),
          description: "Constant rate factor (0-51)".to_string(),
        }],
      },
      CodecInfo {
        name: "VP9".to_string(),
        media_type: ellastic_core::MediaType::Video,
        codec_type: CodecType::Video,
        supported_formats: vec!["webm".to_string()],
        is_lossless: true,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: Some((16384, 16384)),
        max_bit_depth: Some(12),
        max_channels: Some(8),
        max_sample_rate: Some(192000),
        max_bitrate: Some(100000000),
        default_quality: Some(31),
        default_compression: None,
        presets: vec![
          "ultrafast".to_string(),
          "superfast".to_string(),
          "medium".to_string(),
          "slow".to_string(),
        ],
        parameters: vec![CodecParameter {
          name: "crf".to_string(),
          parameter_type: ParameterType::Integer,
          default_value: ParameterValue::Integer(31),
          min_value: Some(ParameterValue::Integer(0)),
          max_value: Some(ParameterValue::Integer(63)),
          description: "Constant rate factor (0-63)".to_string(),
        }],
      },
    ];

    for codec in image_codecs {
      self.default_codecs.insert(codec.name.clone(), codec);
    }

    for codec in audio_codecs {
      self.default_codecs.insert(codec.name.clone(), codec);
    }

    for codec in video_codecs {
      self.default_codecs.insert(codec.name.clone(), codec);
    }
  }

  pub fn register_codec(&self, codec_info: CodecInfo) -> Result<()> {
    let mut codecs = self
      .codecs
      .write()
      .map_err(|e| EllasticError::InternalError(format!("Failed to acquire write lock: {}", e)))?;

    if codecs.contains_key(&codec_info.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Codec {} already registered",
        codec_info.name
      )));
    }

    codecs.insert(codec_info.name.clone(), codec_info);
    Ok(())
  }

  pub fn unregister_codec(&self, name: &str) -> Result<()> {
    let mut codecs = self
      .codecs
      .write()
      .map_err(|e| EllasticError::InternalError(format!("Failed to acquire write lock: {}", e)))?;

    if codecs.remove(name).is_none() {
      return Err(EllasticError::NotFound(format!("Codec {} not found", name)));
    }

    Ok(())
  }

  pub fn get_codec_info(&self, name: &str) -> Option<CodecInfo> {
    let codecs = self.codecs.read().ok()?;

    if let Some(codec) = codecs.get(name) {
      Some(codec.clone())
    } else {
      self.default_codecs.get(name).cloned()
    }
  }

  pub fn list_codecs(&self) -> Vec<CodecInfo> {
    let codecs = self.codecs.read().ok().unwrap_or_default();
    let mut result = codecs.values().cloned().collect::<Vec<_>>();

    for (name, codec) in &self.default_codecs {
      if !codecs.contains_key(name) {
        result.push(codec.clone());
      }
    }

    result
  }

  pub fn list_codecs_by_type(&self, media_type: ellastic_core::MediaType) -> Vec<CodecInfo> {
    self
      .list_codecs()
      .into_iter()
      .filter(|codec| codec.media_type == media_type)
      .collect()
  }

  pub fn list_codecs_by_format(&self, format: &str) -> Vec<CodecInfo> {
    self
      .list_codecs()
      .into_iter()
      .filter(|codec| codec.supported_formats.contains(&format.to_string()))
      .collect()
  }

  pub fn find_best_codec(
    &self,
    media_type: ellastic_core::MediaType,
    requirements: &CodecRequirements,
  ) -> Option<CodecInfo> {
    let mut candidates = self.list_codecs_by_type(media_type);

    if requirements.lossless_only {
      candidates.retain(|codec| codec.is_lossless);
    }

    if requirements.lossy_only {
      candidates.retain(|codec| codec.is_lossy);
    }

    if let Some(max_quality) = requirements.max_quality {
      candidates.retain(|codec| codec.max_quality >= max_quality);
    }

    if let Some(min_quality) = requirements.min_quality {
      candidates.retain(|codec| codec.default_quality.map_or(true, |q| q >= min_quality));
    }

    if requirements.supports_streaming {
      candidates.retain(|codec| codec.supports_streaming);
    }

    if requirements.supports_metadata {
      candidates.retain(|codec| codec.supports_metadata);
    }

    if requirements.supports_animation {
      candidates.retain(|codec| codec.supports_animation);
    }

    if requirements.supports_transparency {
      candidates.retain(|codec| codec.supports_transparency);
    }

    if let Some((max_width, max_height)) = requirements.max_resolution {
      candidates.retain(|codec| {
        codec
          .max_resolution
          .map_or(true, |(w, h)| w >= max_width && h >= max_height)
      });
    }

    if let Some(max_bit_depth) = requirements.max_bit_depth {
      candidates.retain(|codec| codec.max_bit_depth.map_or(true, |bd| bd >= max_bit_depth));
    }

    if let Some(max_channels) = requirements.max_channels {
      candidates.retain(|codec| codec.max_channels.map_or(true, |ch| ch >= max_channels));
    }

    if let Some(max_sample_rate) = requirements.max_sample_rate {
      candidates.retain(|codec| {
        codec
          .max_sample_rate
          .map_or(true, |sr| sr >= max_sample_rate)
      });
    }

    if let Some(max_bitrate) = requirements.max_bitrate {
      candidates.retain(|codec| codec.max_bitrate.map_or(true, |br| br >= max_bitrate));
    }

    candidates.into_iter().min_by(|a, b| {
      let a_score = self.calculate_codec_score(a, requirements);
      let b_score = self.calculate_codec_score(b, requirements);
      b_score
        .partial_cmp(&a_score)
        .unwrap_or(std::cmp::Ordering::Equal)
    })
  }

  fn calculate_codec_score(&self, codec: &CodecInfo, requirements: &CodecRequirements) -> f32 {
    let mut score = 0.0f32;

    if codec.is_lossless && requirements.lossless_only {
      score += 10.0;
    }

    if codec.is_lossy && requirements.lossy_only {
      score += 10.0;
    }

    if codec.supports_streaming && requirements.supports_streaming {
      score += 5.0;
    }

    if codec.supports_metadata && requirements.supports_metadata {
      score += 3.0;
    }

    if codec.supports_animation && requirements.supports_animation {
      score += 2.0;
    }

    if codec.supports_transparency && requirements.supports_transparency {
      score += 2.0;
    }

    score += codec.max_quality as f32 / 10.0;

    score
  }

  pub fn validate_codec(&self, name: &str) -> Result<ValidationResult> {
    if let Some(codec_info) = self.get_codec_info(name) {
      let mut issues = Vec::new();
      let mut warnings = Vec::new();

      if codec_info.name.is_empty() {
        issues.push("Codec name cannot be empty".to_string());
      }

      if codec_info.supported_formats.is_empty() {
        issues.push("Codec must support at least one format".to_string());
      }

      if !codec_info.is_lossless && !codec_info.is_lossy {
        warnings.push("Codec supports neither lossless nor lossy encoding".to_string());
      }

      if codec_info.max_quality == 0 {
        warnings.push("Codec has zero max quality".to_string());
      }

      let is_valid = issues.is_empty();
      let score = if is_valid { 1.0 } else { 0.5 } - (issues.len() as f32 * 0.1);

      Ok(ValidationResult {
        is_valid,
        score,
        issues,
        warnings,
      })
    } else {
      Err(EllasticError::NotFound(format!("Codec {} not found", name)))
    }
  }

  pub fn get_codec_parameters(&self, name: &str) -> Option<Vec<CodecParameter>> {
    self.get_codec_info(name).map(|codec| codec.parameters)
  }

  pub fn get_codec_presets(&self, name: &str) -> Option<Vec<String>> {
    self.get_codec_info(name).map(|codec| codec.presets)
  }

  pub fn create_codec_instance(
    &self,
    name: &str,
    parameters: &HashMap<String, ParameterValue>,
  ) -> Result<Box<dyn CodecInstance>> {
    let codec_info = self
      .get_codec_info(name)
      .ok_or_else(|| EllasticError::NotFound(format!("Codec {} not found", name)))?;

    match codec_info.codec_type {
      CodecType::Image => self.create_image_codec_instance(&codec_info, parameters),
      CodecType::Audio => self.create_audio_codec_instance(&codec_info, parameters),
      CodecType::Video => self.create_video_codec_instance(&codec_info, parameters),
    }
  }

  fn create_image_codec_instance(
    &self,
    codec_info: &CodecInfo,
    parameters: &HashMap<String, ParameterValue>,
  ) -> Result<Box<dyn CodecInstance>> {
    match codec_info.name.as_str() {
      "PNG" => {
        let compression = parameters
          .get("compression")
          .and_then(|v| {
            if let ParameterValue::Integer(i) = v {
              Some(*i)
            } else {
              None
            }
          })
          .unwrap_or(6);
        Ok(Box::new(PNGCodecInstance::new(compression)))
      }
      "JPEG" => {
        let quality = parameters
          .get("quality")
          .and_then(|v| {
            if let ParameterValue::Integer(i) = v {
              Some(*i)
            } else {
              None
            }
          })
          .unwrap_or(85);
        Ok(Box::new(JPEGCodecInstance::new(quality)))
      }
      "WebP" => {
        let quality = parameters
          .get("quality")
          .and_then(|v| {
            if let ParameterValue::Integer(i) = v {
              Some(*i)
            } else {
              None
            }
          })
          .unwrap_or(80);
        Ok(Box::new(WebPCodecInstance::new(quality)))
      }
      _ => Err(EllasticError::UnsupportedOperation(format!(
        "Image codec {} not implemented",
        codec_info.name
      ))),
    }
  }

  fn create_audio_codec_instance(
    &self,
    codec_info: &CodecInfo,
    parameters: &HashMap<String, ParameterValue>,
  ) -> Result<Box<dyn CodecInstance>> {
    match codec_info.name.as_str() {
      "WAV" => {
        let bit_depth = parameters
          .get("bit_depth")
          .and_then(|v| {
            if let ParameterValue::Integer(i) = v {
              Some(*i as u16)
            } else {
              None
            }
          })
          .unwrap_or(16);
        Ok(Box::new(WAVCodecInstance::new(bit_depth)))
      }
      "MP3" => {
        let bitrate = parameters
          .get("bitrate")
          .and_then(|v| {
            if let ParameterValue::Integer(i) = v {
              Some(*i as u32)
            } else {
              None
            }
          })
          .unwrap_or(128000);
        Ok(Box::new(MP3CodecInstance::new(bitrate)))
      }
      "FLAC" => {
        let compression = parameters
          .get("compression")
          .and_then(|v| {
            if let ParameterValue::Integer(i) = v {
              Some(*i)
            } else {
              None
            }
          })
          .unwrap_or(5);
        Ok(Box::new(FLACCodecInstance::new(compression)))
      }
      _ => Err(EllasticError::UnsupportedOperation(format!(
        "Audio codec {} not implemented",
        codec_info.name
      ))),
    }
  }

  fn create_video_codec_instance(
    &self,
    codec_info: &CodecInfo,
    parameters: &HashMap<String, ParameterValue>,
  ) -> Result<Box<dyn CodecInstance>> {
    match codec_info.name.as_str() {
      "H.264" => {
        let crf = parameters
          .get("crf")
          .and_then(|v| {
            if let ParameterValue::Integer(i) = v {
              Some(*i)
            } else {
              None
            }
          })
          .unwrap_or(23);
        Ok(Box::new(H264CodecInstance::new(crf)))
      }
      "VP9" => {
        let crf = parameters
          .get("crf")
          .and_then(|v| {
            if let ParameterValue::Integer(i) = v {
              Some(*i)
            } else {
              None
            }
          })
          .unwrap_or(31);
        Ok(Box::new(VP9CodecInstance::new(crf)))
      }
      _ => Err(EllasticError::UnsupportedOperation(format!(
        "Video codec {} not implemented",
        codec_info.name
      ))),
    }
  }

  pub fn clone(&self) -> CodecRegistry {
    Self {
      codecs: Arc::clone(&self.codecs),
      default_codecs: self.default_codecs.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct CodecInfo {
  pub name: String,
  pub media_type: ellastic_core::MediaType,
  pub codec_type: CodecType,
  pub supported_formats: Vec<String>,
  pub is_lossless: bool,
  pub is_lossy: bool,
  pub max_quality: u8,
  pub supports_streaming: bool,
  pub supports_metadata: bool,
  pub supports_animation: bool,
  pub supports_transparency: bool,
  pub max_resolution: Option<(u32, u32)>,
  pub max_bit_depth: Option<u8>,
  pub max_channels: Option<u8>,
  pub max_sample_rate: Option<u32>,
  pub max_bitrate: Option<u32>,
  pub default_quality: Option<u8>,
  pub default_compression: Option<u8>,
  pub presets: Vec<String>,
  pub parameters: Vec<CodecParameter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecType {
  Image,
  Audio,
  Video,
}

#[derive(Debug, Clone)]
pub struct CodecParameter {
  pub name: String,
  pub parameter_type: ParameterType,
  pub default_value: ParameterValue,
  pub min_value: Option<ParameterValue>,
  pub max_value: Option<ParameterValue>,
  pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
  Integer,
  Float,
  String,
  Boolean,
  Enum(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
  Integer(i64),
  Float(f64),
  String(String),
  Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct CodecRequirements {
  pub lossless_only: bool,
  pub lossy_only: bool,
  pub max_quality: Option<u8>,
  pub min_quality: Option<u8>,
  pub supports_streaming: bool,
  pub supports_metadata: bool,
  pub supports_animation: bool,
  pub supports_transparency: bool,
  pub max_resolution: Option<(u32, u32)>,
  pub max_bit_depth: Option<u8>,
  pub max_channels: Option<u8>,
  pub max_sample_rate: Option<u32>,
  pub max_bitrate: Option<u32>,
}

impl Default for CodecRequirements {
  fn default() -> Self {
    Self {
      lossless_only: false,
      lossy_only: false,
      max_quality: None,
      min_quality: None,
      supports_streaming: false,
      supports_metadata: false,
      supports_animation: false,
      supports_transparency: false,
      max_resolution: None,
      max_bit_depth: None,
      max_channels: None,
      max_sample_rate: None,
      max_bitrate: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
  pub is_valid: bool,
  pub score: f32,
  pub issues: Vec<String>,
  pub warnings: Vec<String>,
}

pub trait CodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>>;
  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>>;
  fn get_info(&self) -> &CodecInfo;
  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()>;
  fn get_parameter(&self, name: &str) -> Option<ParameterValue>;
}

struct PNGCodecInstance {
  compression: u8,
  info: CodecInfo,
}

impl PNGCodecInstance {
  fn new(compression: u8) -> Self {
    Self {
      compression,
      info: CodecInfo {
        name: "PNG".to_string(),
        media_type: ellastic_core::MediaType::Image,
        codec_type: CodecType::Image,
        supported_formats: vec!["png".to_string()],
        is_lossless: true,
        is_lossy: false,
        max_quality: 100,
        supports_streaming: false,
        supports_metadata: true,
        supports_animation: true,
        supports_transparency: true,
        max_resolution: Some((65536, 65536)),
        max_bit_depth: Some(16),
        max_channels: Some(4),
        max_sample_rate: None,
        max_bitrate: None,
        default_quality: Some(6),
        default_compression: Some(6),
        presets: vec!["default".to_string()],
        parameters: vec![],
      },
    }
  }
}

impl CodecInstance for PNGCodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn get_info(&self) -> &CodecInfo {
    &self.info
  }

  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match name {
      "compression" => {
        if let ParameterValue::Integer(i) = value {
          self.compression = i as u8;
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
    match name {
      "compression" => Some(ParameterValue::Integer(self.compression as i64)),
      _ => None,
    }
  }
}

struct JPEGCodecInstance {
  quality: u8,
  info: CodecInfo,
}

impl JPEGCodecInstance {
  fn new(quality: u8) -> Self {
    Self {
      quality,
      info: CodecInfo {
        name: "JPEG".to_string(),
        media_type: ellastic_core::MediaType::Image,
        codec_type: CodecType::Image,
        supported_formats: vec!["jpeg".to_string(), "jpg".to_string()],
        is_lossless: false,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: Some((65536, 65536)),
        max_bit_depth: Some(8),
        max_channels: Some(3),
        max_sample_rate: None,
        max_bitrate: None,
        default_quality: Some(85),
        default_compression: None,
        presets: vec!["default".to_string()],
        parameters: vec![],
      },
    }
  }
}

impl CodecInstance for JPEGCodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn get_info(&self) -> &CodecInfo {
    &self.info
  }

  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match name {
      "quality" => {
        if let ParameterValue::Integer(i) = value {
          self.quality = i as u8;
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
    match name {
      "quality" => Some(ParameterValue::Integer(self.quality as i64)),
      _ => None,
    }
  }
}

struct WebPCodecInstance {
  quality: u8,
  info: CodecInfo,
}

impl WebPCodecInstance {
  fn new(quality: u8) -> Self {
    Self {
      quality,
      info: CodecInfo {
        name: "WebP".to_string(),
        media_type: ellastic_core::MediaType::Image,
        codec_type: CodecType::Image,
        supported_formats: vec!["webp".to_string()],
        is_lossless: true,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: false,
        supports_metadata: true,
        supports_animation: true,
        supports_transparency: true,
        max_resolution: Some((16384, 16384)),
        max_bit_depth: Some(8),
        max_channels: Some(4),
        max_sample_rate: None,
        max_bitrate: None,
        default_quality: Some(80),
        default_compression: None,
        presets: vec!["default".to_string()],
        parameters: vec![],
      },
    }
  }
}

impl CodecInstance for WebPCodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn get_info(&self) -> &CodecInfo {
    &self.info
  }

  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match name {
      "quality" => {
        if let ParameterValue::Integer(i) = value {
          self.quality = i as u8;
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
    match name {
      "quality" => Some(ParameterValue::Integer(self.quality as i64)),
      _ => None,
    }
  }
}

struct WAVCodecInstance {
  bit_depth: u16,
  info: CodecInfo,
}

impl WAVCodecInstance {
  fn new(bit_depth: u16) -> Self {
    Self {
      bit_depth,
      info: CodecInfo {
        name: "WAV".to_string(),
        media_type: ellastic_core::MediaType::Audio,
        codec_type: CodecType::Audio,
        supported_formats: vec!["wav".to_string()],
        is_lossless: true,
        is_lossy: false,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: None,
        max_bit_depth: Some(32),
        max_channels: Some(65535),
        max_sample_rate: Some(192000),
        max_bitrate: None,
        default_quality: None,
        default_compression: None,
        presets: vec!["default".to_string()],
        parameters: vec![],
      },
    }
  }
}

impl CodecInstance for WAVCodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn get_info(&self) -> &CodecInfo {
    &self.info
  }

  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match name {
      "bit_depth" => {
        if let ParameterValue::Integer(i) = value {
          self.bit_depth = i as u16;
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
    match name {
      "bit_depth" => Some(ParameterValue::Integer(self.bit_depth as i64)),
      _ => None,
    }
  }
}

struct MP3CodecInstance {
  bitrate: u32,
  info: CodecInfo,
}

impl MP3CodecInstance {
  fn new(bitrate: u32) -> Self {
    Self {
      bitrate,
      info: CodecInfo {
        name: "MP3".to_string(),
        media_type: ellastic_core::MediaType::Audio,
        codec_type: CodecType::Audio,
        supported_formats: vec!["mp3".to_string()],
        is_lossless: false,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: None,
        max_bit_depth: Some(16),
        max_channels: Some(2),
        max_sample_rate: Some(48000),
        max_bitrate: Some(320000),
        default_quality: Some(128),
        default_compression: None,
        presets: vec!["default".to_string()],
        parameters: vec![],
      },
    }
  }
}

impl CodecInstance for MP3CodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn get_info(&self) -> &CodecInfo {
    &self.info
  }

  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match name {
      "bitrate" => {
        if let ParameterValue::Integer(i) = value {
          self.bitrate = i as u32;
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
    match name {
      "bitrate" => Some(ParameterValue::Integer(self.bitrate as i64)),
      _ => None,
    }
  }
}

struct FLACCodecInstance {
  compression: u8,
  info: CodecInfo,
}

impl FLACCodecInstance {
  fn new(compression: u8) -> Self {
    Self {
      compression,
      info: CodecInfo {
        name: "FLAC".to_string(),
        media_type: ellastic_core::MediaType::Audio,
        codec_type: CodecType::Audio,
        supported_formats: vec!["flac".to_string()],
        is_lossless: true,
        is_lossy: false,
        max_quality: 100,
        supports_streaming: false,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: None,
        max_bit_depth: Some(24),
        max_channels: Some(8),
        max_sample_rate: Some(192000),
        max_bitrate: None,
        default_quality: Some(5),
        default_compression: Some(5),
        presets: vec!["default".to_string()],
        parameters: vec![],
      },
    }
  }
}

impl CodecInstance for FLACCodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn get_info(&self) -> &CodecInfo {
    &self.info
  }

  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match name {
      "compression" => {
        if let ParameterValue::Integer(i) = value {
          self.compression = i as u8;
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
    match name {
      "compression" => Some(ParameterValue::Integer(self.compression as i64)),
      _ => None,
    }
  }
}

struct H264CodecInstance {
  crf: u8,
  info: CodecInfo,
}

impl H264CodecInstance {
  fn new(crf: u8) -> Self {
    Self {
      crf,
      info: CodecInfo {
        name: "H.264".to_string(),
        media_type: ellastic_core::MediaType::Video,
        codec_type: CodecType::Video,
        supported_formats: vec!["mp4".to_string(), "mov".to_string()],
        is_lossless: false,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: Some((7680, 4320)),
        max_bit_depth: Some(10),
        max_channels: Some(8),
        max_sample_rate: Some(192000),
        max_bitrate: Some(10000000),
        default_quality: Some(23),
        default_compression: None,
        presets: vec!["default".to_string()],
        parameters: vec![],
      },
    }
  }
}

impl CodecInstance for H264CodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn get_info(&self) -> &CodecInfo {
    &self.info
  }

  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match name {
      "crf" => {
        if let ParameterValue::Integer(i) = value {
          self.crf = i as u8;
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
    match name {
      "crf" => Some(ParameterValue::Integer(self.crf as i64)),
      _ => None,
    }
  }
}

struct VP9CodecInstance {
  crf: u8,
  info: CodecInfo,
}

impl VP9CodecInstance {
  fn new(crf: u8) -> Self {
    Self {
      crf,
      info: CodecInfo {
        name: "VP9".to_string(),
        media_type: ellastic_core::MediaType::Video,
        codec_type: CodecType::Video,
        supported_formats: vec!["webm".to_string()],
        is_lossless: true,
        is_lossy: true,
        max_quality: 100,
        supports_streaming: true,
        supports_metadata: true,
        supports_animation: false,
        supports_transparency: false,
        max_resolution: Some((16384, 16384)),
        max_bit_depth: Some(12),
        max_channels: Some(8),
        max_sample_rate: Some(192000),
        max_bitrate: Some(100000000),
        default_quality: Some(31),
        default_compression: None,
        presets: vec!["default".to_string()],
        parameters: vec![],
      },
    }
  }
}

impl CodecInstance for VP9CodecInstance {
  fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn get_info(&self) -> &CodecInfo {
    &self.info
  }

  fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match name {
      "crf" => {
        if let ParameterValue::Integer(i) = value {
          self.crf = i as u8;
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
    match name {
      "crf" => Some(ParameterValue::Integer(self.crf as i64)),
      _ => None,
    }
  }
}

pub fn create_codec_registry() -> CodecRegistry {
  CodecRegistry::new()
}

pub fn create_codec_requirements() -> CodecRequirements {
  CodecRequirements::default()
}
