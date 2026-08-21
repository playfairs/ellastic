use ellastic_errors::{Result, EllasticError};
use ellastic_utils::{create_random_generator, create_noise_generator_with_seed, NoiseGenerator};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PatternEffect {
    RepeatingPattern { pattern: u32, period: u32, offset: u32 },
    RandomNoise { noise_type: NoiseType, intensity: f32 },
    GlitchPattern { glitch_type: GlitchType, frequency: f32, intensity: f32 },
    DataPattern { pattern_type: DataPatternType, data: Vec<u8>, intensity: f32 },
    Custom { custom_pattern: Box<dyn Fn(&mut ellastic_media::MediaProcessor) -> Result<()> + Send + Sync> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseType {
    Uniform,
    Gaussian,
    SaltPepper,
    Perlin,
    Simplex,
    Cellular,
    Fractal,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlitchType {
    ByteFlip,
    ByteSwap,
    ByteShift,
    ByteRotate,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataPatternType {
    Overlay,
    Replace,
    XOR,
    AND,
    OR,
    Custom,
}

#[derive(Debug, Clone)]
pub struct PatternGenerator {
    random_seed: Option<u64>,
}

impl PatternGenerator {
    pub fn new() -> Self {
        Self { random_seed: None }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self { random_seed: Some(seed) }
    }

    pub fn random_seed(&self) -> Option<u64> {
        self.random_seed
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.random_seed = Some(seed);
    }

    pub fn generate_pattern(&self, pattern_type: PatternType, size: usize) -> Result<Vec<u8>> {
        match pattern_type {
            PatternType::Repeating { pattern, period } => {
                self.generate_repeating_pattern(pattern, period, size)
            }
            PatternType::Noise { noise_type, intensity } => {
                self.generate_noise_pattern(noise_type, intensity, size)
            }
            PatternType::Glitch { glitch_type, frequency, intensity } => {
                self.generate_glitch_pattern(glitch_type, frequency, intensity, size)
            }
            PatternType::Data { data, pattern_type, intensity } => {
                self.generate_data_pattern(data, pattern_type, intensity, size)
            }
            PatternType::Custom { custom_generator } => {
                custom_generator(size)
            }
        }
    }

    fn generate_repeating_pattern(&self, pattern: u32, period: u32, size: usize) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(size);
        let pattern_bytes = pattern.to_le_bytes();

        for i in 0..size {
            let byte_index = (i % period as usize) % pattern_bytes.len();
            result.push(pattern_bytes[byte_index]);
        }

        Ok(result)
    }

    fn generate_noise_pattern(&self, noise_type: NoiseType, intensity: f32, size: usize) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(size);
        let mut rng = create_random_generator();

        match noise_type {
            NoiseType::Uniform => {
                for _ in 0..size {
                    result.push(rng.gen_range(0, 256) as u8);
                }
            }
            NoiseType::Gaussian => {
                for _ in 0..size {
                    let noise = (rng.gen_range(-1.0, 1.0) * 255.0 * intensity) as i8;
                    result.push(noise as u8);
                }
            }
            NoiseType::SaltPepper => {
                for _ in 0..size {
                    result.push(if rng.gen_range(0.0, 1.0) < intensity { 0 } else { 255 });
                }
            }
            NoiseType::Perlin => {
                let mut noise_gen = create_noise_generator_with_seed(self.random_seed.unwrap_or(42));
                for i in 0..size {
                    let noise = noise_gen.perlin(i as f32 / 100.0, 0.0, 0.0) * 255.0 * intensity;
                    result.push(noise.clamp(0.0, 255.0) as u8);
                }
            }
            NoiseType::Simplex => {
                let mut noise_gen = create_noise_generator_with_seed(self.random_seed.unwrap_or(42));
                for i in 0..size {
                    let noise = noise_gen.simplex(i as f32 / 100.0, 0.0, 0.0) * 255.0 * intensity;
                    result.push(noise.clamp(0.0, 255.0) as u8);
                }
            }
            NoiseType::Cellular => {
                let mut noise_gen = create_noise_generator_with_seed(self.random_seed.unwrap_or(42));
                for i in 0..size {
                    let noise = noise_gen.cellular(i as f32 / 100.0, 0.0, 0.0) * 255.0 * intensity;
                    result.push(noise.clamp(0.0, 255.0) as u8);
                }
            }
            NoiseType::Fractal => {
                let mut noise_gen = create_noise_generator_with_seed(self.random_seed.unwrap_or(42));
                for i in 0..size {
                    let noise = noise_gen.fractal(i as f32 / 100.0, 0.0, 0.0) * 255.0 * intensity;
                    result.push(noise.clamp(0.0, 255.0) as u8);
                }
            }
            NoiseType::Custom => {
                return Err(EllasticError::UnsupportedOperation("Custom noise pattern not implemented".to_string()));
            }
        }

        Ok(result)
    }

    fn generate_glitch_pattern(&self, glitch_type: GlitchType, frequency: f32, intensity: f32, size: usize) -> Result<Vec<u8>> {
        let mut result = vec![0u8; size];
        let mut rng = create_random_generator();

        let glitch_count = (size as f32 * frequency) as usize;

        for _ in 0..glitch_count {
            let pos = rng.gen_range(0, size as u64) as usize;
            let glitch_size = (intensity * 64.0) as usize;

            if pos + glitch_size <= size {
                match glitch_type {
                    GlitchType::ByteFlip => {
                        for i in pos..pos + glitch_size {
                            result[i] ^= 0xFF;
                        }
                    }
                    GlitchType::ByteSwap => {
                        for i in (pos..pos + glitch_size).step_by(2) {
                            if i + 1 < pos + glitch_size {
                                result.swap(i, i + 1);
                            }
                        }
                    }
                    GlitchType::ByteShift => {
                        for i in pos..pos + glitch_size {
                            result[i] <<= 1;
                        }
                    }
                    GlitchType::ByteRotate => {
                        for i in pos..pos + glitch_size {
                            result[i] = result[i].rotate_left(1);
                        }
                    }
                    GlitchType::Custom => {
                        return Err(EllasticError::UnsupportedOperation("Custom glitch pattern not implemented".to_string()));
                    }
                }
            }
        }

        Ok(result)
    }

    fn generate_data_pattern(&self, data: Vec<u8>, pattern_type: DataPatternType, intensity: f32, size: usize) -> Result<Vec<u8>> {
        let mut result = vec![0u8; size];

        match pattern_type {
            DataPatternType::Overlay => {
                for i in 0..size {
                    let data_index = i % data.len();
                    result[i] = result[i].wrapping_add(data[data_index]);
                }
            }
            DataPatternType::Replace => {
                for i in 0..size {
                    let data_index = i % data.len();
                    result[i] = data[data_index];
                }
            }
            DataPatternType::XOR => {
                for i in 0..size {
                    let data_index = i % data.len();
                    result[i] ^= data[data_index];
                }
            }
            DataPatternType::AND => {
                for i in 0..size {
                    let data_index = i % data.len();
                    result[i] &= data[data_index];
                }
            }
            DataPatternType::OR => {
                for i in 0..size {
                    let data_index = i % data.len();
                    result[i] |= data[data_index];
                }
            }
            DataPatternType::Custom => {
                return Err(EllasticError::UnsupportedOperation("Custom data pattern not implemented".to_string()));
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub enum PatternType {
    Repeating { pattern: u32, period: u32 },
    Noise { noise_type: NoiseType, intensity: f32 },
    Glitch { glitch_type: GlitchType, frequency: f32, intensity: f32 },
    Data { data: Vec<u8>, pattern_type: DataPatternType, intensity: f32 },
    Custom { custom_generator: Box<dyn Fn(usize) -> Result<Vec<u8>> + Send + Sync> },
}

#[derive(Debug, Clone)]
pub struct PatternProcessor {
    generator: PatternGenerator,
}

impl PatternProcessor {
    pub fn new() -> Self {
        Self {
            generator: PatternGenerator::new(),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            generator: PatternGenerator::with_seed(seed),
        }
    }

    pub fn generator(&self) -> &PatternGenerator {
        &self.generator
    }

    pub fn generator_mut(&mut self) -> &mut PatternGenerator {
        &mut self.generator
    }

    pub fn apply_pattern(&mut self, data: &mut [u8], pattern: &PatternEffect) -> Result<()> {
        match pattern {
            PatternEffect::RepeatingPattern { pattern, period, offset } => {
                self.apply_repeating_pattern(data, *pattern, *period, *offset)?;
            }
            PatternEffect::RandomNoise { noise_type, intensity } => {
                self.apply_random_noise(data, *noise_type, *intensity)?;
            }
            PatternEffect::GlitchPattern { glitch_type, frequency, intensity } => {
                self.apply_glitch_pattern(data, *glitch_type, *frequency, *intensity)?;
            }
            PatternEffect::DataPattern { pattern_type, data: pattern_data, intensity } => {
                self.apply_data_pattern(data, *pattern_type, pattern_data, *intensity)?;
            }
            PatternEffect::Custom { custom_pattern } => {
This would need access to MediaProcessor, not just byte data
                return Err(EllasticError::UnsupportedOperation("Custom pattern requires MediaProcessor".to_string()));
            }
        }
        Ok(())
    }

    fn apply_repeating_pattern(&mut self, data: &mut [u8], pattern: u32, period: u32, offset: u32) -> Result<()> {
        let pattern_bytes = pattern.to_le_bytes();

        for i in (offset as usize..data.len()).step_by(period as usize) {
            for j in 0..pattern_bytes.len().min(data.len() - i) {
                data[i + j] = pattern_bytes[j];
            }
        }

        Ok(())
    }

    fn apply_random_noise(&mut self, data: &mut [u8], noise_type: NoiseType, intensity: f32) -> Result<()> {
        let mut rng = create_random_generator();

        match noise_type {
            NoiseType::Uniform => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        data[pos] = rng.gen_range(0, 256) as u8;
                    }
                }
            }
            NoiseType::Gaussian => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        let noise = (rng.gen_range(-1.0, 1.0) * 50.0) as i8;
                        data[pos] = data[pos].wrapping_add(noise as u8);
                    }
                }
            }
            NoiseType::SaltPepper => {
                let noise_count = (data.len() as f32 * intensity) as usize;
                for _ in 0..noise_count {
                    let pos = rng.gen_range(0, data.len() as u64) as usize;
                    if pos < data.len() {
                        data[pos] = if rng.gen_range(0.0, 1.0) < 0.5 { 0 } else { 255 };
                    }
                }
            }
            NoiseType::Perlin => {
                let mut noise_gen = create_noise_generator_with_seed(self.generator.random_seed.unwrap_or(42));
                let noise_factor = intensity * 50.0;
                for (i, byte) in data.iter_mut().enumerate() {
                    let noise = noise_gen.perlin(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
                    *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
                }
            }
            NoiseType::Simplex => {
                let mut noise_gen = create_noise_generator_with_seed(self.generator.random_seed.unwrap_or(42));
                let noise_factor = intensity * 50.0;
                for (i, byte) in data.iter_mut().enumerate() {
                    let noise = noise_gen.simplex(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
                    *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
                }
            }
            NoiseType::Cellular => {
                let mut noise_gen = create_noise_generator_with_seed(self.generator.random_seed.unwrap_or(42));
                let noise_factor = intensity * 50.0;
                for (i, byte) in data.iter_mut().enumerate() {
                    let noise = noise_gen.cellular(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
                    *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
                }
            }
            NoiseType::Fractal => {
                let mut noise_gen = create_noise_generator_with_seed(self.generator.random_seed.unwrap_or(42));
                let noise_factor = intensity * 50.0;
                for (i, byte) in data.iter_mut().enumerate() {
                    let noise = noise_gen.fractal(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
                    *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
                }
            }
            NoiseType::Custom => {
                return Err(EllasticError::UnsupportedOperation("Custom noise pattern not implemented".to_string()));
            }
        }

        Ok(())
    }

    fn apply_glitch_pattern(&mut self, data: &mut [u8], glitch_type: GlitchType, frequency: f32, intensity: f32) -> Result<()> {
        let mut rng = create_random_generator();

        let glitch_count = (data.len() as f32 * frequency) as usize;

        for _ in 0..glitch_count {
            let pos = rng.gen_range(0, data.len() as u64) as usize;
            let glitch_size = (intensity * 64.0) as usize;

            if pos + glitch_size <= data.len() {
                match glitch_type {
                    GlitchType::ByteFlip => {
                        for i in pos..pos + glitch_size {
                            data[i] ^= 0xFF;
                        }
                    }
                    GlitchType::ByteSwap => {
                        for i in (pos..pos + glitch_size).step_by(2) {
                            if i + 1 < pos + glitch_size {
                                data.swap(i, i + 1);
                            }
                        }
                    }
                    GlitchType::ByteShift => {
                        for i in pos..pos + glitch_size {
                            data[i] <<= 1;
                        }
                    }
                    GlitchType::ByteRotate => {
                        for i in pos..pos + glitch_size {
                            data[i] = data[i].rotate_left(1);
                        }
                    }
                    GlitchType::Custom => {
                        return Err(EllasticError::UnsupportedOperation("Custom glitch pattern not implemented".to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    fn apply_data_pattern(&mut self, data: &mut [u8], pattern_type: DataPatternType, pattern_data: &[u8], intensity: f32) -> Result<()> {
        match pattern_type {
            DataPatternType::Overlay => {
                let overlay_count = (data.len() as f32 * intensity) as usize;
                for i in 0..overlay_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] = data[i].wrapping_add(pattern_data[data_index]);
                }
            }
            DataPatternType::Replace => {
                let replace_count = (data.len() as f32 * intensity) as usize;
                for i in 0..replace_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] = pattern_data[data_index];
                }
            }
            DataPatternType::XOR => {
                let xor_count = (data.len() as f32 * intensity) as usize;
                for i in 0..xor_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] ^= pattern_data[data_index];
                }
            }
            DataPatternType::AND => {
                let and_count = (data.len() as f32 * intensity) as usize;
                for i in 0..and_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] &= pattern_data[data_index];
                }
            }
            DataPatternType::OR => {
                let or_count = (data.len() as f32 * intensity) as usize;
                for i in 0..or_count.min(data.len()) {
                    let data_index = i % pattern_data.len();
                    data[i] |= pattern_data[data_index];
                }
            }
            DataPatternType::Custom => {
                return Err(EllasticError::UnsupportedOperation("Custom data pattern not implemented".to_string()));
            }
        }

        Ok(())
    }

    pub fn clone(&self) -> PatternProcessor {
        PatternProcessor {
            generator: PatternGenerator::with_seed(self.generator.random_seed.unwrap_or(42)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatternLibrary {
    patterns: HashMap<String, PatternType>,
}

impl PatternLibrary {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    pub fn add_pattern(&mut self, name: String, pattern: PatternType) {
        self.patterns.insert(name, pattern);
    }

    pub fn remove_pattern(&mut self, name: &str) -> Option<PatternType> {
        self.patterns.remove(name)
    }

    pub fn get_pattern(&self, name: &str) -> Option<&PatternType> {
        self.patterns.get(name)
    }

    pub fn list_patterns(&self) -> Vec<&String> {
        self.patterns.keys().collect()
    }

    pub fn clear(&mut self) {
        self.patterns.clear();
    }

    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub fn clone(&self) -> PatternLibrary {
        PatternLibrary {
            patterns: self.patterns.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatternPreset {
    pub name: String,
    pub description: String,
    pub pattern: PatternType,
    pub parameters: HashMap<String, String>,
}

impl PatternPreset {
    pub fn new(name: String, description: String, pattern: PatternType, parameters: HashMap<String, String>) -> Self {
        Self { name, description, pattern, parameters }
    }

    pub fn clone(&self) -> PatternPreset {
        PatternPreset {
            name: self.name.clone(),
            description: self.description.clone(),
            pattern: self.pattern.clone(),
            parameters: self.parameters.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatternPresetLibrary {
    presets: HashMap<String, PatternPreset>,
}

impl PatternPresetLibrary {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
        }
    }

    pub fn add_preset(&mut self, preset: PatternPreset) {
        self.presets.insert(preset.name.clone(), preset);
    }

    pub fn remove_preset(&mut self, name: &str) -> Option<PatternPreset> {
        self.presets.remove(name)
    }

    pub fn get_preset(&self, name: &str) -> Option<&PatternPreset> {
        self.presets.get(name)
    }

    pub fn list_presets(&self) -> Vec<&String> {
        self.presets.keys().collect()
    }

    pub fn clear(&mut self) {
        self.presets.clear();
    }

    pub fn len(&self) -> usize {
        self.presets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.presets.is_empty()
    }

    pub fn clone(&self) -> PatternPresetLibrary {
        PatternPresetLibrary {
            presets: self.presets.clone(),
        }
    }

    pub fn load_default_presets(&mut self) -> Result<()> {
        self.add_preset(PatternPreset::new(
            "digital_glitch".to_string(),
            "Digital glitch pattern with byte flips".to_string(),
            PatternType::Glitch {
                glitch_type: GlitchType::ByteFlip,
                frequency: 0.1,
                intensity: 0.5,
            },
            HashMap::new(),
        ));

        self.add_preset(PatternPreset::new(
            "random_noise".to_string(),
            "Uniform random noise".to_string(),
            PatternType::Noise {
                noise_type: NoiseType::Uniform,
                intensity: 0.3,
            },
            HashMap::new(),
        ));

        self.add_preset(PatternPreset::new(
            "perlin_noise".to_string(),
            "Perlin noise pattern".to_string(),
            PatternType::Noise {
                noise_type: NoiseType::Perlin,
                intensity: 0.4,
            },
            HashMap::new(),
        ));

        self.add_preset(PatternPreset::new(
            "repeating_pattern".to_string(),
            "Simple repeating pattern".to_string(),
            PatternType::Repeating {
                pattern: 0xDEADBEEF,
                period: 16,
            },
            HashMap::new(),
        ));

        self.add_preset(PatternPreset::new(
            "byte_swap_glitch".to_string(),
            "Byte swapping glitch".to_string(),
            PatternType::Glitch {
                glitch_type: GlitchType::ByteSwap,
                frequency: 0.05,
                intensity: 0.3,
            },
            HashMap::new(),
        ));

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PatternAnalyzer {
    data: Vec<u8>,
}

impl PatternAnalyzer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn analyze_entropy(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }

        let mut frequency = [0.0; 256];
        for byte in &self.data {
            frequency[*byte as usize] += 1.0;
        }

        let len = self.data.len() as f32;
        let mut entropy = 0.0;

        for count in frequency.iter() {
            if *count > 0.0 {
                let probability = *count / len;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    pub fn analyze_repetition(&self) -> f32 {
        if self.data.len() < 2 {
            return 0.0;
        }

        let mut repetitions = 0;
        for i in 1..self.data.len() {
            if self.data[i] == self.data[i - 1] {
                repetitions += 1;
            }
        }

        repetitions as f32 / (self.data.len() - 1) as f32
    }

    pub fn analyze_patterns(&self) -> Vec<(Vec<u8>, f32)> {
        let mut patterns = HashMap::new();

        for pattern_length in 1..=8.min(self.data.len()) {
            for i in 0..=(self.data.len() - pattern_length) {
                let pattern = self.data[i..i + pattern_length].to_vec();
                let count = patterns.entry(pattern).or_insert(0);
                *count += 1;
            }
        }

        let mut pattern_list: Vec<(Vec<u8>, f32)> = patterns
            .into_iter()
            .map(|(pattern, count)| (pattern, count as f32 / self.data.len() as f32))
            .collect();

        pattern_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pattern_list
    }

    pub fn analyze_correlation(&self) -> f32 {
        if self.data.len() < 2 {
            return 0.0;
        }

        let mean = self.data.iter().sum::<u8>() as f32 / self.data.len() as f32;
        let mut covariance = 0.0;
        let mut variance = 0.0;

        for i in 1..self.data.len() {
            let diff_current = self.data[i] as f32 - mean;
            let diff_previous = self.data[i - 1] as f32 - mean;
            covariance += diff_current * diff_previous;
            variance += diff_current * diff_current;
        }

        if variance == 0.0 {
            0.0
        } else {
            covariance / variance
        }
    }

    pub fn analyze_frequency(&self) -> [f32; 256] {
        let mut frequency = [0.0; 256];
        for byte in &self.data {
            frequency[*byte as usize] += 1.0;
        }

        let len = self.data.len() as f32;
        for freq in frequency.iter_mut() {
            *freq /= len;
        }

        frequency
    }

    pub fn detect_anomalies(&self) -> Vec<usize> {
        let frequency = self.analyze_frequency();
        let threshold = 1.0 / 256.0 * 0.5;
        let mut anomalies = Vec::new();

        for (i, byte) in self.data.iter().enumerate() {
            if frequency[*byte as usize] < threshold {
                anomalies.push(i);
            }
        }

        anomalies
    }

    pub fn clone(&self) -> PatternAnalyzer {
        PatternAnalyzer {
            data: self.data.clone(),
        }
    }
}

pub fn create_pattern_generator() -> PatternGenerator {
    PatternGenerator::new()
}

pub fn create_pattern_generator_with_seed(seed: u64) -> PatternGenerator {
    PatternGenerator::with_seed(seed)
}

pub fn create_pattern_processor() -> PatternProcessor {
    PatternProcessor::new()
}

pub fn create_pattern_processor_with_seed(seed: u64) -> PatternProcessor {
    PatternProcessor::with_seed(seed)
}

pub fn create_pattern_library() -> PatternLibrary {
    PatternLibrary::new()
}

pub fn create_pattern_preset_library() -> PatternPresetLibrary {
    PatternPresetLibrary::new()
}

pub fn create_pattern_analyzer(data: Vec<u8>) -> PatternAnalyzer {
    PatternAnalyzer::new(data)
}

pub fn create_repeating_pattern_effect(pattern: u32, period: u32, offset: u32) -> PatternEffect {
    PatternEffect::RepeatingPattern { pattern, period, offset }
}

pub fn create_random_noise_effect(noise_type: NoiseType, intensity: f32) -> PatternEffect {
    PatternEffect::RandomNoise { noise_type, intensity }
}

pub fn create_glitch_pattern_effect(glitch_type: GlitchType, frequency: f32, intensity: f32) -> PatternEffect {
    PatternEffect::GlitchPattern { glitch_type, frequency, intensity }
}

pub fn create_data_pattern_effect(pattern_type: DataPatternType, data: Vec<u8>, intensity: f32) -> PatternEffect {
    PatternEffect::DataPattern { pattern_type, data, intensity }
}
