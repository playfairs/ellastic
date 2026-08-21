use ellastic_errors::{Result, EllasticError};
use ellastic_utils::{create_random_generator, create_noise_generator_with_seed, NoiseGenerator};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum GlitchGenerator {
    Image { image_generator: ImageGlitchGenerator },
    Audio { audio_generator: AudioGlitchGenerator },
    Video { video_generator: VideoGlitchGenerator },
    Data { data_generator: DataGlitchGenerator },
    Pattern { pattern_generator: PatternGlitchGenerator },
    Custom { custom_generator: CustomGlitchGenerator },
}

#[derive(Debug, Clone)]
pub struct ImageGlitchGenerator {
    width: u32,
    height: u32,
    channels: u8,
    random_seed: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct AudioGlitchGenerator {
    sample_rate: u32,
    channels: u8,
    duration: f32,
    random_seed: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct VideoGlitchGenerator {
    width: u32,
    height: u32,
    fps: f32,
    frame_count: u32,
    random_seed: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DataGlitchGenerator {
    data_size: usize,
    random_seed: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PatternGlitchGenerator {
    pattern_size: usize,
    repeat_count: usize,
    random_seed: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CustomGlitchGenerator {
    name: String,
    parameters: HashMap<String, String>,
    generate_function: Box<dyn Fn(&HashMap<String, String>) -> Result<Vec<u8>> + Send + Sync>,
}

impl CustomGlitchGenerator {
    pub fn new(name: String, parameters: HashMap<String, String>, generate_function: Box<dyn Fn(&HashMap<String, String>) -> Result<Vec<u8>> + Send + Sync>) -> Self {
        Self { name, parameters, generate_function }
    }

    pub fn generate(&self) -> Result<Vec<u8>> {
        (self.generate_function)(&self.parameters)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageGlitchType {
    PixelSort,
    ChannelShift,
    SliceReorder,
    DataMosh,
    GlitchArt,
    ColorCorruption,
    GeometricDistortion,
    CompressionArtifacts,
    BitManipulation,
    NoiseInjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioGlitchType {
    SampleCorruption,
    TimeStretch,
    PitchShift,
    BitCrush,
    GlitchLoop,
    ReverseSegments,
    Stutter,
    RingModulation,
    FrequencyModulation,
    PhaseVocoder,
    DataBending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoGlitchType {
    FrameCorruption,
    TimeManipulation,
    FrameReordering,
    CompressionArtifacts,
    DataMosh,
    ColorChannelCorruption,
    GeometricDistortion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataGlitchType {
    ByteManipulation,
    DataCorruption,
    FormatBending,
    HeaderCorruption,
    StructuralDamage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternGlitchType {
    RepeatingPattern,
    RandomNoise,
    GlitchPattern,
    DataPattern,
}

impl ImageGlitchGenerator {
    pub fn new(width: u32, height: u32, channels: u8) -> Self {
        Self {
            width,
            height,
            channels,
            random_seed: None,
        }
    }

    pub fn with_seed(width: u32, height: u32, channels: u8, seed: u64) -> Self {
        Self {
            width,
            height,
            channels,
            random_seed: Some(seed),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn random_seed(&self) -> Option<u64> {
        self.random_seed
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.random_seed = Some(seed);
    }

    pub fn generate_glitch(&self, glitch_type: ImageGlitchType, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        let data_size = (self.width * self.height * self.channels as u32) as usize;
        let mut data = vec![0u8; data_size];
        let mut rng = create_random_generator();

        if let Some(seed) = self.random_seed {
            rng.set_seed(seed);
        }

        match glitch_type {
            ImageGlitchType::PixelSort => {
                self.generate_pixel_sort(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::ChannelShift => {
                self.generate_channel_shift(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::SliceReorder => {
                self.generate_slice_reorder(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::DataMosh => {
                self.generate_data_mosh(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::GlitchArt => {
                self.generate_glitch_art(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::ColorCorruption => {
                self.generate_color_corruption(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::GeometricDistortion => {
                self.generate_geometric_distortion(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::CompressionArtifacts => {
                self.generate_compression_artifacts(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::BitManipulation => {
                self.generate_bit_manipulation(&mut data, parameters, &mut rng)?;
            }
            ImageGlitchType::NoiseInjection => {
                self.generate_noise_injection(&mut data, parameters, &mut rng)?;
            }
        }

        Ok(data)
    }

    fn generate_pixel_sort(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let threshold = self.parse_parameter(parameters, "threshold", "0.5")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid threshold".to_string()))?;

        let pixel_size = self.channels as usize;
        let mut pixels = Vec::new();

        for i in (0..data.len()).step_by(pixel_size) {
            if i + pixel_size <= data.len() {
                let brightness = data[i..i + pixel_size].iter().sum::<u8>() as f32 / pixel_size as f32;
                if brightness >= threshold {
                    pixels.push((i, data[i..i + pixel_size].to_vec()));
                }
            }
        }

        pixels.sort_by(|_, a, b| {
            let brightness_a = a.1.iter().sum::<u8>() as f32 / a.1.len() as f32;
            let brightness_b = b.1.iter().sum::<u8>() as f32 / b.1.len() as f32;
            brightness_a.partial_cmp(&brightness_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, pixel_data) in pixels {
            if i + pixel_size <= data.len() {
                data[i..i + pixel_size].copy_from_slice(&pixel_data);
            }
        }

        Ok(())
    }

    fn generate_channel_shift(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let channel = self.parse_parameter(parameters, "channel", "0")?.parse::<u8>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid channel".to_string()))?;
        let amount = self.parse_parameter(parameters, "amount", "10")?.parse::<i32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid amount".to_string()))?;

        let pixel_size = self.channels as usize;

        for i in (0..data.len()).step_by(pixel_size) {
            if i + pixel_size <= data.len() && channel < pixel_size as u8 {
                let channel_index = i + channel as usize;
                data[channel_index] = data[channel_index].wrapping_add(amount as u8);
            }
        }

        Ok(())
    }

    fn generate_slice_reorder(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let slice_size = self.parse_parameter(parameters, "slice_size", "1024")?.parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid slice size".to_string()))?;

        for chunk in data.chunks_mut(slice_size as usize) {
            rng.shuffle(chunk);
        }

        Ok(())
    }

    fn generate_data_mosh(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let intensity = self.parse_parameter(parameters, "intensity", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        let corruption_count = (data.len() as f32 * intensity) as usize;

        for _ in 0..corruption_count {
            let pos = rng.gen_range(0, data.len() as u64) as usize;
            if pos < data.len() {
                data[pos] = rng.gen_range(0, 256) as u8;
            }
        }

        Ok(())
    }

    fn generate_glitch_art(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let style = self.parse_parameter(parameters, "style", "digital")?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.3")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        match style.as_str() {
            "digital" => {
                let glitch_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..glitch_count {
                    let start = rng.gen_range(0, data.len() as u64) as usize;
                    let end = (start + 1024).min(data.len());
                    for i in start..end {
                        data[i] = rng.gen_range(0, 256) as u8;
                    }
                }
            }
            "analog" => {
                let glitch_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..glitch_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        let noise = rng.gen_range(-50, 50) as i8;
                        data[pos] = data[pos].wrapping_add(noise as u8);
                    }
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid glitch style".to_string()));
            }
        }

        Ok(())
    }

    fn generate_color_corruption(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let corruption_type = self.parse_parameter(parameters, "corruption_type", "channel_swap")?;
        let amount = self.parse_parameter(parameters, "amount", "0.5")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid amount".to_string()))?;

        let pixel_size = self.channels as usize;

        match corruption_type.as_str() {
            "channel_swap" => {
                for chunk in data.chunks_mut(pixel_size) {
                    if chunk.len() >= 3 {
                        chunk.swap(0, 1);
                        chunk.swap(1, 2);
                    }
                }
            }
            "color_shift" => {
                let shift_amount = (amount * 255.0) as i32;
                for byte in data.iter_mut() {
                    *byte = (*byte as i32 + shift_amount).clamp(0, 255) as u8;
                }
            }
            "inversion" => {
                for byte in data.iter_mut() {
                    *byte = 255 - *byte;
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid corruption type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_geometric_distortion(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let distortion_type = self.parse_parameter(parameters, "distortion_type", "wave")?;
        let strength = self.parse_parameter(parameters, "strength", "0.3")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid strength".to_string()))?;

        match distortion_type.as_str() {
            "wave" => {
                for y in 0..self.height {
                    for x in 0..self.width {
                        let offset = (strength * 10.0 * ((x as f32 / self.width as f32) * 2.0 * std::f32::consts::PI).sin()) as i32;
                        let pixel_index = ((y as usize * self.width as usize + x as usize) * self.channels as usize).min(data.len() - 1);

                        if offset != 0 && pixel_index + offset < data.len() {
                            let source_pixel = data[pixel_index];
                            data[pixel_index] = data[pixel_index + offset];
                        }
                    }
                }
            }
            "swirl" => {
                let center_x = self.width as f32 / 2.0;
                let center_y = self.height as f32 / 2.0;

                for y in 0..self.height {
                    for x in 0..self.width {
                        let dx = x as f32 - center_x;
                        let dy = y as f32 - center_y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        let angle = strength * distance / 10.0;
                        let cos_angle = angle.cos();
                        let sin_angle = angle.sin();

                        let source_x = (center_x + dx * cos_angle - dy * sin_angle) as i32;
                        let source_y = (center_y + dx * sin_angle + dy * cos_angle) as i32;

                        if source_x >= 0 && source_x < self.width as i32 &&
                           source_y >= 0 && source_y < self.height as i32 {
                            let source_index = (source_y as usize * self.width as usize + source_x as usize) * self.channels as usize;
                            let pixel_index = (y as usize * self.width as usize + x as usize) * self.channels as usize;

                            if source_index < data.len() && pixel_index < data.len() {
                                data[pixel_index] = data[source_index];
                            }
                        }
                    }
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid distortion type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_compression_artifacts(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let artifact_type = self.parse_parameter(parameters, "artifact_type", "jpeg")?;
        let quality = self.parse_parameter(parameters, "quality", "50")?.parse::<u8>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid quality".to_string()))?;

        match artifact_type.as_str() {
            "jpeg" => {
                let block_size = 8;
                let quality_factor = (100 - quality) as f32 / 100.0;

                for chunk in data.chunks_mut(block_size * block_size * self.channels as usize) {
                    for pixel in chunk.chunks_mut(self.channels as usize) {
                        if pixel.len() >= 3 {
                            pixel[0] = ((pixel[0] as f32 / 8.0).round() * 8.0 * (1.0 - quality_factor) + pixel[0] as f32 * quality_factor) as u8;
                            pixel[1] = ((pixel[1] as f32 / 8.0).round() * 8.0 * (1.0 - quality_factor) + pixel[1] as f32 * quality_factor) as u8;
                            pixel[2] = ((pixel[2] as f32 / 8.0).round() * 8.0 * (1.0 - quality_factor) + pixel[2] as f32 * quality_factor) as u8;
                        }
                    }
                }
            }
            "blocking" => {
                let block_size = 16;
                let artifact_factor = (100 - quality) as f32 / 100.0;

                for chunk in data.chunks_mut(block_size * block_size * self.channels as usize) {
                    let mut sum = [0u32; self.channels as usize];
                    let mut count = 0u32;

                    for pixel in chunk.chunks(self.channels as usize) {
                        if pixel.len() >= self.channels as usize {
                            for (i, &channel) in pixel.iter().enumerate().take(self.channels as usize) {
                                sum[i] += channel as u32;
                            }
                            count += 1;
                        }
                    }

                    if count > 0 {
                        let avg: Vec<u8> = sum.iter().map(|&s| (s / count) as u8).collect();

                        for pixel in chunk.chunks_mut(self.channels as usize) {
                            if pixel.len() >= self.channels as usize {
                                for i in 0..self.channels as usize {
                                    pixel[i] = (pixel[i] as f32 * (1.0 - artifact_factor) + avg[i] as f32 * artifact_factor) as u8;
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid artifact type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_bit_manipulation(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let manipulation_type = self.parse_parameter(parameters, "manipulation_type", "bit_flip")?;
        let bits = self.parse_parameter(parameters, "bits", "1")?.parse::<u8>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid bits".to_string()))?;

        match manipulation_type.as_str() {
            "bit_flip" => {
                let flip_count = (data.len() as f32 * bits as f32 / 100.0) as usize;
                for _ in 0..flip_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    let bit_pos = rng.gen_range(0, 8);
                    data[pos] ^= 1 << bit_pos;
                }
            }
            "bit_shift" => {
                let shift_amount = bits % 8;
                for byte in data.iter_mut() {
                    *byte = (*byte << shift_amount) | (*byte >> (8 - shift_amount));
                }
            }
            "bit_rotate" => {
                let rotate_amount = bits % 8;
                for byte in data.iter_mut() {
                    *byte = byte.rotate_left(rotate_amount);
                }
            }
            "bit_invert" => {
                let invert_count = (data.len() as f32 * bits as f32 / 100.0) as usize;
                for _ in 0..invert_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    data[pos] = !data[pos];
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid manipulation type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_noise_injection(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let noise_type = self.parse_parameter(parameters, "noise_type", "uniform")?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.3")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        match noise_type.as_str() {
            "uniform" => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        let noise = rng.gen_range(-50, 50) as i8;
                        data[pos] = data[pos].wrapping_add(noise as u8);
                    }
                }
            }
            "gaussian" => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        let noise = (rng.gen_range(-1.0, 1.0) * 50.0) as i8;
                        data[pos] = data[pos].wrapping_add(noise as u8);
                    }
                }
            }
            "salt_pepper" => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        data[pos] = if rng.gen_range(0.0, 1.0) < 0.5 { 0 } else { 255 };
                    }
                }
            }
            "perlin" => {
                let mut noise_gen = create_noise_generator_with_seed(self.random_seed.unwrap_or(42));
                let noise_factor = intensity * 50.0;
                for (i, byte) in data.iter_mut().enumerate() {
                    let noise = noise_gen.perlin(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
                    *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid noise type".to_string()));
            }
        }

        Ok(())
    }

    fn parse_parameter(&self, parameters: &HashMap<String, String>, key: &str, default: &str) -> Result<String> {
        Ok(parameters.get(key).cloned().unwrap_or_else(|| default.to_string()))
    }

    pub fn clone(&self) -> ImageGlitchGenerator {
        ImageGlitchGenerator {
            width: self.width,
            height: self.height,
            channels: self.channels,
            random_seed: self.random_seed,
        }
    }
}

impl AudioGlitchGenerator {
    pub fn new(sample_rate: u32, channels: u8, duration: f32) -> Self {
        Self {
            sample_rate,
            channels,
            duration,
            random_seed: None,
        }
    }

    pub fn with_seed(sample_rate: u32, channels: u8, duration: f32, seed: u64) -> Self {
        Self {
            sample_rate,
            channels,
            duration,
            random_seed: Some(seed),
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn random_seed(&self) -> Option<u64> {
        self.random_seed
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.random_seed = Some(seed);
    }

    pub fn generate_glitch(&self, glitch_type: AudioGlitchType, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        let sample_count = (self.duration * self.sample_rate as f32) as usize;
        let data_size = sample_count * self.channels as usize * 4;4 bytes per f32 sample
        let mut data = vec![0u8; data_size];
        let mut rng = create_random_generator();

        if let Some(seed) = self.random_seed {
            rng.set_seed(seed);
        }

        match glitch_type {
            AudioGlitchType::SampleCorruption => {
                self.generate_sample_corruption(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::TimeStretch => {
                self.generate_time_stretch(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::PitchShift => {
                self.generate_pitch_shift(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::BitCrush => {
                self.generate_bit_crush(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::GlitchLoop => {
                self.generate_glitch_loop(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::ReverseSegments => {
                self.generate_reverse_segments(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::Stutter => {
                self.generate_stutter(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::RingModulation => {
                self.generate_ring_modulation(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::FrequencyModulation => {
                self.generate_frequency_modulation(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::PhaseVocoder => {
                self.generate_phase_vocoder(&mut data, parameters, &mut rng)?;
            }
            AudioGlitchType::DataBending => {
                self.generate_data_bending(&mut data, parameters, &mut rng)?;
            }
        }

        Ok(data)
    }

    fn generate_sample_corruption(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let corruption_type = self.parse_parameter(parameters, "corruption_type", "random_flip")?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        match corruption_type.as_str() {
            "random_flip" => {
                let flip_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..flip_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        data[pos] = rng.gen_range(0, 256) as u8;
                    }
                }
            }
            "bit_flip" => {
                let flip_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..flip_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        let bit_pos = rng.gen_range(0, 8);
                        data[pos] ^= 1 << bit_pos;
                    }
                }
            }
            "sample_dropout" => {
                let dropout_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..dropout_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        data[pos] = 0;
                    }
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid corruption type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_time_stretch(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let ratio = self.parse_parameter(parameters, "ratio", "1.5")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid ratio".to_string()))?;

        let sample_size = 4 * self.channels as usize;
        let new_length = (data.len() as f32 * ratio) as usize;
        let mut stretched = vec![0u8; new_length];

        for i in (0..new_length).step_by(sample_size) {
            let src_pos = i as f32 / ratio;
            let src_index = src_pos as usize;
            let fraction = src_pos - src_index as f32;

            if src_index + sample_size <= data.len() {
                for j in 0..sample_size {
                    if i + j < stretched.len() {
                        stretched[i + j] = data[src_index + j];
                    }
                }
            }
        }

        data.resize(new_length, 0);
        data.copy_from_slice(&stretched);

        Ok(())
    }

    fn generate_pitch_shift(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let semitones = self.parse_parameter(parameters, "semitones", "2.0")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid semitones".to_string()))?;

        let ratio = 2.0_f32.powf(-semitones / 12.0);
        let sample_size = 4 * self.channels as usize;
        let new_length = (data.len() as f32 / ratio) as usize;
        let mut shifted = vec![0u8; new_length];

        for i in (0..new_length).step_by(sample_size) {
            let src_pos = i as f32 * ratio;
            let src_index = src_pos as usize;

            if src_index + sample_size <= data.len() {
                for j in 0..sample_size {
                    if i + j < shifted.len() {
                        shifted[i + j] = data[src_index + j];
                    }
                }
            }
        }

        data.resize(new_length, 0);
        data.copy_from_slice(&shifted);

        Ok(())
    }

    fn generate_bit_crush(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let bit_depth = self.parse_parameter(parameters, "bit_depth", "8")?.parse::<u8>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid bit depth".to_string()))?;

        let levels = 2.0_f32.powi(bit_depth as i32);
        let step = 2.0 / levels;

        for chunk in data.chunks_mut(4) {
            if chunk.len() >= 4 {
                let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let crushed = (sample / step).round() * step;
                let crushed_bytes = crushed.to_le_bytes();
                chunk.copy_from_slice(&crushed_bytes);
            }
        }

        Ok(())
    }

    fn generate_glitch_loop(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let loop_size = self.parse_parameter(parameters, "loop_size", "1024")?.parse::<usize>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid loop size".to_string()))?;
        let crossfade = self.parse_parameter(parameters, "crossfade", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid crossfade".to_string()))?;

        let sample_size = 4 * self.channels as usize;
        let loop_samples = loop_size / sample_size;

        if data.len() > loop_size {
            let loop_region = data[..loop_size].to_vec();
            let crossfade_samples = (loop_samples as f32 * crossfade) as usize * sample_size;

            for i in loop_size..data.len() {
                let loop_pos = i % loop_size;
                if i < loop_size + crossfade_samples {
                    let fade_out = 1.0 - (i - loop_size) as f32 / crossfade_samples as f32;
                    let fade_in = (i - loop_size) as f32 / crossfade_samples as f32;

                    for j in 0..sample_size {
                        if i + j < data.len() && loop_pos + j < loop_region.len() {
                            let current = f32::from_le_bytes([
                                data[i + j],
                                data[i + j + 1],
                                data[i + j + 2],
                                data[i + j + 3],
                            ]);
                            let loop_sample = f32::from_le_bytes([
                                loop_region[loop_pos + j],
                                loop_region[loop_pos + j + 1],
                                loop_region[loop_pos + j + 2],
                                loop_region[loop_pos + j + 3],
                            ]);
                            let mixed = current * fade_out + loop_sample * fade_in;
                            let mixed_bytes = mixed.to_le_bytes();
                            data[i + j] = mixed_bytes[0];
                            data[i + j + 1] = mixed_bytes[1];
                            data[i + j + 2] = mixed_bytes[2];
                            data[i + j + 3] = mixed_bytes[3];
                        }
                    }
                } else {
                    for j in 0..sample_size {
                        if i + j < data.len() && loop_pos + j < loop_region.len() {
                            data[i + j] = loop_region[loop_pos + j];
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_reverse_segments(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let segment_length = self.parse_parameter(parameters, "segment_length", "1024")?.parse::<usize>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid segment length".to_string()))?;

        for chunk in data.chunks_mut(segment_length) {
            chunk.reverse();
        }

        Ok(())
    }

    fn generate_stutter(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let repeat_count = self.parse_parameter(parameters, "repeat_count", "3")?.parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid repeat count".to_string()))?;
        let variation = self.parse_parameter(parameters, "variation", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid variation".to_string()))?;

        let sample_size = 4 * self.channels as usize;
        let stutter_size = 1024;

        for i in (0..data.len()).step_by(stutter_size) {
            if i + stutter_size <= data.len() {
                let segment = data[i..i + stutter_size].to_vec();
                for j in 1..repeat_count {
                    let start = i + j * stutter_size;
                    if start + stutter_size <= data.len() {
                        for k in 0..stutter_size {
                            let variation_amount = rng.gen_range(-variation, variation);
                            let original = f32::from_le_bytes([
                                segment[k],
                                segment[k + 1],
                                segment[k + 2],
                                segment[k + 3],
                            ]);
                            let varied = original + variation_amount;
                            let varied_bytes = varied.to_le_bytes();
                            data[start + k] = varied_bytes[0];
                            data[start + k + 1] = varied_bytes[1];
                            data[start + k + 2] = varied_bytes[2];
                            data[start + k + 3] = varied_bytes[3];
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_ring_modulation(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let frequency = self.parse_parameter(parameters, "frequency", "440.0")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid frequency".to_string()))?;
        let mix = self.parse_parameter(parameters, "mix", "0.5")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid mix".to_string()))?;

        for chunk in data.chunks_mut(4) {
            if chunk.len() >= 4 {
                let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let carrier = (2.0 * std::f32::consts::PI * frequency).sin();
                let modulated = sample * carrier;
                let mixed = sample * (1.0 - mix) + modulated * mix;
                let mixed_bytes = mixed.to_le_bytes();
                chunk.copy_from_slice(&mixed_bytes);
            }
        }

        Ok(())
    }

    fn generate_frequency_modulation(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let carrier_freq = self.parse_parameter(parameters, "carrier_freq", "440.0")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid carrier frequency".to_string()))?;
        let mod_freq = self.parse_parameter(parameters, "mod_freq", "100.0")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid modulation frequency".to_string()))?;

        for chunk in data.chunks_mut(4) {
            if chunk.len() >= 4 {
                let modulator = (2.0 * std::f32::consts::PI * mod_freq).sin();
                let carrier = (2.0 * std::f32::consts::PI * carrier_freq + modulator).sin();
                let carrier_bytes = carrier.to_le_bytes();
                chunk.copy_from_slice(&carrier_bytes);
            }
        }

        Ok(())
    }

    fn generate_phase_vocoder(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let bands = self.parse_parameter(parameters, "bands", "64")?.parse::<usize>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid bands".to_string()))?;

        for chunk in data.chunks_mut(4) {
            if chunk.len() >= 4 {
                let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let processed = sample * 0.5;
                let processed_bytes = processed.to_le_bytes();
                chunk.copy_from_slice(&processed_bytes);
            }
        }

        Ok(())
    }

    fn generate_data_bending(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let bend_type = self.parse_parameter(parameters, "bend_type", "xor")?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        match bend_type.as_str() {
            "xor" => {
                let xor_value = (intensity * 255.0) as u8;
                for byte in data.iter_mut() {
                    *byte ^= xor_value;
                }
            }
            "add" => {
                let add_value = (intensity * 0.5) as u8;
                for byte in data.iter_mut() {
                    *byte = byte.wrapping_add(add_value);
                }
            }
            "multiply" => {
                let multiply_factor = 1.0 + intensity;
                for chunk in data.chunks_mut(4) {
                    if chunk.len() >= 4 {
                        let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        let multiplied = sample * multiply_factor;
                        let multiplied_bytes = multiplied.to_le_bytes();
                        chunk.copy_from_slice(&multiplied_bytes);
                    }
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid bend type".to_string()));
            }
        }

        Ok(())
    }

    fn parse_parameter(&self, parameters: &HashMap<String, String>, key: &str, default: &str) -> Result<String> {
        Ok(parameters.get(key).cloned().unwrap_or_else(|| default.to_string()))
    }

    pub fn clone(&self) -> AudioGlitchGenerator {
        AudioGlitchGenerator {
            sample_rate: self.sample_rate,
            channels: self.channels,
            duration: self.duration,
            random_seed: self.random_seed,
        }
    }
}

impl VideoGlitchGenerator {
    pub fn new(width: u32, height: u32, fps: f32, frame_count: u32) -> Self {
        Self {
            width,
            height,
            fps,
            frame_count,
            random_seed: None,
        }
    }

    pub fn with_seed(width: u32, height: u32, fps: f32, frame_count: u32, seed: u64) -> Self {
        Self {
            width,
            height,
            fps,
            frame_count,
            random_seed: Some(seed),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn random_seed(&self) -> Option<u64> {
        self.random_seed
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.random_seed = Some(seed);
    }

    pub fn generate_glitch(&self, glitch_type: VideoGlitchType, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        let frame_size = (self.width * self.height * 4) as usize;
        let data_size = frame_size * self.frame_count as usize;
        let mut data = vec![0u8; data_size];
        let mut rng = create_random_generator();

        if let Some(seed) = self.random_seed {
            rng.set_seed(seed);
        }

        match glitch_type {
            VideoGlitchType::FrameCorruption => {
                self.generate_frame_corruption(&mut data, parameters, &mut rng)?;
            }
            VideoGlitchType::TimeManipulation => {
                self.generate_time_manipulation(&mut data, parameters, &mut rng)?;
            }
            VideoGlitchType::FrameReordering => {
                self.generate_frame_reordering(&mut data, parameters, &mut rng)?;
            }
            VideoGlitchType::CompressionArtifacts => {
                self.generate_compression_artifacts(&mut data, parameters, &mut rng)?;
            }
            VideoGlitchType::DataMosh => {
                self.generate_data_mosh(&mut data, parameters, &mut rng)?;
            }
            VideoGlitchType::ColorChannelCorruption => {
                self.generate_color_channel_corruption(&mut data, parameters, &mut rng)?;
            }
            VideoGlitchType::GeometricDistortion => {
                self.generate_geometric_distortion(&mut data, parameters, &mut rng)?;
            }
        }

        Ok(data)
    }

    fn generate_frame_corruption(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let corruption_type = self.parse_parameter(parameters, "corruption_type", "pixel_corruption")?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        let frame_size = (self.width * self.height * 4) as usize;

        for frame in data.chunks_mut(frame_size) {
            match corruption_type.as_str() {
                "pixel_corruption" => {
                    let corruption_count = (frame.len() as f32 * intensity) as usize;
                    for _ in 0..corruption_count {
                        let pos = rng.gen_range(0, frame.len() as u64) as usize;
                        if pos < frame.len() {
                            frame[pos] = rng.gen_range(0, 256) as u8;
                        }
                    }
                }
                "data_corruption" => {
                    let corruption_count = (frame.len() as f32 * intensity * 0.1) as usize;
                    for _ in 0..corruption_count {
                        let start = rng.gen_range(0, frame.len() as u64) as usize;
                        let end = (start + 1024).min(frame.len());
                        if end > start {
                            for i in start..end {
                                frame[i] = rng.gen_range(0, 256) as u8;
                            }
                        }
                    }
                }
                "header_corruption" => {
                    let header_size = 64.min(frame.len());
                    let corruption_count = (header_size as f32 * intensity) as usize;
                    for _ in 0..corruption_count {
                        let pos = rng.gen_range(0, header_size as u64) as usize;
                        frame[pos] = rng.gen_range(0, 256) as u8;
                    }
                }
                _ => {
                    return Err(EllasticError::InvalidParameter("Invalid corruption type".to_string()));
                }
            }
        }

        Ok(())
    }

    fn generate_time_manipulation(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let manipulation_type = self.parse_parameter(parameters, "manipulation_type", "frame_duplication")?;
        let amount = self.parse_parameter(parameters, "amount", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid amount".to_string()))?;

        let frame_size = (self.width * self.height * 4) as usize;
        let frame_count = data.len() / frame_size;

        match manipulation_type.as_str() {
            "frame_duplication" => {
                let duplication_count = (frame_count as f32 * amount) as usize;
                for _ in 0..duplication_count {
                    if frame_count > 0 {
                        let source_frame = rng.gen_range(0, frame_count as u64) as usize;
                        let source_frame_data = data[source_frame * frame_size..(source_frame + 1) * frame_size].to_vec();
                        let target_frame = rng.gen_range(0, frame_count as u64) as usize;
                        data[target_frame * frame_size..(target_frame + 1) * frame_size].copy_from_slice(&source_frame_data);
                    }
                }
            }
            "frame_dropping" => {
                let drop_count = (frame_count as f32 * amount) as usize;
                for _ in 0..drop_count {
                    if frame_count > 0 {
                        let drop_frame = rng.gen_range(0, frame_count as u64) as usize;
                        data[drop_frame * frame_size..(drop_frame + 1) * frame_size].fill(0);
                    }
                }
            }
            "reverse_playback" => {
                for frame in data.chunks_mut(frame_size) {
                    frame.reverse();
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid manipulation type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_frame_reordering(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let reorder_type = self.parse_parameter(parameters, "reorder_type", "random")?;
        let segment_size = self.parse_parameter(parameters, "segment_size", "10")?.parse::<usize>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid segment size".to_string()))?;

        let frame_size = (self.width * self.height * 4) as usize;

        match reorder_type.as_str() {
            "random" => {
                for chunk in data.chunks_mut(frame_size * segment_size) {
                    for frame in chunk.chunks_mut(frame_size) {
                        rng.shuffle(frame);
                    }
                }
            }
            "reverse" => {
                for chunk in data.chunks_mut(frame_size * segment_size) {
                    chunk.reverse();
                }
            }
            "shuffle" => {
                let frames: Vec<_> = data.chunks(frame_size).collect();
                let mut frame_indices: Vec<usize> = (0..frames.len()).collect();
                rng.shuffle(&mut frame_indices);

                let mut new_data = vec![0u8; data.len()];
                for (i, &frame_index) in frame_indices.iter().enumerate() {
                    if i * frame_size + frame_size <= new_data.len() && frame_index * frame_size + frame_size <= data.len() {
                        new_data[i * frame_size..(i + 1) * frame_size].copy_from_slice(&data[frame_index * frame_size..(frame_index + 1) * frame_size]);
                    }
                }
                data.copy_from_slice(&new_data);
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid reorder type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_compression_artifacts(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let artifact_type = self.parse_parameter(parameters, "artifact_type", "blocking")?;
        let quality = self.parse_parameter(parameters, "quality", "50")?.parse::<u8>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid quality".to_string()))?;

        let frame_size = (self.width * self.height * 4) as usize;

        for frame in data.chunks_mut(frame_size) {
            match artifact_type.as_str() {
                "blocking" => {
                    let block_size = 8;
                    let artifact_factor = (100 - quality) as f32 / 100.0;

                    for y in (0..self.height).step_by(block_size) {
                        for x in (0..self.width).step_by(block_size) {
                            let mut sum = [0u32; 4];
                            let mut count = 0u32;

                            for dy in 0..block_size.min(self.height - y) {
                                for dx in 0..block_size.min(self.width - x) {
                                    let pixel_index = ((y + dy) * self.width + (x + dx)) * 4;
                                    if pixel_index + 3 < frame.len() {
                                        sum[0] += frame[pixel_index] as u32;
                                        sum[1] += frame[pixel_index + 1] as u32;
                                        sum[2] += frame[pixel_index + 2] as u32;
                                        sum[3] += frame[pixel_index + 3] as u32;
                                        count += 1;
                                    }
                                }
                            }

                            if count > 0 {
                                let avg = [sum[0] / count, sum[1] / count, sum[2] / count, sum[3] / count];

                                for dy in 0..block_size.min(self.height - y) {
                                    for dx in 0..block_size.min(self.width - x) {
                                        let pixel_index = ((y + dy) * self.width + (x + dx)) * 4;
                                        if pixel_index + 3 < frame.len() {
                                            frame[pixel_index] = (frame[pixel_index] as f32 * (1.0 - artifact_factor) + avg[0] as f32 * artifact_factor) as u8;
                                            frame[pixel_index + 1] = (frame[pixel_index + 1] as f32 * (1.0 - artifact_factor) + avg[1] as f32 * artifact_factor) as u8;
                                            frame[pixel_index + 2] = (frame[pixel_index + 2] as f32 * (1.0 - artifact_factor) + avg[2] as f32 * artifact_factor) as u8;
                                            frame[pixel_index + 3] = (frame[pixel_index + 3] as f32 * (1.0 - artifact_factor) + avg[3] as f32 * artifact_factor) as u8;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "macroblocking" => {
                    let block_size = 16;
                    let artifact_factor = (100 - quality) as f32 / 100.0;

                    for y in (0..self.height).step_by(block_size) {
                        for x in (0..self.width).step_by(block_size) {
                            let mut sum = [0u32; 4];
                            let mut count = 0u32;

                            for dy in 0..block_size.min(self.height - y) {
                                for dx in 0..block_size.min(self.width - x) {
                                    let pixel_index = ((y + dy) * self.width + (x + dx)) * 4;
                                    if pixel_index + 3 < frame.len() {
                                        sum[0] += frame[pixel_index] as u32;
                                        sum[1] += frame[pixel_index + 1] as u32;
                                        sum[2] += frame[pixel_index + 2] as u32;
                                        sum[3] += frame[pixel_index + 3] as u32;
                                        count += 1;
                                    }
                                }
                            }

                            if count > 0 {
                                let avg = [sum[0] / count, sum[1] / count, sum[2] / count, sum[3] / count];

                                for dy in 0..block_size.min(self.height - y) {
                                    for dx in 0..block_size.min(self.width - x) {
                                        let pixel_index = ((y + dy) * self.width + (x + dx)) * 4;
                                        if pixel_index + 3 < frame.len() {
                                            frame[pixel_index] = (frame[pixel_index] as f32 * (1.0 - artifact_factor) + avg[0] as f32 * artifact_factor) as u8;
                                            frame[pixel_index + 1] = (frame[pixel_index + 1] as f32 * (1.0 - artifact_factor) + avg[1] as f32 * artifact_factor) as u8;
                                            frame[pixel_index + 2] = (frame[pixel_index + 2] as f32 * (1.0 - artifact_factor) + avg[2] as f32 * artifact_factor) as u8;
                                            frame[pixel_index + 3] = (frame[pixel_index + 3] as f32 * (1.0 - artifact_factor) + avg[3] as f32 * artifact_factor) as u8;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(EllasticError::InvalidParameter("Invalid artifact type".to_string()));
                }
            }
        }

        Ok(())
    }

    fn generate_data_mosh(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let intensity = self.parse_parameter(parameters, "intensity", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        let frame_size = (self.width * self.height * 4) as usize;
        let frame_count = data.len() / frame_size;

        for i in 1..frame_count {
            let current_frame = &mut data[i * frame_size..(i + 1) * frame_size];
            let previous_frame = &data[(i - 1) * frame_size..i * frame_size];

            let mosh_count = (current_frame.len() as f32 * intensity) as usize;

            for _ in 0..mosh_count {
                let pos = rng.gen_range(0, current_frame.len() as u64) as usize;
                if pos < current_frame.len() && pos < previous_frame.len() {
                    current_frame[pos] = previous_frame[pos];
                }
            }
        }

        Ok(())
    }

    fn generate_color_channel_corruption(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let channel = self.parse_parameter(parameters, "channel", "0")?.parse::<u8>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid channel".to_string()))?;
        let corruption_type = self.parse_parameter(parameters, "corruption_type", "channel_swap")?;
        let amount = self.parse_parameter(parameters, "amount", "0.5")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid amount".to_string()))?;

        let frame_size = (self.width * self.height * 4) as usize;

        for frame in data.chunks_mut(frame_size) {
            for pixel in frame.chunks_mut(4) {
                if pixel.len() > channel as usize {
                    match corruption_type.as_str() {
                        "channel_swap" => {
                            if pixel.len() >= 3 {
                                pixel.swap(0, 1);
                                pixel.swap(1, 2);
                            }
                        }
                        "color_shift" => {
                            let shift = (amount * 255.0) as i32;
                            pixel[channel as usize] = (pixel[channel as usize] as i32 + shift).clamp(0, 255) as u8;
                        }
                        "inversion" => {
                            pixel[channel as usize] = (pixel[channel as usize] as f32 * (1.0 - amount) + (255.0 - pixel[channel as usize] as f32) * amount) as u8;
                        }
                        _ => {
                            return Err(EllasticError::InvalidParameter("Invalid corruption type".to_string()));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_geometric_distortion(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let distortion_type = self.parse_parameter(parameters, "distortion_type", "wave")?;
        let strength = self.parse_parameter(parameters, "strength", "0.3")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid strength".to_string()))?;

        let frame_size = (self.width * self.height * 4) as usize;

        for frame in data.chunks_mut(frame_size) {
            match distortion_type.as_str() {
                "wave" => {
                    for y in 0..self.height {
                        for x in 0..self.width {
                            let offset = (strength * 10.0 * ((x as f32 / self.width as f32) * 2.0 * std::f32::consts::PI).sin()) as i32;
                            let pixel_index = ((y as usize * self.width as usize + x as usize) * 4).min(frame.len() - 1);

                            if offset != 0 && pixel_index + offset < frame.len() {
                                let source_pixel = frame[pixel_index];
                                frame[pixel_index] = frame[pixel_index + offset];
                            }
                        }
                    }
                }
                "swirl" => {
                    let center_x = self.width as f32 / 2.0;
                    let center_y = self.height as f32 / 2.0;

                    for y in 0..self.height {
                        for x in 0..self.width {
                            let dx = x as f32 - center_x;
                            let dy = y as f32 - center_y;
                            let distance = (dx * dx + dy * dy).sqrt();
                            let angle = strength * distance / 10.0;
                            let cos_angle = angle.cos();
                            let sin_angle = angle.sin();

                            let source_x = (center_x + dx * cos_angle - dy * sin_angle) as i32;
                            let source_y = (center_y + dx * sin_angle + dy * cos_angle) as i32;

                            if source_x >= 0 && source_x < self.width as i32 &&
                               source_y >= 0 && source_y < self.height as i32 {
                                let source_index = (source_y as usize * self.width as usize + source_x as usize) * 4;
                                let pixel_index = (y as usize * self.width as usize + x as usize) * 4;

                                if source_index < frame.len() && pixel_index < frame.len() {
                                    frame[pixel_index] = frame[source_index];
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(EllasticError::InvalidParameter("Invalid distortion type".to_string()));
                }
            }
        }

        Ok(())
    }

    fn parse_parameter(&self, parameters: &HashMap<String, String>, key: &str, default: &str) -> Result<String> {
        Ok(parameters.get(key).cloned().unwrap_or_else(|| default.to_string()))
    }

    pub fn clone(&self) -> VideoGlitchGenerator {
        VideoGlitchGenerator {
            width: self.width,
            height: self.height,
            fps: self.fps,
            frame_count: self.frame_count,
            random_seed: self.random_seed,
        }
    }
}

impl DataGlitchGenerator {
    pub fn new(data_size: usize) -> Self {
        Self {
            data_size,
            random_seed: None,
        }
    }

    pub fn with_seed(data_size: usize, seed: u64) -> Self {
        Self {
            data_size,
            random_seed: Some(seed),
        }
    }

    pub fn data_size(&self) -> usize {
        self.data_size
    }

    pub fn random_seed(&self) -> Option<u64> {
        self.random_seed
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.random_seed = Some(seed);
    }

    pub fn generate_glitch(&self, glitch_type: DataGlitchType, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        let mut data = vec![0u8; self.data_size];
        let mut rng = create_random_generator();

        if let Some(seed) = self.random_seed {
            rng.set_seed(seed);
        }

        match glitch_type {
            DataGlitchType::ByteManipulation => {
                self.generate_byte_manipulation(&mut data, parameters, &mut rng)?;
            }
            DataGlitchType::DataCorruption => {
                self.generate_data_corruption(&mut data, parameters, &mut rng)?;
            }
            DataGlitchType::FormatBending => {
                self.generate_format_bending(&mut data, parameters, &mut rng)?;
            }
            DataGlitchType::HeaderCorruption => {
                self.generate_header_corruption(&mut data, parameters, &mut rng)?;
            }
            DataGlitchType::StructuralDamage => {
                self.generate_structural_damage(&mut data, parameters, &mut rng)?;
            }
        }

        Ok(data)
    }

    fn generate_byte_manipulation(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let manipulation_type = self.parse_parameter(parameters, "manipulation_type", "xor")?;

        match manipulation_type.as_str() {
            "xor" => {
                let xor_value = self.parse_parameter(parameters, "xor_value", "0x55")?;
                let xor_bytes = self.parse_hex_value(&xor_value)?;

                for byte in data.iter_mut() {
                    *byte ^= xor_bytes[0];
                }
            }
            "add" => {
                let add_value = self.parse_parameter(parameters, "add_value", "1")?;
                let add_bytes = self.parse_hex_value(&add_value)?;

                for byte in data.iter_mut() {
                    *byte = byte.wrapping_add(add_bytes[0]);
                }
            }
            "multiply" => {
                let mul_value = self.parse_parameter(parameters, "mul_value", "2")?;
                let mul_bytes = self.parse_hex_value(&mul_value)?;

                for byte in data.iter_mut() {
                    *byte = byte.wrapping_mul(mul_bytes[0]);
                }
            }
            "shift_left" => {
                let shift_bits = self.parse_parameter(parameters, "shift_bits", "1")?.parse::<u8>()
                    .map_err(|_| EllasticError::InvalidParameter("Invalid shift bits".to_string()))?;

                for byte in data.iter_mut() {
                    *byte <<= shift_bits;
                }
            }
            "rotate_left" => {
                let rotate_bits = self.parse_parameter(parameters, "rotate_bits", "1")?.parse::<u8>()
                    .map_err(|_| EllasticError::InvalidParameter("Invalid rotate bits".to_string()))?;

                for byte in data.iter_mut() {
                    *byte = byte.rotate_left(rotate_bits);
                }
            }
            "invert" => {
                for byte in data.iter_mut() {
                    *byte = !*byte;
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid manipulation type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_data_corruption(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let corruption_type = self.parse_parameter(parameters, "corruption_type", "random")?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        match corruption_type.as_str() {
            "random" => {
                let corruption_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..corruption_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        data[pos] = rng.gen_range(0, 256) as u8;
                    }
                }
            }
            "pattern" => {
                let pattern = self.generate_corruption_pattern(rng);
                let pattern_size = pattern.len();

                for i in (0..data.len()).step_by(pattern_size) {
                    for j in 0..pattern_size.min(data.len() - i) {
                        data[i + j] ^= pattern[j];
                    }
                }
            }
            "structured" => {
                let corruption_count = (data.len() as f32 * intensity * 0.1) as usize;

                for _ in 0..corruption_count {
                    let start = rng.gen_range(0, data.len() as u64) as usize;
                    let end = (start + rng.gen_range(1, 64) as usize).min(data.len());

                    if end > start {
                        let corruption_pattern = self.generate_corruption_pattern(rng);

                        for i in start..end {
                            data[i] ^= corruption_pattern[i % corruption_pattern.len()];
                        }
                    }
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid corruption type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_format_bending(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let source_format = self.parse_parameter(parameters, "source_format", "png")?;
        let target_format = self.parse_parameter(parameters, "target_format", "jpg")?;
        let bend_type = self.parse_parameter(parameters, "bend_type", "header_injection")?;

        match bend_type.as_str() {
            "header_injection" => {
                let target_header = self.get_format_header(&target_format)?;
                let header_size = target_header.len();

                if data.len() >= header_size {
                    data[..header_size].copy_from_slice(&target_header);
                }
            }
            "footer_injection" => {
                let target_footer = self.get_format_footer(&target_format)?;
                let footer_size = target_footer.len();

                if data.len() >= footer_size {
                    data[data.len() - footer_size..].copy_from_slice(&target_footer);
                }
            }
            "metadata_corruption" => {
                let metadata_start = 64;
                let metadata_size = 128;

                if data.len() >= metadata_start + metadata_size {
                    for i in metadata_start..metadata_start + metadata_size {
                        data[i] = data[i].wrapping_add(1);
                    }
                }
            }
            "structure_rearrangement" => {
                let chunk_size = 1024;
                let mut chunks: Vec<_> = data.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();

                rng.shuffle(&mut chunks);

                let mut pos = 0;
                for chunk in chunks {
                    if pos + chunk.len() <= data.len() {
                        data[pos..pos + chunk.len()].copy_from_slice(&chunk);
                        pos += chunk.len();
                    }
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid bend type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_header_corruption(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let header_type = self.parse_parameter(parameters, "header_type", "image")?;
        let corruption_type = self.parse_parameter(parameters, "corruption_type", "size_manipulation")?;

        match header_type.as_str() {
            "image" => {
                if data.len() >= 8 {
                    data[4] = data[4].wrapping_add(1);
                    data[5] = data[5].wrapping_add(1);
                }
            }
            "audio" => {
                if data.len() >= 12 {
                    data[8] = data[8].wrapping_add(1);
                    data[9] = data[9].wrapping_add(1);
                }
            }
            "video" => {
                if data.len() >= 16 {
                    data[12] = data[12].wrapping_add(1);
                    data[13] = data[13].wrapping_add(1);
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid header type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_structural_damage(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let damage_type = self.parse_parameter(parameters, "damage_type", "fragmentation")?;
        let severity = self.parse_parameter(parameters, "severity", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid severity".to_string()))?;

        match damage_type.as_str() {
            "fragmentation" => {
                let fragment_count = (data.len() as f32 * severity * 0.1) as usize;

                for _ in 0..fragment_count {
                    let start = rng.gen_range(0, data.len() as u64) as usize;
                    let end = (start + rng.gen_range(1, 64) as usize).min(data.len());

                    if end > start {
                        let fragment = data[start..end].to_vec();
                        let new_pos = rng.gen_range(0, (data.len() - fragment.len()) as u64) as usize;

                        if new_pos + fragment.len() <= data.len() {
                            data[new_pos..new_pos + fragment.len()].copy_from_slice(&fragment);
                        }
                    }
                }
            }
            "reordering" => {
                let reorder_count = (data.len() as f32 * severity * 0.05) as usize;

                for _ in 0..reorder_count {
                    let pos1 = rng.gen_range(0, data.len() as u64) as usize;
                    let pos2 = rng.gen_range(0, data.len() as u64) as usize;

                    if pos1 < data.len() && pos2 < data.len() {
                        data.swap(pos1, pos2);
                    }
                }
            }
            "duplication" => {
                let duplicate_count = (data.len() as f32 * severity * 0.02) as usize;

                for _ in 0..duplicate_count {
                    let start = rng.gen_range(0, data.len() as u64) as usize;
                    let length = rng.gen_range(1, 32) as usize;
                    let end = (start + length).min(data.len());

                    if end > start {
                        let duplicate = data[start..end].to_vec();
                        let target_pos = rng.gen_range(0, (data.len() - duplicate.len()) as u64) as usize;

                        if target_pos + duplicate.len() <= data.len() {
                            data[target_pos..target_pos + duplicate.len()].copy_from_slice(&duplicate);
                        }
                    }
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid damage type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_corruption_pattern(&self, rng: &mut ellastic_utils::RandomGenerator) -> Vec<u8> {
        let pattern_length = rng.gen_range(4, 16);
        let mut pattern = Vec::with_capacity(pattern_length);

        for _ in 0..pattern_length {
            pattern.push(rng.gen_range(0, 256) as u8);
        }

        pattern
    }

    fn parse_hex_value(&self, hex_str: &str) -> Result<Vec<u8>> {
        let hex_str = hex_str.trim_start_matches("0x");

        if hex_str.len() % 2 != 0 {
            return Err(EllasticError::InvalidParameter("Invalid hex string length".to_string()));
        }

        let mut bytes = Vec::new();

        for chunk in hex_str.as_bytes().chunks(2) {
            let hex_byte = std::str::from_utf8(chunk)
                .map_err(|_| EllasticError::InvalidParameter("Invalid hex string".to_string()))?;

            let byte = u8::from_str_radix(hex_byte, 16)
                .map_err(|_| EllasticError::InvalidParameter("Invalid hex value".to_string()))?;

            bytes.push(byte);
        }

        Ok(bytes)
    }

    fn get_format_header(&self, format: &str) -> Result<Vec<u8>> {
        match format.to_lowercase().as_str() {
            "png" => Ok(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            "jpg" | "jpeg" => Ok(vec![0xFF, 0xD8, 0xFF, 0xE0]),
            "gif" => Ok(vec![0x47, 0x49, 0x46, 0x38]),
            "bmp" => Ok(vec![0x42, 0x4D]),
            "wav" => Ok(vec![0x52, 0x49, 0x46, 0x46]),
            "mp3" => Ok(vec![0x49, 0x44, 0x33]),
            "flac" => Ok(vec![0x66, 0x4C, 0x61, 0x43]),
            "mp4" => Ok(vec![0x66, 0x74, 0x79, 0x70]),
            "avi" => Ok(vec![0x52, 0x49, 0x46, 0x46]),
            _ => Err(EllasticError::UnsupportedFormat(format!("Unsupported format: {}", format))),
        }
    }

    fn get_format_footer(&self, format: &str) -> Result<Vec<u8>> {
        match format.to_lowercase().as_str() {
            "png" => Ok(vec![0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]),
            "jpg" | "jpeg" => Ok(vec![0xFF, 0xD9]),
            "gif" => Ok(vec![0x00, 0x3B]),
            "bmp" => Ok(vec![]),
            "wav" => Ok(vec![]),
            "mp3" => Ok(vec![]),
            "flac" => Ok(vec![]),
            "mp4" => Ok(vec![]),
            "avi" => Ok(vec![]),
            _ => Err(EllasticError::UnsupportedFormat(format!("Unsupported format: {}", format))),
        }
    }

    fn parse_parameter(&self, parameters: &HashMap<String, String>, key: &str, default: &str) -> Result<String> {
        Ok(parameters.get(key).cloned().unwrap_or_else(|| default.to_string()))
    }

    pub fn clone(&self) -> DataGlitchGenerator {
        DataGlitchGenerator {
            data_size: self.data_size,
            random_seed: self.random_seed,
        }
    }
}

impl PatternGlitchGenerator {
    pub fn new(pattern_size: usize, repeat_count: usize) -> Self {
        Self {
            pattern_size,
            repeat_count,
            random_seed: None,
        }
    }

    pub fn with_seed(pattern_size: usize, repeat_count: usize, seed: u64) -> Self {
        Self {
            pattern_size,
            repeat_count,
            random_seed: Some(seed),
        }
    }

    pub fn pattern_size(&self) -> usize {
        self.pattern_size
    }

    pub fn repeat_count(&self) -> usize {
        self.repeat_count
    }

    pub fn random_seed(&self) -> Option<u64> {
        self.random_seed
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.random_seed = Some(seed);
    }

    pub fn generate_glitch(&self, glitch_type: PatternGlitchType, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        let data_size = self.pattern_size * self.repeat_count;
        let mut data = vec![0u8; data_size];
        let mut rng = create_random_generator();

        if let Some(seed) = self.random_seed {
            rng.set_seed(seed);
        }

        match glitch_type {
            PatternGlitchType::RepeatingPattern => {
                self.generate_repeating_pattern(&mut data, parameters, &mut rng)?;
            }
            PatternGlitchType::RandomNoise => {
                self.generate_random_noise(&mut data, parameters, &mut rng)?;
            }
            PatternGlitchType::GlitchPattern => {
                self.generate_glitch_pattern(&mut data, parameters, &mut rng)?;
            }
            PatternGlitchType::DataPattern => {
                self.generate_data_pattern(&mut data, parameters, &mut rng)?;
            }
        }

        Ok(data)
    }

    fn generate_repeating_pattern(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let pattern = self.parse_parameter(parameters, "pattern", "0xDEADBEEF")?;
        let period = self.parse_parameter(parameters, "period", "16")?.parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid period".to_string()))?;
        let offset = self.parse_parameter(parameters, "offset", "0")?.parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid offset".to_string()))?;

        let pattern_bytes = pattern.to_le_bytes();

        for i in (offset as usize..data.len()).step_by(period as usize) {
            for j in 0..pattern_bytes.len().min(data.len() - i) {
                data[i + j] = pattern_bytes[j];
            }
        }

        Ok(())
    }

    fn generate_random_noise(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let noise_type = self.parse_parameter(parameters, "noise_type", "uniform")?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.3")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        match noise_type.as_str() {
            "uniform" => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        data[pos] = rng.gen_range(0, 256) as u8;
                    }
                }
            }
            "gaussian" => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        let noise = (rng.gen_range(-1.0, 1.0) * 50.0) as i8;
                        data[pos] = data[pos].wrapping_add(noise as u8);
                    }
                }
            }
            "salt_pepper" => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        data[pos] = if rng.gen_range(0.0, 1.0) < 0.5 { 0 } else { 255 };
                    }
                }
            }
            "perlin" => {
                let mut noise_gen = create_noise_generator_with_seed(self.random_seed.unwrap_or(42));
                let noise_factor = intensity * 50.0;
                for (i, byte) in data.iter_mut().enumerate() {
                    let noise = noise_gen.perlin(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
                    *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid noise type".to_string()));
            }
        }

        Ok(())
    }

    fn generate_glitch_pattern(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let glitch_type = self.parse_parameter(parameters, "glitch_type", "byte_flip")?;
        let frequency = self.parse_parameter(parameters, "frequency", "0.1")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid frequency".to_string()))?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.5")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        let glitch_count = (data.len() as f32 * frequency) as usize;

        for _ in 0..glitch_count {
            let pos = rng.gen_range(0, data.len() as u64) as usize;
            let glitch_size = (intensity * 64.0) as usize;

            if pos + glitch_size <= data.len() {
                match glitch_type.as_str() {
                    "byte_flip" => {
                        for i in pos..pos + glitch_size {
                            data[i] ^= 0xFF;
                        }
                    }
                    "byte_swap" => {
                        for i in (pos..pos + glitch_size).step_by(2) {
                            if i + 1 < pos + glitch_size {
                                data.swap(i, i + 1);
                            }
                        }
                    }
                    "byte_shift" => {
                        for i in pos..pos + glitch_size {
                            data[i] <<= 1;
                        }
                    }
                    "byte_rotate" => {
                        for i in pos..pos + glitch_size {
                            data[i] = data[i].rotate_left(1);
                        }
                    }
                    _ => {
                        return Err(EllasticError::InvalidParameter("Invalid glitch type".to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_data_pattern(&self, data: &mut [u8], parameters: &HashMap<String, String>, rng: &mut ellastic_utils::RandomGenerator) -> Result<()> {
        let pattern_type = self.parse_parameter(parameters, "pattern_type", "overlay")?;
        let pattern_data_hex = self.parse_parameter(parameters, "pattern_data", "0x12345678")?;
        let intensity = self.parse_parameter(parameters, "intensity", "0.5")?.parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid intensity".to_string()))?;

        let pattern_data = self.parse_hex_value(&pattern_data_hex)?;

        match pattern_type.as_str() {
            "overlay" => {
                let overlay_count = (data.len() as f32 * intensity) as usize;
                for i in 0..overlay_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] = data[i].wrapping_add(pattern_data[data_index]);
                }
            }
            "replace" => {
                let replace_count = (data.len() as f32 * intensity) as usize;
                for i in 0..replace_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] = pattern_data[data_index];
                }
            }
            "xor" => {
                let xor_count = (data.len() as f32 * intensity) as usize;
                for i in 0..xor_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] ^= pattern_data[data_index];
                }
            }
            "and" => {
                let and_count = (data.len() as f32 * intensity) as usize;
                for i in 0..and_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] &= pattern_data[data_index];
                }
            }
            "or" => {
                let or_count = (data.len() as f32 * intensity) as usize;
                for i in 0..or_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] |= pattern_data[data_index];
                }
            }
            _ => {
                return Err(EllasticError::InvalidParameter("Invalid pattern type".to_string()));
            }
        }

        Ok(())
    }

    fn parse_hex_value(&self, hex_str: &str) -> Result<Vec<u8>> {
        let hex_str = hex_str.trim_start_matches("0x");

        if hex_str.len() % 2 != 0 {
            return Err(EllasticError::InvalidParameter("Invalid hex string length".to_string()));
        }

        let mut bytes = Vec::new();

        for chunk in hex_str.as_bytes().chunks(2) {
            let hex_byte = std::str::from_utf8(chunk)
                .map_err(|_| EllasticError::InvalidParameter("Invalid hex string".to_string()))?;

            let byte = u8::from_str_radix(hex_byte, 16)
                .map_err(|_| EllasticError::InvalidParameter("Invalid hex value".to_string()))?;

            bytes.push(byte);
        }

        Ok(bytes)
    }

    fn parse_parameter(&self, parameters: &HashMap<String, String>, key: &str, default: &str) -> Result<String> {
        Ok(parameters.get(key).cloned().unwrap_or_else(|| default.to_string()))
    }

    pub fn clone(&self) -> PatternGlitchGenerator {
        PatternGlitchGenerator {
            pattern_size: self.pattern_size,
            repeat_count: self.repeat_count,
            random_seed: self.random_seed,
        }
    }
}

pub fn create_image_glitch_generator(width: u32, height: u32, channels: u8) -> ImageGlitchGenerator {
    ImageGlitchGenerator::new(width, height, channels)
}

pub fn create_image_glitch_generator_with_seed(width: u32, height: u32, channels: u8, seed: u64) -> ImageGlitchGenerator {
    ImageGlitchGenerator::with_seed(width, height, channels, seed)
}

pub fn create_audio_glitch_generator(sample_rate: u32, channels: u8, duration: f32) -> AudioGlitchGenerator {
    AudioGlitchGenerator::new(sample_rate, channels, duration)
}

pub fn create_audio_glitch_generator_with_seed(sample_rate: u32, channels: u8, duration: f32, seed: u64) -> AudioGlitchGenerator {
    AudioGlitchGenerator::with_seed(sample_rate, channels, duration, seed)
}

pub fn create_video_glitch_generator(width: u32, height: u32, fps: f32, frame_count: u32) -> VideoGlitchGenerator {
    VideoGlitchGenerator::new(width, height, fps, frame_count)
}

pub fn create_video_glitch_generator_with_seed(width: u32, height: u32, fps: f32, frame_count: u32, seed: u64) -> VideoGlitchGenerator {
    VideoGlitchGenerator::with_seed(width, height, fps, frame_count, seed)
}

pub fn create_data_glitch_generator(data_size: usize) -> DataGlitchGenerator {
    DataGlitchGenerator::new(data_size)
}

pub fn create_data_glitch_generator_with_seed(data_size: usize, seed: u64) -> DataGlitchGenerator {
    DataGlitchGenerator::with_seed(data_size, seed)
}

pub fn create_pattern_glitch_generator(pattern_size: usize, repeat_count: usize) -> PatternGlitchGenerator {
    PatternGlitchGenerator::new(pattern_size, repeat_count)
}

pub fn create_pattern_glitch_generator_with_seed(pattern_size: usize, repeat_count: usize, seed: u64) -> PatternGlitchGenerator {
    PatternGlitchGenerator::with_seed(pattern_size, repeat_count, seed)
}

pub fn create_custom_glitch_generator(name: String, parameters: HashMap<String, String>, generate_function: Box<dyn Fn(&HashMap<String, String>) -> Result<Vec<u8>> + Send + Sync>) -> CustomGlitchGenerator {
    CustomGlitchGenerator::new(name, parameters, generate_function)
}
