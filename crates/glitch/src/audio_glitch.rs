use ellastic_audio::{
  AudioData,
  AudioProcessor,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_utils::{
  NoiseGenerator,
  create_noise_generator_with_seed,
  create_random_generator,
};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AudioGlitch {
  SampleCorruption {
    corruption_type: SampleCorruptionType,
    intensity: f32,
  },
  TimeStretch {
    ratio: f32,
    preserve_pitch: bool,
  },
  PitchShift {
    semitones: f32,
    preserve_duration: bool,
  },
  BitCrush {
    bit_depth: u8,
    sample_rate_reduction: u32,
  },
  GlitchLoop {
    loop_size: usize,
    crossfade: f32,
  },
  ReverseSegments {
    segment_length: usize,
  },
  Stutter {
    repeat_count: u32,
    variation: f32,
  },
  RingModulation {
    frequency: f32,
    mix: f32,
  },
  FrequencyModulation {
    carrier_freq: f32,
    mod_freq: f32,
    mod_type: FMType,
  },
  PhaseVocoder {
    bands: usize,
    carrier_input: Option<Vec<f32>>,
  },
  DataBending {
    bend_type: DataBendType,
    intensity: f32,
  },
  Custom {
    custom_function: Box<dyn Fn(&mut AudioProcessor) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleCorruptionType {
  RandomFlip,
  BitFlip,
  SampleDropout,
  Clipping,
  Quantization,
  Saturation,
  Distortion,
  Overload,
  Undervoltage,
  DigitalGlitch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FMType {
  Sine,
  Triangle,
  Square,
  Sawtooth,
  Noise,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataBendType {
  XOR,
  ADD,
  MULTIPLY,
  BIT_SHIFT,
  BIT_ROTATE,
  COMPLEMENT,
  SWAP,
  REVERSE,
  CUSTOM,
}

#[derive(Debug, Clone)]
pub struct AudioGlitchProcessor {
  audio_processor: AudioProcessor,
  random_seed: Option<u64>,
}

impl AudioGlitchProcessor {
  pub fn new(audio_processor: AudioProcessor) -> Self {
    Self {
      audio_processor,
      random_seed: None,
    }
  }

  pub fn audio_processor(&self) -> &AudioProcessor {
    &self.audio_processor
  }

  pub fn audio_processor_mut(&mut self) -> &mut AudioProcessor {
    &mut self.audio_processor
  }

  pub fn into_audio_processor(self) -> AudioProcessor {
    self.audio_processor
  }

  pub fn random_seed(&self) -> Option<u64> {
    self.random_seed
  }

  pub fn set_random_seed(&mut self, seed: u64) {
    self.random_seed = Some(seed);
  }

  pub fn apply_glitch(&mut self, glitch: &AudioGlitch) -> Result<()> {
    match glitch {
      AudioGlitch::SampleCorruption {
        corruption_type,
        intensity,
      } => {
        self.sample_corruption(*corruption_type, *intensity)?;
      }
      AudioGlitch::TimeStretch {
        ratio,
        preserve_pitch,
      } => {
        self.time_stretch(*ratio, *preserve_pitch)?;
      }
      AudioGlitch::PitchShift {
        semitones,
        preserve_duration,
      } => {
        self.pitch_shift(*semitones, *preserve_duration)?;
      }
      AudioGlitch::BitCrush {
        bit_depth,
        sample_rate_reduction,
      } => {
        self.bit_crush(*bit_depth, *sample_rate_reduction)?;
      }
      AudioGlitch::GlitchLoop {
        loop_size,
        crossfade,
      } => {
        self.glitch_loop(*loop_size, *crossfade)?;
      }
      AudioGlitch::ReverseSegments { segment_length } => {
        self.reverse_segments(*segment_length)?;
      }
      AudioGlitch::Stutter {
        repeat_count,
        variation,
      } => {
        self.stutter(*repeat_count, *variation)?;
      }
      AudioGlitch::RingModulation { frequency, mix } => {
        self.ring_modulation(*frequency, *mix)?;
      }
      AudioGlitch::FrequencyModulation {
        carrier_freq,
        mod_freq,
        mod_type,
      } => {
        self.frequency_modulation(*carrier_freq, *mod_freq, *mod_type)?;
      }
      AudioGlitch::PhaseVocoder {
        bands,
        carrier_input,
      } => {
        self.phase_vocoder(*bands, carrier_input.as_ref())?;
      }
      AudioGlitch::DataBending {
        bend_type,
        intensity,
      } => {
        self.data_bending(*bend_type, *intensity)?;
      }
      AudioGlitch::Custom { custom_function } => {
        custom_function(&mut self.audio_processor)?;
      }
    }
    Ok(())
  }

  pub fn apply_glitch_batch(&mut self, glitches: &[AudioGlitch]) -> Result<Vec<AudioProcessor>> {
    let mut results = Vec::new();

    for glitch in glitches {
      let mut temp_processor = self.audio_processor.clone();
      let mut temp_glitcher = AudioGlitchProcessor::new(temp_processor);

      if let Some(seed) = self.random_seed {
        temp_glitcher.set_random_seed(seed);
      }

      temp_glitcher.apply_glitch(glitch)?;
      results.push(temp_glitcher.into_audio_processor());
    }

    Ok(results)
  }

  pub fn sample_corruption(
    &mut self,
    corruption_type: SampleCorruptionType,
    intensity: f32,
  ) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    match corruption_type {
      SampleCorruptionType::RandomFlip => {
        self.random_flip_corruption(samples, intensity)?;
      }
      SampleCorruptionType::BitFlip => {
        self.bit_flip_corruption(samples, intensity)?;
      }
      SampleCorruptionType::SampleDropout => {
        self.sample_dropout_corruption(samples, intensity)?;
      }
      SampleCorruptionType::Clipping => {
        self.clipping_corruption(samples, intensity)?;
      }
      SampleCorruptionType::Quantization => {
        self.quantization_corruption(samples, intensity)?;
      }
      SampleCorruptionType::Saturation => {
        self.saturation_corruption(samples, intensity)?;
      }
      SampleCorruptionType::Distortion => {
        self.distortion_corruption(samples, intensity)?;
      }
      SampleCorruptionType::Overload => {
        self.overload_corruption(samples, intensity)?;
      }
      SampleCorruptionType::Undervoltage => {
        self.undervoltage_corruption(samples, intensity)?;
      }
      SampleCorruptionType::DigitalGlitch => {
        self.digital_glitch_corruption(samples, intensity)?;
      }
    }

    Ok(())
  }

  pub fn time_stretch(&mut self, ratio: f32, preserve_pitch: bool) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    if preserve_pitch {
      self.phase_vocoder_stretch(samples, ratio)?;
    } else {
      self.simple_stretch(samples, ratio)?;
    }

    Ok(())
  }

  pub fn pitch_shift(&mut self, semitones: f32, preserve_duration: bool) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    if preserve_duration {
      self.phase_vocoder_pitch_shift(samples, semitones)?;
    } else {
      self.resample_pitch_shift(samples, semitones)?;
    }

    Ok(())
  }

  pub fn bit_crush(&mut self, bit_depth: u8, sample_rate_reduction: u32) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    self.apply_bit_crush(samples, bit_depth)?;
    self.apply_sample_rate_reduction(samples, sample_rate_reduction)?;

    Ok(())
  }

  pub fn glitch_loop(&mut self, loop_size: usize, crossfade: f32) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    if samples.len() > loop_size {
      let loop_region = samples[..loop_size].to_vec();
      let crossfade_samples = (loop_size as f32 * crossfade) as usize;

      for i in loop_size..samples.len() {
        let loop_pos = i % loop_size;
        if i < loop_size + crossfade_samples {
          let fade_out = 1.0 - (i - loop_size) as f32 / crossfade_samples as f32;
          let fade_in = (i - loop_size) as f32 / crossfade_samples as f32;
          samples[i] = samples[i] * fade_out + loop_region[loop_pos] * fade_in;
        } else {
          samples[i] = loop_region[loop_pos];
        }
      }
    }

    Ok(())
  }

  pub fn reverse_segments(&mut self, segment_length: usize) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    for chunk in samples.chunks_mut(segment_length) {
      chunk.reverse();
    }

    Ok(())
  }

  pub fn stutter(&mut self, repeat_count: u32, variation: f32) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let mut rng = create_random_generator();

    let stutter_size = 1024;
    for i in (0..samples.len()).step_by(stutter_size) {
      if i + stutter_size <= samples.len() {
        let segment = samples[i..i + stutter_size].to_vec();
        for j in 1..repeat_count {
          let start = i + j * stutter_size;
          if start + stutter_size <= samples.len() {
            for k in 0..stutter_size {
              let variation_amount = rng.gen_range(-variation, variation);
              samples[start + k] = segment[k] + variation_amount;
            }
          }
        }
      }
    }

    Ok(())
  }

  pub fn ring_modulation(&mut self, frequency: f32, mix: f32) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let sample_rate = audio_data.sample_rate;

    for (i, sample) in samples.iter_mut().enumerate() {
      let t = i as f32 / sample_rate as f32;
      let carrier = (2.0 * std::f32::consts::PI * frequency * t).sin();
      let modulated = *sample * carrier;
      *sample = *sample * (1.0 - mix) + modulated * mix;
    }

    Ok(())
  }

  pub fn frequency_modulation(
    &mut self,
    carrier_freq: f32,
    mod_freq: f32,
    mod_type: FMType,
  ) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let sample_rate = audio_data.sample_rate;

    for (i, sample) in samples.iter_mut().enumerate() {
      let t = i as f32 / sample_rate as f32;
      let modulator = match mod_type {
        FMType::Sine => (2.0 * std::f32::consts::PI * mod_freq * t).sin(),
        FMType::Triangle => self.triangle_wave(mod_freq * t),
        FMType::Square => self.square_wave(mod_freq * t),
        FMType::Sawtooth => self.sawtooth_wave(mod_freq * t),
        FMType::Noise => rng.gen_range(-1.0, 1.0),
        FMType::Custom => 0.0,
      };
      let carrier = (2.0 * std::f32::consts::PI * carrier_freq * t + modulator).sin();
      *sample = carrier;
    }

    Ok(())
  }

  pub fn phase_vocoder(&mut self, bands: usize, carrier_input: Option<&Vec<f32>>) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    let fft_size = 2048;
    let hop_size = fft_size / 4;
    let window = self.hann_window(fft_size);

    for i in (0..samples.len()).step_by(hop_size) {
      if i + fft_size <= samples.len() {
        let frame = &samples[i..i + fft_size];
        let windowed_frame: Vec<f32> = frame.iter().zip(&window).map(|(s, w)| s * w).collect();

        let spectrum = self.fft(&windowed_frame);
        let magnitude = self.compute_magnitude(&spectrum);
        let phase = self.compute_phase(&spectrum);

        let carrier_spectrum = if let Some(carrier) = carrier_input {
          let carrier_frame = if i + fft_size <= carrier.len() {
            &carrier[i..i + fft_size]
          } else {
            &carrier[carrier.len() - fft_size..]
          };
          let windowed_carrier: Vec<f32> = carrier_frame
            .iter()
            .zip(&window)
            .map(|(s, w)| s * w)
            .collect();
          self.fft(&windowed_carrier)
        } else {
          spectrum.clone()
        };

        let carrier_phase = self.compute_phase(&carrier_spectrum);

        let modified_spectrum: Vec<(f32, f32)> = magnitude
          .iter()
          .zip(&carrier_phase)
          .map(|(mag, phase)| (mag * 0.5, *phase))
          .collect();

        let reconstructed = self.ifft(&modified_spectrum);

        for j in 0..fft_size {
          if i + j < samples.len() {
            samples[i + j] = reconstructed[j];
          }
        }
      }
    }

    Ok(())
  }

  pub fn data_bending(&mut self, bend_type: DataBendType, intensity: f32) -> Result<()> {
    let audio_data = self.audio_processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    match bend_type {
      DataBendType::XOR => {
        self.xor_bend(samples, intensity)?;
      }
      DataBendType::ADD => {
        self.add_bend(samples, intensity)?;
      }
      DataBendType::MULTIPLY => {
        self.multiply_bend(samples, intensity)?;
      }
      DataBendType::BIT_SHIFT => {
        self.bit_shift_bend(samples, intensity)?;
      }
      DataBendType::BIT_ROTATE => {
        self.bit_rotate_bend(samples, intensity)?;
      }
      DataBendType::COMPLEMENT => {
        self.complement_bend(samples, intensity)?;
      }
      DataBendType::SWAP => {
        self.swap_bend(samples, intensity)?;
      }
      DataBendType::REVERSE => {
        self.reverse_bend(samples, intensity)?;
      }
      DataBendType::CUSTOM => {
        self.custom_bend(samples, intensity)?;
      }
    }

    Ok(())
  }

  fn random_flip_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let mut rng = create_random_generator();
    let flip_count = (samples.len() as f32 * intensity) as usize;

    for _ in 0..flip_count {
      let pos = rng.gen_range(0, samples.len() as u64) as usize;
      samples[pos] = -samples[pos];
    }

    Ok(())
  }

  fn bit_flip_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let mut rng = create_random_generator();
    let flip_count = (samples.len() as f32 * intensity) as usize;

    for _ in 0..flip_count {
      let pos = rng.gen_range(0, samples.len() as u64) as usize;
      let bytes = samples[pos].to_le_bytes();
      let mut new_bytes = bytes;
      let bit_pos = rng.gen_range(0, 8);
      new_bytes[bit_pos / 8] ^= 1 << (bit_pos % 8);
      samples[pos] = f32::from_le_bytes(new_bytes);
    }

    Ok(())
  }

  fn sample_dropout_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let mut rng = create_random_generator();
    let dropout_count = (samples.len() as f32 * intensity) as usize;

    for _ in 0..dropout_count {
      let pos = rng.gen_range(0, samples.len() as u64) as usize;
      samples[pos] = 0.0;
    }

    Ok(())
  }

  fn clipping_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let threshold = 1.0 * (1.0 - intensity);

    for sample in samples.iter_mut() {
      if *sample > threshold {
        *sample = threshold;
      } else if *sample < -threshold {
        *sample = -threshold;
      }
    }

    Ok(())
  }

  fn quantization_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let levels = ((1.0 + intensity * 15.0) as u32).max(2);
    let step = 2.0 / levels as f32;

    for sample in samples.iter_mut() {
      *sample = (*sample / step).round() * step;
    }

    Ok(())
  }

  fn saturation_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let saturation_amount = intensity * 2.0;

    for sample in samples.iter_mut() {
      *sample = *sample.tanh() * saturation_amount;
    }

    Ok(())
  }

  fn distortion_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let distortion_amount = intensity * 5.0;

    for sample in samples.iter_mut() {
      *sample = (*sample * distortion_amount).tanh();
    }

    Ok(())
  }

  fn overload_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let overload_factor = 1.0 + intensity * 10.0;

    for sample in samples.iter_mut() {
      *sample = (*sample * overload_factor).clamp(-1.0, 1.0);
    }

    Ok(())
  }

  fn undervoltage_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let voltage_factor = 1.0 - intensity * 0.9;

    for sample in samples.iter_mut() {
      *sample *= voltage_factor;
    }

    Ok(())
  }

  fn digital_glitch_corruption(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let mut rng = create_random_generator();
    let glitch_count = (samples.len() as f32 * intensity) as usize;

    for _ in 0..glitch_count {
      let start = rng.gen_range(0, samples.len() as u64) as usize;
      let end = (start + 1024).min(samples.len());

      for i in start..end {
        samples[i] = rng.gen_range(-1.0, 1.0);
      }
    }

    Ok(())
  }

  fn simple_stretch(&mut self, samples: &mut [f32], ratio: f32) -> Result<()> {
    let new_length = (samples.len() as f32 * ratio) as usize;
    let mut stretched = vec![0.0; new_length];

    for i in 0..new_length {
      let src_pos = i as f32 / ratio;
      let src_index = src_pos as usize;
      let fraction = src_pos - src_index as f32;

      if src_index + 1 < samples.len() {
        stretched[i] = samples[src_index] * (1.0 - fraction) + samples[src_index + 1] * fraction;
      } else {
        stretched[i] = samples[src_index];
      }
    }

    samples.resize(new_length, 0.0);
    samples.copy_from_slice(&stretched);

    Ok(())
  }

  fn phase_vocoder_stretch(&mut self, samples: &mut [f32], ratio: f32) -> Result<()> {
    let new_length = (samples.len() as f32 * ratio) as usize;
    let mut stretched = vec![0.0; new_length];

    let fft_size = 2048;
    let hop_size = fft_size / 4;
    let window = self.hann_window(fft_size);

    for i in (0..new_length).step_by(hop_size) {
      let src_pos = i as f32 / ratio;
      let src_index = src_pos as usize;

      if src_index + fft_size <= samples.len() {
        let frame = &samples[src_index..src_index + fft_size];
        let windowed_frame: Vec<f32> = frame.iter().zip(&window).map(|(s, w)| s * w).collect();

        let spectrum = self.fft(&windowed_frame);
        let magnitude = self.compute_magnitude(&spectrum);
        let phase = self.compute_phase(&spectrum);

        let modified_spectrum: Vec<(f32, f32)> = magnitude
          .iter()
          .zip(&phase)
          .map(|(mag, phase)| (*mag, *phase))
          .collect();

        let reconstructed = self.ifft(&modified_spectrum);

        for j in 0..fft_size {
          if i + j < stretched.len() {
            stretched[i + j] += reconstructed[j] * window[j];
          }
        }
      }
    }

    samples.resize(new_length, 0.0);
    samples.copy_from_slice(&stretched);

    Ok(())
  }

  fn resample_pitch_shift(&mut self, samples: &mut [f32], semitones: f32) -> Result<()> {
    let ratio = 2.0_f32.powf(-semitones / 12.0);
    let new_length = (samples.len() as f32 / ratio) as usize;
    let mut shifted = vec![0.0; new_length];

    for i in 0..new_length {
      let src_pos = i as f32 * ratio;
      let src_index = src_pos as usize;
      let fraction = src_pos - src_index as f32;

      if src_index + 1 < samples.len() {
        shifted[i] = samples[src_index] * (1.0 - fraction) + samples[src_index + 1] * fraction;
      } else {
        shifted[i] = samples[src_index];
      }
    }

    samples.resize(new_length, 0.0);
    samples.copy_from_slice(&shifted);

    Ok(())
  }

  fn phase_vocoder_pitch_shift(&mut self, samples: &mut [f32], semitones: f32) -> Result<()> {
    let ratio = 2.0_f32.powf(-semitones / 12.0);
    let fft_size = 2048;
    let hop_size = fft_size / 4;
    let window = self.hann_window(fft_size);

    for i in (0..samples.len()).step_by(hop_size) {
      if i + fft_size <= samples.len() {
        let frame = &samples[i..i + fft_size];
        let windowed_frame: Vec<f32> = frame.iter().zip(&window).map(|(s, w)| s * w).collect();

        let spectrum = self.fft(&windowed_frame);
        let magnitude = self.compute_magnitude(&spectrum);
        let phase = self.compute_phase(&spectrum);

        let modified_spectrum: Vec<(f32, f32)> = magnitude
          .iter()
          .zip(&phase)
          .map(|(mag, phase)| (*mag, *phase))
          .collect();

        let reconstructed = self.ifft(&modified_spectrum);

        for j in 0..fft_size {
          if i + j < samples.len() {
            samples[i + j] = reconstructed[j] * window[j];
          }
        }
      }
    }

    Ok(())
  }

  fn apply_bit_crush(&mut self, samples: &mut [f32], bit_depth: u8) -> Result<()> {
    let levels = 2.0_f32.powi(bit_depth as i32);
    let step = 2.0 / levels;

    for sample in samples.iter_mut() {
      *sample = (*sample / step).round() * step;
    }

    Ok(())
  }

  fn apply_sample_rate_reduction(&mut self, samples: &mut [f32], reduction: u32) -> Result<()> {
    if reduction > 1 {
      let step = reduction as usize;
      for i in (0..samples.len()).step_by(step) {
        if i + step < samples.len() {
          for j in 1..step {
            samples[i + j] = samples[i];
          }
        }
      }
    }

    Ok(())
  }

  fn xor_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let xor_value = (intensity * 255.0) as u8;

    for sample in samples.iter_mut() {
      let bytes = sample.to_le_bytes();
      let mut new_bytes = bytes;
      for byte in new_bytes.iter_mut() {
        *byte ^= xor_value;
      }
      *sample = f32::from_le_bytes(new_bytes);
    }

    Ok(())
  }

  fn add_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let add_value = intensity * 0.5;

    for sample in samples.iter_mut() {
      *sample = (*sample + add_value).clamp(-1.0, 1.0);
    }

    Ok(())
  }

  fn multiply_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let multiply_factor = 1.0 + intensity;

    for sample in samples.iter_mut() {
      *sample = (*sample * multiply_factor).clamp(-1.0, 1.0);
    }

    Ok(())
  }

  fn bit_shift_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let shift_amount = (intensity * 8.0) as u8;

    for sample in samples.iter_mut() {
      let bytes = sample.to_le_bytes();
      let mut new_bytes = bytes;
      for byte in new_bytes.iter_mut() {
        *byte = (*byte << shift_amount) | (*byte >> (8 - shift_amount));
      }
      *sample = f32::from_le_bytes(new_bytes);
    }

    Ok(())
  }

  fn bit_rotate_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let rotate_amount = (intensity * 8.0) as u8;

    for sample in samples.iter_mut() {
      let bytes = sample.to_le_bytes();
      let mut new_bytes = bytes;
      for byte in new_bytes.iter_mut() {
        let high_bits = *byte & (0xFF << rotate_amount);
        let low_bits = *byte & (0xFF >> (8 - rotate_amount));
        *byte = (high_bits >> rotate_amount) | (low_bits << (8 - rotate_amount));
      }
      *sample = f32::from_le_bytes(new_bytes);
    }

    Ok(())
  }

  fn complement_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let complement_factor = intensity;

    for sample in samples.iter_mut() {
      *sample = *sample * (1.0 - complement_factor) + (1.0 - *sample) * complement_factor;
    }

    Ok(())
  }

  fn swap_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let swap_count = (samples.len() as f32 * intensity) as usize;
    let mut rng = create_random_generator();

    for _ in 0..swap_count {
      let pos1 = rng.gen_range(0, samples.len() as u64) as usize;
      let pos2 = rng.gen_range(0, samples.len() as u64) as usize;
      samples.swap(pos1, pos2);
    }

    Ok(())
  }

  fn reverse_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    let reverse_count = (samples.len() as f32 * intensity) as usize;

    for i in 0..reverse_count {
      let start = i;
      let end = samples.len() - 1 - i;
      if start < end {
        samples.swap(start, end);
      }
    }

    Ok(())
  }

  fn custom_bend(&mut self, samples: &mut [f32], intensity: f32) -> Result<()> {
    for sample in samples.iter_mut() {
      *sample = (*sample * intensity).sin();
    }

    Ok(())
  }

  fn hann_window(&self, size: usize) -> Vec<f32> {
    (0..size)
      .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32).cos()))
      .collect()
  }

  fn fft(&self, samples: &[f32]) -> Vec<(f32, f32)> {
    let n = samples.len();
    let mut spectrum = Vec::with_capacity(n);

    for k in 0..n {
      let mut real = 0.0;
      let mut imag = 0.0;

      for i in 0..n {
        let angle = -2.0 * std::f32::consts::PI * k as f32 * i as f32 / n as f32;
        real += samples[i] * angle.cos();
        imag += samples[i] * angle.sin();
      }

      spectrum.push((real, imag));
    }

    spectrum
  }

  fn ifft(&self, spectrum: &[(f32, f32)]) -> Vec<f32> {
    let n = spectrum.len();
    let mut samples = Vec::with_capacity(n);

    for i in 0..n {
      let mut real = 0.0;
      let mut imag = 0.0;

      for k in 0..n {
        let angle = 2.0 * std::f32::consts::PI * k as f32 * i as f32 / n as f32;
        real += spectrum[k].0 * angle.cos() - spectrum[k].1 * angle.sin();
        imag += spectrum[k].0 * angle.sin() + spectrum[k].1 * angle.cos();
      }

      samples.push(real / n as f32);
    }

    samples
  }

  fn compute_magnitude(&self, spectrum: &[(f32, f32)]) -> Vec<f32> {
    spectrum
      .iter()
      .map(|(real, imag)| (real * real + imag * imag).sqrt())
      .collect()
  }

  fn compute_phase(&self, spectrum: &[(f32, f32)]) -> Vec<f32> {
    spectrum
      .iter()
      .map(|(real, imag)| imag.atan2(*real))
      .collect()
  }

  fn triangle_wave(&self, phase: f32) -> f32 {
    let t = phase.fract();
    if t < 0.5 {
      4.0 * t - 1.0
    } else {
      3.0 - 4.0 * t
    }
  }

  fn square_wave(&self, phase: f32) -> f32 {
    if phase.fract() < 0.5 { 1.0 } else { -1.0 }
  }

  fn sawtooth_wave(&self, phase: f32) -> f32 {
    2.0 * phase.fract() - 1.0
  }

  pub fn clone(&self) -> AudioGlitchProcessor {
    AudioGlitchProcessor {
      audio_processor: self.audio_processor.clone(),
      random_seed: self.random_seed,
    }
  }
}

