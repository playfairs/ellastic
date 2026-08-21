use crate::{
  ByteBuffer,
  ByteOrder,
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
  create_random_generator_with_seed,
};

#[derive(Debug, Clone)]
pub struct CorruptionEngine {
  buffer: ByteBuffer,
  rng: RandomGenerator,
}

impl CorruptionEngine {
  pub fn new(buffer: ByteBuffer) -> Self {
    Self {
      buffer,
      rng: create_random_generator(),
    }
  }

  pub fn with_seed(buffer: ByteBuffer, seed: u64) -> Self {
    Self {
      buffer,
      rng: create_random_generator_with_seed(seed),
    }
  }

  pub fn buffer(&self) -> &ByteBuffer {
    &self.buffer
  }

  pub fn buffer_mut(&mut self) -> &mut ByteBuffer {
    &mut self.buffer
  }

  pub fn into_buffer(self) -> ByteBuffer {
    self.buffer
  }

  pub fn random_corruption(&mut self, intensity: f64) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len == 0 {
      return Ok(());
    }

    let corruption_count = (data_len as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let position = self.rng.gen_range(0, data_len as u64) as usize;
      let byte = self.rng.gen_u8();
      self.buffer.data_mut()[position] = byte;
    }

    Ok(())
  }

  pub fn bit_flip_corruption(&mut self, intensity: f64) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len == 0 {
      return Ok(());
    }

    let flip_count = (data_len as f64 * intensity.clamp(0.0, 1.0) * 8.0) as usize;

    for _ in 0..flip_count {
      let position = self.rng.gen_range(0, data_len as u64) as usize;
      let bit_position = self.rng.gen_range(0, 8);
      self.buffer.data_mut()[position] ^= 1 << bit_position;
    }

    Ok(())
  }

  pub fn byte_substitution(&mut self, old_byte: u8, new_byte: u8, probability: f64) -> Result<()> {
    for byte in self.buffer.data_mut() {
      if *byte == old_byte && self.rng.gen_range(0, 1000) as f64 / 1000.0 < probability {
        *byte = new_byte;
      }
    }
    Ok(())
  }

  pub fn byte_insertion(&mut self, insertion_rate: f64) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len == 0 {
      return Ok(());
    }

    let insertions = (data_len as f64 * insertion_rate.clamp(0.0, 1.0)) as usize;
    let data = self.buffer.data().to_vec();
    let mut new_data = Vec::with_capacity(data_len + insertions);

    for (i, &byte) in data.iter().enumerate() {
      new_data.push(byte);

      if self.rng.gen_range(0, 1000) as f64 / 1000.0 < insertion_rate {
        new_data.push(self.rng.gen_u8());
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&new_data);

    Ok(())
  }

  pub fn byte_deletion(&mut self, deletion_rate: f64) -> Result<()> {
    let data = self.buffer.data().to_vec();
    let mut new_data = Vec::with_capacity(data.len());

    for &byte in &data {
      if self.rng.gen_range(0, 1000) as f64 / 1000.0 >= deletion_rate {
        new_data.push(byte);
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&new_data);

    Ok(())
  }

  pub fn byte_swapping(&mut self, swap_probability: f64) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len < 2 {
      return Ok(());
    }

    let swap_count = (data_len as f64 * swap_probability.clamp(0.0, 1.0) / 2.0) as usize;

    for _ in 0..swap_count {
      let pos1 = self.rng.gen_range(0, data_len as u64) as usize;
      let pos2 = self.rng.gen_range(0, data_len as u64) as usize;

      if pos1 != pos2 {
        let data = self.buffer.data_mut();
        data.swap(pos1, pos2);
      }
    }

    Ok(())
  }

  pub fn chunk_duplication(&mut self, chunk_size: usize, duplication_count: usize) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len == 0 || chunk_size == 0 {
      return Ok(());
    }

    let start_pos = self.rng.gen_range(0, (data_len - chunk_size + 1) as u64) as usize;
    let chunk = self.buffer.data()[start_pos..start_pos + chunk_size].to_vec();

    let insert_pos = self.rng.gen_range(0, (data_len - chunk_size + 1) as u64) as usize;

    for _ in 0..duplication_count {
      let data = self.buffer.data_mut();
      data.splice(insert_pos..insert_pos, chunk.iter().cloned());
    }

    Ok(())
  }

  pub fn chunk_reversal(&mut self, chunk_size: usize) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len == 0 || chunk_size == 0 {
      return Ok(());
    }

    let start_pos = self.rng.gen_range(0, (data_len - chunk_size + 1) as u64) as usize;
    let end_pos = start_pos + chunk_size;

    let data = self.buffer.data_mut();
    data[start_pos..end_pos].reverse();

    Ok(())
  }

  pub fn endian_corruption(&mut self, chunk_size: usize) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len < 2 || chunk_size < 2 {
      return Ok(());
    }

    let start_pos = self.rng.gen_range(0, (data_len - chunk_size + 1) as u64) as usize;
    let end_pos = std::cmp::min(start_pos + chunk_size, data_len);

    let data = self.buffer.data_mut();
    for chunk in data[start_pos..end_pos].chunks_mut(2) {
      if chunk.len() == 2 {
        chunk.swap(0, 1);
      }
    }

    Ok(())
  }

  pub fn noise_injection(&mut self, noise_type: NoiseType, intensity: f64) -> Result<()> {
    let noise_gen = create_noise_generator_with_seed(self.rng.gen_seed());
    noise_gen.apply_to_bytes(self.buffer.data_mut(), intensity as f32);
    Ok(())
  }

  pub fn pattern_corruption(&mut self, pattern: &[u8], replacement: &[u8]) -> Result<()> {
    let data_len = self.buffer.len();
    if pattern.is_empty() || data_len == 0 {
      return Ok(());
    }

    let data = self.buffer.data().to_vec();
    let mut new_data = Vec::with_capacity(data_len);
    let mut i = 0;

      while i <= data_len.saturating_sub(pattern.len()) {
        if &data[i..i + pattern.len()] == pattern {
        new_data.extend_from_slice(replacement);
        i += pattern.len();
      } else {
        new_data.push(data[i]);
        i += 1;
      }
    }

    new_data.extend_from_slice(&data[i..]);

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&new_data);

    Ok(())
  }

  pub fn header_corruption(&mut self, header_size: usize, corruption_level: f64) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len == 0 {
      return Ok(());
    }

    let actual_header_size = std::cmp::min(header_size, data_len);
    let corruption_count = (actual_header_size as f64 * corruption_level.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let position = self.rng.gen_range(0, actual_header_size as u64) as usize;
      self.buffer.data_mut()[position] = self.rng.gen_u8();
    }

    Ok(())
  }

  pub fn checksum_corruption(&mut self) -> Result<()> {
    let data_len = self.buffer.len();
    if data_len < 4 {
      return Ok(());
    }

    let checksum_pos = self.rng.gen_range(0, (data_len - 3) as u64) as usize;
    let fake_checksum = self.rng.gen_u32();

    let data = self.buffer.data_mut();
    data[checksum_pos] = (fake_checksum >> 24) as u8;
    data[checksum_pos + 1] = (fake_checksum >> 16) as u8;
    data[checksum_pos + 2] = (fake_checksum >> 8) as u8;
    data[checksum_pos + 3] = fake_checksum as u8;

    Ok(())
  }

  pub fn structural_corruption(&mut self, corruption_type: StructuralCorruption) -> Result<()> {
    match corruption_type {
      StructuralCorruption::ShuffleChunks { chunk_size } => {
        let data_len = self.buffer.len();
        if data_len < chunk_size || chunk_size == 0 {
          return Ok(());
        }

        let data = self.buffer.data_mut();
        let mut chunks: Vec<_> = data.chunks_mut(chunk_size).collect();
        self.rng.shuffle(&mut chunks.iter_mut().collect::<Vec<_>>());
      }
      StructuralCorruption::SplitAndMerge { split_positions } => {
        let data = self.buffer.data().to_vec();
        let mut chunks = Vec::new();
        let mut last_pos = 0;

        for &pos in &split_positions {
          if pos > last_pos && pos <= data.len() {
            chunks.push(data[last_pos..pos].to_vec());
            last_pos = pos;
          }
        }

        if last_pos < data.len() {
          chunks.push(data[last_pos..].to_vec());
        }

        self.rng.shuffle(&mut chunks);

        let mut new_data = Vec::new();
        for chunk in chunks {
          new_data.extend_from_slice(&chunk);
        }

        self.buffer.data_mut().clear();
        self.buffer.data_mut().extend_from_slice(&new_data);
      }
      StructuralCorruption::OffsetShift { offset } => {
        let data_len = self.buffer.len();
        if data_len == 0 {
          return Ok(());
        }

        let data = self.buffer.data().to_vec();
        let mut new_data = vec![0u8; data_len];

        for (i, &byte) in data.iter().enumerate() {
          let new_pos = (i as isize + offset).rem_euclid(data_len as isize) as usize;
          new_data[new_pos] = byte;
        }

        self.buffer.data_mut().clear();
        self.buffer.data_mut().extend_from_slice(&new_data);
      }
    }

    Ok(())
  }

  pub fn compression_artifact(&mut self, artifact_type: CompressionArtifact) -> Result<()> {
    match artifact_type {
      CompressionArtifact::Blockiness { block_size } => {
        let data_len = self.buffer.len();
        if data_len == 0 || block_size == 0 {
          return Ok(());
        }

        let data = self.buffer.data_mut();
        for chunk in data.chunks_mut(block_size) {
          if !chunk.is_empty() {
            let average = chunk.iter().map(|&b| b as u32).sum::<u32>() / chunk.len() as u32;
            for byte in chunk {
              *byte = average as u8;
            }
          }
        }
      }
      CompressionArtifact::ColorBanding { levels } => {
        let data = self.buffer.data_mut();
        for byte in data {
          *byte = (*byte as f32 / 255.0 * levels as f32).round() as u8 * 255 / levels;
        }
      }
      CompressionArtifact::Quantization { step_size } => {
        let data = self.buffer.data_mut();
        for byte in data {
          *byte = (*byte as u32 / step_size as u32 * step_size as u32) as u8;
        }
      }
    }

    Ok(())
  }

  pub fn databending_effect(&mut self, effect: DatabendingEffect) -> Result<()> {
    match effect {
      DatabendingEffect::XOR { key } => {
        self.buffer.xor(key);
      }
      DatabendingEffect::XORWithKey { key } => {
        self.buffer.xor_with_key(&key);
      }
      DatabendingEffect::Add { value } => {
        self.buffer.add(value);
      }
      DatabendingEffect::Subtract { value } => {
        self.buffer.subtract(value);
      }
      DatabendingEffect::Multiply { value } => {
        self.buffer.multiply(value);
      }
      DatabendingEffect::Divide { value } => {
        self.buffer.divide(value);
      }
      DatabendingEffect::ShiftLeft { bits } => {
        self.buffer.shift_left(bits);
      }
      DatabendingEffect::ShiftRight { bits } => {
        self.buffer.shift_right(bits);
      }
      DatabendingEffect::RotateLeft { bits } => {
        self.buffer.rotate_left(bits);
      }
      DatabendingEffect::RotateRight { bits } => {
        self.buffer.rotate_right(bits);
      }
      DatabendingEffect::Complement => {
        self.buffer.complement();
      }
      DatabendingEffect::Reverse => {
        self.buffer.reverse();
      }
      DatabendingEffect::ReverseBits => {
        self.buffer.reverse_bytes();
      }
    }

    Ok(())
  }

  pub fn recursive_corruption(&mut self, depth: usize, intensity: f64) -> Result<()> {
    if depth == 0 {
      return Ok(());
    }

    let corruption_types = [
      CorruptionType::Random,
      CorruptionType::BitFlip,
      CorruptionType::ByteSwap,
      CorruptionType::ChunkReverse,
    ];

    for _ in 0..depth {
      let corruption_type =
        corruption_types[self.rng.gen_range(0, corruption_types.len() as u64) as usize].clone();

      match corruption_type {
        CorruptionType::Random => self.random_corruption(intensity)?,
        CorruptionType::BitFlip => self.bit_flip_corruption(intensity)?,
        CorruptionType::ByteSwap => self.byte_swapping(intensity)?,
        CorruptionType::ChunkReverse => self.chunk_reversal(64)?,
        CorruptionType::Noise | CorruptionType::Structural => {}
      }
    }

    Ok(())
  }

  pub fn layered_corruption(&mut self, layers: &[CorruptionLayer]) -> Result<()> {
    for layer in layers {
      match layer.corruption_type {
        CorruptionType::Random => self.random_corruption(layer.intensity)?,
        CorruptionType::BitFlip => self.bit_flip_corruption(layer.intensity)?,
        CorruptionType::ByteSwap => self.byte_swapping(layer.intensity)?,
        CorruptionType::ChunkReverse => self.chunk_reversal(64)?,
        CorruptionType::Noise => self.noise_injection(NoiseType::Uniform, layer.intensity)?,
        CorruptionType::Structural => {
          self.structural_corruption(StructuralCorruption::ShuffleChunks { chunk_size: 32 })?;
        }
      }
    }

    Ok(())
  }

  pub fn controlled_corruption(&mut self, parameters: &CorruptionParameters) -> Result<()> {
    if parameters.random_corruption {
      self.random_corruption(parameters.random_intensity)?;
    }

    if parameters.bit_flip_corruption {
      self.bit_flip_corruption(parameters.bit_flip_intensity)?;
    }

    if parameters.byte_swapping {
      self.byte_swapping(parameters.swap_intensity)?;
    }

      if parameters.noise_injection {
        self.noise_injection(parameters.noise_type.clone(), parameters.noise_intensity)?;
    }

    if parameters.header_corruption {
      self.header_corruption(parameters.header_size, parameters.header_intensity)?;
    }

    if parameters.structural_corruption {
      self.structural_corruption(parameters.structural_type.clone())?;
    }

    Ok(())
  }

  pub fn corruption_statistics(&self) -> CorruptionStats {
    let data = self.buffer.data();
    let original_entropy = self.calculate_entropy(&data);
    let histogram = self.buffer.get_histogram();

    let zero_bytes = histogram[0];
    let max_bytes = histogram[255];
    let unique_bytes = histogram.iter().filter(|&&count| count > 0).count();

    let mut runs = Vec::new();
    if !data.is_empty() {
      let mut current_byte = data[0];
      let mut current_run = 1;

      for &byte in data.iter().skip(1) {
        if byte == current_byte {
          current_run += 1;
        } else {
          runs.push(current_run);
          current_byte = byte;
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

    CorruptionStats {
      entropy: original_entropy,
      zero_bytes,
      max_bytes,
      unique_bytes,
      total_bytes: data.len(),
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
}

#[derive(Debug, Clone)]
pub enum NoiseType {
  Uniform,
  Gaussian,
  Perlin,
  SaltAndPepper,
}

#[derive(Debug, Clone)]
pub enum StructuralCorruption {
  ShuffleChunks { chunk_size: usize },
  SplitAndMerge { split_positions: Vec<usize> },
  OffsetShift { offset: isize },
}

#[derive(Debug, Clone)]
pub enum CompressionArtifact {
  Blockiness { block_size: usize },
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
  Divide { value: u8 },
  ShiftLeft { bits: u8 },
  ShiftRight { bits: u8 },
  RotateLeft { bits: u8 },
  RotateRight { bits: u8 },
  Complement,
  Reverse,
  ReverseBits,
}

#[derive(Debug, Clone)]
pub enum CorruptionType {
  Random,
  BitFlip,
  ByteSwap,
  ChunkReverse,
  Noise,
  Structural,
}

#[derive(Debug, Clone)]
pub struct CorruptionLayer {
  pub corruption_type: CorruptionType,
  pub intensity: f64,
}

#[derive(Debug, Clone)]
pub struct CorruptionParameters {
  pub random_corruption: bool,
  pub random_intensity: f64,
  pub bit_flip_corruption: bool,
  pub bit_flip_intensity: f64,
  pub byte_swapping: bool,
  pub swap_intensity: f64,
  pub noise_injection: bool,
  pub noise_type: NoiseType,
  pub noise_intensity: f64,
  pub header_corruption: bool,
  pub header_size: usize,
  pub header_intensity: f64,
  pub structural_corruption: bool,
  pub structural_type: StructuralCorruption,
}

#[derive(Debug, Clone)]
pub struct CorruptionStats {
  pub entropy: f64,
  pub zero_bytes: u64,
  pub max_bytes: u64,
  pub unique_bytes: usize,
  pub total_bytes: usize,
  pub avg_run_length: f64,
  pub max_run_length: usize,
  pub run_count: usize,
}

impl Default for CorruptionParameters {
  fn default() -> Self {
    Self {
      random_corruption: false,
      random_intensity: 0.1,
      bit_flip_corruption: false,
      bit_flip_intensity: 0.05,
      byte_swapping: false,
      swap_intensity: 0.1,
      noise_injection: false,
      noise_type: NoiseType::Uniform,
      noise_intensity: 0.1,
      header_corruption: false,
      header_size: 1024,
      header_intensity: 0.2,
      structural_corruption: false,
      structural_type: StructuralCorruption::ShuffleChunks { chunk_size: 64 },
    }
  }
}

pub fn create_corruption_engine(buffer: ByteBuffer) -> CorruptionEngine {
  CorruptionEngine::new(buffer)
}

pub fn create_corruption_engine_with_seed(buffer: ByteBuffer, seed: u64) -> CorruptionEngine {
  CorruptionEngine::with_seed(buffer, seed)
}
