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
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EffectExporter {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub supported_formats: Vec<ExportFormat>,
  pub export_options: ExportOptions,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
  PNG,
  JPEG,
  WEBP,
  BMP,
  TIFF,
  GIF,
  MP4,
  AVI,
  MOV,
  WAV,
  MP3,
  FLAC,
  OGG,
  JSON,
  XML,
  YAML,
  CSV,
  Custom(String),
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
  pub quality: ExportQuality,
  pub compression_level: u8,
  pub preserve_metadata: bool,
  pub include_effects_info: bool,
  pub batch_mode: bool,
  pub parallel_processing: bool,
  pub output_directory: Option<String>,
  pub filename_template: String,
  pub overwrite_existing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportQuality {
  Low,
  Medium,
  High,
  Ultra,
  Custom { quality_factor: f32 },
}

#[derive(Debug, Clone)]
pub struct ExportContext {
  pub id: Uuid,
  pub exporter: EffectExporter,
  pub media_processor: MediaProcessor,
  pub target_format: ExportFormat,
  pub output_path: String,
  pub export_options: ExportOptions,
  pub progress_callback: Option<Box<dyn ProgressCallback + Send + Sync>>,
  pub cancellation_token: Arc<RwLock<CancellationToken>>,
  pub start_time: DateTime<Utc>,
  pub end_time: Option<DateTime<Utc>>,
  pub status: ExportStatus,
  pub error: Option<String>,
  pub bytes_exported: u64,
  pub total_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStatus {
  Pending,
  Running,
  Completed,
  Failed,
  Cancelled,
}

#[derive(Debug, Clone)]
pub struct ExportResult {
  pub context_id: Uuid,
  pub output_path: String,
  pub file_size: u64,
  pub export_time: std::time::Duration,
  pub format: ExportFormat,
  pub metadata: HashMap<String, String>,
  pub checksum: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatchExportConfig {
  pub input_files: Vec<String>,
  pub output_directory: String,
  pub target_format: ExportFormat,
  pub export_options: ExportOptions,
  pub filename_pattern: String,
  pub preserve_structure: bool,
  pub continue_on_error: bool,
}

#[derive(Debug, Clone)]
pub struct BatchExportResult {
  pub total_files: usize,
  pub successful_exports: usize,
  pub failed_exports: usize,
  pub skipped_files: usize,
  pub total_time: std::time::Duration,
  pub total_size: u64,
  pub results: Vec<ExportResult>,
  pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExportProfile {
  pub name: String,
  pub description: String,
  pub format: ExportFormat,
  pub export_options: ExportOptions,
  pub supported_media_types: Vec<MediaType>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ExportManager {
  exporters: HashMap<Uuid, EffectExporter>,
  contexts: HashMap<Uuid, ExportContext>,
  profiles: HashMap<String, ExportProfile>,
  active_contexts: Vec<Uuid>,
  max_concurrent_exports: usize,
  default_exporter: Option<Uuid>,
}

pub trait ProgressCallback: Send + Sync {
  fn on_progress(&self, progress: f32, message: Option<&str>);
  fn on_complete(&self);
  fn on_error(&self, error: &str);
}

#[derive(Debug, Clone)]
pub struct CancellationToken {
  pub cancelled: bool,
  pub reason: Option<String>,
}

impl CancellationToken {
  pub fn new() -> Self {
    Self {
      cancelled: false,
      reason: None,
    }
  }

  pub fn cancel(&mut self, reason: String) {
    self.cancelled = true;
    self.reason = Some(reason);
  }

  pub fn is_cancelled(&self) -> bool {
    self.cancelled
  }
}

impl EffectExporter {
  pub fn new(name: String, description: String) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name,
      description,
      supported_formats: Vec::new(),
      export_options: ExportOptions::default(),
      created_at: now,
      updated_at: now,
    }
  }

  pub fn with_supported_formats(mut self, formats: Vec<ExportFormat>) -> Self {
    self.supported_formats = formats;
    self
  }

  pub fn with_export_options(mut self, options: ExportOptions) -> Self {
    self.export_options = options;
    self
  }

  pub fn supports_format(&self, format: ExportFormat) -> bool {
    self.supported_formats.contains(&format)
  }

  pub fn export(
    &self,
    media_processor: &MediaProcessor,
    target_format: ExportFormat,
    output_path: String,
  ) -> Result<ExportContext> {
    if !self.supports_format(target_format) {
      return Err(EllasticError::InvalidParameter(format!(
        "Exporter '{}' does not support format: {:?}",
        self.name, target_format
      )));
    }

    let context = ExportContext {
      id: Uuid::new_v4(),
      exporter: self.clone(),
      media_processor: media_processor.clone(),
      target_format,
      output_path,
      export_options: self.export_options.clone(),
      progress_callback: None,
      cancellation_token: Arc::new(RwLock::new(CancellationToken::new())),
      start_time: Utc::now(),
      end_time: None,
      status: ExportStatus::Pending,
      error: None,
      bytes_exported: 0,
      total_bytes: 0,
    };

    Ok(context)
  }

  pub fn export_with_progress(
    &self,
    media_processor: &MediaProcessor,
    target_format: ExportFormat,
    output_path: String,
    progress_callback: Box<dyn ProgressCallback + Send + Sync>,
  ) -> Result<ExportContext> {
    if !self.supports_format(target_format) {
      return Err(EllasticError::InvalidParameter(format!(
        "Exporter '{}' does not support format: {:?}",
        self.name, target_format
      )));
    }

    let context = ExportContext {
      id: Uuid::new_v4(),
      exporter: self.clone(),
      media_processor: media_processor.clone(),
      target_format,
      output_path,
      export_options: self.export_options.clone(),
      progress_callback: Some(progress_callback),
      cancellation_token: Arc::new(RwLock::new(CancellationToken::new())),
      start_time: Utc::now(),
      end_time: None,
      status: ExportStatus::Pending,
      error: None,
      bytes_exported: 0,
      total_bytes: 0,
    };

    Ok(context)
  }

  pub fn execute_export(&self, context: &mut ExportContext) -> Result<ExportResult> {
    let start_time = std::time::Instant::now();

    context.status = ExportStatus::Running;

    if let Some(ref callback) = context.progress_callback {
      callback.on_progress(0.0, Some("Starting export..."));
    }

    if context.cancellation_token.read().is_cancelled() {
      context.status = ExportStatus::Cancelled;
      return Err(EllasticError::Cancelled("Export cancelled".to_string()));
    }

    context.total_bytes =
      self.estimate_output_size(&context.media_processor, context.target_format);

    let result = match context.media_processor.media_type() {
      MediaType::Image => self.export_image(context)?,
      MediaType::Audio => self.export_audio(context)?,
      MediaType::Video => self.export_video(context)?,
      _ => {
        return Err(EllasticError::UnsupportedOperation(
          "Unsupported media type".to_string(),
        ));
      }
    };

    let end_time = Utc::now();
    let export_time = start_time.elapsed();

    context.end_time = Some(end_time);
    context.status = ExportStatus::Completed;

    if let Some(ref callback) = context.progress_callback {
      callback.on_progress(1.0, Some("Export completed"));
      callback.on_complete();
    }

    Ok(ExportResult {
      context_id: context.id,
      output_path: context.output_path.clone(),
      file_size: context.bytes_exported,
      export_time,
      format: context.target_format,
      metadata: self.extract_export_metadata(context),
      checksum: self.calculate_checksum(&result),
    })
  }

  fn export_image(&self, context: &mut ExportContext) -> Result<Vec<u8>> {
    let image_processor = context
      .media_processor
      .image_processor()
      .ok_or_else(|| EllasticError::InvalidParameter("Not an image processor".to_string()))?;

    let output_data = match context.target_format {
      ExportFormat::PNG => image_processor.encode_png(context.export_options.preserve_metadata)?,
      ExportFormat::JPEG => image_processor.encode_jpeg(
        context.export_options.compression_level,
        context.export_options.preserve_metadata,
      )?,
      ExportFormat::WEBP => image_processor.encode_webp(
        context.export_options.compression_level,
        context.export_options.preserve_metadata,
      )?,
      ExportFormat::BMP => image_processor.encode_bmp()?,
      ExportFormat::TIFF => {
        image_processor.encode_tiff(context.export_options.preserve_metadata)?
      }
      ExportFormat::GIF => image_processor.encode_gif()?,
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Format '{:?}' not supported for images",
          context.target_format
        )));
      }
    };

    context.bytes_exported = output_data.len() as u64;

    std::fs::write(&context.output_path, &output_data)
      .map_err(|e| EllasticError::IOError(format!("Failed to write output file: {}", e)))?;

    Ok(output_data)
  }

  fn export_audio(&self, context: &mut ExportContext) -> Result<Vec<u8>> {
    let audio_processor = context
      .media_processor
      .audio_processor()
      .ok_or_else(|| EllasticError::InvalidParameter("Not an audio processor".to_string()))?;

    let output_data = match context.target_format {
      ExportFormat::WAV => audio_processor.encode_wav(16)?,
      ExportFormat::MP3 => {
        audio_processor.encode_mp3(context.export_options.compression_level, 16)?
      }
      ExportFormat::FLAC => audio_processor.encode_flac(16)?,
      ExportFormat::OGG => audio_processor.encode_ogg(16)?,
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Format '{:?}' not supported for audio",
          context.target_format
        )));
      }
    };

    context.bytes_exported = output_data.len() as u64;

    std::fs::write(&context.output_path, &output_data)
      .map_err(|e| EllasticError::IOError(format!("Failed to write output file: {}", e)))?;

    Ok(output_data)
  }

  fn export_video(&self, context: &mut ExportContext) -> Result<Vec<u8>> {
    let video_processor = context
      .media_processor
      .video_processor()
      .ok_or_else(|| EllasticError::InvalidParameter("Not a video processor".to_string()))?;

    let output_data = match context.target_format {
      ExportFormat::MP4 => {
        video_processor.encode_mp4(context.export_options.compression_level, Some(30.0))?
      }
      ExportFormat::AVI => video_processor.encode_avi()?,
      ExportFormat::MOV => video_processor.encode_mov()?,
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Format '{:?}' not supported for video",
          context.target_format
        )));
      }
    };

    context.bytes_exported = output_data.len() as u64;

    std::fs::write(&context.output_path, &output_data)
      .map_err(|e| EllasticError::IOError(format!("Failed to write output file: {}", e)))?;

    Ok(output_data)
  }

  fn export_metadata(&self, context: &ExportContext) -> Result<Vec<u8>> {
    let metadata = self.extract_export_metadata(context);

    let output_data = match context.target_format {
      ExportFormat::JSON => serde_json::to_vec_pretty(&metadata).map_err(|e| {
        EllasticError::SerializationError(format!("Failed to serialize JSON: {}", e))
      })?,
      ExportFormat::XML => {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metadata>\n");
        for (key, value) in &metadata {
          xml.push_str(&format!("  <{}>{}</{}>\n", key, value, key));
        }
        xml.push_str("</metadata>");
        xml.into_bytes()
      }
      ExportFormat::YAML => serde_yaml::to_vec(&metadata).map_err(|e| {
        EllasticError::SerializationError(format!("Failed to serialize YAML: {}", e))
      })?,
      ExportFormat::CSV => {
        let mut csv = String::new();
        for (key, value) in &metadata {
          csv.push_str(&format!("{},{}\n", key, value));
        }
        csv.into_bytes()
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Format '{:?}' not supported for metadata",
          context.target_format
        )));
      }
    };

    Ok(output_data)
  }

  fn estimate_output_size(&self, media_processor: &MediaProcessor, format: ExportFormat) -> u64 {
    let base_size = media_processor.data().len() as u64;

    let compression_factor = match format {
      ExportFormat::PNG => 0.7,
      ExportFormat::JPEG => 0.3,
      ExportFormat::WEBP => 0.4,
      ExportFormat::MP3 => 0.1,
      ExportFormat::MP4 => 0.2,
      ExportFormat::FLAC => 0.5,
      _ => 1.0,
    };

    (base_size as f64 * compression_factor) as u64
  }

  fn extract_export_metadata(&self, context: &ExportContext) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    metadata.insert("exporter_name".to_string(), context.exporter.name.clone());
    metadata.insert("exporter_id".to_string(), context.exporter.id.to_string());
    metadata.insert(
      "target_format".to_string(),
      format!("{:?}", context.target_format),
    );
    metadata.insert("output_path".to_string(), context.output_path.clone());
    metadata.insert(
      "quality".to_string(),
      format!("{:?}", context.export_options.quality),
    );
    metadata.insert(
      "compression_level".to_string(),
      context.export_options.compression_level.to_string(),
    );
    metadata.insert(
      "preserve_metadata".to_string(),
      context.export_options.preserve_metadata.to_string(),
    );
    metadata.insert(
      "include_effects_info".to_string(),
      context.export_options.include_effects_info.to_string(),
    );
    metadata.insert("start_time".to_string(), context.start_time.to_rfc3339());

    if let Some(end_time) = context.end_time {
      metadata.insert("end_time".to_string(), end_time.to_rfc3339());
    }

    metadata.insert("status".to_string(), format!("{:?}", context.status));
    metadata.insert(
      "bytes_exported".to_string(),
      context.bytes_exported.to_string(),
    );
    metadata.insert("total_bytes".to_string(), context.total_bytes.to_string());

    metadata.insert(
      "media_type".to_string(),
      format!("{:?}", context.media_processor.media_type()),
    );
    metadata.insert(
      "media_format".to_string(),
      context.media_processor.format().to_string(),
    );
    metadata.insert(
      "media_size".to_string(),
      context.media_processor.data().len().to_string(),
    );

    let (width, height) = context.media_processor.dimensions();
    metadata.insert("dimensions".to_string(), format!("{}x{}", width, height));

    metadata
  }

  fn calculate_checksum(&self, data: &[u8]) -> Option<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{
      Hash,
      Hasher,
    };

    let mut hasher = DefaultHasher::new();
    hasher.write(data);
    Some(format!("{:x}", hasher.finish()))
  }

  pub fn clone(&self) -> EffectExporter {
    EffectExporter {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      supported_formats: self.supported_formats.clone(),
      export_options: self.export_options.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
    }
  }
}

