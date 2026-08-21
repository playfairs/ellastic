use ellastic_errors::{Result, EllasticError};
use ellastic_core::{MediaData, MediaType};
use ellastic_image::{ImageProcessor, ImageData};
use ellastic_audio::{AudioProcessor, AudioData};
use ellastic_media::{MediaProcessor};
use ellastic_glitch::{GlitchProcessor, GlitchEffect};
use ellastic_utils::{create_random_generator};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct EffectRenderer {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub supported_media_types: Vec<MediaType>,
    pub rendering_options: RenderingOptions,
    pub performance_settings: PerformanceSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RenderingOptions {
    pub quality: RenderingQuality,
    pub resolution: Option<(u32, u32)>,
    pub format: OutputFormat,
    pub color_space: ColorSpace,
    pub bit_depth: u8,
    pub frame_rate: Option<f32>,
    pub compression_level: u8,
    pub enable_alpha: bool,
    pub enable_metadata: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom { quality_factor: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
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
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    RGB,
    RGBA,
    SRGB,
    CMYK,
    YUV,
    GRAY,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct PerformanceSettings {
    pub parallel_processing: bool,
    pub max_threads: Option<usize>,
    pub memory_limit_mb: Option<usize>,
    pub tile_size: Option<u32>,
    pub chunk_size: Option<usize>,
    pub enable_gpu_acceleration: bool,
    pub enable_caching: bool,
}

#[derive(Debug, Clone)]
pub struct RenderingContext {
    pub id: Uuid,
    pub renderer: EffectRenderer,
    pub media_processor: MediaProcessor,
    pub target_format: OutputFormat,
    pub rendering_options: RenderingOptions,
    pub progress_callback: Option<Box<dyn ProgressCallback + Send + Sync>>,
    pub cancellation_token: Arc<RwLock<CancellationToken>>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: RenderingStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct RenderingResult {
    pub context_id: Uuid,
    pub output_data: Vec<u8>,
    pub output_format: OutputFormat,
    pub rendering_time: std::time::Duration,
    pub file_size: usize,
    pub metadata: HashMap<String, String>,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub processing_time: std::time::Duration,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub gpu_usage_percent: Option<f64>,
    pub cache_hit_rate: f64,
    pub threads_used: usize,
    pub frames_processed: u32,
    pub pixels_processed: u64,
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

impl EffectRenderer {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            supported_media_types: Vec::new(),
            rendering_options: RenderingOptions::default(),
            performance_settings: PerformanceSettings::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_supported_media_types(mut self, media_types: Vec<MediaType>) -> Self {
        self.supported_media_types = media_types;
        self
    }

    pub fn with_rendering_options(mut self, options: RenderingOptions) -> Self {
        self.rendering_options = options;
        self
    }

    pub fn with_performance_settings(mut self, settings: PerformanceSettings) -> Self {
        self.performance_settings = settings;
        self
    }

    pub fn supports_media_type(&self, media_type: MediaType) -> bool {
        self.supported_media_types.contains(&media_type)
    }

    pub fn render(&self, media_processor: &mut MediaProcessor, target_format: OutputFormat) -> Result<RenderingContext> {
        if !self.supports_media_type(media_processor.media_type()) {
            return Err(EllasticError::InvalidParameter(format!(
                "Renderer '{}' does not support media type: {:?}",
                self.name, media_processor.media_type()
            )));
        }

        let context = RenderingContext {
            id: Uuid::new_v4(),
            renderer: self.clone(),
            media_processor: media_processor.clone(),
            target_format,
            rendering_options: self.rendering_options.clone(),
            progress_callback: None,
            cancellation_token: Arc::new(RwLock::new(CancellationToken::new())),
            start_time: Utc::now(),
            end_time: None,
            status: RenderingStatus::Pending,
            error: None,
        };

        Ok(context)
    }

    pub fn render_with_progress(
        &self,
        media_processor: &mut MediaProcessor,
        target_format: OutputFormat,
        progress_callback: Box<dyn ProgressCallback + Send + Sync>,
    ) -> Result<RenderingContext> {
        if !self.supports_media_type(media_processor.media_type()) {
            return Err(EllasticError::InvalidParameter(format!(
                "Renderer '{}' does not support media type: {:?}",
                self.name, media_processor.media_type()
            )));
        }

        let context = RenderingContext {
            id: Uuid::new_v4(),
            renderer: self.clone(),
            media_processor: media_processor.clone(),
            target_format,
            rendering_options: self.rendering_options.clone(),
            progress_callback: Some(progress_callback),
            cancellation_token: Arc::new(RwLock::new(CancellationToken::new())),
            start_time: Utc::now(),
            end_time: None,
            status: RenderingStatus::Pending,
            error: None,
        };

        Ok(context)
    }

    pub fn execute_rendering(&self, context: &mut RenderingContext) -> Result<RenderingResult> {
        let start_time = std::time::Instant::now();

        context.status = RenderingStatus::Running;

        if let Some(ref callback) = context.progress_callback {
            callback.on_progress(0.0, Some("Starting rendering..."));
        }

        if context.cancellation_token.read().is_cancelled() {
            context.status = RenderingStatus::Cancelled;
            return Err(EllasticError::Cancelled("Rendering cancelled".to_string()));
        }

        let result = match context.media_processor.media_type() {
            MediaType::Image => self.render_image(context)?,
            MediaType::Audio => self.render_audio(context)?,
            MediaType::Video => self.render_video(context)?,
            _ => return Err(EllasticError::UnsupportedOperation("Unsupported media type".to_string())),
        };

        let end_time = Utc::now();
        let rendering_time = start_time.elapsed();

        context.end_time = Some(end_time);
        context.status = RenderingStatus::Completed;

        if let Some(ref callback) = context.progress_callback {
            callback.on_progress(1.0, Some("Rendering completed"));
            callback.on_complete();
        }

        Ok(RenderingResult {
            context_id: context.id,
            output_data: result,
            output_format: context.target_format,
            rendering_time,
            file_size: result.len(),
            metadata: self.extract_metadata(context),
            performance_metrics: self.calculate_performance_metrics(context, rendering_time),
        })
    }

    fn render_image(&self, context: &mut RenderingContext) -> Result<Vec<u8>> {
        let image_processor = context.media_processor.image_processor()
            .ok_or_else(|| EllasticError::InvalidParameter("Not an image processor".to_string()))?;

        let (width, height) = (image_processor.width(), image_processor.height());
        let target_size = context.rendering_options.resolution.unwrap_or((width, height));

        let mut final_processor = if (width, height) != target_size {
            let mut resized = image_processor.clone();
            resized.resize(target_size.0, target_size.1)?;
            resized
        } else {
            image_processor.clone()
        };

        match context.rendering_options.quality {
            RenderingQuality::Low => {
                final_processor.adjust_quality(0.5)?;
            }
            RenderingQuality::High => {
                final_processor.adjust_quality(1.5)?;
            }
            RenderingQuality::Ultra => {
                final_processor.adjust_quality(2.0)?;
            }
            RenderingQuality::Custom { quality_factor } => {
                final_processor.adjust_quality(quality_factor)?;
            }
        }

        match context.rendering_options.color_space {
            ColorSpace::GRAY => {
                final_processor.grayscale()?;
            }
            ColorSpace::CMYK => {
                final_processor.convert_to_cmyk()?;
            }
            _ => {}
        }

        let output_data = match context.target_format {
            OutputFormat::PNG => {
                final_processor.encode_png(context.rendering_options.enable_metadata)?
            }
            OutputFormat::JPEG => {
                final_processor.encode_jpeg(
                    context.rendering_options.compression_level,
                    context.rendering_options.enable_metadata
                )?
            }
            OutputFormat::WEBP => {
                final_processor.encode_webp(
                    context.rendering_options.compression_level,
                    context.rendering_options.enable_metadata
                )?
            }
            OutputFormat::BMP => {
                final_processor.encode_bmp()?
            }
            OutputFormat::TIFF => {
                final_processor.encode_tiff(context.rendering_options.enable_metadata)?
            }
            OutputFormat::GIF => {
                final_processor.encode_gif()?
            }
            OutputFormat::Custom(ref format_name) => {
                return Err(EllasticError::UnsupportedOperation(format!(
                    "Custom format '{}' not supported",
                    format_name
                )));
            }
            _ => {
                return Err(EllasticError::UnsupportedOperation(format!(
                    "Format '{:?}' not supported for images",
                    context.target_format
                )));
            }
        };

        Ok(output_data)
    }

    fn render_audio(&self, context: &mut RenderingContext) -> Result<Vec<u8>> {
        let audio_processor = context.media_processor.audio_processor()
            .ok_or_else(|| EllasticError::InvalidParameter("Not an audio processor".to_string()))?;

        let mut final_processor = audio_processor.clone();

        match context.rendering_options.quality {
            RenderingQuality::Low => {
                final_processor.adjust_quality(0.5)?;
            }
            RenderingQuality::High => {
                final_processor.adjust_quality(1.5)?;
            }
            RenderingQuality::Ultra => {
                final_processor.adjust_quality(2.0)?;
            }
            RenderingQuality::Custom { quality_factor } => {
                final_processor.adjust_quality(quality_factor)?;
            }
        }

        let output_data = match context.target_format {
            OutputFormat::WAV => {
                final_processor.encode_wav(context.rendering_options.bit_depth)?
            }
            OutputFormat::MP3 => {
                final_processor.encode_mp3(
                    context.rendering_options.compression_level,
                    context.rendering_options.bit_depth
                )?
            }
            OutputFormat::FLAC => {
                final_processor.encode_flac(context.rendering_options.bit_depth)?
            }
            OutputFormat::OGG => {
                final_processor.encode_ogg(context.rendering_options.bit_depth)?
            }
            OutputFormat::Custom(ref format_name) => {
                return Err(EllasticError::UnsupportedOperation(format!(
                    "Custom format '{}' not supported",
                    format_name
                )));
            }
            _ => {
                return Err(EllasticError::UnsupportedOperation(format!(
                    "Format '{:?}' not supported for audio",
                    context.target_format
                )));
            }
        };

        Ok(output_data)
    }

    fn render_video(&self, context: &mut RenderingContext) -> Result<Vec<u8>> {
        let video_processor = context.media_processor.video_processor()
            .ok_or_else(|| EllasticError::InvalidParameter("Not a video processor".to_string()))?;

        let mut final_processor = video_processor.clone();

        match context.rendering_options.quality {
            RenderingQuality::Low => {
                final_processor.set_quality(0.5)?;
            }
            RenderingQuality::High => {
                final_processor.set_quality(1.5)?;
            }
            RenderingQuality::Ultra => {
                final_processor.set_quality(2.0)?;
            }
            RenderingQuality::Custom { quality_factor } => {
                final_processor.set_quality(quality_factor)?;
            }
        }

        let output_data = match context.target_format {
            OutputFormat::MP4 => {
                final_processor.encode_mp4(
                    context.rendering_options.compression_level,
                    context.rendering_options.frame_rate
                )?
            }
            OutputFormat::AVI => {
                final_processor.encode_avi()?
            }
            OutputFormat::MOV => {
                final_processor.encode_mov()?
            }
            OutputFormat::Custom(ref format_name) => {
                return Err(EllasticError::UnsupportedOperation(format!(
                    "Custom format '{}' not supported",
                    format_name
                )));
            }
            _ => {
                return Err(EllasticError::UnsupportedOperation(format!(
                    "Format '{:?}' not supported for video",
                    context.target_format
                )));
            }
        };

        Ok(output_data)
    }

    fn extract_metadata(&self, context: &RenderingContext) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        metadata.insert("renderer_name".to_string(), context.renderer.name.clone());
        metadata.insert("renderer_id".to_string(), context.renderer.id.to_string());
        metadata.insert("target_format".to_string(), format!("{:?}", context.target_format));
        metadata.insert("rendering_quality".to_string(), format!("{:?}", context.rendering_options.quality));
        metadata.insert("color_space".to_string(), format!("{:?}", context.rendering_options.color_space));
        metadata.insert("bit_depth".to_string(), context.rendering_options.bit_depth.to_string());
        metadata.insert("compression_level".to_string(), context.rendering_options.compression_level.to_string());
        metadata.insert("enable_alpha".to_string(), context.rendering_options.enable_alpha.to_string());
        metadata.insert("enable_metadata".to_string(), context.rendering_options.enable_metadata.to_string());

        if let Some((width, height)) = context.rendering_options.resolution {
            metadata.insert("resolution".to_string(), format!("{}x{}", width, height));
        }

        if let Some(frame_rate) = context.rendering_options.frame_rate {
            metadata.insert("frame_rate".to_string(), frame_rate.to_string());
        }

        metadata.insert("start_time".to_string(), context.start_time.to_rfc3339());

        if let Some(end_time) = context.end_time {
            metadata.insert("end_time".to_string(), end_time.to_rfc3339());
        }

        metadata.insert("status".to_string(), format!("{:?}", context.status));

        metadata
    }

    fn calculate_performance_metrics(&self, context: &RenderingContext, rendering_time: std::time::Duration) -> PerformanceMetrics {
        PerformanceMetrics {
            processing_time: rendering_time,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            gpu_usage_percent: None,
            cache_hit_rate: 0.0,
            threads_used: self.performance_settings.max_threads.unwrap_or(1),
            frames_processed: match context.media_processor.media_type() {
                MediaType::Video => context.media_processor.frame_count().unwrap_or(0),
                _ => 1,
            },
            pixels_processed: match context.media_processor.media_type() {
                MediaType::Image => {
                    let (width, height) = context.media_processor.dimensions();
                    (width * height) as u64
                }
                MediaType::Video => {
                    let (width, height) = context.media_processor.dimensions();
                    let frames = context.media_processor.frame_count().unwrap_or(0);
                    (width * height * frames) as u64
                }
                _ => 0,
            },
        }
    }

    pub fn clone(&self) -> EffectRenderer {
        EffectRenderer {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            supported_media_types: self.supported_media_types.clone(),
            rendering_options: self.rendering_options.clone(),
            performance_settings: self.performance_settings.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl Default for RenderingOptions {
    fn default() -> Self {
        Self {
            quality: RenderingQuality::Medium,
            resolution: None,
            format: OutputFormat::PNG,
            color_space: ColorSpace::RGB,
            bit_depth: 8,
            frame_rate: None,
            compression_level: 75,
            enable_alpha: true,
            enable_metadata: true,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            parallel_processing: true,
            max_threads: None,
            memory_limit_mb: None,
            tile_size: None,
            chunk_size: None,
            enable_gpu_acceleration: false,
            enable_caching: true,
        }
    }

#[derive(Debug, Clone)]
pub struct RenderingManager {
    renderers: HashMap<Uuid, EffectRenderer>,
    contexts: HashMap<Uuid, RenderingContext>,
    active_contexts: Vec<Uuid>,
    max_concurrent_renders: usize,
    default_renderer: Option<Uuid>,
}

impl RenderingManager {
    pub fn new() -> Self {
        Self {
            renderers: HashMap::new(),
            contexts: HashMap::new(),
            active_contexts: Vec::new(),
            max_concurrent_renders: 4,
            default_renderer: None,
        }
    }

    pub fn with_max_concurrent_renders(mut self, max_concurrent: usize) -> Self {
        self.max_concurrent_renders = max_concurrent;
        self
    }

    pub fn register_renderer(&mut self, renderer: EffectRenderer) -> Result<()> {
        if self.renderers.contains_key(&renderer.id) {
            return Err(EllasticError::AlreadyExists(format!(
                "Renderer '{}' already registered",
                renderer.name
            )));
        }

        self.renderers.insert(renderer.id, renderer);

        if self.default_renderer.is_none() {
            self.default_renderer = Some(renderer.id);
        }

        Ok(())
    }

    pub fn unregister_renderer(&mut self, id: Uuid) -> Option<EffectRenderer> {
        let renderer = self.renderers.remove(&id);

        if self.default_renderer == Some(id) {
            self.default_renderer = self.renderers.keys().next();
        }

        renderer
    }

    pub fn get_renderer(&self, id: Uuid) -> Option<&EffectRenderer> {
        self.renderers.get(&id)
    }

    pub fn get_renderer_by_name(&self, name: &str) -> Option<&EffectRenderer> {
        self.renderers.values().find(|r| r.name == name)
    }

    pub fn get_default_renderer(&self) -> Option<&EffectRenderer> {
        self.default_renderer.and_then(|id| self.renderers.get(&id))
    }

    pub fn list_renderers(&self) -> Vec<&EffectRenderer> {
        self.renderers.values().collect()
    }

    pub fn create_rendering_context(&mut self, media_processor: MediaProcessor, target_format: OutputFormat) -> Result<Uuid> {
        let renderer = self.get_default_renderer()
            .ok_or_else(|| Err(EllasticError::InvalidParameter("No default renderer available".to_string()))?;

        if !renderer.supports_media_type(media_processor.media_type()) {
            return Err(EllasticError::InvalidParameter(format!(
                "Default renderer does not support media type: {:?}",
                media_processor.media_type()
            )));
        }

        let context = renderer.render(&mut media_processor.clone(), target_format)?;
        let context_id = context.id;

        self.contexts.insert(context_id, context);
        Ok(context_id)
    }

    pub fn create_rendering_context_with_renderer(
        &mut self,
        renderer_id: Uuid,
        media_processor: MediaProcessor,
        target_format: OutputFormat,
    ) -> Result<Uuid> {
        let renderer = self.get_renderer(renderer_id)
            .ok_or_else(|| EllasticError::InvalidParameter("Renderer not found".to_string()))?;

        if !renderer.supports_media_type(media_processor.media_type()) {
            return Err(EllasticError::InvalidParameter(format!(
                "Renderer does not support media type: {:?}",
                media_processor.media_type()
            )));
        }

        let context = renderer.render(&mut media_processor.clone(), target_format)?;
        let context_id = context.id;

        self.contexts.insert(context_id, context);
        Ok(context_id)
    }

    pub fn execute_rendering(&mut self, context_id: Uuid) -> Result<RenderingResult> {
        let context = self.contexts.get_mut(&context_id)
            .ok_or_else(|| EllasticError::InvalidParameter("Rendering context not found".to_string()))?;

        if self.active_contexts.len() >= self.max_concurrent_renders {
            return Err(EllasticError::ResourceExhausted("Maximum concurrent renders reached".to_string()));
        }

        self.active_contexts.push(context_id);

        let result = context.renderer.execute_rendering(context);

        self.active_contexts.retain(|&id| {
            self.contexts.get(&id).map_or(false, |ctx| ctx.status != RenderingStatus::Running)
        });

        result
    }

    pub fn cancel_rendering(&mut self, context_id: Uuid, reason: String) -> Result<()> {
        if let Some(context) = self.contexts.get_mut(&context_id) {
            context.cancellation_token.write().cancel(reason);
            context.status = RenderingStatus::Cancelled;
            Ok(())
        } else {
            Err(EllasticError::InvalidParameter("Rendering context not found".to_string()))
        }
    }

    pub fn get_context(&self, context_id: Uuid) -> Option<&RenderingContext> {
        self.contexts.get(&context_id)
    }

    pub fn get_context_mut(&mut self, context_id: Uuid) -> Option<&mut RenderingContext> {
        self.contexts.get_mut(&context_id)
    }

    pub fn list_contexts(&self) -> Vec<&RenderingContext> {
        self.contexts.values().collect()
    }

    pub fn list_active_contexts(&self) -> Vec<&RenderingContext> {
        self.active_contexts
            .iter()
            .filter_map(|id| self.contexts.get(id))
            .collect()
    }

    pub fn cleanup_completed_contexts(&mut self) -> usize {
        let initial_count = self.contexts.len();

        self.contexts.retain(|_, context| {
            matches!(context.status, RenderingStatus::Pending | RenderingStatus::Running | RenderingStatus::Failed)
        });

        initial_count - self.contexts.len()
    }

    pub fn clear_all_contexts(&mut self) {
        self.contexts.clear();
        self.active_contexts.clear();
    }

    pub fn get_stats(&self) -> RenderingStats {
        let total_contexts = self.contexts.len();
        let active_contexts = self.active_contexts.len();
        let completed_contexts = self.contexts
            .values()
            .filter(|ctx| matches!(ctx.status, RenderingStatus::Completed))
            .count();
        let failed_contexts = self.contexts
            .values()
            .filter(|ctx| matches!(ctx.status, RenderingStatus::Failed))
            .count();
        let cancelled_contexts = self.contexts
            .values()
            .filter(|ctx| matches!(ctx.status, RenderingStatus::Cancelled))
            .count();

        RenderingStats {
            total_renderers: self.renderers.len(),
            total_contexts,
            active_contexts,
            completed_contexts,
            failed_contexts,
            cancelled_contexts,
            max_concurrent_renders: self.max_concurrent_renders,
        }
    }

    pub fn clone(&self) -> RenderingManager {
        RenderingManager {
            renderers: self.renderers.clone(),
            contexts: HashMap::new(),
            active_contexts: Vec::new(),
            max_concurrent_renders: self.max_concurrent_renders,
            default_renderer: self.default_renderer,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderingStats {
    pub total_renderers: usize,
    pub total_contexts: usize,
    pub active_contexts: usize,
    pub completed_contexts: usize,
    pub failed_contexts: usize,
    pub cancelled_contexts: usize,
    pub max_concurrent_renders: usize,
}

pub fn create_effect_renderer(name: String, description: String) -> EffectRenderer {
    EffectRenderer::new(name, description)
}

pub fn create_rendering_options() -> RenderingOptions {
    RenderingOptions::default()
}

pub fn create_performance_settings() -> PerformanceSettings {
    PerformanceSettings::default()
}

pub fn create_rendering_manager() -> RenderingManager {
    RenderingManager::new()
}

pub fn create_cancellation_token() -> CancellationToken {
    CancellationToken::new()
}
