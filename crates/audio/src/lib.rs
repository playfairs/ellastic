use ellastic_core::{
  AudioData,
  MediaData,
  MediaType,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use std::path::Path;

pub mod analysis;
pub mod corruption;
pub mod effects;
pub mod formats;
pub mod processing;

pub use analysis::*;
pub use corruption::*;
pub use effects::*;
pub use formats::*;
pub use processing::*;

#[derive(Debug, Clone)]
pub struct AudioProcessor {
  data: AudioData,
}

impl AudioProcessor {
  pub fn new(sample_rate: u32, channels: u8, samples: Vec<f32>) -> Result<Self> {
    let audio_data = AudioData::new(sample_rate, channels, samples)?;
    Ok(Self { data: audio_data })
  }

  pub fn from_audio_data(data: AudioData) -> Self {
    Self { data }
  }

  pub fn from_buffer(buffer: Vec<u8>, format: AudioFormat) -> Result<Self> {
    let audio_data = decode_audio(&buffer, format)?;
    Ok(Self { data: audio_data })
  }

  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    let buffer = std::fs::read(path)
      .map_err(|e| EllasticError::IoError(format!("Failed to read audio file: {}", e)))?;

    let format = detect_format(&buffer)?;
    Self::from_buffer(buffer, format)
  }

  pub fn data(&self) -> &AudioData {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut AudioData {
    &mut self.data
  }

  pub fn into_data(self) -> AudioData {
    self.data
  }

  pub fn sample_rate(&self) -> u32 {
    self.data.sample_rate
  }

  pub fn channels(&self) -> u8 {
    self.data.channels
  }

  pub fn sample_count(&self) -> usize {
    self.data.samples.len()
  }

  pub fn duration_seconds(&self) -> f64 {
    self.data.duration_seconds()
  }

  pub fn get_sample(&self, index: usize) -> Option<f32> {
    self.data.samples.get(index).copied()
  }

  pub fn get_sample_mut(&mut self, index: usize) -> Option<&mut f32> {
    self.data.samples.get_mut(index)
  }

  pub fn get_channel(&self, channel: u8) -> Result<Vec<f32>> {
    if channel >= self.data.channels {
      return Err(EllasticError::InvalidParameter(
        "Channel out of range".to_string(),
      ));
    }

    let mut channel_samples = Vec::with_capacity(self.sample_count() / self.channels as usize);
    for (i, &sample) in self.data.samples.iter().enumerate() {
      if i % self.channels as usize == channel as usize {
        channel_samples.push(sample);
      }
    }

    Ok(channel_samples)
  }

  pub fn set_channel(&mut self, channel: u8, samples: &[f32]) -> Result<()> {
    if channel >= self.data.channels {
      return Err(EllasticError::InvalidParameter(
        "Channel out of range".to_string(),
      ));
    }

    if samples.len() != self.sample_count() / self.channels as usize {
      return Err(EllasticError::InvalidParameter(
        "Channel sample count mismatch".to_string(),
      ));
    }

    for (i, &sample) in samples.iter().enumerate() {
      let sample_index = i * self.channels as usize + channel as usize;
      if let Some(target) = self.data.samples.get_mut(sample_index) {
        *target = sample;
      }
    }

    Ok(())
  }

  pub fn get_frame(&self, frame_index: usize) -> Option<Vec<f32>> {
    let start_index = frame_index * self.channels as usize;
    let end_index = start_index + self.channels as usize;

    if end_index > self.data.samples.len() {
      None
    } else {
      Some(self.data.samples[start_index..end_index].to_vec())
    }
  }

  pub fn set_frame(&mut self, frame_index: usize, frame: &[f32]) -> Result<()> {
    if frame.len() != self.channels as usize {
      return Err(EllasticError::InvalidParameter(
        "Frame size mismatch".to_string(),
      ));
    }

    let start_index = frame_index * self.channels as usize;
    let end_index = start_index + self.channels as usize;

    if end_index > self.data.samples.len() {
      return Err(EllasticError::InvalidParameter(
        "Frame index out of range".to_string(),
      ));
    }

    for (i, &sample) in frame.iter().enumerate() {
      self.data.samples[start_index + i] = sample;
    }

    Ok(())
  }

  pub fn get_stereo_samples(&self) -> Option<(Vec<f32>, Vec<f32>)> {
    if self.data.channels != 2 {
      return None;
    }

    let mut left = Vec::with_capacity(self.sample_count() / 2);
    let mut right = Vec::with_capacity(self.sample_count() / 2);

    for chunk in self.data.samples.chunks_exact(2) {
      left.push(chunk[0]);
      right.push(chunk[1]);
    }

    Some((left, right))
  }

  pub fn set_stereo_samples(&mut self, left: &[f32], right: &[f32]) -> Result<()> {
    if self.data.channels != 2 {
      return Err(EllasticError::InvalidParameter(
        "Not stereo audio".to_string(),
      ));
    }

    if left.len() != right.len() {
      return Err(EllasticError::InvalidParameter(
        "Channel length mismatch".to_string(),
      ));
    }

    if left.len() * 2 != self.data.samples.len() {
      return Err(EllasticError::InvalidParameter(
        "Sample count mismatch".to_string(),
      ));
    }

    for (i, (&l, &r)) in left.iter().zip(right.iter()).enumerate() {
      self.data.samples[i * 2] = l;
      self.data.samples[i * 2 + 1] = r;
    }

    Ok(())
  }

  pub fn get_mono_samples(&self) -> Option<Vec<f32>> {
    if self.data.channels == 1 {
      Some(self.data.samples.clone())
    } else {
      let mut mono = Vec::with_capacity(self.sample_count() / self.channels as usize);

      for chunk in self.data.samples.chunks_exact(self.channels as usize) {
        let sum: f32 = chunk.iter().sum();
        mono.push(sum / self.channels as f32);
      }

      Some(mono)
    }
  }

  pub fn set_mono_samples(&mut self, samples: &[f32]) -> Result<()> {
    for (i, &sample) in samples.iter().enumerate() {
      for channel in 0..self.data.channels {
        let sample_index = i * self.channels as usize + channel as usize;
        if let Some(target) = self.data.samples.get_mut(sample_index) {
          *target = sample;
        }
      }
    }

    Ok(())
  }

  pub fn resample(&self, new_sample_rate: u32, method: ResampleMethod) -> Result<AudioProcessor> {
    let resampled_data = resample_audio(&self.data, new_sample_rate, method)?;
    Ok(AudioProcessor::from_audio_data(resampled_data))
  }

  pub fn change_channels(
    &self,
    new_channels: u8,
    method: ChannelConversionMethod,
  ) -> Result<AudioProcessor> {
    let converted_data = convert_channels(&self.data, new_channels, method)?;
    Ok(AudioProcessor::from_audio_data(converted_data))
  }

  pub fn trim(&self, start_seconds: f64, end_seconds: f64) -> Result<AudioProcessor> {
    let start_sample = (start_seconds * self.data.sample_rate as f64) as usize;
    let end_sample = (end_seconds * self.data.sample_rate as f64) as usize;

    if start_sample >= self.data.samples.len()
      || end_sample > self.data.samples.len()
      || start_sample >= end_sample
    {
      return Err(EllasticError::InvalidParameter(
        "Invalid trim range".to_string(),
      ));
    }

    let trimmed_samples = self.data.samples[start_sample..end_sample].to_vec();
    let trimmed_data = AudioData::new(self.data.sample_rate, self.data.channels, trimmed_samples)?;
    Ok(AudioProcessor::from_audio_data(trimmed_data))
  }

  pub fn append(&self, other: &AudioProcessor) -> Result<AudioProcessor> {
    if self.data.sample_rate != other.data.sample_rate || self.data.channels != other.data.channels
    {
      return Err(EllasticError::InvalidParameter(
        "Audio properties must match".to_string(),
      ));
    }

    let mut combined_samples = self.data.samples.clone();
    combined_samples.extend_from_slice(&other.data.samples);

    let combined_data =
      AudioData::new(self.data.sample_rate, self.data.channels, combined_samples)?;
    Ok(AudioProcessor::from_audio_data(combined_data))
  }

  pub fn repeat(&self, times: usize) -> Result<AudioProcessor> {
    let mut repeated_samples = Vec::with_capacity(self.data.samples.len() * times);

    for _ in 0..times {
      repeated_samples.extend_from_slice(&self.data.samples);
    }

    let repeated_data =
      AudioData::new(self.data.sample_rate, self.data.channels, repeated_samples)?;
    Ok(AudioProcessor::from_audio_data(repeated_data))
  }

  pub fn reverse(&self) -> Result<AudioProcessor> {
    let mut reversed_samples = self.data.samples.clone();
    reversed_samples.reverse();

    let reversed_data =
      AudioData::new(self.data.sample_rate, self.data.channels, reversed_samples)?;
    Ok(AudioProcessor::from_audio_data(reversed_data))
  }

  pub fn apply_filter(&mut self, filter: &AudioFilter) -> Result<()> {
    apply_audio_filter(&mut self.data, filter)
  }

  pub fn apply_effect(&mut self, effect: &AudioEffect) -> Result<()> {
    apply_audio_effect(&mut self.data, effect)
  }

  pub fn normalize(&mut self, target_level: f32) -> Result<()> {
    normalize_audio(&mut self.data, target_level)
  }

  pub fn fade_in(&mut self, duration_seconds: f64) -> Result<()> {
    let fade_samples = (duration_seconds * self.data.sample_rate as f64) as usize;
    fade_in_audio(&mut self.data, fade_samples)
  }

  pub fn fade_out(&mut self, duration_seconds: f64) -> Result<()> {
    let fade_samples = (duration_seconds * self.data.sample_rate as f64) as usize;
    fade_out_audio(&mut self.data, fade_samples)
  }

  pub fn crossfade(&self, other: &AudioProcessor, duration_seconds: f64) -> Result<AudioProcessor> {
    let fade_samples = (duration_seconds * self.data.sample_rate as f64) as usize;
    crossfade_audio(&self.data, &other.data, fade_samples)
  }

  pub fn get_amplitude_envelope(&self, window_size: usize) -> Vec<f32> {
    calculate_amplitude_envelope(&self.data, window_size)
  }

  pub fn get_rms(&self) -> f32 {
    calculate_rms(&self.data)
  }

  pub fn get_peak(&self) -> f32 {
    calculate_peak(&self.data)
  }

  pub fn get_dynamic_range(&self) -> f32 {
    calculate_dynamic_range(&self.data)
  }

  pub fn get_zero_crossing_rate(&self) -> f64 {
    calculate_zero_crossing_rate(&self.data)
  }

  pub fn get_frequency_spectrum(&self, fft_size: usize) -> Vec<f32> {
    calculate_frequency_spectrum(&self.data, fft_size)
  }

  pub fn get_spectrogram(&self, fft_size: usize, hop_size: usize) -> Vec<Vec<f32>> {
    calculate_spectrogram(&self.data, fft_size, hop_size)
  }

  pub fn encode(&self, format: AudioFormat, quality: Option<u8>) -> Result<Vec<u8>> {
    encode_audio(&self.data, format, quality)
  }

  pub fn save<P: AsRef<Path>>(
    &self,
    path: P,
    format: AudioFormat,
    quality: Option<u8>,
  ) -> Result<()> {
    let encoded = self.encode(format, quality)?;
    std::fs::write(path, encoded)
      .map_err(|e| EllasticError::IoError(format!("Failed to save audio: {}", e)))
  }

  pub fn clone(&self) -> AudioProcessor {
    AudioProcessor {
      data: AudioData::new(
        self.data.sample_rate,
        self.data.channels,
        self.data.samples.clone(),
      )
      .unwrap(),
    }
  }

  pub fn to_media_data(self) -> MediaData {
    MediaData::Audio(self.data)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
  WAV,
  MP3,
  FLAC,
  OGG,
  AAC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResampleMethod {
  Nearest,
  Linear,
  Cubic,
  Sinc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelConversionMethod {
  Downmix,
  Upmix,
  Duplicate,
  Average,
}

#[derive(Debug, Clone)]
pub struct AudioFilter {
  pub filter_type: FilterType,
  pub parameters: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum FilterType {
  LowPass {
    cutoff: f32,
  },
  HighPass {
    cutoff: f32,
  },
  BandPass {
    low_cutoff: f32,
    high_cutoff: f32,
  },
  Notch {
    center: f32,
    bandwidth: f32,
  },
  LowShelf {
    cutoff: f32,
    gain: f32,
  },
  HighShelf {
    cutoff: f32,
    gain: f32,
  },
  Peaking {
    center: f32,
    bandwidth: f32,
    gain: f32,
  },
}

#[derive(Debug, Clone)]
pub struct AudioEffect {
  pub effect_type: EffectType,
  pub parameters: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum EffectType {
  Reverb {
    room_size: f32,
    damping: f32,
  },
  Delay {
    time: f32,
    feedback: f32,
    mix: f32,
  },
  Chorus {
    rate: f32,
    depth: f32,
    mix: f32,
  },
  Flanger {
    rate: f32,
    depth: f32,
    feedback: f32,
  },
  Distortion {
    drive: f32,
    tone: f32,
  },
  Compressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
  },
  Limiter {
    threshold: f32,
    release: f32,
  },
  Gate {
    threshold: f32,
    attack: f32,
    release: f32,
    hold: f32,
  },
  Pitch {
    semitones: f32,
  },
  TimeStretch {
    ratio: f32,
  },
}

pub fn detect_format(buffer: &[u8]) -> Result<AudioFormat> {
  if buffer.len() < 12 {
    return Err(EllasticError::UnsupportedFormat(
      "Insufficient data".to_string(),
    ));
  }

  if buffer.starts_with(b"RIFF") && buffer.len() > 12 && &buffer[8..12] == b"WAVE" {
    Ok(AudioFormat::WAV)
  } else if buffer.starts_with(b"ID3") || (buffer[0] == 0xFF && buffer[1] == 0xFB) {
    Ok(AudioFormat::MP3)
  } else if buffer.starts_with(b"fLaC") {
    Ok(AudioFormat::FLAC)
  } else if buffer.starts_with(b"OggS") {
    Ok(AudioFormat::OGG)
  } else {
    Err(EllasticError::UnsupportedFormat(
      "Unknown audio format".to_string(),
    ))
  }
}

pub fn create_audio_processor(
  sample_rate: u32,
  channels: u8,
  samples: Vec<f32>,
) -> Result<AudioProcessor> {
  AudioProcessor::new(sample_rate, channels, samples)
}

pub fn load_audio<P: AsRef<Path>>(path: P) -> Result<AudioProcessor> {
  AudioProcessor::from_file(path)
}

pub fn create_silence(
  sample_rate: u32,
  channels: u8,
  duration_seconds: f64,
) -> Result<AudioProcessor> {
  let sample_count = (duration_seconds * sample_rate as f64) as usize * channels as usize;
  let samples = vec![0.0f32; sample_count];
  AudioProcessor::new(sample_rate, channels, samples)
}

pub fn create_sine_wave(
  sample_rate: u32,
  channels: u8,
  frequency: f32,
  duration_seconds: f64,
  amplitude: f32,
) -> Result<AudioProcessor> {
  let sample_count = (duration_seconds * sample_rate as f64) as usize * channels as usize;
  let mut samples = Vec::with_capacity(sample_count);

  for i in 0..sample_count {
    let time = i as f32 / sample_rate as f32;
    let sample = amplitude * (2.0 * std::f32::consts::PI * frequency * time).sin();
    samples.push(sample);
  }

  AudioProcessor::new(sample_rate, channels, samples)
}

pub fn create_square_wave(
  sample_rate: u32,
  channels: u8,
  frequency: f32,
  duration_seconds: f64,
  amplitude: f32,
) -> Result<AudioProcessor> {
  let sample_count = (duration_seconds * sample_rate as f64) as usize * channels as usize;
  let mut samples = Vec::with_capacity(sample_count);

  for i in 0..sample_count {
    let time = i as f32 / sample_rate as f32;
    let phase = (frequency * time) % 1.0;
    let sample = amplitude * if phase < 0.5 { 1.0 } else { -1.0 };
    samples.push(sample);
  }

  AudioProcessor::new(sample_rate, channels, samples)
}

pub fn create_sawtooth_wave(
  sample_rate: u32,
  channels: u8,
  frequency: f32,
  duration_seconds: f64,
  amplitude: f32,
) -> Result<AudioProcessor> {
  let sample_count = (duration_seconds * sample_rate as f64) as usize * channels as usize;
  let mut samples = Vec::with_capacity(sample_count);

  for i in 0..sample_count {
    let time = i as f32 / sample_rate as f32;
    let phase = (frequency * time) % 1.0;
    let sample = amplitude * (2.0 * phase - 1.0);
    samples.push(sample);
  }

  AudioProcessor::new(sample_rate, channels, samples)
}

pub fn create_triangle_wave(
  sample_rate: u32,
  channels: u8,
  frequency: f32,
  duration_seconds: f64,
  amplitude: f32,
) -> Result<AudioProcessor> {
  let sample_count = (duration_seconds * sample_rate as f64) as usize * channels as usize;
  let mut samples = Vec::with_capacity(sample_count);

  for i in 0..sample_count {
    let time = i as f32 / sample_rate as f32;
    let phase = (frequency * time) % 1.0;
    let sample = amplitude
      * if phase < 0.5 {
        4.0 * phase - 1.0
      } else {
        3.0 - 4.0 * phase
      };
    samples.push(sample);
  }

  AudioProcessor::new(sample_rate, channels, samples)
}

pub fn create_white_noise(
  sample_rate: u32,
  channels: u8,
  duration_seconds: f64,
  amplitude: f32,
) -> Result<AudioProcessor> {
  let sample_count = (duration_seconds * sample_rate as f64) as usize * channels as usize;
  let mut samples = Vec::with_capacity(sample_count);

  for _ in 0..sample_count {
    let sample = amplitude * (rand::random::<f32>() * 2.0 - 1.0);
    samples.push(sample);
  }

  AudioProcessor::new(sample_rate, channels, samples)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_audio_processor_creation() {
    let processor = create_audio_processor(44100, 2, vec![0.0f32; 44100 * 2]).unwrap();
    assert_eq!(processor.sample_rate(), 44100);
    assert_eq!(processor.channels(), 2);
    assert_eq!(processor.sample_count(), 88200);
  }

  #[test]
  fn test_sine_wave_generation() {
    let processor = create_sine_wave(44100, 1, 440.0, 1.0, 1.0).unwrap();
    assert_eq!(processor.sample_rate(), 44100);
    assert_eq!(processor.channels(), 1);
    assert_eq!(processor.sample_count(), 44100);
  }

  #[test]
  fn test_stereo_operations() {
    let mut processor = create_audio_processor(44100, 2, vec![0.0f32; 88200]).unwrap();

    let left = vec![1.0f32; 44100];
    let right = vec![-1.0f32; 44100];
    processor.set_stereo_samples(&left, &right).unwrap();

    let (retrieved_left, retrieved_right) = processor.get_stereo_samples().unwrap();
    assert_eq!(retrieved_left, left);
    assert_eq!(retrieved_right, right);
  }

  #[test]
  fn test_format_detection() {
    let wav_header = b"RIFF\x24\x08\x00\x00\x00WAVE";
    assert_eq!(detect_format(wav_header).unwrap(), AudioFormat::WAV);

    let mp3_header = b"ID3\x04\x00\x00\x00\x00\x00\x00";
    assert_eq!(detect_format(mp3_header).unwrap(), AudioFormat::MP3);
  }
}