impl Default for ExportOptions {
  fn default() -> Self {
    Self {
      quality: ExportQuality::Medium,
      compression_level: 75,
      preserve_metadata: true,
      include_effects_info: false,
      batch_mode: false,
      parallel_processing: true,
      output_directory: None,
      filename_template: "{name}_{timestamp}".to_string(),
      overwrite_existing: false,
    }
  }
}

impl ExportManager {
  pub fn new() -> Self {
    Self {
      exporters: HashMap::new(),
      contexts: HashMap::new(),
      profiles: HashMap::new(),
      active_contexts: Vec::new(),
      max_concurrent_exports: 4,
      default_exporter: None,
    }
  }

  pub fn with_max_concurrent_exports(mut self, max_concurrent: usize) -> Self {
    self.max_concurrent_exports = max_concurrent;
    self
  }

  pub fn register_exporter(&mut self, exporter: EffectExporter) -> Result<()> {
    if self.exporters.contains_key(&exporter.id) {
      return Err(EllasticError::AlreadyExists(format!(
        "Exporter '{}' already registered",
        exporter.name
      )));
    }

    self.exporters.insert(exporter.id, exporter);

    if self.default_exporter.is_none() {
      self.default_exporter = Some(exporter.id);
    }

    Ok(())
  }

  pub fn unregister_exporter(&mut self, id: Uuid) -> Option<EffectExporter> {
    let exporter = self.exporters.remove(&id);

    if self.default_exporter == Some(id) {
      self.default_exporter = self.exporters.keys().next();
    }

    exporter
  }

