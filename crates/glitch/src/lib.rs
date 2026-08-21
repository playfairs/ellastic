use ellastic_errors::{Result, EllasticError};
use ellastic_core::{MediaData, MediaType};
use ellastic_image::{ImageProcessor, ImageData};
use ellastic_audio::{AudioProcessor, AudioData};
use ellastic_media::{MediaProcessor};
use ellastic_bytes::{ByteBuffer, ByteOrder};
use ellastic_utils::{create_random_generator, create_noise_generator_with_seed, NoiseGenerator};
use rayon::prelude::*;
use std::collections::HashMap;

pub mod image_glitch;
pub mod audio_glitch;
pub mod video_glitch;
pub mod databending;
pub mod effects;
pub mod generators;
pub mod patterns;

pub use image_glitch::*;
pub use audio_glitch::*;
pub use video_glitch::*;
pub use databending::*;
pub use effects::*;
pub use generators::*;
pub use patterns::*;

#[derive(Debug, Clone)]
pub struct GlitchProcessor {
    processor: MediaProcessor,
    random_seed: Option<u64>,
}

impl GlitchProcessor {
    pub fn new(processor: MediaProcessor) -> Self {
        Self {
            processor,
            random_seed: None,
        }
    }

    pub fn processor(&self) -> &MediaProcessor {
        &self.processor
    }

    pub fn processor_mut(&mut self) -> &mut MediaProcessor {
        &mut self.processor
    }

    pub fn into_processor(self) -> MediaProcessor {
        self.processor
    }

    pub fn random_seed(&self) -> Option<u64> {
        self.random_seed
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.random_seed = Some(seed);
    }

    pub fn apply_glitch(&mut self, glitch: &GlitchEffect) -> Result<()> {
        match glitch.effect_type {
            GlitchType::Image { image_glitch } => {
                if let Some(image_processor) = self.processor.as_image_processor() {
                    self.apply_image_glitch(image_processor, image_glitch)
                } else {
                    Err(EllasticError::InvalidParameter("Cannot apply image glitch to non-image media".to_string()))
                }
            }
            GlitchType::Audio { audio_glitch } => {
                if let Some(audio_processor) = self.processor.as_audio_processor() {
                    self.apply_audio_glitch(audio_processor, audio_glitch)
                } else {
                    Err(EllasticError::InvalidParameter("Cannot apply audio glitch to non-audio media".to_string()))
                }
            }
            GlitchType::Video { video_glitch } => {
                if let Some(video_processor) = self.processor.as_video_processor() {
                    self.apply_video_glitch(video_processor, video_glitch)
                } else {
                    Err(EllasticError::InvalidParameter("Cannot apply video glitch to non-video media".to_string()))
                }
            }
            GlitchType::Databending { databending_effect } => {
                self.apply_databending_effect(databending_effect)
            }
            GlitchType::Pattern { pattern_effect } => {
                self.apply_pattern_effect(pattern_effect)
            }
            GlitchType::Custom { custom_effect } => {
                self.apply_custom_effect(custom_effect)
            }
        }
    }

    pub fn apply_glitch_batch(&mut self, glitches: &[GlitchEffect]) -> Result<Vec<MediaProcessor>> {
        let mut results = Vec::new();

        for glitch in glitches {
            let mut temp_processor = self.processor.clone();
            let mut temp_glitcher = GlitchProcessor::new(temp_processor);

            if let Some(seed) = self.random_seed {
                temp_glitcher.set_random_seed(seed);
            }

            temp_glitcher.apply_glitch(glitch)?;
            results.push(temp_glitcher.into_processor());
        }

        Ok(results)
    }

    pub fn apply_glitch_pipeline(&mut self, pipeline: &GlitchPipeline) -> Result<MediaProcessor> {
        let mut current_processor = self.processor.clone();

        for step in &pipeline.steps {
            let mut temp_glitcher = GlitchProcessor::new(current_processor);

            if let Some(seed) = self.random_seed {
                temp_glitcher.set_random_seed(seed);
            }

            current_processor = match step {
                PipelineStep::Glitch { glitch } => {
                    temp_glitcher.apply_glitch(glitch)?;
                    temp_glitcher.into_processor()
                }
                PipelineStep::Blend { blend_mode, mix_ratio } => {
                    self.blend_media(&temp_glitcher.processor(), blend_mode, *mix_ratio)?
                }
                PipelineStep::Transform { transform } => {
                    self.transform_media(&temp_glitcher.processor(), transform)?
                }
                PipelineStep::Filter { filter } => {
                    self.filter_media(&temp_glitcher.processor(), filter)?
                }
            };
        }

        Ok(current_processor)
    }

