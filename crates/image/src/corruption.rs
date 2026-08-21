use crate::{
  ImageData,
  ImageProcessor,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use ellastic_utils::{
  NoiseGenerator,
  RandomGenerator,
  create_noise_generator_with_seed,
  create_random_generator,
};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct ImageCorruptionEngine {
  image: ImageProcessor,
  rng: RandomGenerator,
}

impl ImageCorruptionEngine {
  pub fn new(image: ImageProcessor) -> Self {
    Self {
      image,
      rng: create_random_generator(),
    }
  }

  pub fn with_seed(image: ImageProcessor, seed: u64) -> Self {
    Self {
      image,
      rng: create_random_generator_with_seed(seed),
    }
  }

  pub fn image(&self) -> &ImageProcessor {
    &self.image
  }

  pub fn image_mut(&mut self) -> &mut ImageProcessor {
    &mut self.image
  }

  pub fn into_image(self) -> ImageProcessor {
    self.image
  }

  pub fn random_pixel_corruption(&mut self, intensity: f64) -> Result<()> {
    let pixel_count = self.image.pixel_count();
    let corruption_count = (pixel_count as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let x = self.rng.gen_range(0, self.image.width() as u64) as u32;
      let y = self.rng.gen_range(0, self.image.height() as u64) as u32;

      let channels = self.image.channels();
      let mut new_pixel = Vec::with_capacity(channels as usize);
      for _ in 0..channels {
        new_pixel.push(self.rng.gen_u8());
      }

      self.image.set_pixel(x, y, &new_pixel)?;
    }

    Ok(())
  }

  pub fn channel_corruption(&mut self, channel: u8, intensity: f64) -> Result<()> {
    if channel >= self.image.channels() {
      return Err(EllasticError::InvalidParameter(
        "Channel out of range".to_string(),
      ));
    }

    let channel_data = self.image.get_channel(channel)?;
    let corruption_count = (channel_data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let position = self.rng.gen_range(0, channel_data.len() as u64) as usize;
      let new_value = self.rng.gen_u8();

      let x = (position as u32 % self.image.width()) as u32;
      let y = (position as u32 / self.image.width()) as u32;
      self.image.set_pixel_value(x, y, channel, new_value)?;
    }

    Ok(())
  }

  pub fn scanline_corruption(&mut self, intensity: f64) -> Result<()> {
    let height = self.image.height();
    let corruption_count = (height as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let y = self.rng.gen_range(0, height as u64) as u32;
      let row_start = (y * self.image.width() * self.image.channels() as u32) as usize;
      let row_end = row_start + (self.image.width() * self.image.channels() as u32) as usize;

      let data = self.image.data_mut();
      for i in row_start..row_end {
        data[i] = self.rng.gen_u8();
      }
    }

    Ok(())
  }

  pub fn column_corruption(&mut self, intensity: f64) -> Result<()> {
    let width = self.image.width();
    let corruption_count = (width as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let x = self.rng.gen_range(0, width as u64) as u32;

      for y in 0..self.image.height() {
        let channels = self.image.channels();
        for c in 0..channels {
          let new_value = self.rng.gen_u8();
          self.image.set_pixel_value(x, y, c, new_value)?;
        }
      }
    }

    Ok(())
  }

  pub fn block_corruption(&mut self, block_size: u32, intensity: f64) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let blocks_x = (width + block_size - 1) / block_size;
    let blocks_y = (height + block_size - 1) / block_size;
    let total_blocks = blocks_x * blocks_y;
    let corruption_count = (total_blocks as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let block_x = self.rng.gen_range(0, blocks_x as u64) as u32;
      let block_y = self.rng.gen_range(0, blocks_y as u64) as u32;

      let start_x = block_x * block_size;
      let start_y = block_y * block_size;
      let end_x = std::cmp::min(start_x + block_size, width);
      let end_y = std::cmp::min(start_y + block_size, height);

      for y in start_y..end_y {
        for x in start_x..end_x {
          let channels = self.image.channels();
          let mut new_pixel = Vec::with_capacity(channels as usize);
          for _ in 0..channels {
            new_pixel.push(self.rng.gen_u8());
          }
          self.image.set_pixel(x, y, &new_pixel)?;
        }
      }
    }

    Ok(())
  }

  pub fn bit_flip_corruption(&mut self, intensity: f64) -> Result<()> {
    let data = self.image.data_mut();
    let flip_count = (data.len() as f64 * intensity.clamp(0.0, 1.0) * 8.0) as usize;

    for _ in 0..flip_count {
      let byte_index = self.rng.gen_range(0, data.len() as u64) as usize;
      let bit_position = self.rng.gen_range(0, 8);
      data[byte_index] ^= 1 << bit_position;
    }

    Ok(())
  }

  pub fn byte_swap_corruption(&mut self, intensity: f64) -> Result<()> {
    let data = self.image.data_mut();
    let swap_count = (data.len() as f64 * intensity.clamp(0.0, 1.0) / 2.0) as usize;

    for _ in 0..swap_count {
      let pos1 = self.rng.gen_range(0, data.len() as u64) as usize;
      let pos2 = self.rng.gen_range(0, data.len() as u64) as usize;

      if pos1 != pos2 {
        data.swap(pos1, pos2);
      }
    }

    Ok(())
  }

  pub fn color_channel_corruption(
    &mut self,
    channel_corruption_type: ColorChannelCorruption,
    intensity: f64,
  ) -> Result<()> {
    match channel_corruption_type {
      ColorChannelCorruption::Red => self.channel_corruption(0, intensity),
      ColorChannelCorruption::Green => self.channel_corruption(1, intensity),
      ColorChannelCorruption::Blue => self.channel_corruption(2, intensity),
      ColorChannelCorruption::Alpha => {
        if self.image.channels() >= 4 {
          self.channel_corruption(3, intensity)
        } else {
          Ok(())
        }
      }
      ColorChannelCorruption::Random => {
        let channel = self.rng.gen_range(0, self.image.channels() as u64) as u8;
        self.channel_corruption(channel, intensity)
      }
    }
  }

  pub fn color_shift_corruption(&mut self, shift_amount: i8, intensity: f64) -> Result<()> {
    let data = self.image.data_mut();
    let shift_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..shift_count {
      let position = self.rng.gen_range(0, data.len() as u64) as usize;
      data[position] = data[position].wrapping_add(shift_amount);
    }

    Ok(())
  }

  pub fn noise_injection(&mut self, noise_type: NoiseType, intensity: f64) -> Result<()> {
    let noise_gen = create_noise_generator_with_seed(self.rng.gen_seed());
    let data = self.image.data_mut();

    noise_gen.apply_to_bytes(data, intensity);
    Ok(())
  }

  pub fn compression_artifact(
    &mut self,
    artifact_type: CompressionArtifact,
    intensity: f64,
  ) -> Result<()> {
    match artifact_type {
      CompressionArtifact::Blockiness { block_size } => {
        let data = self.image.data_mut();
        let width = self.image.width();
        let height = self.image.height();
        let channels = self.image.channels();

        for y in (0..height).step_by(block_size as usize) {
          for x in (0..width).step_by(block_size as usize) {
            let end_x = std::cmp::min(x + block_size, width);
            let end_y = std::cmp::min(y + block_size, height);

            let mut sum = vec![0u32; channels as usize];
            let mut count = 0;

            for py in y..end_y {
              for px in x..end_x {
                for c in 0..channels {
                  if let Some(value) = self.image.get_pixel_value(px, py, c) {
                    sum[c as usize] += value as u32;
                  }
                }
                count += 1;
              }
            }

            for py in y..end_y {
              for px in x..end_x {
                for c in 0..channels {
                  let avg = (sum[c as usize] / count) as u8;
                  self.image.set_pixel_value(px, py, c, avg)?;
                }
              }
            }
          }
        }
      }
      CompressionArtifact::ColorBanding { levels } => {
        let data = self.image.data_mut();
        for byte in data {
          *byte = (*byte as f32 / 255.0 * levels as f32).round() as u8 * 255 / levels;
        }
      }
      CompressionArtifact::Quantization { step_size } => {
        let data = self.image.data_mut();
        for byte in data {
          *byte = (*byte as u32 / step_size * step_size) as u8;
        }
      }
    }

    Ok(())
  }

  pub fn databending_effect(&mut self, effect: DatabendingEffect, intensity: f64) -> Result<()> {
    let data = self.image.data_mut();
    let affected_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    match effect {
      DatabendingEffect::XOR { key } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] ^= key;
        }
      }
      DatabendingEffect::XORWithKey { key } => {
        if key.is_empty() {
          return Ok(());
        }

        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] ^= key[position % key.len()];
        }
      }
      DatabendingEffect::Add { value } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] = data[position].wrapping_add(value);
        }
      }
      DatabendingEffect::Subtract { value } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] = data[position].wrapping_sub(value);
        }
      }
      DatabendingEffect::Multiply { value } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] = data[position].wrapping_mul(value);
        }
      }
      DatabendingEffect::ShiftLeft { bits } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] <<= bits;
        }
      }
      DatabendingEffect::ShiftRight { bits } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] >>= bits;
        }
      }
      DatabendingEffect::Complement => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] = !data[position];
        }
      }
    }

    Ok(())
  }

  pub fn geometric_corruption(
    &mut self,
    corruption_type: GeometricCorruption,
    intensity: f64,
  ) -> Result<()> {
    match corruption_type {
      GeometricCorruption::Pixelation { block_size } => {
        let width = self.image.width();
        let height = self.image.height();
        let channels = self.image.channels();

        for y in (0..height).step_by(block_size as usize) {
          for x in (0..width).step_by(block_size as usize) {
            let end_x = std::cmp::min(x + block_size, width);
            let end_y = std::cmp::min(y + block_size, height);

            if let Some(pixel) = self.image.get_pixel(x, y) {
              for py in y..end_y {
                for px in x..end_x {
                  self.image.set_pixel(px, py, pixel)?;
                }
              }
            }
          }
        }
      }
      GeometricCorruption::GlitchBlocks { block_size } => {
        let width = self.image.width();
        let height = self.image.height();

        let block_count =
          ((width as f64 * height as f64) / (block_size * block_size) as f64) as usize;
        let glitch_count = (block_count as f64 * intensity) as usize;

        for _ in 0..glitch_count {
          let block_x = self.rng.gen_range(0, (width / block_size) as u64) as u32;
          let block_y = self.rng.gen_range(0, (height / block_size) as u64) as u32;

          let start_x = block_x * block_size;
          let start_y = block_y * block_size;
          let end_x = std::cmp::min(start_x + block_size, width);
          let end_y = std::cmp::min(start_y + block_size, height);

          for py in start_y..end_y {
            for px in start_x..end_x {
              let channels = self.image.channels();
              let mut glitch_pixel = Vec::with_capacity(channels as usize);
              for _ in 0..channels {
                glitch_pixel.push(self.rng.gen_u8());
              }
              self.image.set_pixel(px, py, &glitch_pixel)?;
            }
          }
        }
      }
      GeometricCorruption::DataMosaic { tile_size } => {
        let width = self.image.width();
        let height = self.image.height();
        let tiles_x = (width + tile_size - 1) / tile_size;
        let tiles_y = (height + tile_size - 1) / tile_size;

        let mut tiles = Vec::new();
        for ty in 0..tiles_y {
          for tx in 0..tiles_x {
            let start_x = tx * tile_size;
            let start_y = ty * tile_size;
            let end_x = std::cmp::min(start_x + tile_size, width);
            let end_y = std::cmp::min(start_y + tile_size, height);

            let mut tile_data = Vec::new();
            for py in start_y..end_y {
              for px in start_x..end_x {
                if let Some(pixel) = self.image.get_pixel(px, py) {
                  tile_data.push(pixel.to_vec());
                }
              }
            }
            tiles.push(tile_data);
          }
        }

        self.rng.shuffle(&mut tiles);

        let mut tile_index = 0;
        for ty in 0..tiles_y {
          for tx in 0..tiles_x {
            let start_x = tx * tile_size;
            let start_y = ty * tile_size;
            let end_x = std::cmp::min(start_x + tile_size, width);
            let end_y = std::cmp::min(start_y + tile_size, height);

            if let Some(tile_data) = tiles.get(tile_index) {
              let mut data_index = 0;
              for py in start_y..end_y {
                for px in start_x..end_x {
                  if let Some(pixel) = tile_data.get(data_index) {
                    self.image.set_pixel(px, py, pixel)?;
                    data_index += 1;
                  }
                }
              }
            }
            tile_index += 1;
          }
        }
      }
    }

    Ok(())
  }

  pub fn temporal_corruption(
    &mut self,
    corruption_type: TemporalCorruption,
    intensity: f64,
  ) -> Result<()> {
    match corruption_type {
      TemporalCorruption::FrameDrop { drop_rate } => {
        let height = self.image.height();
        let drop_count = (height as f64 * drop_rate.clamp(0.0, 1.0)) as usize;

        for _ in 0..drop_count {
          let y = self.rng.gen_range(0, height as u64) as u32;
          if let Some(row) = self.image.get_row_mut(y) {
            for byte in row {
              *byte = self.rng.gen_u8();
            }
          }
        }
      }
      TemporalCorruption::Interference { frequency } => {
        let data = self.image.data_mut();
        let interference_count = (data.len() as f64 * intensity) as usize;

        for i in 0..interference_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          let interference = (self.rng.gen_range(-128, 128) as f64 * frequency) as i8;
          data[position] = data[position].wrapping_add(interference as u8);
        }
      }
    }

    Ok(())
  }

  pub fn recursive_corruption(&mut self, depth: usize, intensity: f64) -> Result<()> {
    if depth == 0 {
      return Ok(());
    }

    let corruption_types = [
      ImageCorruptionType::RandomPixel,
      ImageCorruptionType::BitFlip,
      ImageCorruptionType::Scanline,
      ImageCorruptionType::Block,
    ];

    for _ in 0..depth {
      let corruption_type =
        corruption_types[self.rng.gen_range(0, corruption_types.len() as u64) as usize];

      match corruption_type {
        ImageCorruptionType::RandomPixel => self.random_pixel_corruption(intensity)?,
        ImageCorruptionType::BitFlip => self.bit_flip_corruption(intensity)?,
        ImageCorruptionType::Scanline => self.scanline_corruption(intensity)?,
        ImageCorruptionType::Block => self.block_corruption(16, intensity)?,
      }
    }

    Ok(())
  }

  pub fn layered_corruption(&mut self, layers: &[CorruptionLayer]) -> Result<()> {
    for layer in layers {
      match layer.corruption_type {
        ImageCorruptionType::RandomPixel => self.random_pixel_corruption(layer.intensity)?,
        ImageCorruptionType::BitFlip => self.bit_flip_corruption(layer.intensity)?,
        ImageCorruptionType::Scanline => self.scanline_corruption(layer.intensity)?,
        ImageCorruptionType::Block => self.block_corruption(32, layer.intensity)?,
        ImageCorruptionType::Channel => {
          self.channel_corruption(layer.channel.unwrap_or(0), layer.intensity)?
        }
        ImageCorruptionType::Noise => self.noise_injection(NoiseType::Uniform, layer.intensity)?,
        ImageCorruptionType::Databending => {
          self.databending_effect(DatabendingEffect::XOR { key: 42 }, layer.intensity)?;
        }
        ImageCorruptionType::Geometric => {
          self.geometric_corruption(
            GeometricCorruption::Pixelation { block_size: 8 },
            layer.intensity,
          )?;
        }
      }
    }

    Ok(())
  }

  pub fn controlled_corruption(&mut self, parameters: &ImageCorruptionParameters) -> Result<()> {
    if parameters.random_pixel_corruption {
      self.random_pixel_corruption(parameters.random_pixel_intensity)?;
    }

    if parameters.bit_flip_corruption {
      self.bit_flip_corruption(parameters.bit_flip_intensity)?;
    }

    if parameters.scanline_corruption {
      self.scanline_corruption(parameters.scanline_intensity)?;
    }

    if parameters.block_corruption {
      self.block_corruption(parameters.block_size, parameters.block_intensity)?;
    }

    if parameters.channel_corruption {
      if let Some(channel) = parameters.corruption_channel {
        self.channel_corruption(channel, parameters.channel_intensity)?;
      }
    }

    if parameters.noise_injection {
      self.noise_injection(parameters.noise_type, parameters.noise_intensity)?;
    }

    if parameters.compression_artifacts {
      self.compression_artifact(
        CompressionArtifact::Blockiness { block_size: 16 },
        parameters.artifact_intensity,
      )?;
    }

    Ok(())
  }

  pub fn corruption_statistics(&self) -> ImageCorruptionStats {
    let data = self.image.data();
    let original_entropy = self.calculate_entropy(data);
    let histogram = self.image.histogram();

    let zero_pixels = histogram[0];
    let max_pixels = histogram[255];
    let unique_colors = self.count_unique_colors();

    let mut runs = Vec::new();
    if !data.is_empty() {
      let mut current_value = data[0];
      let mut current_run = 1;

      for &byte in data.iter().skip(1) {
        if byte == current_value {
          current_run += 1;
        } else {
          runs.push(current_run);
          current_value = byte;
          current_run = 1;
        }
      }
      runs.push(current_run);
    }

    let avg_run_length = if runs.is_empty() {
      0.0
    } else {
      runs.iter().sum::<usize>() as f64 / runs.len() as f64
    };
    let max_run_length = runs.iter().max().copied().unwrap_or(0);

    ImageCorruptionStats {
      entropy: original_entropy,
      zero_pixels,
      max_pixels,
      unique_colors,
      total_pixels: self.image.pixel_count(),
      avg_run_length,
      max_run_length,
      run_count: runs.len(),
    }
  }

  fn calculate_entropy(&self, data: &[u8]) -> f64 {
    if data.is_empty() {
      return 0.0;
    }

    let mut frequency = [0u64; 256];
    for &byte in data {
      frequency[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &frequency {
      if count > 0 {
        let probability = count as f64 / len;
        entropy -= probability * probability.log2();
      }
    }

    entropy
  }

  fn count_unique_colors(&self) -> usize {
    let mut unique_colors = std::collections::HashSet::new();
    let channels = self.image.channels() as usize;

    for chunk in self.image.data().chunks(channels) {
      unique_colors.insert(chunk.to_vec());
    }

    unique_colors.len()
  }
}

#[derive(Debug, Clone)]
pub enum NoiseType {
  Uniform,
  Gaussian,
  Perlin,
  SaltAndPepper,
}

#[derive(Debug, Clone)]
pub enum ColorChannelCorruption {
  Red,
  Green,
  Blue,
  Alpha,
  Random,
}

#[derive(Debug, Clone)]
pub enum CompressionArtifact {
  Blockiness { block_size: u32 },
  ColorBanding { levels: u8 },
  Quantization { step_size: u8 },
}

#[derive(Debug, Clone)]
pub enum DatabendingEffect {
  XOR { key: u8 },
  XORWithKey { key: Vec<u8> },
  Add { value: u8 },
  Subtract { value: u8 },
  Multiply { value: u8 },
  ShiftLeft { bits: u8 },
  ShiftRight { bits: u8 },
  Complement,
}

#[derive(Debug, Clone)]
pub enum GeometricCorruption {
  Pixelation { block_size: u32 },
  GlitchBlocks { block_size: u32 },
  DataMosaic { tile_size: u32 },
}

#[derive(Debug, Clone)]
pub enum TemporalCorruption {
  FrameDrop { drop_rate: f64 },
  Interference { frequency: f64 },
}

#[derive(Debug, Clone)]
pub enum ImageCorruptionType {
  RandomPixel,
  BitFlip,
  Scanline,
  Block,
  Channel,
  Noise,
  Databending,
  Geometric,
}

#[derive(Debug, Clone)]
pub struct CorruptionLayer {
  pub corruption_type: ImageCorruptionType,
  pub intensity: f64,
  pub channel: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct ImageCorruptionParameters {
  pub random_pixel_corruption: bool,
  pub random_pixel_intensity: f64,
  pub bit_flip_corruption: bool,
  pub bit_flip_intensity: f64,
  pub scanline_corruption: bool,
  pub scanline_intensity: f64,
  pub block_corruption: bool,
  pub block_size: u32,
  pub block_intensity: f64,
  pub channel_corruption: bool,
  pub corruption_channel: Option<u8>,
  pub channel_intensity: f64,
  pub noise_injection: bool,
  pub noise_type: NoiseType,
  pub noise_intensity: f64,
  pub compression_artifacts: bool,
  pub artifact_intensity: f64,
}

#[derive(Debug, Clone)]
pub struct ImageCorruptionStats {
  pub entropy: f64,
  pub zero_pixels: u32,
  pub max_pixels: u32,
  pub unique_colors: usize,
  pub total_pixels: usize,
  pub avg_run_length: f64,
  pub max_run_length: usize,
  pub run_count: usize,
}

impl Default for ImageCorruptionParameters {
  fn default() -> Self {
    Self {
      random_pixel_corruption: false,
      random_pixel_intensity: 0.1,
      bit_flip_corruption: false,
      bit_flip_intensity: 0.05,
      scanline_corruption: false,
      scanline_intensity: 0.1,
      block_corruption: false,
      block_size: 32,
      block_intensity: 0.1,
      channel_corruption: false,
      corruption_channel: None,
      channel_intensity: 0.1,
      noise_injection: false,
      noise_type: NoiseType::Uniform,
      noise_intensity: 0.1,
      compression_artifacts: false,
      artifact_intensity: 0.1,
    }
  }
}

pub fn create_corruption_engine(image: ImageProcessor) -> ImageCorruptionEngine {
  ImageCorruptionEngine::new(image)
}

pub fn create_corruption_engine_with_seed(
  image: ImageProcessor,
  seed: u64,
) -> ImageCorruptionEngine {
  ImageCorruptionEngine::with_seed(image, seed)
}
