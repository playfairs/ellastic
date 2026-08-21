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

#[derive(Debug, Clone)]
pub enum ProcessorType {
  Image {
    image_processor: ImageProcessor,
  },
  Audio {
    audio_processor: AudioProcessor,
  },
  Video {
    video_processor: ellastic_media::VideoProcessor,
  },
  Composite {
    composite_processor: MediaProcessor,
  },
  Custom {
    custom_processor: CustomProcessor,
  },
}

#[derive(Debug, Clone)]
pub struct CustomProcessor {
  pub name: String,
  pub description: String,
  pub supported_media_types: Vec<MediaType>,
  pub process_function:
    Box<dyn Fn(&mut MediaProcessor, &HashMap<String, String>) -> Result<()> + Send + Sync>,
}

impl CustomProcessor {
  pub fn new(
    name: String,
    description: String,
    supported_media_types: Vec<MediaType>,
    process_function: Box<
      dyn Fn(&mut MediaProcessor, &HashMap<String, String>) -> Result<()> + Send + Sync,
    >,
  ) -> Self {
    Self {
      name,
      description,
      supported_media_types,
      process_function,
    }
  }

  pub fn process(
    &self,
    media_processor: &mut MediaProcessor,
    parameters: &HashMap<String, String>,
  ) -> Result<()> {
    (self.process_function)(media_processor, parameters)
  }

  pub fn clone(&self) -> CustomProcessor {
    CustomProcessor {
      name: self.name.clone(),
      description: self.description.clone(),
      supported_media_types: self.supported_media_types.clone(),
      process_function: Box::new(|_, _| {
        Err(EllasticError::UnsupportedOperation(
          "Custom processor cloning not supported".to_string(),
        ))
      }),
    }
  }
}

#[derive(Debug, Clone)]
pub struct EffectProcessor {
  processor_type: ProcessorType,
  parameters: HashMap<String, String>,
  cache: Arc<RwLock<ProcessorCache>>,
  performance_stats: Arc<RwLock<ProcessorPerformanceStats>>,
}

impl EffectProcessor {
  pub fn new(processor_type: ProcessorType) -> Self {
    Self {
      processor_type,
      parameters: HashMap::new(),
      cache: Arc::new(RwLock::new(ProcessorCache::new())),
      performance_stats: Arc::new(RwLock::new(ProcessorPerformanceStats::new())),
    }
  }

  pub fn processor_type(&self) -> &ProcessorType {
    &self.processor_type
  }

  pub fn parameters(&self) -> &HashMap<String, String> {
    &self.parameters
  }

  pub fn parameters_mut(&mut self) -> &mut HashMap<String, String> {
    &mut self.parameters
  }

  pub fn cache(&self) -> Arc<RwLock<ProcessorCache>> {
    self.cache.clone()
  }

  pub fn performance_stats(&self) -> Arc<RwLock<ProcessorPerformanceStats>> {
    self.performance_stats.clone()
  }

  pub fn set_parameter(&mut self, name: String, value: String) -> Result<()> {
    self.validate_parameter(&name, &value)?;
    self.parameters.insert(name, value);
    Ok(())
  }

  pub fn get_parameter(&self, name: &str) -> Option<&String> {
    self.parameters.get(name)
  }

  pub fn reset_parameters(&mut self) {
    self.parameters.clear();
  }

  pub fn process(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
    let start_time = std::time::Instant::now();

    self.validate_media_type(media_processor)?;

    let cache_key = self.generate_cache_key(media_processor);

    if let Some(cached_result) = self.cache.read().get(&cache_key) {
      *media_processor = cached_result.clone();
      return Ok(());
    }

    match &mut self.processor_type {
      ProcessorType::Image { image_processor } => {
        self.process_image(media_processor, image_processor)?;
      }
      ProcessorType::Audio { audio_processor } => {
        self.process_audio(media_processor, audio_processor)?;
      }
      ProcessorType::Video { video_processor } => {
        self.process_video(media_processor, video_processor)?;
      }
      ProcessorType::Composite {
        composite_processor,
      } => {
        self.process_composite(media_processor, composite_processor)?;
      }
      ProcessorType::Custom { custom_processor } => {
        self.process_custom(media_processor, custom_processor)?;
      }
    }

    let processing_time = start_time.elapsed();
    self
      .performance_stats
      .write()
      .record_execution(processing_time);

    self.cache.write().put(cache_key, media_processor.clone());

    Ok(())
  }

