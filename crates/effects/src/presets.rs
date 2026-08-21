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
pub struct EffectPreset {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: PresetCategory,
    pub effects: Vec<PresetEffect>,
    pub parameters: HashMap<String, String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
    pub author: String,
    pub supported_media_types: Vec<MediaType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetCategory {
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
pub struct PresetEffect {
    pub effect_type: String,
    pub parameters: HashMap<String, String>,
    pub enabled: bool,
    pub weight: f32,
}

impl EffectPreset {
    pub fn new(name: String, description: String, category: PresetCategory) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            category,
            effects: Vec::new(),
            parameters: HashMap::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            author: "Ellastic Team".to_string(),
            supported_media_types: Vec::new(),
        }
    }

    pub fn with_effects(mut self, effects: Vec<PresetEffect>) -> Self {
        self.effects = effects;
        self
    }

    pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
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

    pub fn add_effect(&mut self, effect: PresetEffect) {
        self.effects.push(effect);
        self.update_timestamp();
    }

    pub fn remove_effect(&mut self, index: usize) -> Option<PresetEffect> {
        if index < self.effects.len() {
            let effect = self.effects.remove(index);
            self.update_timestamp();
            Some(effect)
        } else {
            None
        }
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn clone(&self) -> EffectPreset {
        EffectPreset {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category,
            effects: self.effects.clone(),
            parameters: self.parameters.clone(),
            tags: self.tags.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            version: self.version.clone(),
            author: self.author.clone(),
            supported_media_types: self.supported_media_types.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresetProcessor {
    preset: EffectPreset,
    cache: Arc<RwLock<PresetCache>>,
    performance_stats: Arc<RwLock<PresetPerformanceStats>>,
}

impl PresetProcessor {
    pub fn new(preset: EffectPreset) -> Self {
        Self {
            preset,
            cache: Arc::new(RwLock::new(PresetCache::new())),
            performance_stats: Arc::new(RwLock::new(PresetPerformanceStats::new())),
        }
    }

    pub fn preset(&self) -> &EffectPreset {
        &self.preset
    }

    pub fn preset_mut(&mut self) -> &mut EffectPreset {
        &mut self.preset
    }

    pub fn cache(&self) -> Arc<RwLock<PresetCache>> {
        self.cache.clone()
    }

    pub fn performance_stats(&self) -> Arc<RwLock<PresetPerformanceStats>> {
        self.performance_stats.clone()
    }

    pub fn apply_preset(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
        let start_time = std::time::Instant::now();

        self.validate_media_type(media_processor)?;

        let cache_key = self.generate_cache_key(media_processor);

        if let Some(cached_result) = self.cache.read().get(&cache_key) {
            *media_processor = cached_result.clone();
            return Ok(());
        }

        let mut current_processor = media_processor.clone();

        for effect in &self.preset.effects {
            if effect.enabled {
                self.apply_preset_effect(&mut current_processor, effect)?;
            }
        }

        *media_processor = current_processor;

        let processing_time = start_time.elapsed();
        self.performance_stats.write().record_execution(processing_time);

        self.cache.write().put(cache_key, media_processor.clone());

        Ok(())
    }

    pub fn apply_preset_async(&mut self, media_processor: MediaProcessor) -> Result<tokio::task::JoinHandle<Result<MediaProcessor>>> {
        let preset = self.preset.clone();
        let cache = self.cache.clone();
        let performance_stats = self.performance_stats.clone();

        let handle = tokio::spawn(async move {
            let start_time = std::time::Instant::now();

            let mut processor = media_processor;

            let cache_key = Self::generate_cache_key_static(&preset, &processor);

            if let Some(cached_result) = cache.read().get(&cache_key) {
                return Ok(cached_result);
            }

            for effect in &preset.effects {
                if effect.enabled {
                    Self::apply_preset_effect_static(&mut processor, effect)?;
                }
            }

            let processing_time = start_time.elapsed();
            performance_stats.write().record_execution(processing_time);

            cache.write().put(cache_key, processor.clone());

            Ok(processor)
        });

        Ok(handle)
    }

    pub fn preview_preset(&mut self, media_processor: &mut MediaProcessor, preview_size: (u32, u32)) -> Result<()> {
        let original_size = (media_processor.width(), media_processor.height());

        if original_size != preview_size {
            media_processor.resize(preview_size.0, preview_size.1)?;
        }

        self.apply_preset(media_processor)?;

        if original_size != preview_size {
            media_processor.resize(original_size.0, original_size.1)?;
        }

        Ok(())
    }

    pub fn batch_apply_preset(&mut self, media_processors: &mut [MediaProcessor]) -> Result<Vec<Result<()>>> {
        media_processors
            .par_iter_mut()
            .map(|processor| {
                let mut temp_processor = processor.clone();
                let mut temp_preset_processor = PresetProcessor::new(self.preset.clone());

                match temp_preset_processor.apply_preset(&mut temp_processor) {
                    Ok(()) => {
                        *processor = temp_processor;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            })
            .collect()
    }

    pub fn batch_apply_preset_async(&mut self, media_processors: Vec<MediaProcessor>) -> Result<Vec<tokio::task::JoinHandle<Result<MediaProcessor>>>> {
        let preset = self.preset.clone();
        let cache = self.cache.clone();
        let performance_stats = self.performance_stats.clone();

        let handles: Vec<_> = media_processors
            .into_iter()
            .map(|processor| {
                let preset = preset.clone();
                let cache = cache.clone();
                let performance_stats = performance_stats.clone();

                tokio::spawn(async move {
                    let start_time = std::time::Instant::now();

                    let mut temp_processor = processor;

                    let cache_key = Self::generate_cache_key_static(&preset, &temp_processor);

                    if let Some(cached_result) = cache.read().get(&cache_key) {
                        return Ok(cached_result);
                    }

                    for effect in &preset.effects {
                        if effect.enabled {
                            Self::apply_preset_effect_static(&mut temp_processor, effect)?;
                        }
                    }

                    let processing_time = start_time.elapsed();
                    performance_stats.write().record_execution(processing_time);

                    cache.write().put(cache_key, temp_processor.clone());

                    Ok(temp_processor)
                })
            })
            .collect();

        Ok(handles)
    }

    fn validate_media_type(&self, media_processor: &MediaProcessor) -> Result<()> {
        if !self.preset.supported_media_types.contains(&media_processor.media_type()) {
            return Err(EllasticError::InvalidParameter(format!(
                "Preset does not support media type: {:?}",
                media_processor.media_type()
            )));
        }

        Ok(())
    }

    fn generate_cache_key(&self, media_processor: &MediaProcessor) -> String {
        Self::generate_cache_key_static(&self.preset, media_processor)
    }

    fn generate_cache_key_static(preset: &EffectPreset, media_processor: &MediaProcessor) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        preset.id.hash(&mut hasher);

        for effect in &preset.effects {
            if effect.enabled {
                effect.effect_type.hash(&mut hasher);
                effect.weight.to_bits().hash(&mut hasher);

                let mut param_keys: Vec<_> = effect.parameters.keys().collect();
                param_keys.sort();

                for key in param_keys {
                    key.hash(&mut hasher);
                    effect.parameters[key].hash(&mut hasher);
                }
            }
        }

        media_processor.data().hash(&mut hasher);

        format!("{}:{}:{}", preset.id, hasher.finish(), media_processor.data().len())
    }

    fn apply_preset_effect(&mut self, media_processor: &mut MediaProcessor, effect: &PresetEffect) -> Result<()> {
        match effect.effect_type.as_str() {
            "brightness" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid brightness amount".to_string()))?;
                    media_processor.adjust_brightness(amount * effect.weight)?;
                }
            }
            "contrast" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid contrast amount".to_string()))?;
                    media_processor.adjust_contrast(amount * effect.weight)?;
                }
            }
            "saturation" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid saturation amount".to_string()))?;
                    media_processor.adjust_saturation(amount * effect.weight)?;
                }
            }
            "gamma" => {
                if let Some(gamma) = effect.parameters.get("gamma") {
                    let gamma = gamma.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid gamma value".to_string()))?;
                    media_processor.adjust_gamma(gamma)?;
                }
            }
            "blur" => {
                if let Some(radius) = effect.parameters.get("radius") {
                    let radius = radius.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid blur radius".to_string()))?;
                    media_processor.blur(radius * effect.weight)?;
                }
            }
            "sharpen" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid sharpen amount".to_string()))?;
                    media_processor.sharpen(amount * effect.weight)?;
                }
            }
            "edge_detection" => {
                media_processor.edge_detection()?;
            }
            "emboss" => {
                media_processor.emboss()?;
            }
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
                if let Some(angle) = effect.parameters.get("angle") {
                    let angle = angle.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid hue rotate angle".to_string()))?;
                    media_processor.hue_rotate(angle * effect.weight)?;
                }
            }
            "reverb" => {
                if let Some(room_size) = effect.parameters.get("room_size") {
                    let room_size = room_size.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid reverb room_size".to_string()))?;
                    media_processor.reverb(room_size * effect.weight)?;
                }
            }
            "echo" => {
                if let Some(delay) = effect.parameters.get("delay") {
                    let delay = delay.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid echo delay".to_string()))?;
                    media_processor.echo(delay * effect.weight)?;
                }
            }
            "distortion" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid distortion amount".to_string()))?;
                    media_processor.distortion(amount * effect.weight)?;
                }
            }
            "compressor" => {
                if let Some(ratio) = effect.parameters.get("ratio") {
                    let ratio = ratio.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid compressor ratio".to_string()))?;
                    media_processor.compressor(ratio)?;
                }
            }
            "low_pass" => {
                if let Some(cutoff) = effect.parameters.get("cutoff") {
                    let cutoff = cutoff.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid low_pass cutoff".to_string()))?;
                    media_processor.low_pass_filter(cutoff)?;
                }
            }
            "high_pass" => {
                if let Some(cutoff) = effect.parameters.get("cutoff") {
                    let cutoff = cutoff.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid high_pass cutoff".to_string()))?;
                    media_processor.high_pass_filter(cutoff)?;
                }
            }
            "rotate" => {
                if let Some(angle) = effect.parameters.get("angle") {
                    let angle = angle.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid rotate angle".to_string()))?;
                    media_processor.rotate(angle * effect.weight)?;
                }
            }
            "scale" => {
                if let (Some(scale_x), Some(scale_y)) =
                    (effect.parameters.get("scale_x"), effect.parameters.get("scale_y")) {
                    let scale_x = scale_x.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid scale_x".to_string()))?;
                    let scale_y = scale_y.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid scale_y".to_string()))?;
                    media_processor.scale(scale_x * effect.weight, scale_y * effect.weight)?;
                }
            }
            "flip" => {
                if let Some(direction) = effect.parameters.get("direction") {
                    match direction.as_str() {
                        "horizontal" => media_processor.flip_horizontal()?,
                        "vertical" => media_processor.flip_vertical()?,
                        "both" => {
                            media_processor.flip_horizontal()?;
                            media_processor.flip_vertical()?;
                        }
                        _ => {
                            return Err(EllasticError::InvalidParameter("Invalid flip direction".to_string()));
                        }
                    }
                }
            }
            "crop" => {
                if let (Some(x), Some(y), Some(width), Some(height)) =
                    (effect.parameters.get("x"), effect.parameters.get("y"),
                     effect.parameters.get("width"), effect.parameters.get("height")) {
                    let x = x.parse::<u32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid crop x".to_string()))?;
                    let y = y.parse::<u32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid crop y".to_string()))?;
                    let width = width.parse::<u32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid crop width".to_string()))?;
                    let height = height.parse::<u32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid crop height".to_string()))?;
                    media_processor.crop(x, y, width, height)?;
                }
            }
            "pixel_sort" => {
                if let Some(threshold) = effect.parameters.get("threshold") {
                    let threshold = threshold.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid pixel_sort threshold".to_string()))?;
                    let mode = effect.parameters.get("mode")
                        .map(|s| match s.as_str() {
                            "brightness" => crate::effects::PixelSortMode::Brightness,
                            "hue" => crate::effects::PixelSortMode::Hue,
                            "saturation" => crate::effects::PixelSortMode::Saturation,
                            "random" => crate::effects::PixelSortMode::Random,
                            _ => crate::effects::PixelSortMode::Brightness,
                        })
                        .unwrap_or(crate::effects::PixelSortMode::Brightness);

                    self.apply_pixel_sort(media_processor, threshold * effect.weight, mode)?;
                }
            }
            "data_mosh" => {
                if let Some(intensity) = effect.parameters.get("intensity") {
                    let intensity = intensity.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid data_mosh intensity".to_string()))?;
                    let preserve_size = effect.parameters.get("preserve_size")
                        .and_then(|s| s.parse::<bool>().ok())
                        .unwrap_or(true);

                    self.apply_data_mosh(media_processor, intensity * effect.weight, preserve_size)?;
                }
            }
            "bit_crush" => {
                if let Some(bit_depth) = effect.parameters.get("bit_depth") {
                    let bit_depth = bit_depth.parse::<u8>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid bit_crush bit_depth".to_string()))?;
                    let sample_rate_reduction = effect.parameters.get("sample_rate_reduction")
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(1);

                    media_processor.bit_crush(bit_depth, sample_rate_reduction)?;
                }
            }
            "frame_duplication" => {
                if let Some(count) = effect.parameters.get("count") {
                    let count = count.parse::<u32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid frame_duplication count".to_string()))?;
                    media_processor.duplicate_frames(count)?;
                }
            }
            "frame_dropping" => {
                if let Some(count) = effect.parameters.get("count") {
                    let count = count.parse::<u32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid frame_dropping count".to_string()))?;
                    media_processor.drop_frames(count)?;
                }
            }
            "time_stretch" => {
                if let Some(ratio) = effect.parameters.get("ratio") {
                    let ratio = ratio.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid time_stretch ratio".to_string()))?;
                    media_processor.time_stretch(ratio)?;
                }
            }
            "reverse_playback" => {
                media_processor.reverse_playback()?;
            }
            "blend" => {
                if let (Some(overlay_path), Some(mode), Some(mix_ratio)) =
                    (effect.parameters.get("overlay"), effect.parameters.get("mode"), effect.parameters.get("mix_ratio")) {
                    let overlay_processor = MediaProcessor::from_file(overlay_path)?;
                    let blend_mode = match mode.as_str() {
                        "add" => ellastic_media::BlendMode::Add,
                        "multiply" => ellastic_media::BlendMode::Multiply,
                        "screen" => ellastic_media::BlendMode::Screen,
                        "overlay" => ellastic_media::BlendMode::Overlay,
                        "difference" => ellastic_media::BlendMode::Difference,
                        _ => {
                            return Err(EllasticError::InvalidParameter("Invalid blend mode".to_string()));
                        }
                    };

                    let mix_ratio = mix_ratio.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid blend mix_ratio".to_string()))?;

                    media_processor.blend(&overlay_processor, blend_mode, mix_ratio * effect.weight)?;
                }
            }
            "composite" => {
                if let Some(composite_path) = effect.parameters.get("composite") {
                    let composite_processor = MediaProcessor::from_file(composite_path)?;
                    media_processor.composite(&composite_processor)?;
                }
            }
            "mask" => {
                if let Some(mask_path) = effect.parameters.get("mask") {
                    let mask_processor = MediaProcessor::from_file(mask_path)?;
                    media_processor.apply_mask(&mask_processor)?;
                }
            }
            _ => {
                return Err(EllasticError::UnsupportedOperation(format!("Unknown preset effect: {}", effect.effect_type)));
            }
        }

        Ok(())
    }

    fn apply_preset_effect_static(media_processor: &mut MediaProcessor, effect: &PresetEffect) -> Result<()> {
        match effect.effect_type.as_str() {
            "brightness" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid brightness amount".to_string()))?;
                    media_processor.adjust_brightness(amount * effect.weight)?;
                }
            }
            "contrast" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid contrast amount".to_string()))?;
                    media_processor.adjust_contrast(amount * effect.weight)?;
                }
            }
            "saturation" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid saturation amount".to_string()))?;
                    media_processor.adjust_saturation(amount * effect.weight)?;
                }
            }
            "gamma" => {
                if let Some(gamma) = effect.parameters.get("gamma") {
                    let gamma = gamma.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid gamma value".to_string()))?;
                    media_processor.adjust_gamma(gamma)?;
                }
            }
            "blur" => {
                if let Some(radius) = effect.parameters.get("radius") {
                    let radius = radius.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid blur radius".to_string()))?;
                    media_processor.blur(radius * effect.weight)?;
                }
            }
            "sharpen" => {
                if let Some(amount) = effect.parameters.get("amount") {
                    let amount = amount.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid sharpen amount".to_string()))?;
                    media_processor.sharpen(amount * effect.weight)?;
                }
            }
            "grayscale" => {
                media_processor.grayscale()?;
            }
            "sepia" => {
                media_processor.sepia()?;
            }
            "invert" => {
                media_processor.invert()?;
            }
            "reverb" => {
                if let Some(room_size) = effect.parameters.get("room_size") {
                    let room_size = room_size.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid reverb room_size".to_string()))?;
                    media_processor.reverb(room_size * effect.weight)?;
                }
            }
            "echo" => {
                if let Some(delay) = effect.parameters.get("delay") {
                    let delay = delay.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid echo delay".to_string()))?;
                    media_processor.echo(delay * effect.weight)?;
                }
            }
            "rotate" => {
                if let Some(angle) = effect.parameters.get("angle") {
                    let angle = angle.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid rotate angle".to_string()))?;
                    media_processor.rotate(angle * effect.weight)?;
                }
            }
            "scale" => {
                if let (Some(scale_x), Some(scale_y)) =
                    (effect.parameters.get("scale_x"), effect.parameters.get("scale_y")) {
                    let scale_x = scale_x.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid scale_x".to_string()))?;
                    let scale_y = scale_y.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid scale_y".to_string()))?;
                    media_processor.scale(scale_x * effect.weight, scale_y * effect.weight)?;
                }
            }
            "flip" => {
                if let Some(direction) = effect.parameters.get("direction") {
                    match direction.as_str() {
                        "horizontal" => media_processor.flip_horizontal()?,
                        "vertical" => media_processor.flip_vertical()?,
                        "both" => {
                            media_processor.flip_horizontal()?;
                            media_processor.flip_vertical()?;
                        }
                        _ => {
                            return Err(EllasticError::InvalidParameter("Invalid flip direction".to_string()));
                        }
                    }
                }
            }
            "blend" => {
                if let (Some(overlay_path), Some(mode), Some(mix_ratio)) =
                    (effect.parameters.get("overlay"), effect.parameters.get("mode"), effect.parameters.get("mix_ratio")) {
                    let overlay_processor = MediaProcessor::from_file(overlay_path)?;
                    let blend_mode = match mode.as_str() {
                        "add" => ellastic_media::BlendMode::Add,
                        "multiply" => ellastic_media::BlendMode::Multiply,
                        "screen" => ellastic_media::BlendMode::Screen,
                        "overlay" => ellastic_media::BlendMode::Overlay,
                        "difference" => ellastic_media::BlendMode::Difference,
                        _ => {
                            return Err(EllasticError::InvalidParameter("Invalid blend mode".to_string()));
                        }
                    };

                    let mix_ratio = mix_ratio.parse::<f32>()
                        .map_err(|_| EllasticError::InvalidParameter("Invalid blend mix_ratio".to_string()))?;

                    media_processor.blend(&overlay_processor, blend_mode, mix_ratio * effect.weight)?;
                }
            }
            _ => {
                return Err(EllasticError::UnsupportedOperation(format!("Unknown preset effect: {}", effect.effect_type)));
            }
        }

        Ok(())
    }

    fn apply_pixel_sort(&self, media_processor: &mut MediaProcessor, threshold: f32, mode: crate::effects::PixelSortMode) -> Result<()> {
        if let Some(image_processor) = media_processor.image_processor_mut() {
            let image_data = image_processor.data();
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
                        brightness_a.partial_cmp(&brightness_b).unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                crate::effects::PixelSortMode::Hue => {
                    pixels.sort_by(|_, a, b| {
                        let hue_a = self.calculate_hue(&a.2);
                        let hue_b = self.calculate_hue(&b.2);
                        hue_a.partial_cmp(&hue_b).unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                crate::effects::PixelSortMode::Saturation => {
                    pixels.sort_by(|_, a, b| {
                        let sat_a = self.calculate_saturation(&a.2);
                        let sat_b = self.calculate_saturation(&b.2);
                        sat_a.partial_cmp(&sat_b).unwrap_or(std::cmp::Ordering::Equal)
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

            *image_processor = ImageProcessor::from_image_data(new_image_data);
        }

        Ok(())
    }

    fn apply_data_mosh(&self, media_processor: &mut MediaProcessor, intensity: f32, preserve_size: bool) -> Result<()> {
        if let Some(image_processor) = media_processor.image_processor_mut() {
            let image_data = image_processor.data();
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

            *image_processor = ImageProcessor::from_image_data(new_image_data);
        }

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

    pub fn clone(&self) -> PresetProcessor {
        PresetProcessor {
            preset: self.preset.clone(),
            cache: self.cache.clone(),
            performance_stats: self.performance_stats.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresetCache {
    cache: HashMap<String, MediaProcessor>,
    max_size: usize,
}

impl PresetCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 500,
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
Remove oldest entry (simple LRU simulation)
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

    pub fn clone(&self) -> PresetCache {
        PresetCache {
            cache: self.cache.clone(),
            max_size: self.max_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresetPerformanceStats {
    execution_times: Vec<std::time::Duration>,
    total_executions: u64,
    total_time: std::time::Duration,
    min_time: std::time::Duration,
    max_time: std::time::Duration,
}

impl PresetPerformanceStats {
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

        if self.execution_times.len() > 500 {
            self.execution_times.drain(0..250);
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

    pub fn clone(&self) -> PresetPerformanceStats {
        PresetPerformanceStats {
            execution_times: self.execution_times.clone(),
            total_executions: self.total_executions,
            total_time: self.total_time,
            min_time: self.min_time,
            max_time: self.max_time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresetLibrary {
    presets: HashMap<Uuid, EffectPreset>,
    presets_by_name: HashMap<String, Uuid>,
    presets_by_category: HashMap<PresetCategory, Vec<Uuid>>,
}

impl PresetLibrary {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
            presets_by_name: HashMap::new(),
            presets_by_category: HashMap::new(),
        }
    }

    pub fn add_preset(&mut self, preset: EffectPreset) -> Result<()> {
        if self.presets_by_name.contains_key(&preset.name) {
            return Err(EllasticError::AlreadyExists(format!("Preset '{}' already exists", preset.name)));
        }

        let id = preset.id;
        self.presets_by_name.insert(preset.name.clone(), id);

        self.presets_by_category
            .entry(preset.category.clone())
            .or_insert_with(Vec::new)
            .push(id);

        self.presets.insert(id, preset);

        Ok(())
    }

    pub fn remove_preset(&mut self, id: Uuid) -> Option<EffectPreset> {
        if let Some(preset) = self.presets.remove(&id) {
            self.presets_by_name.remove(&preset.name);

            if let Some(presets) = self.presets_by_category.get_mut(&preset.category) {
                presets.retain(|&preset_id| preset_id != id);
            }

            Some(preset)
        } else {
            None
        }
    }

    pub fn get_preset(&self, id: Uuid) -> Option<&EffectPreset> {
        self.presets.get(&id)
    }

    pub fn get_preset_by_name(&self, name: &str) -> Option<&EffectPreset> {
        self.presets_by_name.get(name).and_then(|id| self.presets.get(id))
    }

    pub fn list_presets(&self) -> Vec<&EffectPreset> {
        self.presets.values().collect()
    }

    pub fn list_presets_by_category(&self, category: PresetCategory) -> Vec<&EffectPreset> {
        self.presets_by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.presets.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn search_presets(&self, query: &str) -> Vec<&EffectPreset> {
        let query = query.to_lowercase();
        self.presets
            .values()
            .filter(|preset| {
                preset.name.to_lowercase().contains(&query) ||
                preset.description.to_lowercase().contains(&query) ||
                preset.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.presets.clear();
        self.presets_by_name.clear();
        self.presets_by_category.clear();
    }

    pub fn len(&self) -> usize {
        self.presets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.presets.is_empty()
    }

    pub fn clone(&self) -> PresetLibrary {
        PresetLibrary {
            presets: self.presets.clone(),
            presets_by_name: self.presets_by_name.clone(),
            presets_by_category: self.presets_by_category.clone(),
        }
    }

    pub fn load_default_presets(&mut self) -> Result<()> {
        self.load_basic_presets()?;
        self.load_glitch_presets()?;
        self.load_filter_presets()?;
        self.load_transform_presets()?;
        self.load_color_presets()?;
        self.load_audio_presets()?;
        self.load_video_presets()?;
        self.load_composite_presets()?;

        Ok(())
    }

    fn load_basic_presets(&mut self) -> Result<()> {
        let presets = vec![
            EffectPreset::new("vintage".to_string(), "Vintage photo effect".to_string(), PresetCategory::Basic)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "sepia".to_string(),
                        parameters: HashMap::new(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "brightness".to_string(),
                        parameters: [("amount".to_string(), "-0.1".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.8,
                    },
                    PresetEffect {
                        effect_type: "contrast".to_string(),
                        parameters: [("amount".to_string(), "0.2".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.6,
                    },
                ])
                .with_tags(vec!["photo".to_string(), "vintage".to_string(), "sepia".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),

            EffectPreset::new("dramatic".to_string(), "Dramatic contrast enhancement".to_string(), PresetCategory::Basic)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "contrast".to_string(),
                        parameters: [("amount".to_string(), "0.5".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "saturation".to_string(),
                        parameters: [("amount".to_string(), "0.3".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.7,
                    },
                    PresetEffect {
                        effect_type: "brightness".to_string(),
                        parameters: [("amount".to_string(), "0.1".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.5,
                    },
                ])
                .with_tags(vec!["photo".to_string(), "dramatic".to_string(), "contrast".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),

            EffectPreset::new("soft".to_string(), "Soft dreamy effect".to_string(), PresetCategory::Basic)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "blur".to_string(),
                        parameters: [("radius".to_string(), "2.0".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.8,
                    },
                    PresetEffect {
                        effect_type: "brightness".to_string(),
                        parameters: [("amount".to_string(), "0.2".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.6,
                    },
                    PresetEffect {
                        effect_type: "saturation".to_string(),
                        parameters: [("amount".to_string(), "-0.1".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.4,
                    },
                ])
                .with_tags(vec!["photo".to_string(), "soft".to_string(), "dreamy".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),
        ];

        for preset in presets {
            self.add_preset(preset)?;
        }

        Ok(())
    }

    fn load_glitch_presets(&mut self) -> Result<()> {
        let presets = vec![
            EffectPreset::new("digital_glitch".to_string(), "Digital glitch effect".to_string(), PresetCategory::Glitch)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "pixel_sort".to_string(),
                        parameters: [("threshold".to_string(), "0.5".to_string()), ("mode".to_string(), "brightness".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "data_mosh".to_string(),
                        parameters: [("intensity".to_string(), "0.3".to_string()), ("preserve_size".to_string(), "true".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.7,
                    },
                    PresetEffect {
                        effect_type: "hue_rotate".to_string(),
                        parameters: [("angle".to_string(), "15.0".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.5,
                    },
                ])
                .with_tags(vec!["glitch".to_string(), "digital".to_string(), "pixel_sort".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),

            EffectPreset::new("audio_bit_crush".to_string(), "Audio bit crushing effect".to_string(), PresetCategory::Glitch)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "bit_crush".to_string(),
                        parameters: [("bit_depth".to_string(), "6".to_string()), ("sample_rate_reduction".to_string(), "2".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "distortion".to_string(),
                        parameters: [("amount".to_string(), "0.5".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.6,
                    },
                    PresetEffect {
                        effect_type: "low_pass".to_string(),
                        parameters: [("cutoff".to_string(), "8000.0".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.4,
                    },
                ])
                .with_tags(vec!["glitch".to_string(), "audio".to_string(), "bit_crush".to_string()])
                .with_supported_media_types(vec![MediaType::Audio]),
        ];

        for preset in presets {
            self.add_preset(preset)?;
        }

        Ok(())
    }

    fn load_filter_presets(&mut self) -> Result<()> {
        let presets = vec![
            EffectPreset::new("blur_background".to_string(), "Blur background while keeping subject sharp".to_string(), PresetCategory::Filter)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "blur".to_string(),
                        parameters: [("radius".to_string(), "5.0".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "sharpen".to_string(),
                        parameters: [("amount".to_string(), "0.8".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.6,
                    },
                ])
                .with_tags(vec!["filter".to_string(), "blur".to_string(), "background".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),

            EffectPreset::new("edge_enhance".to_string(), "Enhance edges in the image".to_string(), PresetCategory::Filter)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "edge_detection".to_string(),
                        parameters: HashMap::new(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "sharpen".to_string(),
                        parameters: [("amount".to_string(), "0.5".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.7,
                    },
                    PresetEffect {
                        effect_type: "contrast".to_string(),
                        parameters: [("amount".to_string(), "0.2".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.5,
                    },
                ])
                .with_tags(vec!["filter".to_string(), "edge".to_string(), "enhance".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),
        ];

        for preset in presets {
            self.add_preset(preset)?;
        }

        Ok(())
    }

    fn load_transform_presets(&mut self) -> Result<()> {
        let presets = vec![
            EffectPreset::new("rotate_90".to_string(), "Rotate image 90 degrees clockwise".to_string(), PresetCategory::Transform)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "rotate".to_string(),
                        parameters: [("angle".to_string(), "90.0".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                ])
                .with_tags(vec!["transform".to_string(), "rotate".to_string(), "90_degrees".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),

            EffectPreset::new("flip_horizontal".to_string(), "Flip image horizontally".to_string(), PresetCategory::Transform)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "flip".to_string(),
                        parameters: [("direction".to_string(), "horizontal".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                ])
                .with_tags(vec!["transform".to_string(), "flip".to_string(), "horizontal".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),
        ];

        for preset in presets {
            self.add_preset(preset)?;
        }

        Ok(())
    }

    fn load_color_presets(&mut self) -> Result<()> {
        let presets = vec![
            EffectPreset::new("black_and_white".to_string(), "Convert to black and white".to_string(), PresetCategory::Color)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "grayscale".to_string(),
                        parameters: HashMap::new(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "contrast".to_string(),
                        parameters: [("amount".to_string(), "0.2".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.6,
                    },
                ])
                .with_tags(vec!["color".to_string(), "black_and_white".to_string(), "grayscale".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),

            EffectPreset::new("color_shift".to_string(), "Shift colors for artistic effect".to_string(), PresetCategory::Color)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "hue_rotate".to_string(),
                        parameters: [("angle".to_string(), "30.0".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "saturation".to_string(),
                        parameters: [("amount".to_string(), "0.3".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.7,
                    },
                ])
                .with_tags(vec!["color".to_string(), "hue".to_string(), "artistic".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),
        ];

        for preset in presets {
            self.add_preset(preset)?;
        }

        Ok(())
    }

    fn load_audio_presets(&mut self) -> Result<()> {
        let presets = vec![
            EffectPreset::new("reverb_hall".to_string(), "Hall reverb effect".to_string(), PresetCategory::Audio)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "reverb".to_string(),
                        parameters: [("room_size".to_string(), "0.8".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "low_pass".to_string(),
                        parameters: [("cutoff".to_string(), "12000.0".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.5,
                    },
                ])
                .with_tags(vec!["audio".to_string(), "reverb".to_string(), "hall".to_string()])
                .with_supported_media_types(vec![MediaType::Audio]),

            EffectPreset::new("echo_delay".to_string(), "Echo delay effect".to_string(), PresetCategory::Audio)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "echo".to_string(),
                        parameters: [("delay".to_string(), "0.3".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                    PresetEffect {
                        effect_type: "compressor".to_string(),
                        parameters: [("ratio".to_string(), "3.0".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 0.6,
                    },
                ])
                .with_tags(vec!["audio".to_string(), "echo".to_string(), "delay".to_string()])
                .with_supported_media_types(vec![MediaType::Audio]),
        ];

        for preset in presets {
            self.add_preset(preset)?;
        }

        Ok(())
    }

    fn load_video_presets(&mut self) -> Result<()> {
        let presets = vec![
            EffectPreset::new("slow_motion".to_string(), "Slow motion effect".to_string(), PresetCategory::Video)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "time_stretch".to_string(),
                        parameters: [("ratio".to_string(), "0.5".to_string())].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                ])
                .with_tags(vec!["video".to_string(), "slow_motion".to_string(), "time_stretch".to_string()])
                .with_supported_media_types(vec![MediaType::Video]),

            EffectPreset::new("reverse_video".to_string(), "Reverse video playback".to_string(), PresetCategory::Video)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "reverse_playback".to_string(),
                        parameters: HashMap::new(),
                        enabled: true,
                        weight: 1.0,
                    },
                ])
                .with_tags(vec!["video".to_string(), "reverse".to_string(), "playback".to_string()])
                .with_supported_media_types(vec![MediaType::Video]),
        ];

        for preset in presets {
            self.add_preset(preset)?;
        }

        Ok(())
    }

    fn load_composite_presets(&mut self) -> Result<()> {
        let presets = vec![
            EffectPreset::new("overlay_blend".to_string(), "Overlay blend effect".to_string(), PresetCategory::Composite)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "blend".to_string(),
                        parameters: [
                            ("overlay".to_string(), "".to_string()),
                            ("mode".to_string(), "overlay".to_string()),
                            ("mix_ratio".to_string(), "0.7".to_string()),
                        ].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                ])
                .with_tags(vec!["composite".to_string(), "blend".to_string(), "overlay".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),

            EffectPreset::new("multiply_blend".to_string(), "Multiply blend effect".to_string(), PresetCategory::Composite)
                .with_effects(vec![
                    PresetEffect {
                        effect_type: "blend".to_string(),
                        parameters: [
                            ("overlay".to_string(), "".to_string()),
                            ("mode".to_string(), "multiply".to_string()),
                            ("mix_ratio".to_string(), "0.5".to_string()),
                        ].into_iter().collect(),
                        enabled: true,
                        weight: 1.0,
                    },
                ])
                .with_tags(vec!["composite".to_string(), "blend".to_string(), "multiply".to_string()])
                .with_supported_media_types(vec![MediaType::Image]),
        ];

        for preset in presets {
            self.add_preset(preset)?;
        }

        Ok(())
    }
}

pub fn create_preset_processor(preset: EffectPreset) -> PresetProcessor {
    PresetProcessor::new(preset)
}

pub fn create_effect_preset(name: String, description: String, category: PresetCategory) -> EffectPreset {
    EffectPreset::new(name, description, category)
}

pub fn create_preset_effect(effect_type: String, parameters: HashMap<String, String>) -> PresetEffect {
    PresetEffect {
        effect_type,
        parameters,
        enabled: true,
        weight: 1.0,
    }
}

pub fn create_preset_cache() -> PresetCache {
    PresetCache::new()
}

pub fn create_preset_cache_with_max_size(max_size: usize) -> PresetCache {
    PresetCache::with_max_size(max_size)
}

pub fn create_preset_performance_stats() -> PresetPerformanceStats {
    PresetPerformanceStats::new()
}

pub fn create_preset_library() -> PresetLibrary {
    PresetLibrary::new()
}