    fn apply_image_glitch(&mut self, image_processor: &ImageProcessor, image_glitch: &ImageGlitch) -> Result<()> {
        match image_glitch {
            ImageGlitch::PixelSort { threshold, mode } => {
                self.pixel_sort(image_processor, *threshold, *mode)?;
            }
            ImageGlitch::ChannelShift { channel, amount } => {
                self.channel_shift(image_processor, *channel, *amount)?;
            }
            ImageGlitch::SliceAndReorder { slice_size, reorder } => {
                self.slice_and_reorder(image_processor, *slice_size, *reorder)?;
            }
            ImageGlitch::DataMosh { intensity, preserve_size } => {
                self.data_mosh(image_processor, *intensity, *preserve_size)?;
            }
            ImageGlitch::GlitchArt { style, intensity } => {
                self.glitch_art(image_processor, *style, *intensity)?;
            }
            ImageGlitch::ColorCorruption { corruption_type, amount } => {
                self.color_corruption(image_processor, *corruption_type, *amount)?;
            }
            ImageGlitch::GeometricDistortion { distortion_type, strength } => {
                self.geometric_distortion(image_processor, *distortion_type, *strength)?;
            }
            ImageGlitch::CompressionArtifacts { artifact_type, quality } => {
                self.compression_artifacts(image_processor, *artifact_type, *quality)?;
            }
            ImageGlitch::BitManipulation { manipulation_type, bits } => {
                self.bit_manipulation(image_processor, *manipulation_type, *bits)?;
            }
            ImageGlitch::NoiseInjection { noise_type, intensity } => {
                self.noise_injection(image_processor, *noise_type, *intensity)?;
            }
            ImageGlitch::Custom { custom_function } => {
                custom_function(image_processor)?;
            }
        }
        Ok(())
    }

    fn apply_audio_glitch(&mut self, audio_processor: &AudioProcessor, audio_glitch: &AudioGlitch) -> Result<()> {
        match audio_glitch {
            AudioGlitch::SampleCorruption { corruption_type, intensity } => {
                self.sample_corruption(audio_processor, *corruption_type, *intensity)?;
            }
            AudioGlitch::TimeStretch { ratio, preserve_pitch } => {
                self.time_stretch(audio_processor, *ratio, *preserve_pitch)?;
            }
            AudioGlitch::PitchShift { semitones, preserve_duration } => {
                self.pitch_shift(audio_processor, *semitones, *preserve_duration)?;
            }
            AudioGlitch::BitCrush { bit_depth, sample_rate_reduction } => {
                self.bit_crush(audio_processor, *bit_depth, *sample_rate_reduction)?;
            }
            AudioGlitch::GlitchLoop { loop_size, crossfade } => {
                self.glitch_loop(audio_processor, *loop_size, *crossfade)?;
            }
            AudioGlitch::ReverseSegments { segment_length } => {
                self.reverse_segments(audio_processor, *segment_length)?;
            }
            AudioGlitch::Stutter { repeat_count, variation } => {
                self.stutter(audio_processor, *repeat_count, *variation)?;
            }
            AudioGlitch::RingModulation { frequency, mix } => {
                self.ring_modulation(audio_processor, *frequency, *mix)?;
            }
            AudioGlitch::FrequencyModulation { carrier_freq, mod_freq, mod_type } => {
                self.frequency_modulation(audio_processor, *carrier_freq, *mod_freq, *mod_type)?;
            }
            AudioGlitch::PhaseVocoder { bands, carrier_input } => {
                self.phase_vocoder(audio_processor, *bands, carrier_input)?;
            }
            AudioGlitch::DataBending { bend_type, intensity } => {
                self.data_bending(audio_processor, *bend_type, *intensity)?;
            }
            AudioGlitch::Custom { custom_function } => {
                custom_function(audio_processor)?;
            }
        }
        Ok(())
    }