  pub fn process_async(
    &mut self,
    media_processor: MediaProcessor,
  ) -> Result<tokio::task::JoinHandle<Result<MediaProcessor>>> {
    let processor_type = self.processor_type.clone();
    let parameters = self.parameters.clone();
    let cache = self.cache.clone();
    let performance_stats = self.performance_stats.clone();

    let handle = tokio::spawn(async move {
      let start_time = std::time::Instant::now();

      let mut processor = media_processor;

      let cache_key = Self::generate_cache_key_static(&processor_type, &parameters, &processor);

      if let Some(cached_result) = cache.read().get(&cache_key) {
        return Ok(cached_result);
      }

      Self::process_media_static(&mut processor, &processor_type, &parameters)?;

      let processing_time = start_time.elapsed();
      performance_stats.write().record_execution(processing_time);

      cache.write().put(cache_key, processor.clone());

      Ok(processor)
    });

    Ok(handle)
  }

  pub fn preview(
    &mut self,
    media_processor: &mut MediaProcessor,
    preview_size: (u32, u32),
  ) -> Result<()> {
    let original_size = (media_processor.width(), media_processor.height());

    if original_size != preview_size {
      media_processor.resize(preview_size.0, preview_size.1)?;
    }

    self.process(media_processor)?;

    if original_size != preview_size {
      media_processor.resize(original_size.0, original_size.1)?;
    }

    Ok(())
  }

  pub fn batch_process(
    &mut self,
    media_processors: &mut [MediaProcessor],
  ) -> Result<Vec<Result<()>>> {
    media_processors
      .par_iter_mut()
      .map(|processor| {
        let mut temp_processor = processor.clone();
        let mut temp_effect_processor = EffectProcessor::new(self.processor_type.clone());
        temp_effect_processor.parameters = self.parameters.clone();

        match temp_effect_processor.process(&mut temp_processor) {
          Ok(()) => {
            *processor = temp_processor;
            Ok(())
          }
          Err(e) => Err(e),
        }
      })
      .collect()
  }

  pub fn batch_process_async(
    &mut self,
    media_processors: Vec<MediaProcessor>,
  ) -> Result<Vec<tokio::task::JoinHandle<Result<MediaProcessor>>>> {
    let processor_type = self.processor_type.clone();
    let parameters = self.parameters.clone();
    let cache = self.cache.clone();
    let performance_stats = self.performance_stats.clone();

    let handles: Vec<_> = media_processors
      .into_iter()
      .map(|processor| {
        let processor_type = processor_type.clone();
        let parameters = parameters.clone();
        let cache = cache.clone();
        let performance_stats = performance_stats.clone();

        tokio::spawn(async move {
          let start_time = std::time::Instant::now();

          let mut temp_processor = processor;

          let cache_key =
            Self::generate_cache_key_static(&processor_type, &parameters, &temp_processor);

          if let Some(cached_result) = cache.read().get(&cache_key) {
            return Ok(cached_result);
          }

          Self::process_media_static(&mut temp_processor, &processor_type, &parameters)?;

          let processing_time = start_time.elapsed();
          performance_stats.write().record_execution(processing_time);

          cache.write().put(cache_key, temp_processor.clone());

          Ok(temp_processor)
        })
      })
      .collect();

    Ok(handles)
  }

  fn validate_parameter(&self, name: &str, value: &str) -> Result<()> {
    match name {
      "brightness" | "contrast" | "saturation" | "gamma" => {
        value
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid float value".to_string()))?;
      }
      "radius" | "amount" | "delay" | "ratio" => {
        value
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid float value".to_string()))?;
      }
      "angle" => {
        value
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid angle value".to_string()))?;
      }
      "count" | "bit_depth" | "sample_rate_reduction" => {
        value
          .parse::<u32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid integer value".to_string()))?;
      }
      "threshold" | "intensity" | "mix_ratio" => {
        value
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid float value".to_string()))?;
      }
      "preserve_size" | "preserve_duration" => {
        value
          .parse::<bool>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid boolean value".to_string()))?;
      }
      "mode" | "direction" => match value {
        "brightness" | "hue" | "saturation" | "random" => Ok(()),
        "horizontal" | "vertical" | "both" => Ok(()),
        _ => Err(EllasticError::InvalidParameter(
          "Invalid enum value".to_string(),
        )),
      },
      "cutoff" | "room_size" => {
        value
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid float value".to_string()))?;
      }
      _ => Ok(()),
    }
  }