  pub fn get_exporter(&self, id: Uuid) -> Option<&EffectExporter> {
    self.exporters.get(&id)
  }

  pub fn get_exporter_by_name(&self, name: &str) -> Option<&EffectExporter> {
    self.exporters.values().find(|e| e.name == name)
  }

  pub fn get_default_exporter(&self) -> Option<&EffectExporter> {
    self.default_exporter.and_then(|id| self.exporters.get(&id))
  }

  pub fn list_exporters(&self) -> Vec<&EffectExporter> {
    self.exporters.values().collect()
  }

  pub fn create_export_context(
    &mut self,
    media_processor: MediaProcessor,
    target_format: ExportFormat,
    output_path: String,
  ) -> Result<Uuid> {
    let exporter = self.get_default_exporter().ok_or_else(|| {
      Err(EllasticError::InvalidParameter(
        "No default exporter available".to_string(),
      ))
    })?;

    if !exporter.supports_format(target_format) {
      return Err(EllasticError::InvalidParameter(format!(
        "Default exporter does not support format: {:?}",
        target_format
      )));
    }

    let context = exporter.export(&media_processor, target_format, output_path)?;
    let context_id = context.id;

    self.contexts.insert(context_id, context);
    Ok(context_id)
  }

  pub fn create_export_context_with_exporter(
    &mut self,
    exporter_id: Uuid,
    media_processor: MediaProcessor,
    target_format: ExportFormat,
    output_path: String,
  ) -> Result<Uuid> {
    let exporter = self
      .get_exporter(exporter_id)
      .ok_or_else(|| EllasticError::InvalidParameter("Exporter not found".to_string()))?;

    if !exporter.supports_format(target_format) {
      return Err(EllasticError::InvalidParameter(format!(
        "Exporter does not support format: {:?}",
        target_format
      )));
    }

    let context = exporter.export(&media_processor, target_format, output_path)?;
    let context_id = context.id;

    self.contexts.insert(context_id, context);
    Ok(context_id)
  }