    fn apply_video_glitch(&mut self, video_processor: &crate::VideoProcessor, video_glitch: &VideoGlitch) -> Result<()> {
        match video_glitch {
            VideoGlitch::FrameCorruption { corruption_type, intensity } => {
                self.frame_corruption(video_processor, *corruption_type, *intensity)?;
            }
            VideoGlitch::TimeManipulation { manipulation_type, amount } => {
                self.time_manipulation(video_processor, *manipulation_type, *amount)?;
            }
            VideoGlitch::FrameReordering { reorder_type, segment_size } => {
                self.frame_reordering(video_processor, *reorder_type, *segment_size)?;
            }
            VideoGlitch::CompressionArtifacts { artifact_type, quality } => {
                self.compression_artifacts_video(video_processor, *artifact_type, *quality)?;
            }
            VideoGlitch::DataMosh { intensity, preserve_duration } => {
                self.data_mosh_video(video_processor, *intensity, *preserve_duration)?;
            }
            VideoGlitch::ColorChannelCorruption { channel, corruption_type, amount } => {
                self.color_channel_corruption(video_processor, *channel, *corruption_type, *amount)?;
            }
            VideoGlitch::GeometricDistortion { distortion_type, strength } => {
                self.geometric_distortion_video(video_processor, *distortion_type, *strength)?;
            }
            VideoGlitch::Custom { custom_function } => {
                custom_function(video_processor)?;
            }
        }
        Ok(())
    }

    fn apply_databending_effect(&mut self, databending_effect: &DatabendingEffect) -> Result<()> {
        match databending_effect {
            DatabendingEffect::ByteManipulation { manipulation_type, parameters } => {
                self.byte_manipulation(*manipulation_type, parameters)?;
            }
            DatabendingEffect::DataCorruption { corruption_type, intensity } => {
                self.data_corruption(*corruption_type, *intensity)?;
            }
            DatabendingEffect::FormatBending { source_format, target_format, bend_type } => {
                self.format_bending(*source_format, *target_format, *bend_type)?;
            }
            DatabendingEffect::HeaderCorruption { header_type, corruption_type } => {
                self.header_corruption(*header_type, *corruption_type)?;
            }
            DatabendingEffect::StructuralDamage { damage_type, severity } => {
                self.structural_damage(*damage_type, *severity)?;
            }
            DatabendingEffect::Custom { custom_function } => {
                custom_function(&mut self.processor)?;
            }
        }
        Ok(())
    }

    fn apply_pattern_effect(&mut self, pattern_effect: &PatternEffect) -> Result<()> {
        match pattern_effect {
            PatternEffect::RepeatingPattern { pattern, period, offset } => {
                self.repeating_pattern(*pattern, *period, *offset)?;
            }
            PatternEffect::RandomNoise { noise_type, intensity } => {
                self.random_noise(*noise_type, *intensity)?;
            }
            PatternEffect::GlitchPattern { glitch_type, frequency, intensity } => {
                self.glitch_pattern(*glitch_type, *frequency, *intensity)?;
            }
            PatternEffect::DataPattern { pattern_type, data, intensity } => {
                self.data_pattern(*pattern_type, data, *intensity)?;
            }
            PatternEffect::Custom { custom_pattern } => {
                custom_pattern(&mut self.processor)?;
            }
        }
        Ok(())
    }

    fn apply_custom_effect(&mut self, custom_effect: &CustomEffect) -> Result<()> {
        custom_effect.apply(&mut self.processor)
    }

