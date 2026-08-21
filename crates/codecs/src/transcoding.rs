use ellastic_audio::{
  AudioFormat,
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
use ellastic_image::{
  ImageFormat,
  ImageProcessor,
};
use ellastic_media::{
  MediaProcessor,
  VideoFormat,
  VideoProcessor,
};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Transcoder {
  source: MediaProcessor,
  target_format: String,
  options: TranscodeOptions,
  progress_callback: Option<Box<dyn Fn(f32) + Send>>,
}

impl Transcoder {
  pub fn new(source: MediaProcessor, target_format: String, options: TranscodeOptions) -> Self {
    Self {
      source,
      target_format,
      options,
      progress_callback: None,
    }
  }

  pub fn source(&self) -> &MediaProcessor {
    &self.source
  }

  pub fn target_format(&self) -> &str {
    &self.target_format
  }

  pub fn options(&self) -> &TranscodeOptions {
    &self.options
  }

  pub fn set_progress_callback<F>(&mut self, callback: F)
  where
    F: Fn(f32) + Send + 'static,
  {
    self.progress_callback = Some(Box::new(callback));
  }

  pub fn transcode(&mut self) -> Result<MediaProcessor> {
    let media_type = self.source.media_type();

    match media_type {
      MediaType::Image => self.transcode_image(),
      MediaType::Audio => self.transcode_audio(),
      MediaType::Video => self.transcode_video(),
    }
  }

  pub fn transcode_async(&self) -> Result<mpsc::Receiver<TranscodeProgress>> {
    let (tx, rx) = mpsc::channel();
    let source = self.source.clone();
    let target_format = self.target_format.clone();
    let options = self.options.clone();

    thread::spawn(move || {
      let mut transcoder = Transcoder::new(source, target_format, options);

      match transcoder.transcode() {
        Ok(result) => {
          let _ = tx.send(TranscodeProgress {
            progress: 1.0,
            status: TranscodeStatus::Completed,
            result: Some(result),
            error: None,
          });
        }
        Err(error) => {
          let _ = tx.send(TranscodeProgress {
            progress: 0.0,
            status: TranscodeStatus::Error,
            result: None,
            error: Some(error),
          });
        }
      }
    });

    Ok(rx)
  }

  pub fn estimate_output_size(&self) -> Result<usize> {
    let media_type = self.source.media_type();

    match media_type {
      MediaType::Image => self.estimate_image_size(),
      MediaType::Audio => self.estimate_audio_size(),
      MediaType::Video => self.estimate_video_size(),
    }
  }

  pub fn estimate_transcode_time(&self) -> Result<Duration> {
    let media_type = self.source.media_type();

    match media_type {
      MediaType::Image => self.estimate_image_time(),
      MediaType::Audio => self.estimate_audio_time(),
      MediaType::Video => self.estimate_video_time(),
    }
  }

  pub fn validate_transcode(&self) -> Result<TranscodeValidation> {
    let media_type = self.source.media_type();

    match media_type {
      MediaType::Image => self.validate_image_transcode(),
      MediaType::Audio => self.validate_audio_transcode(),
      MediaType::Video => self.validate_video_transcode(),
    }
  }

  fn transcode_image(&mut self) -> Result<MediaProcessor> {
    if let Some(image_processor) = self.source.as_image_processor() {
      let target_format = self.parse_image_format(&self.target_format)?;
      let quality = self.options.quality.unwrap_or(85);

      self.report_progress(0.1);

      let encoded = image_processor.encode(target_format, Some(quality))?;

      self.report_progress(0.5);

      let decoded = ellastic_image::decode_image(&encoded, target_format)?;

      self.report_progress(1.0);

      Ok(MediaProcessor::new(MediaData::Image(decoded)))
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not an image".to_string(),
      ))
    }
  }

  fn transcode_audio(&mut self) -> Result<MediaProcessor> {
    if let Some(audio_processor) = self.source.as_audio_processor() {
      let target_format = self.parse_audio_format(&self.target_format)?;
      let quality = self.options.quality.unwrap_or(128);
      let bitrate = self.options.bitrate.unwrap_or(128000);

      self.report_progress(0.1);

      let encoded = audio_processor.encode(target_format, Some(quality))?;

      self.report_progress(0.5);

      let decoded = ellastic_audio::decode_audio(&encoded, target_format)?;

      self.report_progress(1.0);

      Ok(MediaProcessor::new(MediaData::Audio(decoded)))
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not audio".to_string(),
      ))
    }
  }

  fn transcode_video(&mut self) -> Result<MediaProcessor> {
    if let Some(video_processor) = self.source.as_video_processor() {
      let target_format = self.parse_video_format(&self.target_format)?;
      let quality = self.options.quality.unwrap_or(75);
      let bitrate = self.options.bitrate.unwrap_or(5000000);

      self.report_progress(0.05);

      let encoded = video_processor.encode(target_format, Some(quality))?;

      self.report_progress(0.5);

      let decoded = crate::decode_video(&encoded, target_format)?;

      self.report_progress(1.0);

      Ok(MediaProcessor::new(MediaData::Video(decoded)))
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not video".to_string(),
      ))
    }
  }

  fn parse_image_format(&self, format_str: &str) -> Result<ImageFormat> {
    match format_str.to_lowercase().as_str() {
      "png" => Ok(ImageFormat::PNG),
      "jpeg" | "jpg" => Ok(ImageFormat::JPEG),
      "webp" => Ok(ImageFormat::WEBP),
      "bmp" => Ok(ImageFormat::BMP),
      "gif" => Ok(ImageFormat::GIF),
      "tiff" | "tif" => Ok(ImageFormat::TIFF),
      _ => Err(EllasticError::UnsupportedFormat(format!(
        "Unsupported image format: {}",
        format_str
      ))),
    }
  }

  fn parse_audio_format(&self, format_str: &str) -> Result<AudioFormat> {
    match format_str.to_lowercase().as_str() {
      "wav" => Ok(AudioFormat::WAV),
      "mp3" => Ok(AudioFormat::MP3),
      "flac" => Ok(AudioFormat::FLAC),
      "ogg" => Ok(AudioFormat::OGG),
      "aac" => Ok(AudioFormat::AAC),
      _ => Err(EllasticError::UnsupportedFormat(format!(
        "Unsupported audio format: {}",
        format_str
      ))),
    }
  }

  fn parse_video_format(&self, format_str: &str) -> Result<VideoFormat> {
    match format_str.to_lowercase().as_str() {
      "mp4" => Ok(VideoFormat::MP4),
      "avi" => Ok(VideoFormat::AVI),
      "mov" => Ok(VideoFormat::MOV),
      "webm" => Ok(VideoFormat::WEBM),
      "mkv" => Ok(VideoFormat::MKV),
      _ => Err(EllasticError::UnsupportedFormat(format!(
        "Unsupported video format: {}",
        format_str
      ))),
    }
  }

  fn estimate_image_size(&self) -> Result<usize> {
    if let Some(image_processor) = self.source.as_image_processor() {
      let target_format = self.parse_image_format(&self.target_format)?;
      let base_size = image_processor.data().byte_size();

      let compression_factor = match target_format {
        ImageFormat::PNG => 0.7,
        ImageFormat::JPEG => 0.2,
        ImageFormat::WEBP => 0.3,
        ImageFormat::BMP => 1.0,
        ImageFormat::GIF => 0.5,
        ImageFormat::TIFF => 0.8,
      };

      Ok((base_size as f32 * compression_factor) as usize)
    } else {
      Ok(0)
    }
  }

  fn estimate_audio_size(&self) -> Result<usize> {
    if let Some(audio_processor) = self.source.as_audio_processor() {
      let target_format = self.parse_audio_format(&self.target_format)?;
      let duration = audio_processor.data().duration_seconds();
      let bitrate = self.options.bitrate.unwrap_or(128000);

      match target_format {
        AudioFormat::WAV => {
          let bit_depth = self.options.bit_depth.unwrap_or(16) as u32;
          let channels = audio_processor.channels() as u32;
          let sample_rate = audio_processor.sample_rate();
          (duration * sample_rate * channels * bit_depth / 8.0) as usize
        }
        AudioFormat::MP3 | AudioFormat::OGG | AudioFormat::AAC => {
          (duration * bitrate as f64 / 8.0) as usize
        }
        AudioFormat::FLAC => {
          let bit_depth = self.options.bit_depth.unwrap_or(16) as u32;
          let channels = audio_processor.channels() as u32;
          let sample_rate = audio_processor.sample_rate();
          let compression_ratio = 0.6;
          (duration * sample_rate * channels * bit_depth / 8.0 * compression_ratio) as usize
        }
      }
    } else {
      Ok(0)
    }
  }

  fn estimate_video_size(&self) -> Result<usize> {
    if let Some(video_processor) = self.source.as_video_processor() {
      let target_format = self.parse_video_format(&self.target_format)?;
      let duration = video_processor.data().duration_seconds();
      let bitrate = self.options.bitrate.unwrap_or(5000000);

      match target_format {
        VideoFormat::MP4 | VideoFormat::WEBM | VideoFormat::MKV => {
          (duration * bitrate as f64 / 8.0) as usize
        }
        VideoFormat::AVI => {
          let width = video_processor.width() as u64;
          let height = video_processor.height() as u64;
          let frame_rate = video_processor.frame_rate();
          let bit_depth = self.options.bit_depth.unwrap_or(24) as u64;
          let compression_ratio = 0.8;
          (duration * width * height * frame_rate * bit_depth / 8.0 * compression_ratio) as usize
        }
        VideoFormat::MOV => (duration * bitrate as f64 / 8.0) as usize,
      }
    } else {
      Ok(0)
    }
  }

  fn estimate_image_time(&self) -> Result<Duration> {
    if let Some(image_processor) = self.source.as_image_processor() {
      let pixel_count = image_processor.width() * image_processor.height();
      let base_time_ms = (pixel_count as f64 / 1000000.0) * 1000.0;

      let format_multiplier = match self.parse_image_format(&self.target_format)? {
        ImageFormat::PNG => 2.0,
        ImageFormat::JPEG => 0.5,
        ImageFormat::WEBP => 1.0,
        ImageFormat::BMP => 0.1,
        ImageFormat::GIF => 0.8,
        ImageFormat::TIFF => 1.5,
      };

      Ok(Duration::from_millis(
        (base_time_ms * format_multiplier) as u64,
      ))
    } else {
      Ok(Duration::from_millis(0))
    }
  }

  fn estimate_audio_time(&self) -> Result<Duration> {
    if let Some(audio_processor) = self.source.as_audio_processor() {
      let duration = audio_processor.data().duration_seconds();
      let sample_count = audio_processor.sample_count();

      let base_time_ms = (duration * 1000.0) / 10.0;

      let format_multiplier = match self.parse_audio_format(&self.target_format)? {
        AudioFormat::WAV => 0.1,
        AudioFormat::MP3 => 2.0,
        AudioFormat::FLAC => 1.5,
        AudioFormat::OGG => 1.8,
        AudioFormat::AAC => 1.6,
      };

      Ok(Duration::from_millis(
        (base_time_ms * format_multiplier) as u64,
      ))
    } else {
      Ok(Duration::from_millis(0))
    }
  }

  fn estimate_video_time(&self) -> Result<Duration> {
    if let Some(video_processor) = self.source.as_video_processor() {
      let duration = video_processor.data().duration_seconds();
      let frame_count = video_processor.frame_count();
      let resolution = video_processor.width() * video_processor.height();

      let base_time_ms = (duration * 1000.0) / 5.0;

      let format_multiplier = match self.parse_video_format(&self.target_format)? {
        VideoFormat::MP4 => 3.0,
        VideoFormat::AVI => 1.0,
        VideoFormat::MOV => 2.5,
        VideoFormat::WEBM => 2.8,
        VideoFormat::MKV => 2.7,
      };

      Ok(Duration::from_millis(
        (base_time_ms * format_multiplier) as u64,
      ))
    } else {
      Ok(Duration::from_millis(0))
    }
  }

  fn validate_image_transcode(&self) -> Result<TranscodeValidation> {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    if let Some(image_processor) = self.source.as_image_processor() {
      let target_format = self.parse_image_format(&self.target_format)?;

      if self.options.quality.is_some() && target_format == ImageFormat::PNG {
        warnings.push("Quality parameter ignored for PNG format".to_string());
      }

      if image_processor.channels() > 4 && target_format == ImageFormat::JPEG {
        issues.push("JPEG format supports maximum 4 channels".to_string());
      }

      if image_processor.width() > 65536 || image_processor.height() > 65536 {
        warnings.push("Image dimensions exceed recommended maximum".to_string());
      }
    }

    let is_valid = issues.is_empty();
    Ok(TranscodeValidation {
      is_valid,
      issues,
      warnings,
    })
  }

  fn validate_audio_transcode(&self) -> Result<TranscodeValidation> {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    if let Some(audio_processor) = self.source.as_audio_processor() {
      let target_format = self.parse_audio_format(&self.target_format)?;

      if self.options.bitrate.is_some() && target_format == AudioFormat::WAV {
        warnings.push("Bitrate parameter ignored for WAV format".to_string());
      }

      if self.options.sample_rate.is_some() && self.options.sample_rate.unwrap() > 192000 {
        warnings.push("Sample rate exceeds recommended maximum".to_string());
      }

      if audio_processor.channels() > 2 && target_format == AudioFormat::MP3 {
        warnings.push("MP3 format typically uses 2 channels maximum".to_string());
      }
    }

    let is_valid = issues.is_empty();
    Ok(TranscodeValidation {
      is_valid,
      issues,
      warnings,
    })
  }

  fn validate_video_transcode(&self) -> Result<TranscodeValidation> {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    if let Some(video_processor) = self.source.as_video_processor() {
      let target_format = self.parse_video_format(&self.target_format)?;

      if self.options.resolution.is_some() {
        let (width, height) = self.options.resolution.unwrap();
        if width > 7680 || height > 4320 {
          warnings.push("Target resolution exceeds 8K maximum".to_string());
        }
      }

      if self.options.frame_rate.is_some() && self.options.frame_rate.unwrap() > 120.0 {
        warnings.push("Frame rate exceeds recommended maximum".to_string());
      }

      if video_processor.width() > 4096 && target_format == VideoFormat::AVI {
        warnings.push("AVI format has resolution limitations".to_string());
      }
    }

    let is_valid = issues.is_empty();
    Ok(TranscodeValidation {
      is_valid,
      issues,
      warnings,
    })
  }

  fn report_progress(&mut self, progress: f32) {
    if let Some(ref callback) = self.progress_callback {
      callback(progress);
    }
  }

  pub fn clone(&self) -> Transcoder {
    Transcoder {
      source: self.source.clone(),
      target_format: self.target_format.clone(),
      options: self.options.clone(),
      progress_callback: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct TranscodeOptions {
  pub quality: Option<u8>,
  pub bitrate: Option<u32>,
  pub sample_rate: Option<u32>,
  pub channels: Option<u8>,
  pub resolution: Option<(u32, u32)>,
  pub frame_rate: Option<f64>,
  pub bit_depth: Option<u8>,
  pub compression_level: Option<u8>,
  pub preset: Option<String>,
  pub custom_options: HashMap<String, String>,
}

impl Default for TranscodeOptions {
  fn default() -> Self {
    Self {
      quality: None,
      bitrate: None,
      sample_rate: None,
      channels: None,
      resolution: None,
      frame_rate: None,
      bit_depth: None,
      compression_level: None,
      preset: None,
      custom_options: HashMap::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct TranscodeProgress {
  pub progress: f32,
  pub status: TranscodeStatus,
  pub result: Option<MediaProcessor>,
  pub error: Option<EllasticError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscodeStatus {
  Starting,
  Processing,
  Completed,
  Error,
  Cancelled,
}

#[derive(Debug, Clone)]
pub struct TranscodeValidation {
  pub is_valid: bool,
  pub issues: Vec<String>,
  pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BatchTranscoder {
  transcoders: Vec<Transcoder>,
  parallel: bool,
  max_concurrent: usize,
}

impl BatchTranscoder {
  pub fn new() -> Self {
    Self {
      transcoders: Vec::new(),
      parallel: false,
      max_concurrent: 1,
    }
  }

  pub fn add_transcoder(&mut self, transcoder: Transcoder) {
    self.transcoders.push(transcoder);
  }

  pub fn parallel(&mut self, parallel: bool) {
    self.parallel = parallel;
  }

  pub fn max_concurrent(&mut self, max_concurrent: usize) {
    self.max_concurrent = max_concurrent;
  }

  pub fn transcode_all(&mut self) -> Result<Vec<TranscodeProgress>> {
    if self.parallel {
      self.transcode_parallel()
    } else {
      self.transcode_sequential()
    }
  }

  fn transcode_sequential(&mut self) -> Result<Vec<TranscodeProgress>> {
    let mut results = Vec::new();

    for (i, transcoder) in self.transcoders.iter_mut().enumerate() {
      match transcoder.transcode() {
        Ok(result) => {
          results.push(TranscodeProgress {
            progress: 1.0,
            status: TranscodeStatus::Completed,
            result: Some(result),
            error: None,
          });
        }
        Err(error) => {
          results.push(TranscodeProgress {
            progress: 0.0,
            status: TranscodeStatus::Error,
            result: None,
            error: Some(error),
          });
        }
      }
    }

    Ok(results)
  }

  fn transcode_parallel(&mut self) -> Result<Vec<TranscodeProgress>> {
    let mut handles = Vec::new();
    let mut results = Vec::new();

    for transcoder in &self.transcoders {
      let receiver = transcoder.transcode_async()?;
      handles.push(receiver);
    }

    for handle in handles {
      while let Ok(progress) = handle.recv() {
        if progress.status == TranscodeStatus::Completed
          || progress.status == TranscodeStatus::Error
        {
          results.push(progress);
          break;
        }
      }
    }

    Ok(results)
  }

  pub fn estimate_total_time(&self) -> Result<Duration> {
    let mut total_time = Duration::from_millis(0);

    for transcoder in &self.transcoders {
      total_time += transcoder.estimate_transcode_time()?;
    }

    if self.parallel {
      total_time /= self.max_concurrent as u32;
    }

    Ok(total_time)
  }

  pub fn estimate_total_size(&self) -> Result<usize> {
    let mut total_size = 0;

    for transcoder in &self.transcoders {
      total_size += transcoder.estimate_output_size()?;
    }

    Ok(total_size)
  }

  pub fn clone(&self) -> BatchTranscoder {
    Self {
      transcoders: self.transcoders.clone(),
      parallel: self.parallel,
      max_concurrent: self.max_concurrent,
    }
  }
}

#[derive(Debug, Clone)]
pub struct TranscodeProfile {
  pub name: String,
  pub target_format: String,
  pub options: TranscodeOptions,
  pub description: String,
}

impl TranscodeProfile {
  pub fn new(
    name: String,
    target_format: String,
    options: TranscodeOptions,
    description: String,
  ) -> Self {
    Self {
      name,
      target_format,
      options,
      description,
    }
  }

  pub fn create_preset(name: &str, target_format: &str) -> Result<Self> {
    match (name, target_format) {
      ("web_optimized", "jpg") => Ok(Self::new(
        "Web Optimized".to_string(),
        "jpg".to_string(),
        TranscodeOptions {
          quality: Some(75),
          resolution: Some((1920, 1080)),
          ..Default::default()
        },
        "Optimized for web viewing".to_string(),
      )),
      ("high_quality", "png") => Ok(Self::new(
        "High Quality".to_string(),
        "png".to_string(),
        TranscodeOptions {
          quality: Some(9),
          ..Default::default()
        },
        "Maximum quality PNG".to_string(),
      )),
      ("fast_encoding", "mp4") => Ok(Self::new(
        "Fast Encoding".to_string(),
        "mp4".to_string(),
        TranscodeOptions {
          quality: Some(50),
          preset: Some("fast".to_string()),
          ..Default::default()
        },
        "Fast MP4 encoding".to_string(),
      )),
      ("lossless_audio", "flac") => Ok(Self::new(
        "Lossless Audio".to_string(),
        "flac".to_string(),
        TranscodeOptions {
          compression_level: Some(5),
          ..Default::default()
        },
        "Lossless FLAC audio".to_string(),
      )),
      _ => Err(EllasticError::NotFound(format!(
        "Preset {} not found for format {}",
        name, target_format
      ))),
    }
  }

  pub fn get_available_presets() -> Vec<Self> {
    vec![
      Self::new(
        "Web Optimized".to_string(),
        "jpg".to_string(),
        TranscodeOptions {
          quality: Some(75),
          resolution: Some((1920, 1080)),
          ..Default::default()
        },
        "Optimized for web viewing".to_string(),
      ),
      Self::new(
        "High Quality".to_string(),
        "png".to_string(),
        TranscodeOptions {
          quality: Some(9),
          ..Default::default()
        },
        "Maximum quality PNG".to_string(),
      ),
      Self::new(
        "Fast Encoding".to_string(),
        "mp4".to_string(),
        TranscodeOptions {
          quality: Some(50),
          preset: Some("fast".to_string()),
          ..Default::default()
        },
        "Fast MP4 encoding".to_string(),
      ),
      Self::new(
        "Lossless Audio".to_string(),
        "flac".to_string(),
        TranscodeOptions {
          compression_level: Some(5),
          ..Default::default()
        },
        "Lossless FLAC audio".to_string(),
      ),
    ]
  }

  pub fn clone(&self) -> Self {
    Self {
      name: self.name.clone(),
      target_format: self.target_format.clone(),
      options: self.options.clone(),
      description: self.description.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct TranscodeManager {
  profiles: HashMap<String, TranscodeProfile>,
  history: Vec<TranscodeHistory>,
}

impl TranscodeManager {
  pub fn new() -> Self {
    Self {
      profiles: HashMap::new(),
      history: Vec::new(),
    }
  }

  pub fn add_profile(&mut self, profile: TranscodeProfile) {
    self.profiles.insert(profile.name.clone(), profile);
  }

  pub fn get_profile(&self, name: &str) -> Option<&TranscodeProfile> {
    self.profiles.get(name)
  }

  pub fn list_profiles(&self) -> Vec<&TranscodeProfile> {
    self.profiles.values().collect()
  }

  pub fn remove_profile(&mut self, name: &str) -> bool {
    self.profiles.remove(name).is_some()
  }

  pub fn create_transcoder(
    &self,
    source: MediaProcessor,
    profile_name: &str,
  ) -> Result<Transcoder> {
    let profile = self
      .get_profile(profile_name)
      .ok_or_else(|| EllasticError::NotFound(format!("Profile {} not found", profile_name)))?;

    Ok(Transcoder::new(
      source,
      profile.target_format.clone(),
      profile.options.clone(),
    ))
  }

  pub fn add_to_history(&mut self, history: TranscodeHistory) {
    self.history.push(history);
    if self.history.len() > 1000 {
      self.history.remove(0);
    }
  }

  pub fn get_history(&self) -> &[TranscodeHistory] {
    &self.history
  }

  pub fn clear_history(&mut self) {
    self.history.clear();
  }

  pub fn get_statistics(&self) -> TranscodeStatistics {
    let mut stats = TranscodeStatistics::default();

    for history in &self.history {
      stats.total_transcodes += 1;

      if history.success {
        stats.successful_transcodes += 1;
        stats.total_time += history.duration;
        stats.total_input_size += history.input_size;
        stats.total_output_size += history.output_size;
      } else {
        stats.failed_transcodes += 1;
      }
    }

    stats
  }

  pub fn clone(&self) -> Self {
    Self {
      profiles: self.profiles.clone(),
      history: self.history.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct TranscodeHistory {
  pub timestamp: std::time::SystemTime,
  pub source_format: String,
  pub target_format: String,
  pub options: TranscodeOptions,
  pub duration: Duration,
  pub input_size: usize,
  pub output_size: usize,
  pub success: bool,
  pub error: Option<String>,
}

impl TranscodeHistory {
  pub fn new(
    source_format: String,
    target_format: String,
    options: TranscodeOptions,
    duration: Duration,
    input_size: usize,
    output_size: usize,
    success: bool,
    error: Option<String>,
  ) -> Self {
    Self {
      timestamp: std::time::SystemTime::now(),
      source_format,
      target_format,
      options,
      duration,
      input_size,
      output_size,
      success,
      error,
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct TranscodeStatistics {
  pub total_transcodes: u64,
  pub successful_transcodes: u64,
  pub failed_transcodes: u64,
  pub total_time: Duration,
  pub total_input_size: usize,
  pub total_output_size: usize,
}

impl TranscodeStatistics {
  pub fn success_rate(&self) -> f64 {
    if self.total_transcodes == 0 {
      0.0
    } else {
      self.successful_transcodes as f64 / self.total_transcodes as f64
    }
  }

  pub fn average_time(&self) -> Duration {
    if self.successful_transcodes == 0 {
      Duration::from_millis(0)
    } else {
      self.total_time / self.successful_transcodes as u32
    }
  }

  pub fn compression_ratio(&self) -> f64 {
    if self.total_input_size == 0 {
      1.0
    } else {
      self.total_input_size as f64 / self.total_output_size as f64
    }
  }
}

pub fn create_transcoder(
  source: MediaProcessor,
  target_format: String,
  options: TranscodeOptions,
) -> Transcoder {
  Transcoder::new(source, target_format, options)
}

pub fn create_batch_transcoder() -> BatchTranscoder {
  BatchTranscoder::new()
}

pub fn create_transcode_manager() -> TranscodeManager {
  TranscodeManager::new()
}

pub fn create_transcode_preset(name: &str, target_format: &str) -> Result<TranscodeProfile> {
  TranscodeProfile::create_preset(name, target_format)
}