  fn validate_media_type(&self, media_processor: &MediaProcessor) -> Result<()> {
    let supported_media_types = match &self.processor_type {
      ProcessorType::Image { .. } => vec![MediaType::Image],
      ProcessorType::Audio { .. } => vec![MediaType::Audio],
      ProcessorType::Video { .. } => vec![MediaType::Video],
      ProcessorType::Composite { .. } => vec![MediaType::Image, MediaType::Video],
      ProcessorType::Custom { custom_processor } => custom_processor.supported_media_types.clone(),
    };

    if !supported_media_types.contains(&media_processor.media_type()) {
      return Err(EllasticError::InvalidParameter(format!(
        "Processor does not support media type: {:?}",
        media_processor.media_type()
      )));
    }

    Ok(())
  }

  fn generate_cache_key(&self, media_processor: &MediaProcessor) -> String {
    Self::generate_cache_key_static(&self.processor_type, &self.parameters, media_processor)
  }

  fn generate_cache_key_static(
    processor_type: &ProcessorType,
    parameters: &HashMap<String, String>,
    media_processor: &MediaProcessor,
  ) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{
      Hash,
      Hasher,
    };

    let mut hasher = DefaultHasher::new();

    match processor_type {
      ProcessorType::Image { .. } => "image".hash(&mut hasher),
      ProcessorType::Audio { .. } => "audio".hash(&mut hasher),
      ProcessorType::Video { .. } => "video".hash(&mut hasher),
      ProcessorType::Composite { .. } => "composite".hash(&mut hasher),
      ProcessorType::Custom { custom_processor } => custom_processor.name.hash(&mut hasher),
    }

    let mut param_keys: Vec<_> = parameters.keys().collect();
    param_keys.sort();

    for key in param_keys {
      key.hash(&mut hasher);
      parameters[key].hash(&mut hasher);
    }

    media_processor.data().hash(&mut hasher);