    fn pixel_sort(&mut self, image_processor: &ImageProcessor, threshold: f32, mode: PixelSortMode) -> Result<()> {
        let image_data = image_processor.data();
        let mut pixels = Vec::new();

        for y in 0..image_data.height {
            for x in 0..image_data.width {
                if let Some(pixel) = image_processor.get_pixel(x, y) {
                    let brightness = pixel.iter().sum::<u8>() as f32 / pixel.len() as f32;
                    if brightness >= threshold {
                        pixels.push((x, y, pixel));
                    }
                }
            }
        }

        match mode {
            PixelSortMode::Brightness => {
                pixels.sort_by(|_, a, b| {
                    let brightness_a = a.2.iter().sum::<u8>() as f32 / a.2.len() as f32;
                    let brightness_b = b.2.iter().sum::<u8>() as f32 / b.2.len() as f32;
                    brightness_a.partial_cmp(&brightness_b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            PixelSortMode::Hue => {
                pixels.sort_by(|_, a, b| {
                    let hue_a = self.calculate_hue(&a.2);
                    let hue_b = self.calculate_hue(&b.2);
                    hue_a.partial_cmp(&hue_b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            PixelSortMode::Saturation => {
                pixels.sort_by(|_, a, b| {
                    let sat_a = self.calculate_saturation(&a.2);
                    let sat_b = self.calculate_saturation(&b.2);
                    sat_a.partial_cmp(&sat_b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            PixelSortMode::Random => {
                let mut rng = create_random_generator();
                rng.shuffle(&mut pixels);
            }
        }

        let mut new_image_data = image_data.clone();
        for (x, y, pixel) in pixels {
            if let Some(_) = new_image_data.set_pixel(x, y, &pixel) {
            }
        }

        Ok(())
    }

    fn channel_shift(&mut self, image_processor: &ImageProcessor, channel: u8, amount: i32) -> Result<()> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();

        for y in 0..image_data.height {
            for x in 0..image_data.width {
                if let Some(mut pixel) = image_processor.get_pixel(x, y) {
                    if channel < pixel.len() as u8 {
                        let shifted_value = pixel[channel as usize] as i32;
                        let shifted_value = (shifted_value + amount).clamp(0, 255) as u8;
                        pixel[channel as usize] = shifted_value;
                    }

                    if let Some(_) = new_image_data.set_pixel(x, y, &pixel) {
                    }
                }
            }
        }

        Ok(())
    }

    fn slice_and_reorder(&mut self, image_processor: &ImageProcessor, slice_size: u32, reorder: ReorderMode) -> Result<()> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();
        let mut slices = Vec::new();

        for y in (0..image_data.height).step_by(slice_size) {
            for x in (0..image_data.width).step_by(slice_size) {
                let slice = image_processor.get_region(x, y, slice_size, slice_size)?;
                slices.push(slice);
            }
        }

        match reorder {
            ReorderMode::Random => {
                let mut rng = create_random_generator();
                rng.shuffle(&mut slices);
            }
            ReorderMode::Reverse => {
                slices.reverse();
            }
            ReorderMode::Rotate => {
                slices.rotate_left(1);
            }
            ReorderMode::Shuffle => {
                rng.shuffle(&mut slices);
            }
        }

        let mut offset_y = 0;
        for slice in slices {
            if let Some(_) = new_image_data.set_region(0, offset_y, slice.width(), slice.height(), &slice.data()) {
                offset_y += slice.height();
            }
        }

        Ok(())
    }

    fn data_mosh(&mut self, image_processor: &ImageProcessor, intensity: f32, preserve_size: bool) -> Result<()> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();
        let data = &image_data.data;

        let corruption_count = (data.len() as f32 * intensity) as usize;
        let mut rng = create_random_generator();

        for _ in 0..corruption_count {
            let pos = rng.gen_range(0, data.len() as u64) as usize;
            if pos < data.len() {
                new_image_data.data[pos] = rng.gen_range(0, 256) as u8;
            }
        }

        Ok(())
    }

    fn glitch_art(&mut self, image_processor: &ImageProcessor, style: GlitchStyle, intensity: f32) -> Result<()> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();

        match style {
            GlitchStyle::Digital => {
                self.digital_glitch(&mut new_image_data, intensity)?;
            }
            GlitchStyle::Analog => {
                self.analog_glitch(&mut new_image_data, intensity)?;
            }
            GlitchStyle::Compression => {
                self.compression_glitch(&mut new_image_data, intensity)?;
            }
            GlitchStyle::DataLoss => {
                self.data_loss_glitch(&mut new_image_data, intensity)?;
            }
            GlitchStyle::ColorShift => {
                self.color_shift_glitch(&mut new_image_data, intensity)?;
            }
        }

        Ok(())
    }

    fn digital_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
        let data = &mut image_data.data;
        let glitch_count = (data.len() as f32 * intensity) as usize;
        let mut rng = create_random_generator();

        for _ in 0..glitch_count {
            let start = rng.gen_range(0, data.len() as u64) as usize;
            let end = (start + 1024).min(data.len());

            if end > start {
                for i in start..end {
                    data[i] = rng.gen_range(0, 256) as u8;
                }
            }
        }

        Ok(())
    }

    fn analog_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
        let data = &mut image_data.data;
        let glitch_count = (data.len() as f32 * intensity) as usize;
        let mut rng = create_random_generator();

        for _ in 0..glitch_count {
            let pos = rng.gen_range(0, data.len() as u64) as usize;
            if pos < data.len() {
                let original = data[pos];
                let noise = rng.gen_range(-50, 50) as i8;
                let corrupted = (original as i16).saturating_add(noise) as u8;
                data[pos] = corrupted;
            }
        }

        Ok(())
    }

    fn compression_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
        let data = &mut image_data.data;
        let chunk_size = (1024.0 * intensity) as usize;
        let mut rng = create_random_generator();

        for chunk in data.chunks_mut(chunk_size) {
            if rng.gen_range(0.0, 1.0) < 0.5 {
                chunk.fill(0);
            }
        }

        Ok(())
    }

    fn data_loss_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
        let data = &mut image_data.data;
        let loss_count = (data.len() as f32 * intensity) as usize;
        let mut rng = create_random_generator();

        for _ in 0..loss_count {
            let pos = rng.gen_range(0, data.len() as u64) as usize;
            if pos < data.len() {
                data[pos] = 0;
            }
        }

        Ok(())
    }

    fn color_shift_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
        let data = &mut image_data.data;
        let shift_count = (data.len() / 4) as usize;
        let mut rng = create_random_generator();

        for _ in 0..shift_count {
            let pos = rng.gen_range(0, data.len() as u64) as usize;
            if pos + 3 < data.len() {
                data[pos..pos + 4].rotate_right(1);
            }
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
            (g - b) / delta
        } else if max == g {
            2.0 + (b - r) / delta
        } else {
            4.0 + (r - g) / delta
        };

        (hue * 360.0 / 6.0).fract() * 360.0
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
            delta / (2.0 - lightness * 2.0)
        } else {
            delta / (lightness * 2.0 - 1.0)
        };

        saturation
    }

    fn blend_media(&mut self, processor: &MediaProcessor, blend_mode: BlendMode, mix_ratio: f32) -> Result<MediaProcessor> {
        match blend_mode {
            BlendMode::Add => self.blend_add(processor, mix_ratio),
            BlendMode::Multiply => self.blend_multiply(processor, mix_ratio),
            BlendMode::Screen => self.blend_screen(processor, mix_ratio),
            BlendMode::Overlay => self.blend_overlay(processor, mix_ratio),
            BlendMode::Difference => self.blend_difference(processor, mix_ratio),
        }
    }

    fn blend_add(&self, processor: &MediaProcessor, mix_ratio: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                for pixel in image_data.data.iter_mut() {
                    *pixel = (*pixel as f32 * mix_ratio + 255.0 * (1.0 - mix_ratio)) as u8;
                }
            }
            MediaData::Audio(audio_data) => {
                for sample in audio_data.samples.iter_mut() {
                    *sample = *sample * mix_ratio + audio_data.samples.iter().sum::<f32>() / audio_data.samples.len() as f32 * (1.0 - mix_ratio);
                }
            }
            _ => {}
        }

        Ok(result)
    }