  pub fn execute_export(&mut self, context_id: Uuid) -> Result<ExportResult> {
    let context = self
      .contexts
      .get_mut(&context_id)
      .ok_or_else(|| EllasticError::InvalidParameter("Export context not found".to_string()))?;

    if self.active_contexts.len() >= self.max_concurrent_exports {
      return Err(EllasticError::ResourceExhausted(
        "Maximum concurrent exports reached".to_string(),
      ));
    }

    self.active_contexts.push(context_id);

    let result = context.exporter.execute_export(context);

    self.active_contexts.retain(|&id| {
      self
        .contexts
        .get(&id)
        .map_or(false, |ctx| ctx.status != ExportStatus::Running)
    });

    result
  }

  pub fn cancel_export(&mut self, context_id: Uuid, reason: String) -> Result<()> {
    if let Some(context) = self.contexts.get_mut(&context_id) {
      context.cancellation_token.write().cancel(reason);
      context.status = ExportStatus::Cancelled;
      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(
        "Export context not found".to_string(),
      ))
    }
  }

  pub fn get_context(&self, context_id: Uuid) -> Option<&ExportContext> {
    self.contexts.get(&context_id)
  }

  pub fn get_context_mut(&mut self, context_id: Uuid) -> Option<&mut ExportContext> {
    self.contexts.get_mut(&context_id)
  }

  pub fn list_contexts(&self) -> Vec<&ExportContext> {
    self.contexts.values().collect()
  }

  pub fn list_active_contexts(&self) -> Vec<&ExportContext> {
    self
      .active_contexts
      .iter()
      .filter_map(|id| self.contexts.get(id))
      .collect()
  }

  pub fn batch_export(&mut self, config: BatchExportConfig) -> Result<BatchExportResult> {
    let start_time = std::time::Instant::now();

    let exporter = self.get_default_exporter().ok_or_else(|| {
      Err(EllasticError::InvalidParameter(
        "No default exporter available".to_string(),
      ))
    })?;

    let mut results = Vec::new();
    let mut errors = Vec::new();
    let mut successful = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut total_size = 0;

    for input_file in &config.input_files {
      if !Path::new(input_file).exists() {
        errors.push(format!("Input file not found: {}", input_file));
        failed += 1;
        if !config.continue_on_error {
          break;
        }
        continue;
      }

      let output_filename =
        self.generate_filename(&config.filename_pattern, input_file, &config.target_format);
      let output_path = Path::new(&config.output_directory)
        .join(&output_filename)
        .to_string_lossy()
        .to_string();

      if Path::new(&output_path).exists() && !config.overwrite_existing {
        skipped += 1;
        continue;
      }

      let media_processor = match MediaProcessor::from_file(input_file) {
        Ok(processor) => processor,
        Err(e) => {
          errors.push(format!("Failed to load {}: {}", input_file, e));
          failed += 1;
          if !config.continue_on_error {
            break;
          }
          continue;
        }
      };

      let context_id = match self.create_export_context_with_exporter(
        exporter.id,
        media_processor,
        config.target_format,
        output_path,
      ) {
        Ok(id) => id,
        Err(e) => {
          errors.push(format!(
            "Failed to create export context for {}: {}",
            input_file, e
          ));
          failed += 1;
          if !config.continue_on_error {
            break;
          }
          continue;
        }
      };

      match self.execute_export(context_id) {
        Ok(result) => {
          total_size += result.file_size;
          results.push(result);
          successful += 1;
        }
        Err(e) => {
          errors.push(format!("Failed to export {}: {}", input_file, e));
          failed += 1;
          if !config.continue_on_error {
            break;
          }
        }
      }
    }

    let total_time = start_time.elapsed();

    Ok(BatchExportResult {
      total_files: config.input_files.len(),
      successful_exports: successful,
      failed_exports: failed,
      skipped_files: skipped,
      total_time,
      total_size,
      results,
      errors,
    })
  }

  fn generate_filename(&self, pattern: &str, input_file: &str, format: &ExportFormat) -> String {
    let path = Path::new(input_file);
    let filename = path
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("output");

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let extension = self.get_file_extension(format);

    pattern
      .replace("{name}", filename)
      .replace("{timestamp}", &timestamp)
      .replace("{ext}", &extension)
  }

  fn get_file_extension(&self, format: &ExportFormat) -> String {
    match format {
      ExportFormat::PNG => "png".to_string(),
      ExportFormat::JPEG => "jpg".to_string(),
      ExportFormat::WEBP => "webp".to_string(),
      ExportFormat::BMP => "bmp".to_string(),
      ExportFormat::TIFF => "tiff".to_string(),
      ExportFormat::GIF => "gif".to_string(),
      ExportFormat::MP4 => "mp4".to_string(),
      ExportFormat::AVI => "avi".to_string(),
      ExportFormat::MOV => "mov".to_string(),
      ExportFormat::WAV => "wav".to_string(),
      ExportFormat::MP3 => "mp3".to_string(),
      ExportFormat::FLAC => "flac".to_string(),
      ExportFormat::OGG => "ogg".to_string(),
      ExportFormat::JSON => "json".to_string(),
      ExportFormat::XML => "xml".to_string(),
      ExportFormat::YAML => "yaml".to_string(),
      ExportFormat::CSV => "csv".to_string(),
      ExportFormat::Custom(ref ext) => ext.clone(),
    }
  }

  pub fn add_profile(&mut self, profile: ExportProfile) -> Result<()> {
    if self.profiles.contains_key(&profile.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Profile '{}' already exists",
        profile.name
      )));
    }

    self.profiles.insert(profile.name.clone(), profile);
    Ok(())
  }

  pub fn get_profile(&self, name: &str) -> Option<&ExportProfile> {
    self.profiles.get(name)
  }

  pub fn list_profiles(&self) -> Vec<&ExportProfile> {
    self.profiles.values().collect()
  }

  pub fn cleanup_completed_contexts(&mut self) -> usize {
    let initial_count = self.contexts.len();

    self.contexts.retain(|_, context| {
      matches!(
        context.status,
        ExportStatus::Pending | ExportStatus::Running | ExportStatus::Failed
      )
    });

    initial_count - self.contexts.len()
  }

  pub fn clear_all_contexts(&mut self) {
    self.contexts.clear();
    self.active_contexts.clear();
  }

  pub fn get_stats(&self) -> ExportStats {
    let total_contexts = self.contexts.len();
    let active_contexts = self.active_contexts.len();
    let completed_contexts = self
      .contexts
      .values()
      .filter(|ctx| matches!(ctx.status, ExportStatus::Completed))
      .count();
    let failed_contexts = self
      .contexts
      .values()
      .filter(|ctx| matches!(ctx.status, ExportStatus::Failed))
      .count();
    let cancelled_contexts = self
      .contexts
      .values()
      .filter(|ctx| matches!(ctx.status, ExportStatus::Cancelled))
      .count();

    ExportStats {
      total_exporters: self.exporters.len(),
      total_contexts,
      active_contexts,
      completed_contexts,
      failed_contexts,
      cancelled_contexts,
      max_concurrent_exports: self.max_concurrent_exports,
    }
  }

  pub fn clone(&self) -> ExportManager {
    ExportManager {
      exporters: self.exporters.clone(),
      contexts: HashMap::new(),
      profiles: self.profiles.clone(),
      active_contexts: Vec::new(),
      max_concurrent_exports: self.max_concurrent_exports,
      default_exporter: self.default_exporter,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ExportStats {
  pub total_exporters: usize,
  pub total_contexts: usize,
  pub active_contexts: usize,
  pub completed_contexts: usize,
  pub failed_contexts: usize,
  pub cancelled_contexts: usize,
  pub max_concurrent_exports: usize,
}

pub fn create_effect_exporter(name: String, description: String) -> EffectExporter {
  EffectExporter::new(name, description)
}

pub fn create_export_options() -> ExportOptions {
  ExportOptions::default()
}

pub fn create_export_manager() -> ExportManager {
  ExportManager::new()
}

pub fn create_export_profile(
  name: String,
  description: String,
  format: ExportFormat,
  media_types: Vec<MediaType>,
) -> ExportProfile {
  ExportProfile {
    name,
    description,
    format,
    export_options: ExportOptions::default(),
    supported_media_types: media_types,
    created_at: Utc::now(),
  }
}