    format!(
      "{}:{}:{}",
      hasher.finish(),
      parameters.len(),
      media_processor.data().len()
    )
  }

  fn process_image(
    &mut self,
    media_processor: &mut MediaProcessor,
    image_processor: &mut ImageProcessor,
  ) -> Result<()> {
    if let Some(processor) = media_processor.image_processor_mut() {
      *processor = image_processor.clone();

      if let Some(brightness) = self.parameters.get("brightness") {
        let amount = brightness
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid brightness value".to_string()))?;
        processor.adjust_brightness(amount)?;
      }

      if let Some(contrast) = self.parameters.get("contrast") {
        let amount = contrast
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid contrast value".to_string()))?;
        processor.adjust_contrast(amount)?;
      }

      if let Some(saturation) = self.parameters.get("saturation") {
        let amount = saturation
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid saturation value".to_string()))?;
        processor.adjust_saturation(amount)?;
      }

      if let Some(gamma) = self.parameters.get("gamma") {
        let gamma = gamma
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid gamma value".to_string()))?;
        processor.adjust_gamma(gamma)?;
      }

      if let Some(radius) = self.parameters.get("radius") {
        let radius = radius
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid radius value".to_string()))?;
        processor.blur(radius)?;
      }

      if let Some(amount) = self.parameters.get("amount") {
        let amount = amount
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid amount value".to_string()))?;
        processor.sharpen(amount)?;
      }

      if self.parameters.contains_key("edge_detection") {
        processor.edge_detection()?;
      }

      if self.parameters.contains_key("emboss") {
        processor.emboss()?;
      }

      if let Some(threshold) = self.parameters.get("threshold") {
        let threshold = threshold
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid threshold value".to_string()))?;
        let mode = self
          .parameters
          .get("mode")
          .map(|s| match s.as_str() {
            "brightness" => crate::effects::PixelSortMode::Brightness,
            "hue" => crate::effects::PixelSortMode::Hue,
            "saturation" => crate::effects::PixelSortMode::Saturation,
            "random" => crate::effects::PixelSortMode::Random,
            _ => crate::effects::PixelSortMode::Brightness,
          })
          .unwrap_or(crate::effects::PixelSortMode::Brightness);

        self.apply_pixel_sort(processor, threshold, mode)?;
      }

      if let Some(intensity) = self.parameters.get("intensity") {
        let intensity = intensity
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid intensity value".to_string()))?;
        let preserve_size = self
          .parameters
          .get("preserve_size")
          .and_then(|s| s.parse::<bool>().ok())
          .unwrap_or(true);

        self.apply_data_mosh(processor, intensity, preserve_size)?;
      }

      if self.parameters.contains_key("grayscale") {
        processor.grayscale()?;
      }

      if self.parameters.contains_key("sepia") {
        processor.sepia()?;
      }

      if self.parameters.contains_key("invert") {
        processor.invert()?;
      }

      if let Some(angle) = self.parameters.get("angle") {
        let angle = angle
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid angle value".to_string()))?;
        processor.hue_rotate(angle)?;
      }

      *image_processor = processor.clone();
    }

    Ok(())
  }

  fn process_audio(
    &mut self,
    media_processor: &mut MediaProcessor,
    audio_processor: &mut AudioProcessor,
  ) -> Result<()> {
    if let Some(processor) = media_processor.audio_processor_mut() {
      *processor = audio_processor.clone();

      if let Some(room_size) = self.parameters.get("room_size") {
        let room_size = room_size
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid room_size value".to_string()))?;
        processor.reverb(room_size)?;
      }

      if let Some(delay) = self.parameters.get("delay") {
        let delay = delay
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid delay value".to_string()))?;
        processor.echo(delay)?;
      }

      if let Some(amount) = self.parameters.get("amount") {
        let amount = amount
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid amount value".to_string()))?;
        processor.distortion(amount)?;
      }

      if let Some(ratio) = self.parameters.get("ratio") {
        let ratio = ratio
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid ratio value".to_string()))?;
        processor.compressor(ratio)?;
      }

      if let Some(bit_depth) = self.parameters.get("bit_depth") {
        let bit_depth = bit_depth
          .parse::<u8>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid bit_depth value".to_string()))?;
        let sample_rate_reduction = self
          .parameters
          .get("sample_rate_reduction")
          .and_then(|s| s.parse::<u32>().ok())
          .unwrap_or(1);

        processor.bit_crush(bit_depth, sample_rate_reduction)?;
      }

      if let Some(cutoff) = self.parameters.get("cutoff") {
        let cutoff = cutoff
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid cutoff value".to_string()))?;
        processor.low_pass_filter(cutoff)?;
      }

      *audio_processor = processor.clone();
    }

    Ok(())
  }

  fn process_video(
    &mut self,
    media_processor: &mut MediaProcessor,
    video_processor: &mut ellastic_media::VideoProcessor,
  ) -> Result<()> {
    if let Some(processor) = media_processor.video_processor_mut() {
      *processor = video_processor.clone();

      if let Some(count) = self.parameters.get("count") {
        let count = count
          .parse::<u32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid count value".to_string()))?;
        processor.duplicate_frames(count)?;
      }

      if let Some(count) = self.parameters.get("count") {
        let count = count
          .parse::<u32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid count value".to_string()))?;
        processor.drop_frames(count)?;
      }

      if let Some(ratio) = self.parameters.get("ratio") {
        let ratio = ratio
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid ratio value".to_string()))?;
        processor.time_stretch(ratio)?;
      }

      if self.parameters.contains_key("reverse_playback") {
        processor.reverse_playback()?;
      }

      if let Some(intensity) = self.parameters.get("intensity") {
        let intensity = intensity
          .parse::<f32>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid intensity value".to_string()))?;
        let preserve_duration = self
          .parameters
          .get("preserve_duration")
          .and_then(|s| s.parse::<bool>().ok())
          .unwrap_or(true);

        processor.data_mosh(intensity, preserve_duration)?;
      }

      *video_processor = processor.clone();
    }

    Ok(())
  }

  fn process_composite(
    &mut self,
    media_processor: &mut MediaProcessor,
    composite_processor: &mut MediaProcessor,
  ) -> Result<()> {
    *media_processor = composite_processor.clone();

    if let Some(overlay_path) = self.parameters.get("overlay") {
      let overlay_processor = MediaProcessor::from_file(overlay_path)?;
      let mode = self
        .parameters
        .get("mode")
        .map(|s| match s.as_str() {
          "add" => ellastic_media::BlendMode::Add,
          "multiply" => ellastic_media::BlendMode::Multiply,
          "screen" => ellastic_media::BlendMode::Screen,
          "overlay" => ellastic_media::BlendMode::Overlay,
          "difference" => ellastic_media::BlendMode::Difference,
          _ => ellastic_media::BlendMode::Overlay,
        })
        .unwrap_or(ellastic_media::BlendMode::Overlay);

      let mix_ratio = self
        .parameters
        .get("mix_ratio")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.5);

      media_processor.blend(&overlay_processor, mode, mix_ratio)?;
    }

    if let Some(composite_path) = self.parameters.get("composite") {
      let composite_processor = MediaProcessor::from_file(composite_path)?;
      media_processor.composite(&composite_processor)?;
    }

    if let Some(mask_path) = self.parameters.get("mask") {
      let mask_processor = MediaProcessor::from_file(mask_path)?;
      media_processor.apply_mask(&mask_processor)?;
    }

    *composite_processor = media_processor.clone();

    Ok(())
  }

  fn process_custom(
    &mut self,
    media_processor: &mut MediaProcessor,
    custom_processor: &CustomProcessor,
  ) -> Result<()> {
    custom_processor.process(media_processor, &self.parameters)
  }

  fn process_media_static(
    media_processor: &mut MediaProcessor,
    processor_type: &ProcessorType,
    parameters: &HashMap<String, String>,
  ) -> Result<()> {
    match processor_type {
      ProcessorType::Image { .. } => {
        if let Some(brightness) = parameters.get("brightness") {
          let amount = brightness
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid brightness value".to_string()))?;
          media_processor.adjust_brightness(amount)?;
        }

        if let Some(contrast) = parameters.get("contrast") {
          let amount = contrast
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid contrast value".to_string()))?;
          media_processor.adjust_contrast(amount)?;
        }

        if let Some(saturation) = parameters.get("saturation") {
          let amount = saturation
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid saturation value".to_string()))?;
          media_processor.adjust_saturation(amount)?;
        }

        if let Some(gamma) = parameters.get("gamma") {
          let gamma = gamma
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid gamma value".to_string()))?;
          media_processor.adjust_gamma(gamma)?;
        }
      }
      ProcessorType::Audio { .. } => {
        if let Some(room_size) = parameters.get("room_size") {
          let room_size = room_size
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid room_size value".to_string()))?;
          media_processor.reverb(room_size)?;
        }

        if let Some(delay) = parameters.get("delay") {
          let delay = delay
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid delay value".to_string()))?;
          media_processor.echo(delay)?;
        }

        if let Some(amount) = parameters.get("amount") {
          let amount = amount
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid amount value".to_string()))?;
          media_processor.distortion(amount)?;
        }
      }
      ProcessorType::Video { .. } => {
        if let Some(count) = parameters.get("count") {
          let count = count
            .parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid count value".to_string()))?;
          media_processor.duplicate_frames(count)?;
        }

        if let Some(ratio) = parameters.get("ratio") {
          let ratio = ratio
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid ratio value".to_string()))?;
          media_processor.time_stretch(ratio)?;
        }

        if parameters.contains_key("reverse_playback") {
          media_processor.reverse_playback()?;
        }
      }
      ProcessorType::Composite { .. } => {
        if let Some(overlay_path) = parameters.get("overlay") {
          let overlay_processor = MediaProcessor::from_file(overlay_path)?;
          let mode = parameters
            .get("mode")
            .map(|s| match s.as_str() {
              "add" => ellastic_media::BlendMode::Add,
              "multiply" => ellastic_media::BlendMode::Multiply,
              "screen" => ellastic_media::BlendMode::Screen,
              "overlay" => ellastic_media::BlendMode::Overlay,
              "difference" => ellastic_media::BlendMode::Difference,
              _ => ellastic_media::BlendMode::Overlay,
            })
            .unwrap_or(ellastic_media::BlendMode::Overlay);

          let mix_ratio = parameters
            .get("mix_ratio")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.5);

          media_processor.blend(&overlay_processor, mode, mix_ratio)?;
        }
      }
      ProcessorType::Custom { custom_processor } => {
        return Err(EllasticError::UnsupportedOperation(
          "Static custom processing not supported".to_string(),
        ));
      }
    }

    Ok(())
  }

  fn apply_pixel_sort(
    &self,
    processor: &mut ImageProcessor,
    threshold: f32,
    mode: crate::effects::PixelSortMode,
  ) -> Result<()> {
    let image_data = processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let channels = image_data.channels as usize;

    let mut pixels = Vec::new();

    for y in 0..height {
      for x in 0..width {
        let pixel_index = (y * width + x) * channels as u32;
        let pixel_start = pixel_index as usize;

        if pixel_start + channels <= data.len() {
          let pixel_data = data[pixel_start..pixel_start + channels].to_vec();
          let brightness = pixel_data.iter().take(3).sum::<u8>() as f32 / 3.0;

          if brightness >= threshold {
            pixels.push((x, y, pixel_data));
          }
        }
      }
    }

    match mode {
      crate::effects::PixelSortMode::Brightness => {
        pixels.sort_by(|_, a, b| {
          let brightness_a = a.2.iter().take(3).sum::<u8>() as f32 / 3.0;
          let brightness_b = b.2.iter().take(3).sum::<u8>() as f32 / 3.0;
          brightness_a
            .partial_cmp(&brightness_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        });
      }
      crate::effects::PixelSortMode::Hue => {
        pixels.sort_by(|_, a, b| {
          let hue_a = self.calculate_hue(&a.2);
          let hue_b = self.calculate_hue(&b.2);
          hue_a
            .partial_cmp(&hue_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        });
      }
      crate::effects::PixelSortMode::Saturation => {
        pixels.sort_by(|_, a, b| {
          let sat_a = self.calculate_saturation(&a.2);
          let sat_b = self.calculate_saturation(&b.2);
          sat_a
            .partial_cmp(&sat_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        });
      }
      crate::effects::PixelSortMode::Random => {
        let mut rng = create_random_generator();
        rng.shuffle(&mut pixels);
      }
    }

    for (x, y, pixel_data) in pixels {
      let pixel_index = (y * width + x) * channels as u32;
      let pixel_start = pixel_index as usize;

      if pixel_start + channels <= data.len() {
        data[pixel_start..pixel_start + channels].copy_from_slice(&pixel_data);
      }
    }

    *processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  fn apply_data_mosh(
    &self,
    processor: &mut ImageProcessor,
    intensity: f32,
    preserve_size: bool,
  ) -> Result<()> {
    let image_data = processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    let corruption_count = (data.len() as f32 * intensity) as usize;
    let mut rng = create_random_generator();

    for _ in 0..corruption_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      if pos < data.len() {
        data[pos] = rng.gen_range(0, 256) as u8;
      }
    }

    *processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  fn calculate_hue(&self, pixel: &[u8]) -> f32 {
    if pixel.len() < 3 {
      return 0.0;
    }

    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta == 0.0 {
      return 0.0;
    }

    let hue = if max == r {
      ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
      ((b - r) / delta + 2.0) / 6.0
    } else {
      ((r - g) / delta + 4.0) / 6.0
    };

    hue
  }

  fn calculate_saturation(&self, pixel: &[u8]) -> f32 {
    if pixel.len() < 3 {
      return 0.0;
    }

    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta == 0.0 {
      return 0.0;
    }

    let lightness = (max + min) / 2.0;

    if lightness == 0.0 {
      return 0.0;
    }

    let saturation = if lightness <= 0.5 {
      delta / (max + min)
    } else {
      delta / (2.0 - max - min)
    };

    saturation
  }

  pub fn clone(&self) -> EffectProcessor {
    EffectProcessor {
      processor_type: self.processor_type.clone(),
      parameters: self.parameters.clone(),
      cache: self.cache.clone(),
      performance_stats: self.performance_stats.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProcessorCache {
  cache: HashMap<String, MediaProcessor>,
  max_size: usize,
}

impl ProcessorCache {
  pub fn new() -> Self {
    Self {
      cache: HashMap::new(),
      max_size: 1000,
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

  pub fn clone(&self) -> ProcessorCache {
    ProcessorCache {
      cache: self.cache.clone(),
      max_size: self.max_size,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProcessorPerformanceStats {
  execution_times: Vec<std::time::Duration>,
  total_executions: u64,
  total_time: std::time::Duration,
  min_time: std::time::Duration,
  max_time: std::time::Duration,
}

impl ProcessorPerformanceStats {
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

    if self.execution_times.len() > 1000 {
      self.execution_times.drain(0..500);
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

  pub fn clone(&self) -> ProcessorPerformanceStats {
    ProcessorPerformanceStats {
      execution_times: self.execution_times.clone(),
      total_executions: self.total_executions,
      total_time: self.total_time,
      min_time: self.min_time,
      max_time: self.max_time,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProcessorRegistry {
  processors: HashMap<Uuid, EffectProcessor>,
  processors_by_name: HashMap<String, Uuid>,
  processors_by_type: HashMap<ProcessorType, Vec<Uuid>>,
}

impl ProcessorRegistry {
  pub fn new() -> Self {
    Self {
      processors: HashMap::new(),
      processors_by_name: HashMap::new(),
      processors_by_type: HashMap::new(),
    }
  }

  pub fn register_processor(&mut self, processor: EffectProcessor) -> Result<()> {
    let id = Uuid::new_v4();

    self.processors.insert(id, processor);
    self
      .processors_by_type
      .insert(processor.processor_type.clone(), Vec::new());

    Ok(())
  }

  pub fn unregister_processor(&mut self, id: Uuid) -> Option<EffectProcessor> {
    if let Some(processor) = self.processors.remove(&id) {
      self.processors_by_type.remove(&processor.processor_type);
      Some(processor)
    } else {
      None
    }
  }

  pub fn get_processor(&self, id: Uuid) -> Option<&EffectProcessor> {
    self.processors.get(&id)
  }

  pub fn list_processors(&self) -> Vec<&EffectProcessor> {
    self.processors.values().collect()
  }

  pub fn list_processors_by_type(&self, processor_type: &ProcessorType) -> Vec<&EffectProcessor> {
    self
      .processors
      .values()
      .filter(|p| {
        std::mem::discriminant(&p.processor_type) == std::mem::discriminant(processor_type)
      })
      .collect()
  }

  pub fn clear(&mut self) {
    self.processors.clear();
    self.processors_by_name.clear();
    self.processors_by_type.clear();
  }

  pub fn len(&self) -> usize {
    self.processors.len()
  }

  pub fn is_empty(&self) -> bool {
    self.processors.is_empty()
  }

  pub fn clone(&self) -> ProcessorRegistry {
    ProcessorRegistry {
      processors: self.processors.clone(),
      processors_by_name: self.processors_by_name.clone(),
      processors_by_type: self.processors_by_type.clone(),
    }
  }
}

pub fn create_effect_processor(processor_type: ProcessorType) -> EffectProcessor {
  EffectProcessor::new(processor_type)
}

pub fn create_image_processor(processor: ImageProcessor) -> ProcessorType {
  ProcessorType::Image {
    image_processor: processor,
  }
}

pub fn create_audio_processor(processor: AudioProcessor) -> ProcessorType {
  ProcessorType::Audio {
    audio_processor: processor,
  }
}

pub fn create_video_processor(processor: ellastic_media::VideoProcessor) -> ProcessorType {
  ProcessorType::Video {
    video_processor: processor,
  }
}

pub fn create_composite_processor(processor: MediaProcessor) -> ProcessorType {
  ProcessorType::Composite {
    composite_processor: processor,
  }
}

pub fn create_custom_processor(
  name: String,
  description: String,
  supported_media_types: Vec<MediaType>,
  process_function: Box<
    dyn Fn(&mut MediaProcessor, &HashMap<String, String>) -> Result<()> + Send + Sync,
  >,
) -> ProcessorType {
  ProcessorType::Custom {
    custom_processor: CustomProcessor::new(
      name,
      description,
      supported_media_types,
      process_function,
    ),
  }
}

pub fn create_processor_cache() -> ProcessorCache {
  ProcessorCache::new()
}

pub fn create_processor_cache_with_max_size(max_size: usize) -> ProcessorCache {
  ProcessorCache::with_max_size(max_size)
}

pub fn create_processor_performance_stats() -> ProcessorPerformanceStats {
  ProcessorPerformanceStats::new()
}

pub fn create_processor_registry() -> ProcessorRegistry {
  ProcessorRegistry::new()
}