    fn blend_multiply(&self, processor: &MediaProcessor, mix_ratio: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                for pixel in image_data.data.iter_mut() {
                    *pixel = (*pixel as f32 * mix_ratio) as u8;
                }
            }
            MediaData::Audio(audio_data) => {
                for sample in audio_data.samples.iter_mut() {
                    *sample = *sample * mix_ratio;
                }
            }
            _ => {}
        }

        Ok(result)
    }

    fn blend_screen(&self, processor: &MediaProcessor, mix_ratio: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                for pixel in image_data.data.iter_mut() {
                    let original = *pixel as f32 / 255.0;
                    let blended = 1.0 - (1.0 - original) * (1.0 - mix_ratio);
                    *pixel = (blended * 255.0) as u8;
                }
            }
            MediaData::Audio(audio_data) => {
                for sample in audio_data.samples.iter_mut() {
                    let original = *sample;
                    let blended = 1.0 - (1.0 - original) * (1.0 - mix_ratio);
                    *sample = blended;
                }
            }
            _ => {}
        }

        Ok(result)
    }

    fn blend_overlay(&self, processor: &MediaProcessor, mix_ratio: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                for pixel in image_data.data.iter_mut() {
                    let original = *pixel as f32 / 255.0;
                    let blended = if original < 0.5 {
                        2.0 * original * mix_ratio
                    } else {
                        1.0 - 2.0 * (1.0 - original) * (1.0 - mix_ratio)
                    };
                    *pixel = (blended * 255.0) as u8;
                }
            }
            MediaData::Audio(audio_data) => {
                for sample in audio_data.samples.iter_mut() {
                    let original = *sample;
                    let blended = if original < 0.0 {
                        2.0 * original * mix_ratio
                    } else {
                        1.0 - 2.0 * (1.0 - original) * (1.0 - mix_ratio)
                    };
                    *sample = blended;
                }
            }
            _ => {}
        }

        Ok(result)
    }

    fn blend_difference(&self, processor: &MediaProcessor, mix_ratio: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                for pixel in image_data.data.iter_mut() {
                    let original = *pixel as f32 / 255.0;
                    let difference = (original - 0.5).abs() * 2.0;
                    let blended = original * mix_ratio + difference * (1.0 - mix_ratio);
                    *pixel = (blended * 255.0) as u8;
                }
            }
            MediaData::Audio(audio_data) => {
                for sample in audio_data.samples.iter_mut() {
                    let original = *sample;
                    let difference = (original - audio_data.samples.iter().sum::<f32>() / audio_data.samples.len() as f32).abs();
                    let blended = original * mix_ratio + difference * (1.0 - mix_ratio);
                    *sample = blended;
                }
            }
            _ => {}
        }

        Ok(result)
    }

    fn transform_media(&mut self, processor: &MediaProcessor, transform: &Transform) -> Result<MediaProcessor> {
        match transform {
            Transform::Rotate { angle } => self.rotate_media(processor, *angle),
            Transform::Scale { scale_x, scale_y } => self.scale_media(processor, *scale_x, *scale_y),
            Transform::Flip { direction } => self.flip_media(processor, *direction),
            Transform::Crop { x, y, width, height } => self.crop_media(processor, *x, *y, *width, *height),
            Transform::Distortion { distortion_type, strength } => self.distort_media(processor, *distortion_type, *strength),
        }
    }

    fn rotate_media(&mut self, processor: MediaProcessor, angle: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                let mut image_processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
                let rotated = image_processor.rotate(angle)?;
                *image_data = rotated.into_data();
            }
            MediaData::Audio(audio_data) => {
            }
            _ => {}
        }

        Ok(result)
    }

    fn scale_media(&mut self, processor: MediaProcessor, scale_x: f32, scale_y: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                let new_width = (image_data.width as f32 * scale_x) as u32;
                let new_height = (image_data.height as f32 * scale_y) as u32;
                let mut image_processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
                let scaled = image_processor.resize(new_width, new_height, ellastic_image::ResampleMethod::Linear)?;
                *image_data = scaled.into_data();
            }
            MediaData::Audio(audio_data) => {
            }
            _ => {}
        }

        Ok(result)
    }

    fn flip_media(&mut self, processor: MediaProcessor, direction: FlipDirection) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                let mut image_processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
                let flipped = image_processor.flip(match direction {
                    FlipDirection::Horizontal => ellastic_image::FlipDirection::Horizontal,
                    FlipDirection::Vertical => ellastic_image::FlipDirection::Vertical,
                    FlipDirection::Both => ellastic_image::FlipDirection::Both,
                })?;
                *image_data = flipped.into_data();
            }
            MediaData::Audio(audio_data) => {
            }
            _ => {}
        }

        Ok(result)
    }

    fn crop_media(&mut self, processor: MediaProcessor, x: u32, y: u32, width: u32, height: u32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                let mut image_processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
                let cropped = image_processor.crop(x, y, width, height)?;
                *image_data = cropped.into_data();
            }
            MediaData::Audio(audio_data) => {
            }
            _ => {}
        }

        Ok(result)
    }

    fn distort_media(&mut self, processor: MediaProcessor, distortion_type: DistortionType, strength: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Image(image_data) => {
                let mut image_processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
                let distorted = self.apply_image_distortion(&image_processor, distortion_type, strength)?;
                *image_data = distorted.into_data();
            }
            MediaData::Audio(audio_data) => {
            }
            _ => {}
        }

        Ok(result)
    }

    fn apply_image_distortion(&mut self, image_processor: &ImageProcessor, distortion_type: &DistortionType, strength: f32) -> Result<ellastic_image::ImageProcessor> {
        match distortion_type {
            DistortionType::Wave => self.wave_distortion(image_processor, strength),
            Distortion::Ripple => self.ripple_distortion(image_processor, strength),
            Distortion::Swirl => self.swirl_distortion(image_processor, strength),
            Distortion::Fisheye => self.fisheye_distortion(image_processor, strength),
            Distortion::Pixelate => self.pixelate_distortion(image_processor, strength),
            Distortion::Noise => self.noise_distortion(image_processor, strength),
            Distortion::Custom { custom_function } => custom_function(image_processor),
        }
    }

    fn wave_distortion(&mut self, image_processor: &ellastic_image::ImageProcessor, strength: f32) -> Result<ellastic_image::ImageProcessor> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();
        let data = &mut new_image_data.data;

        for y in 0..image_data.height {
            for x in 0..image_data.width {
                let offset = (strength * 10.0 * ((x as f32 / image_data.width as f32) * 2.0 * std::f32::consts::PI).sin()) as i32;
                let pixel_index = ((y as usize * image_data.width as usize + x as usize) * image_data.channels as usize).min(data.len() - 1);

                if offset != 0 && pixel_index + offset < data.len() {
                    let source_pixel = data[pixel_index];
                    data[pixel_index] = data[pixel_index + offset];
                }
            }
        }

        Ok(ellastic_image::ImageProcessor::from_image_data(new_image_data))
    }

    fn ripple_distortion(&mut self, image_processor: &ellastic_image::ImageProcessor, strength: f32) -> Result<ellastic_image::ImageProcessor> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();
        let data = &mut new_image_data.data;

        for y in 0..image_data.height {
            for x in 0..image_data.width {
                let distance = ((x as f32 - image_data.width as f32 / 2.0).powi(2) +
                                (y as f32 - image_data.height as f32 / 2.0).powi(2)).sqrt();
                let wave_height = (strength * 20.0 * (distance / 100.0)).sin();
                let offset = wave_height as i32;

                let pixel_index = ((y as usize * image_data.width as usize + x as usize) * image_data.channels as usize).min(data.len() - 1);

                if offset != 0 && pixel_index + offset < data.len() {
                    let source_pixel = data[pixel_index];
                    data[pixel_index] = data[pixel_index + offset];
                }
            }
        }

        Ok(ellastic_image::ImageProcessor::from_image_data(new_image_data))
    }

    fn swirl_distortion(&mut self, image_processor: &ellastic_image::ImageProcessor, strength: f32) -> Result<ellastic_image::ImageProcessor> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();
        let data = &mut new_image_data.data;
        let center_x = image_data.width as f32 / 2.0;
        let center_y = image_data.height as f32 / 2.0;

        for y in 0..image_data.height {
            for x in 0..image_data.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                let angle = strength * distance / 10.0;
                let cos_angle = angle.cos();
                let sin_angle = angle.sin();

                let source_x = (center_x + dx * cos_angle - dy * sin_angle) as i32;
                let source_y = (center_y + dx * sin_angle + dy * cos_angle) as i32;

                if source_x >= 0 && source_x < image_data.width as i32 &&
                   source_y >= 0 && source_y < image_data.height as i32 {
                    let source_index = (source_y as usize * image_data.width as usize + source_x as usize) * image_data.channels as usize).min(data.len() - 1);
                    let pixel_index = ((y as usize * image_data.width as usize + x as usize) * image_data.channels as usize).min(data.len() - 1);

                    if source_index < data.len() {
                        data[pixel_index] = data[source_index];
                    }
                }
            }
        }

        Ok(ellastic_image::ImageProcessor::from_image_data(new_image_data))
    }

    fn fisheye_distortion(&mut self, image_processor: &ellastic_image::ImageProcessor, strength: f32) -> Result<ellastic_image::ImageProcessor> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();
        let data = &mut new_image_data.data;
        let center_x = image_data.width as f32 / 2.0;
        let center_y = image_data.height as f32 / 2.0;

        for y in 0..image_data.height {
            for x in 0..image_data.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                let radius = strength * 100.0;

                if distance < radius {
                    let source_x = (center_x + dx / (1.0 + distance / radius)) as i32;
                    let source_y = (center_y + dy / (1.0 + distance / radius)) as i32;

                    if source_x >= 0 && source_x < image_data.width as i32 &&
                       source_y >= 0 && source_y < image_data.height as i32 {
                        let source_index = (source_y as usize * image_data.width as usize + source_x as usize) * image_data.channels as usize).min(data.len() - 1);
                        let pixel_index = ((y as usize * image_data.width as usize + x as usize) * image_data.channels as usize).min(data.len() - 1);

                        if source_index < data.len() {
                            data[pixel_index] = data[source_index];
                        }
                    }
                }
            }
        }

        Ok(ellastic_image::ImageProcessor::from_image_data(new_image_data))
    }

    fn pixelate_distortion(&mut self, image_processor: &ellastic_image::ImageProcessor, strength: f32) -> Result<ellastic_image::ImageProcessor> {
        let image_data = image_processor.data();
        let pixel_size = (1.0 / strength).max(2.0) as u32;
        let mut new_image_data = image_data.clone();
        let data = &mut new_image_data.data;

        for y in (0..image_data.height).step_by(pixel_size) {
            for x in (0..image_data.width).step_by(pixel_size) {
                let pixel_index = (y as usize * image_data.width as usize + x as usize) * image_data.channels as usize).min(data.len() - 1);
                let pixel = data[pixel_index];

                for dy in 0..pixel_size {
                    for dx in 0..pixel_size {
                        let target_index = ((y + dy) as usize * image_data.width as usize + (x + dx) as usize) * image_data.channels as usize).min(data.len() - 1);
                        if target_index < data.len() {
                            data[target_index] = pixel;
                        }
                    }
                }
            }
        }

        Ok(ellastic_image::ImageProcessor::from_image_data(new_image_data))
    }

    fn noishe_distortion(&mut self, image_processor: &ellastic_image::ImageProcessor, strength: f32) -> Result<ellastic_image::ImageProcessor> {
        let image_data = image_processor.data();
        let mut new_image_data = image_data.clone();
        let data = &mut new_image_data.data;
        let mut rng = create_random_generator();

        for pixel in data.iter_mut() {
            let noise = rng.gen_range(-strength, strength) as i8;
            let corrupted = (*pixel as i16).saturating_add(noise) as u8;
            *pixel = corrupted;
        }

        Ok(ellastic_image::ImageProcessor::from_image_data(new_image_data))
    }

    fn filter_media(&mut self, processor: &MediaProcessor, filter: &Filter) -> Result<MediaProcessor> {
        match filter {
            Filter::LowPass { cutoff } => self.low_pass_filter(processor, *cutoff),
            Filter::HighPass { cutoff } => self.high_pass_filter(processor, *cutoff),
            Filter::BandPass { low_cutoff, high_cutoff } => self.band_pass_filter(processor, *low_cutoff, *high_cutoff),
            Filter::Custom { custom_filter } => custom_filter(processor),
        }
    }

    fn low_pass_filter(&mut self, processor: MediaProcessor, cutoff: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Audio(audio_data) => {
                let sample_rate = audio_data.sample_rate;
                let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff / sample_rate as f32);
                let mut prev_sample = 0.0f32;

                for sample in audio_data.samples.iter_mut() {
                    let filtered = *sample * rc + prev_sample * (1.0 - rc);
                    *sample = filtered;
                    prev_sample = *sample;
                }
            }
            _ => {}
        }

        Ok(result)
    }

    fn high_pass_filter(&mut self, processor: MediaProcessor, cutoff: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Audio(audio_data) => {
                let sample_rate = audio_data.sample_rate;
                let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff / sample_rate as f32);
                let mut prev_sample = 0.0f32;

                for sample in audio_data.samples.iter_mut() {
                    let filtered = *sample * (1.0 - rc) + prev_sample * rc;
                    *sample = filtered;
                    prev_sample = *sample;
                }
            }
            _ => {}
        }

        Ok(result)
    }

    fn band_pass_filter(&mut self, processor: MediaProcessor, low_cutoff: f32, high_cutoff: f32) -> Result<MediaProcessor> {
        let mut result = processor.clone();

        match result.data_mut() {
            MediaData::Audio(audio_data) => {
                let sample_rate = audio_data.sample_rate;
                let rc_low = 1.0 / (2.0 * std::f32::consts::PI * low_cutoff / sample_rate as f32);
                let rc_high = 1.0 / (2.0 * std::f32::consts::PI * high_cutoff / sample_rate as f32);
                let mut prev_low_sample = 0.0f32;
                let mut prev_high_sample = 0.0f32;

                for sample in audio_data.samples.iter_mut() {
                    let low_filtered = *sample * rc_low + prev_low_sample * (1.0 - rc_low);
                    let high_filtered = *sample * rc_high + prev_high_sample * (1.0 - rc_high);
                    let band_passed = low_filtered - high_filtered;
                    *sample = band_passed;
                    prev_low_sample = low_filtered;
                    prev_high_sample = high_filtered;
                }
            }
            _ => {}
        }

        Ok(result)
    }

    pub fn clone(&self) -> GlitchProcessor {
        GlitchProcessor {
            processor: self.processor.clone(),
            random_seed: self.random_seed,
        }
    }
}

pub fn create_glitch_processor(processor: MediaProcessor) -> GlitchProcessor {
    GlitchProcessor::new(processor)
}

pub fn create_glitch_effect(effect_type: GlitchType) -> GlitchEffect {
    GlitchEffect {
        effect_type,
        parameters: Vec<GlitchParameter>,
    }
}

pub fn create_glitch_pipeline() -> GlitchPipeline {
    GlitchPipeline {
        steps: Vec::new(),
    }
}
