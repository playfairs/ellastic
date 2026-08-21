use crate::{
  AudioData,
  AudioProcessor,
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
pub struct AudioCorruptionEngine {
  audio: AudioProcessor,
  rng: RandomGenerator,
}

impl AudioCorruptionEngine {
  pub fn new(audio: AudioProcessor) -> Self {
    Self {
      audio,
      rng: create_random_generator(),
    }
  }

  pub fn with_seed(audio: AudioProcessor, seed: u64) -> Self {
    Self {
      audio,
      rng: create_random_generator_with_seed(seed),
    }
  }

  pub fn audio(&self) -> &AudioProcessor {
    &self.audio
  }

  pub fn audio_mut(&mut self) -> &mut AudioProcessor {
    &mut self.audio
  }

  pub fn into_audio(self) -> AudioProcessor {
    self.audio
  }

  pub fn random_sample_corruption(&mut self, intensity: f64) -> Result<()> {
    let sample_count = self.audio.sample_count();
    let corruption_count = (sample_count as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let index = self.rng.gen_range(0, sample_count as u64) as usize;
      let new_value = self.rng.gen_range(-1.0, 1.0);
      if let Some(sample) = self.audio.get_sample_mut(index) {
        *sample = new_value;
      }
    }

    Ok(())
  }

  pub fn channel_corruption(&mut self, channel: u8, intensity: f64) -> Result<()> {
    if channel >= self.audio.channels() {
      return Err(EllasticError::InvalidParameter(
        "Channel out of range".to_string(),
      ));
    }

    let channel_samples = self.audio.get_channel(channel)?;
    let corruption_count = (channel_samples.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..corruption_count {
      let position = self.rng.gen_range(0, channel_samples.len() as u64) as usize;
      let new_value = self.rng.gen_range(-1.0, 1.0);

      let sample_index = position * self.audio.channels() as usize + channel as usize;
      if let Some(sample) = self.audio.get_sample_mut(sample_index) {
        *sample = new_value;
      }
    }

    Ok(())
  }

  pub fn bit_flip_corruption(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let flip_count = (data.len() as f64 * intensity.clamp(0.0, 1.0) * 32.0) as usize;

    for _ in 0..flip_count {
      let byte_index = self.rng.gen_range(0, data.len() as u64) as usize;
      let bit_position = self.rng.gen_range(0, 32);

      let bits = data[byte_index].to_bits();
      let mut bits_array = [0u8; 4];
      bits_array.copy_from_slice(&bits.to_le_bytes());

      let byte_pos = bit_position / 8;
      let bit_pos = bit_position % 8;

      if byte_pos < 4 {
        bits_array[byte_pos] ^= 1 << bit_pos;
        let new_bits = u32::from_le_bytes(bits_array);
        data[byte_index] = f32::from_bits(new_bits);
      }
    }

    Ok(())
  }

  pub fn sample_swap_corruption(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
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

  pub fn clipping_corruption(&mut self, threshold: f32, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let clip_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..clip_count {
      let index = self.rng.gen_range(0, data.len() as u64) as usize;
      let sample = data[index];

      if sample.abs() > threshold {
        data[index] = if sample > 0.0 { 1.0 } else { -1.0 };
      }
    }

    Ok(())
  }

  pub fn quantization_corruption(&mut self, bits: u8, intensity: f64) -> Result<()> {
    if bits == 0 || bits > 32 {
      return Err(EllasticError::InvalidParameter(
        "Bits must be between 1 and 32".to_string(),
      ));
    }

    let data = self.audio.data_mut().samples;
    let quant_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;
    let levels = (1 << bits) as f32;

    for _ in 0..quant_count {
      let index = self.rng.gen_range(0, data.len() as u64) as usize;
      let sample = data[index];

      let quantized = (sample * levels / 2.0).round() / (levels / 2.0);
      data[index] = quantized.clamp(-1.0, 1.0);
    }

    Ok(())
  }

  pub fn noise_injection(&mut self, noise_type: NoiseType, intensity: f64) -> Result<()> {
    let noise_gen = create_noise_generator_with_seed(self.rng.gen_seed());
    let data = self.audio.data_mut().samples;

    for sample in data.iter_mut() {
      let noise = noise_gen.next_f32() * intensity;
      *sample = (*sample + noise).clamp(-1.0, 1.0);
    }

    Ok(())
  }

  pub fn dropout_corruption(&mut self, dropout_rate: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let dropout_count = (data.len() as f64 * dropout_rate.clamp(0.0, 1.0)) as usize;

    for _ in 0..dropout_count {
      let index = self.rng.gen_range(0, data.len() as u64) as usize;
      data[index] = 0.0;
    }

    Ok(())
  }

  pub fn glitch_corruption(&mut self, glitch_type: AudioGlitchType, intensity: f64) -> Result<()> {
    match glitch_type {
      AudioGlitchType::Stutter => self.stutter_glitch(intensity),
      AudioGlitchType::Repeat => self.repeat_glitch(intensity),
      AudioGlitchType::Reverse => self.reverse_glitch(intensity),
      AudioGlitchType::PitchShift => self.pitch_shift_glitch(intensity),
      AudioGlitchType::TimeStretch => self.time_stretch_glitch(intensity),
      AudioGlitchType::BitCrush => self.bit_crush_glitch(intensity),
      AudioGlitchType::RingModulation => self.ring_modulation_glitch(intensity),
      AudioGlitchType::FrequencyModulation => self.frequency_modulation_glitch(intensity),
    }
  }

  fn stutter_glitch(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let sample_rate = self.audio.sample_rate() as f32;
    let glitch_count = (intensity * 10.0) as usize;

    for _ in 0..glitch_count {
      let start_pos = self.rng.gen_range(0, data.len() as u64) as usize;
      let glitch_duration = self.rng.gen_range(10, 100) as usize;
      let repeat_count = self.rng.gen_range(2, 8);

      if start_pos + glitch_duration < data.len() {
        let glitch_segment = data[start_pos..start_pos + glitch_duration].to_vec();

        for _ in 0..repeat_count {
          let insert_pos = self.rng.gen_range(0, data.len() as u64) as usize;
          if insert_pos + glitch_duration <= data.len() {
            data[insert_pos..insert_pos + glitch_duration].copy_from_slice(&glitch_segment);
          }
        }
      }
    }

    Ok(())
  }

  fn repeat_glitch(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let repeat_count = (intensity * 5.0) as usize;

    for _ in 0..repeat_count {
      let start_pos = self.rng.gen_range(0, data.len() as u64) as usize;
      let repeat_length = self.rng.gen_range(100, 1000) as usize;
      let repeat_times = self.rng.gen_range(2, 5);

      if start_pos + repeat_length < data.len() {
        let repeat_segment = data[start_pos..start_pos + repeat_length].to_vec();

        for _ in 0..repeat_times {
          let insert_pos = self.rng.gen_range(0, data.len() as u64) as usize;
          if insert_pos + repeat_length <= data.len() {
            data[insert_pos..insert_pos + repeat_length].copy_from_slice(&repeat_segment);
          }
        }
      }
    }

    Ok(())
  }

  fn reverse_glitch(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let reverse_count = (intensity * 3.0) as usize;

    for _ in 0..reverse_count {
      let start_pos = self.rng.gen_range(0, data.len() as u64) as usize;
      let reverse_length = self.rng.gen_range(100, 2000) as usize;

      if start_pos + reverse_length < data.len() {
        data[start_pos..start_pos + reverse_length].reverse();
      }
    }

    Ok(())
  }

  fn pitch_shift_glitch(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let shift_count = (intensity * 2.0) as usize;

    for _ in 0..shift_count {
      let start_pos = self.rng.gen_range(0, data.len() as u64) as usize;
      let segment_length = self.rng.gen_range(1000, 5000) as usize;
      let pitch_shift = self.rng.gen_range(-12.0, 12.0);

      if start_pos + segment_length < data.len() {
        let pitch_factor = (2.0f32).powf(pitch_shift / 12.0);

        for i in 0..segment_length {
          let source_index = (i as f32 / pitch_factor) as usize;
          if source_index < segment_length {
            data[start_pos + i] = data[start_pos + source_index];
          }
        }
      }
    }

    Ok(())
  }

  fn time_stretch_glitch(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let stretch_count = (intensity * 2.0) as usize;

    for _ in 0..stretch_count {
      let start_pos = self.rng.gen_range(0, data.len() as u64) as usize;
      let segment_length = self.rng.gen_range(1000, 5000) as usize;
      let stretch_factor = self.rng.gen_range(0.5, 2.0);

      if start_pos + segment_length < data.len() {
        let stretched_length = (segment_length as f32 * stretch_factor) as usize;

        for i in 0..stretched_length.min(segment_length) {
          let source_index = (i as f32 / stretch_factor) as usize;
          if source_index < segment_length {
            data[start_pos + i] = data[start_pos + source_index];
          }
        }
      }
    }

    Ok(())
  }

  fn bit_crush_glitch(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let crush_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;
    let bits = self.rng.gen_range(1, 16);
    let levels = (1 << bits) as f32;

    for _ in 0..crush_count {
      let index = self.rng.gen_range(0, data.len() as u64) as usize;
      let sample = data[index];

      let crushed = (sample * levels / 2.0).round() / (levels / 2.0);
      data[index] = crushed.clamp(-1.0, 1.0);
    }

    Ok(())
  }

  fn ring_modulation_glitch(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let sample_rate = self.audio.sample_rate() as f32;
    let mod_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..mod_count {
      let index = self.rng.gen_range(0, data.len() as u64) as usize;
      let mod_freq = self.rng.gen_range(20.0, 2000.0);
      let mod_phase = self.rng.gen_range(0.0, 2.0 * std::f32::consts::PI);

      let sample = data[index];
      let time = index as f32 / sample_rate;
      let modulation = (2.0 * std::f32::consts::PI * mod_freq * time + mod_phase).sin();

      data[index] = (sample * modulation).clamp(-1.0, 1.0);
    }

    Ok(())
  }

  fn frequency_modulation_glitch(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let sample_rate = self.audio.sample_rate() as f32;
    let fm_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..fm_count {
      let index = self.rng.gen_range(0, data.len() as u64) as usize;
      let carrier_freq = self.rng.gen_range(100.0, 2000.0);
      let mod_freq = self.rng.gen_range(1.0, 100.0);
      let mod_index = self.rng.gen_range(0.1, 5.0);

      let time = index as f32 / sample_rate;
      let carrier_phase = 2.0 * std::f32::consts::PI * carrier_freq * time;
      let mod_phase = 2.0 * std::f32::consts::PI * mod_freq * time;

      let modulation = (carrier_phase + mod_index * mod_phase.sin()).sin();
      data[index] = modulation.clamp(-1.0, 1.0);
    }

    Ok(())
  }

  pub fn compression_artifact(
    &mut self,
    artifact_type: CompressionArtifact,
    intensity: f64,
  ) -> Result<()> {
    match artifact_type {
      CompressionArtifact::Aliasing => self.aliasing_artifact(intensity),
      CompressionArtifact::PreEcho => self.pre_echo_artifact(intensity),
      CompressionArtifact::BlockSwapping => self.block_swapping_artifact(intensity),
      CompressionArtifact::QuantizationNoise => self.quantization_noise_artifact(intensity),
    }
  }

  fn aliasing_artifact(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let alias_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..alias_count {
      let index = self.rng.gen_range(0, data.len() as u64) as usize;
      if index > 0 && index < data.len() - 1 {
        let prev = data[index - 1];
        let next = data[index + 1];
        let aliased = (prev + next) * 0.5 + self.rng.gen_range(-0.1, 0.1);
        data[index] = aliased.clamp(-1.0, 1.0);
      }
    }

    Ok(())
  }

  fn pre_echo_artifact(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let echo_count = (intensity * 5.0) as usize;

    for _ in 0..echo_count {
      let start_pos = self.rng.gen_range(100, data.len() as u64) as usize;
      let echo_length = self.rng.gen_range(10, 100) as usize;
      let echo_gain = self.rng.gen_range(0.1, 0.3);

      if start_pos >= echo_length {
        for i in 0..echo_length.min(start_pos) {
          data[start_pos - echo_length + i] += data[start_pos + i] * echo_gain;
          data[start_pos - echo_length + i] = data[start_pos - echo_length + i].clamp(-1.0, 1.0);
        }
      }
    }

    Ok(())
  }

  fn block_swapping_artifact(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let block_size = self.rng.gen_range(64, 1024) as usize;
    let swap_count = (intensity * 10.0) as usize;

    for _ in 0..swap_count {
      let pos1 = self.rng.gen_range(0, (data.len() - block_size) as u64) as usize;
      let pos2 = self.rng.gen_range(0, (data.len() - block_size) as u64) as usize;

      if pos1 != pos2 && pos1 + block_size <= data.len() && pos2 + block_size <= data.len() {
        for i in 0..block_size {
          data.swap(pos1 + i, pos2 + i);
        }
      }
    }

    Ok(())
  }

  fn quantization_noise_artifact(&mut self, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let noise_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    for _ in 0..noise_count {
      let index = self.rng.gen_range(0, data.len() as u64) as usize;
      let quantization_error = self.rng.gen_range(-0.01, 0.01);
      data[index] = (data[index] + quantization_error).clamp(-1.0, 1.0);
    }

    Ok(())
  }

  pub fn databending_effect(&mut self, effect: DatabendingEffect, intensity: f64) -> Result<()> {
    let data = self.audio.data_mut().samples;
    let affected_count = (data.len() as f64 * intensity.clamp(0.0, 1.0)) as usize;

    match effect {
      DatabendingEffect::XOR { key } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          let bits = data[position].to_bits();
          let key_bits = key.to_bits();
          let xor_bits = bits ^ key_bits;
          data[position] = f32::from_bits(xor_bits);
        }
      }
      DatabendingEffect::Add { value } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] = (data[position] + value).clamp(-1.0, 1.0);
        }
      }
      DatabendingEffect::Multiply { value } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          data[position] = (data[position] * value).clamp(-1.0, 1.0);
        }
      }
      DatabendingEffect::BitShift { bits } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          let bits_array = data[position].to_bits().to_le_bytes();
          let mut shifted_bits = bits_array;

          for byte in shifted_bits.iter_mut() {
            *byte = byte.wrapping_shl(bits);
          }

          data[position] = f32::from_bits(u32::from_le_bytes(shifted_bits));
        }
      }
      DatabendingEffect::BitRotate { bits } => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          let bits = data[position].to_bits();
          let rotated_bits = bits.rotate_left(bits as u32);
          data[position] = f32::from_bits(rotated_bits);
        }
      }
      DatabendingEffect::Complement => {
        for i in 0..affected_count {
          let position = self.rng.gen_range(0, data.len() as u64) as usize;
          let bits = data[position].to_bits();
          let complemented_bits = !bits;
          data[position] = f32::from_bits(complemented_bits);
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
      AudioCorruptionType::RandomSample,
      AudioCorruptionType::BitFlip,
      AudioCorruptionType::Noise,
      AudioCorruptionType::Glitch,
    ];

    for _ in 0..depth {
      let corruption_type =
        corruption_types[self.rng.gen_range(0, corruption_types.len() as u64) as usize];

      match corruption_type {
        AudioCorruptionType::RandomSample => self.random_sample_corruption(intensity)?,
        AudioCorruptionType::BitFlip => self.bit_flip_corruption(intensity)?,
        AudioCorruptionType::Noise => self.noise_injection(NoiseType::Uniform, intensity)?,
        AudioCorruptionType::Glitch => {
          self.glitch_corruption(AudioGlitchType::Stutter, intensity)?
        }
      }
    }

    Ok(())
  }

  pub fn layered_corruption(&mut self, layers: &[AudioCorruptionLayer]) -> Result<()> {
    for layer in layers {
      match layer.corruption_type {
        AudioCorruptionType::RandomSample => self.random_sample_corruption(layer.intensity)?,
        AudioCorruptionType::BitFlip => self.bit_flip_corruption(layer.intensity)?,
        AudioCorruptionType::ChannelSwap => {
          self.channel_corruption(layer.channel.unwrap_or(0), layer.intensity)?
        }
        AudioCorruptionType::Noise => self.noise_injection(
          layer.noise_type.unwrap_or(NoiseType::Uniform),
          layer.intensity,
        )?,
        AudioCorruptionType::Glitch => self.glitch_corruption(
          layer.glitch_type.unwrap_or(AudioGlitchType::Stutter),
          layer.intensity,
        )?,
        AudioCorruptionType::Databending => self.databending_effect(
          layer
            .databending_effect
            .unwrap_or(DatabendingEffect::XOR { key: 42 }),
          layer.intensity,
        )?,
        AudioCorruptionType::Compression => self.compression_artifact(
          layer
            .compression_artifact
            .unwrap_or(CompressionArtifact::Aliasing),
          layer.intensity,
        )?,
      }
    }

    Ok(())
  }

  pub fn controlled_corruption(&mut self, parameters: &AudioCorruptionParameters) -> Result<()> {
    if parameters.random_sample_corruption {
      self.random_sample_corruption(parameters.random_sample_intensity)?;
    }

    if parameters.bit_flip_corruption {
      self.bit_flip_corruption(parameters.bit_flip_intensity)?;
    }

    if parameters.channel_corruption {
      if let Some(channel) = parameters.corruption_channel {
        self.channel_corruption(channel, parameters.channel_intensity)?;
      }
    }

    if parameters.noise_injection {
      self.noise_injection(parameters.noise_type, parameters.noise_intensity)?;
    }

    if parameters.glitch_corruption {
      self.glitch_corruption(parameters.glitch_type, parameters.glitch_intensity)?;
    }

    if parameters.compression_artifacts {
      self.compression_artifact(
        parameters.compression_artifact,
        parameters.artifact_intensity,
      )?;
    }

    Ok(())
  }

  pub fn corruption_statistics(&self) -> AudioCorruptionStats {
    let data = self.audio.data().samples;
    let original_entropy = self.calculate_entropy(&data);
    let histogram = self.calculate_histogram(&data);

    let zero_samples = histogram.iter().filter(|&&count| count > 0).count();
    let peak_samples = histogram.iter().filter(|&&count| count > 0).count();
    let unique_values = histogram.iter().filter(|&&count| count > 0).count();

    let mut runs = Vec::new();
    if !data.is_empty() {
      let mut current_value = data[0];
      let mut current_run = 1;

      for &sample in data.iter().skip(1) {
        if (sample - current_value).abs() < 1e-6 {
          current_run += 1;
        } else {
          runs.push(current_run);
          current_value = sample;
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

    AudioCorruptionStats {
      entropy: original_entropy,
      zero_samples,
      peak_samples,
      unique_values,
      total_samples: data.len(),
      avg_run_length,
      max_run_length,
      run_count: runs.len(),
    }
  }

  fn calculate_entropy(&self, data: &[f32]) -> f64 {
    if data.is_empty() {
      return 0.0;
    }

    let mut frequency = std::collections::HashMap::new();
    for &sample in data {
      let bucket = (sample * 1000.0).round() as i32;
      *frequency.entry(bucket).or_insert(0) += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in frequency.values() {
      if count > 0 {
        let probability = count as f64 / len;
        entropy -= probability * probability.log2();
      }
    }

    entropy
  }

  fn calculate_histogram(&self, data: &[f32]) -> Vec<usize> {
    let mut histogram = vec![0usize; 2001];

    for &sample in data {
      let bucket = ((sample + 1.0) * 1000.0).round() as i32;
      if bucket >= 0 && bucket <= 2000 {
        histogram[bucket as usize] += 1;
      }
    }

    histogram
  }
}

#[derive(Debug, Clone)]
pub enum NoiseType {
  Uniform,
  Gaussian,
  Pink,
  Brownian,
}

#[derive(Debug, Clone)]
pub enum AudioGlitchType {
  Stutter,
  Repeat,
  Reverse,
  PitchShift,
  TimeStretch,
  BitCrush,
  RingModulation,
  FrequencyModulation,
}

#[derive(Debug, Clone)]
pub enum CompressionArtifact {
  Aliasing,
  PreEcho,
  BlockSwapping,
  QuantizationNoise,
}

#[derive(Debug, Clone)]
pub enum DatabendingEffect {
  XOR { key: u32 },
  Add { value: f32 },
  Multiply { value: f32 },
  BitShift { bits: u8 },
  BitRotate { bits: u8 },
  Complement,
}

#[derive(Debug, Clone)]
pub enum AudioCorruptionType {
  RandomSample,
  BitFlip,
  ChannelSwap,
  Noise,
  Glitch,
  Databending,
  Compression,
}

#[derive(Debug, Clone)]
pub struct AudioCorruptionLayer {
  pub corruption_type: AudioCorruptionType,
  pub intensity: f64,
  pub channel: Option<u8>,
  pub noise_type: Option<NoiseType>,
  pub glitch_type: Option<AudioGlitchType>,
  pub databending_effect: Option<DatabendingEffect>,
  pub compression_artifact: Option<CompressionArtifact>,
}

#[derive(Debug, Clone)]
pub struct AudioCorruptionParameters {
  pub random_sample_corruption: bool,
  pub random_sample_intensity: f64,
  pub bit_flip_corruption: bool,
  pub bit_flip_intensity: f64,
  pub channel_corruption: bool,
  pub corruption_channel: Option<u8>,
  pub channel_intensity: f64,
  pub noise_injection: bool,
  pub noise_type: NoiseType,
  pub noise_intensity: f64,
  pub glitch_corruption: bool,
  pub glitch_type: AudioGlitchType,
  pub glitch_intensity: f64,
  pub compression_artifacts: bool,
  pub compression_artifact: CompressionArtifact,
  pub artifact_intensity: f64,
}

#[derive(Debug, Clone)]
pub struct AudioCorruptionStats {
  pub entropy: f64,
  pub zero_samples: usize,
  pub peak_samples: usize,
  pub unique_values: usize,
  pub total_samples: usize,
  pub avg_run_length: f64,
  pub max_run_length: usize,
  pub run_count: usize,
}

impl Default for AudioCorruptionParameters {
  fn default() -> Self {
    Self {
      random_sample_corruption: false,
      random_sample_intensity: 0.1,
      bit_flip_corruption: false,
      bit_flip_intensity: 0.05,
      channel_corruption: false,
      corruption_channel: None,
      channel_intensity: 0.1,
      noise_injection: false,
      noise_type: NoiseType::Uniform,
      noise_intensity: 0.1,
      glitch_corruption: false,
      glitch_type: AudioGlitchType::Stutter,
      glitch_intensity: 0.1,
      compression_artifacts: false,
      compression_artifact: CompressionArtifact::Aliasing,
      artifact_intensity: 0.1,
    }
  }
}

pub fn create_corruption_engine(audio: AudioProcessor) -> AudioCorruptionEngine {
  AudioCorruptionEngine::new(audio)
}

pub fn create_corruption_engine_with_seed(
  audio: AudioProcessor,
  seed: u64,
) -> AudioCorruptionEngine {
  AudioCorruptionEngine::with_seed(audio, seed)
}