pub fn create_audio_glitch_processor(audio_processor: AudioProcessor) -> AudioGlitchProcessor {
  AudioGlitchProcessor::new(audio_processor)
}

pub fn create_sample_corruption_glitch(
  corruption_type: SampleCorruptionType,
  intensity: f32,
) -> AudioGlitch {
  AudioGlitch::SampleCorruption {
    corruption_type,
    intensity,
  }
}

pub fn create_time_stretch_glitch(ratio: f32, preserve_pitch: bool) -> AudioGlitch {
  AudioGlitch::TimeStretch {
    ratio,
    preserve_pitch,
  }
}

pub fn create_pitch_shift_glitch(semitones: f32, preserve_duration: bool) -> AudioGlitch {
  AudioGlitch::PitchShift {
    semitones,
    preserve_duration,
  }
}

pub fn create_bit_crush_glitch(bit_depth: u8, sample_rate_reduction: u32) -> AudioGlitch {
  AudioGlitch::BitCrush {
    bit_depth,
    sample_rate_reduction,
  }
}

pub fn create_glitch_loop_glitch(loop_size: usize, crossfade: f32) -> AudioGlitch {
  AudioGlitch::GlitchLoop {
    loop_size,
    crossfade,
  }
}

pub fn create_reverse_segments_glitch(segment_length: usize) -> AudioGlitch {
  AudioGlitch::ReverseSegments { segment_length }
}

pub fn create_stutter_glitch(repeat_count: u32, variation: f32) -> AudioGlitch {
  AudioGlitch::Stutter {
    repeat_count,
    variation,
  }
}

pub fn create_ring_modulation_glitch(frequency: f32, mix: f32) -> AudioGlitch {
  AudioGlitch::RingModulation { frequency, mix }
}

pub fn create_frequency_modulation_glitch(
  carrier_freq: f32,
  mod_freq: f32,
  mod_type: FMType,
) -> AudioGlitch {
  AudioGlitch::FrequencyModulation {
    carrier_freq,
    mod_freq,
    mod_type,
  }
}

pub fn create_phase_vocoder_glitch(bands: usize, carrier_input: Option<Vec<f32>>) -> AudioGlitch {
  AudioGlitch::PhaseVocoder {
    bands,
    carrier_input,
  }
}

pub fn create_data_bending_glitch(bend_type: DataBendType, intensity: f32) -> AudioGlitch {
  AudioGlitch::DataBending {
    bend_type,
    intensity,
  }
}
