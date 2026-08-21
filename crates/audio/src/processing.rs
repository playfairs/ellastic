use crate::{
  AudioData,
  AudioFormat,
  AudioProcessor,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use hound::{
  WavReader,
  WavSpec,
  WavWriter,
};
use std::io::{
  Cursor,
  Read,
  Write,
};

pub fn decode_audio(data: &[u8], format: AudioFormat) -> Result<AudioData> {
  match format {
    AudioFormat::WAV => decode_wav(data),
    AudioFormat::MP3 => decode_mp3(data),
    AudioFormat::FLAC => decode_flac(data),
    AudioFormat::OGG => decode_ogg(data),
    AudioFormat::AAC => decode_aac(data),
  }
}

pub fn encode_audio(
  audio_data: &AudioData,
  format: AudioFormat,
  quality: Option<u8>,
) -> Result<Vec<u8>> {
  match format {
    AudioFormat::WAV => encode_wav(audio_data),
    AudioFormat::MP3 => encode_mp3(audio_data, quality),
    AudioFormat::FLAC => encode_flac(audio_data),
    AudioFormat::OGG => encode_ogg(audio_data, quality),
    AudioFormat::AAC => encode_aac(audio_data, quality),
  }
}

fn decode_wav(data: &[u8]) -> Result<AudioData> {
  let cursor = Cursor::new(data);
  let mut reader = WavReader::new(cursor)
    .map_err(|e| EllasticError::IoError(format!("WAV decode error: {}", e)))?;

  let spec = reader.spec();
  let samples: Vec<f32> = reader
    .into_samples()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| EllasticError::IoError(format!("WAV sample conversion error: {}", e)))?;

  AudioData::new(spec.sample_rate, spec.channels as u8, samples)
}

fn encode_wav(audio_data: &AudioData) -> Result<Vec<u8>> {
  let spec = WavSpec {
    channels: audio_data.channels as u16,
    sample_rate: audio_data.sample_rate,
    bits_per_sample: 32,
    sample_format: hound::SampleFormat::Float,
  };

  let mut buffer = Vec::new();
  {
    let cursor = Cursor::new(&mut buffer);
    let mut writer = WavWriter::new(cursor, spec)
      .map_err(|e| EllasticError::IoError(format!("WAV writer error: {}", e)))?;

    for &sample in &audio_data.samples {
      writer
        .write_sample(sample)
        .map_err(|e| EllasticError::IoError(format!("WAV write error: {}", e)))?;
    }

    writer
      .finalize()
      .map_err(|e| EllasticError::IoError(format!("WAV finalize error: {}", e)))?;
  }

  Ok(buffer)
}

fn decode_mp3(data: &[u8]) -> Result<AudioData> {
  Err(EllasticError::UnsupportedFormat(
    "MP3 decoding not implemented".to_string(),
  ))
}

fn encode_mp3(audio_data: &AudioData, quality: Option<u8>) -> Result<Vec<u8>> {
  Err(EllasticError::UnsupportedFormat(
    "MP3 encoding not implemented".to_string(),
  ))
}

fn decode_flac(data: &[u8]) -> Result<AudioData> {
  Err(EllasticError::UnsupportedFormat(
    "FLAC decoding not implemented".to_string(),
  ))
}

fn encode_flac(audio_data: &AudioData) -> Result<Vec<u8>> {
  Err(EllasticError::UnsupportedFormat(
    "FLAC encoding not implemented".to_string(),
  ))
}

fn decode_ogg(data: &[u8]) -> Result<AudioData> {
  Err(EllasticError::UnsupportedFormat(
    "OGG decoding not implemented".to_string(),
  ))
}

fn encode_ogg(audio_data: &AudioData, quality: Option<u8>) -> Result<Vec<u8>> {
  Err(EllasticError::UnsupportedFormat(
    "OGG encoding not implemented".to_string(),
  ))
}

fn decode_aac(data: &[u8]) -> Result<AudioData> {
  Err(EllasticError::UnsupportedFormat(
    "AAC decoding not implemented".to_string(),
  ))
}

fn encode_aac(audio_data: &AudioData, quality: Option<u8>) -> Result<Vec<u8>> {
  Err(EllasticError::UnsupportedFormat(
    "AAC encoding not implemented".to_string(),
  ))
}

pub fn resample_audio(
  audio_data: &AudioData,
  new_sample_rate: u32,
  method: crate::ResampleMethod,
) -> Result<AudioData> {
  if audio_data.sample_rate == new_sample_rate {
    return Ok(audio_data.clone());
  }

  let ratio = new_sample_rate as f64 / audio_data.sample_rate as f64;
  let new_sample_count = (audio_data.samples.len() as f64 * ratio) as usize;
  let mut resampled_samples = Vec::with_capacity(new_sample_count);

  match method {
    crate::ResampleMethod::Nearest => {
      for i in 0..new_sample_count {
        let source_index = (i as f64 / ratio) as usize;
        resampled_samples.push(audio_data.samples[source_index]);
      }
    }
    crate::ResampleMethod::Linear => {
      for i in 0..new_sample_count {
        let source_index = i as f64 / ratio;
        let index0 = source_index as usize;
        let index1 = std::cmp::min(index0 + 1, audio_data.samples.len() - 1);
        let fraction = source_index - index0 as f64;

        let sample0 = audio_data.samples[index0];
        let sample1 = audio_data.samples[index1];
        let interpolated = sample0 + fraction * (sample1 - sample0);

        resampled_samples.push(interpolated);
      }
    }
    crate::ResampleMethod::Cubic => {
      for i in 0..new_sample_count {
        let source_index = i as f64 / ratio;
        let index0 = source_index as usize;

        let p0 = if index0 > 0 {
          audio_data.samples[index0 - 1]
        } else {
          audio_data.samples[index0]
        };
        let p1 = audio_data.samples[index0];
        let p2 = if index0 + 1 < audio_data.samples.len() {
          audio_data.samples[index0 + 1]
        } else {
          audio_data.samples[index0]
        };
        let p3 = if index0 + 2 < audio_data.samples.len() {
          audio_data.samples[index0 + 2]
        } else {
          audio_data.samples[index0]
        };

        let t = source_index - index0 as f64;
        let t2 = t * t;
        let t3 = t2 * t;

        let interpolated = 0.5
          * ((2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3);

        resampled_samples.push(interpolated);
      }
    }
    crate::ResampleMethod::Sinc => {
      let window_size = 8;
      for i in 0..new_sample_count {
        let source_index = i as f64 / ratio;
        let center = source_index as usize;

        let mut sum = 0.0f32;
        let mut weight_sum = 0.0f32;

        for j in -window_size..=window_size {
          let source_pos = center as isize + j;
          if source_pos >= 0 && source_pos < audio_data.samples.len() as isize {
            let sample = audio_data.samples[source_pos as usize];
            let x = source_index - source_pos as f64;

            if x.abs() < 1e-6 {
              sum += sample;
              weight_sum += 1.0;
            } else {
              let weight =
                (std::f32::consts::PI * x as f32).sin() / (std::f32::consts::PI * x as f32);
              sum += sample * weight;
              weight_sum += weight;
            }
          }
        }

        resampled_samples.push(if weight_sum > 0.0 {
          sum / weight_sum
        } else {
          0.0
        });
      }
    }
  }

  AudioData::new(new_sample_rate, audio_data.channels, resampled_samples)
}

pub fn convert_channels(
  audio_data: &AudioData,
  new_channels: u8,
  method: crate::ChannelConversionMethod,
) -> Result<AudioData> {
  if audio_data.channels == new_channels {
    return Ok(audio_data.clone());
  }

  let mut converted_samples = Vec::with_capacity(
    (audio_data.samples.len() / audio_data.channels as usize) * new_channels as usize,
  );

  match method {
    crate::ChannelConversionMethod::Downmix => {
      if new_channels == 1 && audio_data.channels > 1 {
        for chunk in audio_data
          .samples
          .chunks_exact(audio_data.channels as usize)
        {
          let sum: f32 = chunk.iter().sum();
          converted_samples.push(sum / audio_data.channels as f32);
        }
      } else {
        return Err(EllasticError::UnsupportedFormat(
          "Unsupported channel conversion".to_string(),
        ));
      }
    }
    crate::ChannelConversionMethod::Upmix => {
      if new_channels > 1 && audio_data.channels == 1 {
        for &sample in &audio_data.samples {
          for _ in 0..new_channels {
            converted_samples.push(sample);
          }
        }
      } else {
        return Err(EllasticError::UnsupportedFormat(
          "Unsupported channel conversion".to_string(),
        ));
      }
    }
    crate::ChannelConversionMethod::Duplicate => {
      if new_channels > 1 && audio_data.channels == 1 {
        for &sample in &audio_data.samples {
          converted_samples.push(sample);
          converted_samples.push(sample);
          for _ in 2..new_channels {
            converted_samples.push(0.0);
          }
        }
      } else {
        return Err(EllasticError::UnsupportedFormat(
          "Unsupported channel conversion".to_string(),
        ));
      }
    }
    crate::ChannelConversionMethod::Average => {
      if new_channels == 1 && audio_data.channels > 1 {
        for chunk in audio_data
          .samples
          .chunks_exact(audio_data.channels as usize)
        {
          let sum: f32 = chunk.iter().sum();
          converted_samples.push(sum / audio_data.channels as f32);
        }
      } else {
        return Err(EllasticError::UnsupportedFormat(
          "Unsupported channel conversion".to_string(),
        ));
      }
    }
  }

  AudioData::new(audio_data.sample_rate, new_channels, converted_samples)
}

pub fn normalize_audio(audio_data: &mut AudioData, target_level: f32) -> Result<()> {
  let current_peak = calculate_peak(audio_data);
  if current_peak == 0.0 {
    return Ok(());
  }

  let scale_factor = target_level / current_peak;
  for sample in audio_data.samples.iter_mut() {
    *sample *= scale_factor;
  }

  Ok(())
}

pub fn fade_in_audio(audio_data: &mut AudioData, fade_samples: usize) -> Result<()> {
  if fade_samples == 0 {
    return Ok(());
  }

  let fade_samples = std::cmp::min(fade_samples, audio_data.samples.len());
  for (i, sample) in audio_data.samples.iter_mut().enumerate().take(fade_samples) {
    let fade_factor = i as f32 / fade_samples as f32;
    *sample *= fade_factor;
  }

  Ok(())
}

pub fn fade_out_audio(audio_data: &mut AudioData, fade_samples: usize) -> Result<()> {
  if fade_samples == 0 {
    return Ok(());
  }

  let fade_samples = std::cmp::min(fade_samples, audio_data.samples.len());
  let start_index = audio_data.samples.len() - fade_samples;

  for (i, sample) in audio_data.samples.iter_mut().enumerate().skip(start_index) {
    let fade_factor = (fade_samples - i) as f32 / fade_samples as f32;
    *sample *= fade_factor;
  }

  Ok(())
}

pub fn crossfade_audio(
  audio1: &AudioData,
  audio2: &AudioData,
  fade_samples: usize,
) -> Result<AudioProcessor> {
  if audio1.sample_rate != audio2.sample_rate || audio1.channels != audio2.channels {
    return Err(EllasticError::InvalidParameter(
      "Audio properties must match".to_string(),
    ));
  }

  let fade_samples = std::cmp::min(fade_samples, audio1.samples.len().min(audio2.samples.len()));
  let mut crossfaded_samples =
    Vec::with_capacity(audio1.samples.len() + audio2.samples.len() - fade_samples);

  crossfaded_samples.extend_from_slice(&audio1.samples[..audio1.samples.len() - fade_samples]);

  for i in 0..fade_samples {
    let fade_factor = i as f32 / fade_samples as f32;
    let sample1 = audio1.samples[audio1.samples.len() - fade_samples + i];
    let sample2 = audio2.samples[i];
    let crossfaded = sample1 * (1.0 - fade_factor) + sample2 * fade_factor;
    crossfaded_samples.push(crossfaded);
  }

  crossfaded_samples.extend_from_slice(&audio2.samples[fade_samples..]);

  AudioData::new(audio1.sample_rate, audio1.channels, crossfaded_samples)
    .map(AudioProcessor::from_audio_data)
}

pub fn apply_audio_filter(audio_data: &mut AudioData, filter: &crate::AudioFilter) -> Result<()> {
  match filter.filter_type {
    crate::FilterType::LowPass { cutoff } => apply_lowpass_filter(audio_data, cutoff),
    crate::FilterType::HighPass { cutoff } => apply_highpass_filter(audio_data, cutoff),
    crate::FilterType::BandPass {
      low_cutoff,
      high_cutoff,
    } => apply_bandpass_filter(audio_data, low_cutoff, high_cutoff),
    crate::FilterType::Notch { center, bandwidth } => {
      apply_notch_filter(audio_data, center, bandwidth)
    }
    crate::FilterType::LowShelf { cutoff, gain } => apply_lowshelf_filter(audio_data, cutoff, gain),
    crate::FilterType::HighShelf { cutoff, gain } => {
      apply_highshelf_filter(audio_data, cutoff, gain)
    }
    crate::FilterType::Peaking {
      center,
      bandwidth,
      gain,
    } => apply_peaking_filter(audio_data, center, bandwidth, gain),
  }
}

fn apply_lowpass_filter(audio_data: &mut AudioData, cutoff: f32) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let dt = 1.0 / sample_rate;
  let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
  let alpha = dt / (rc + dt);
  let mut previous_output = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let output = alpha * *sample + (1.0 - alpha) * previous_output;
    previous_output = output;
    *sample = output;
  }

  Ok(())
}

fn apply_highpass_filter(audio_data: &mut AudioData, cutoff: f32) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let dt = 1.0 / sample_rate;
  let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
  let alpha = rc / (rc + dt);
  let mut previous_input = 0.0f32;
  let mut previous_output = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let output = alpha * (previous_output + *sample - previous_input);
    previous_input = *sample;
    previous_output = output;
    *sample = output;
  }

  Ok(())
}

fn apply_bandpass_filter(
  audio_data: &mut AudioData,
  low_cutoff: f32,
  high_cutoff: f32,
) -> Result<()> {
  let mut temp_data = audio_data.samples.clone();
  apply_lowpass_filter(&mut temp_data, high_cutoff)?;
  apply_highpass_filter(&mut temp_data, low_cutoff)?;

  audio_data.samples.copy_from_slice(&temp_data);
  Ok(())
}

fn apply_notch_filter(audio_data: &mut AudioData, center: f32, bandwidth: f32) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let dt = 1.0 / sample_rate;
  let rc = 1.0 / (2.0 * std::f32::consts::PI * bandwidth);
  let alpha = rc / (rc + dt);
  let omega = 2.0 * std::f32::consts::PI * center;
  let cos_omega = omega.cos();
  let mut previous_input = 0.0f32;
  let mut previous_output = 0.0f32;
  let mut previous_previous_output = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let notch_output =
      2.0 * cos_omega * previous_output - previous_previous_output + *sample - previous_input;
    let filtered_output = alpha * notch_output + (1.0 - alpha) * previous_output;

    previous_previous_output = previous_output;
    previous_output = filtered_output;
    previous_input = *sample;
    *sample = filtered_output;
  }

  Ok(())
}

fn apply_lowshelf_filter(audio_data: &mut AudioData, cutoff: f32, gain: f32) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let omega = 2.0 * std::f32::consts::PI * cutoff / sample_rate;
  let sin_omega = omega.sin();
  let cos_omega = omega.cos();
  let a = (gain / 20.0).powf(10.0).sqrt();
  let alpha = sin_omega / 2.0 * ((a + 1.0) / a).sqrt();
  let beta = 2.0 * ((a + 1.0).sqrt() * cos_omega);
  let gamma = ((a + 1.0).sqrt() - (a - 1.0).sqrt()) / ((a + 1.0).sqrt() + (a - 1.0).sqrt());

  let a0 = (a + 1.0 + (a - 1.0) * cos_omega + beta) / 2.0;
  let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega - beta) / 2.0;
  let a2 = (a + 1.0 + (a - 1.0) * cos_omega - beta) / 2.0;
  let b1 = 2.0 * gamma * cos_omega;
  let b2 = -gamma;

  let mut x1 = 0.0f32;
  let mut x2 = 0.0f32;
  let mut y1 = 0.0f32;
  let mut y2 = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let output = a0 * *sample + a1 * x1 + a2 * x2 - b1 * y1 - b2 * y2;
    x2 = x1;
    x1 = *sample;
    y2 = y1;
    y1 = output;
    *sample = output;
  }

  Ok(())
}

fn apply_highshelf_filter(audio_data: &mut AudioData, cutoff: f32, gain: f32) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let omega = 2.0 * std::f32::consts::PI * cutoff / sample_rate;
  let sin_omega = omega.sin();
  let cos_omega = omega.cos();
  let a = (gain / 20.0).powf(10.0).sqrt();
  let alpha = sin_omega / 2.0 * ((a + 1.0) / a).sqrt();
  let beta = 2.0 * ((a + 1.0).sqrt() * cos_omega);
  let gamma = ((a + 1.0).sqrt() - (a - 1.0).sqrt()) / ((a + 1.0).sqrt() + (a - 1.0).sqrt());

  let a0 = (a + 1.0 - (a - 1.0) * cos_omega + beta) / 2.0;
  let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega + beta) / 2.0;
  let a2 = (a + 1.0 - (a - 1.0) * cos_omega - beta) / 2.0;
  let b1 = 2.0 * gamma * cos_omega;
  let b2 = -gamma;

  let mut x1 = 0.0f32;
  let mut x2 = 0.0f32;
  let mut y1 = 0.0f32;
  let mut y2 = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let output = a0 * *sample + a1 * x1 + a2 * x2 - b1 * y1 - b2 * y2;
    x2 = x1;
    x1 = *sample;
    y2 = y1;
    y1 = output;
    *sample = output;
  }

  Ok(())
}

fn apply_peaking_filter(
  audio_data: &mut AudioData,
  center: f32,
  bandwidth: f32,
  gain: f32,
) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let omega = 2.0 * std::f32::consts::PI * center / sample_rate;
  let sin_omega = omega.sin();
  let cos_omega = omega.cos();
  let a = (gain / 40.0).powf(10.0).sqrt();
  let alpha = sin_omega * std::sinh(bandwidth / 2.0);
  let beta = 2.0 * cos_omega * std::cosh(bandwidth / 2.0);
  let gamma = 2.0 * a * std::cosh(bandwidth / 2.0);

  let a0 = 1.0 + alpha * a;
  let a1 = -2.0 * cos_omega * std::cosh(bandwidth / 2.0);
  let a2 = 1.0 - alpha * a;
  let b1 = -2.0 * cos_omega * std::cosh(bandwidth / 2.0);
  let b2 = 1.0 - alpha;

  let mut x1 = 0.0f32;
  let mut x2 = 0.0f32;
  let mut y1 = 0.0f32;
  let mut y2 = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let output = (a0 * *sample + a1 * x1 + a2 * x2 - b1 * y1 - b2 * y2) / gamma;
    x2 = x1;
    x1 = *sample;
    y2 = y1;
    y1 = output;
    *sample = output;
  }

  Ok(())
}

pub fn apply_audio_effect(audio_data: &mut AudioData, effect: &crate::AudioEffect) -> Result<()> {
  match effect.effect_type {
    crate::EffectType::Reverb { room_size, damping } => {
      apply_reverb_effect(audio_data, room_size, damping)
    }
    crate::EffectType::Delay {
      time,
      feedback,
      mix,
    } => apply_delay_effect(audio_data, time, feedback, mix),
    crate::EffectType::Chorus { rate, depth, mix } => {
      apply_chorus_effect(audio_data, rate, depth, mix)
    }
    crate::EffectType::Flanger {
      rate,
      depth,
      feedback,
    } => apply_flanger_effect(audio_data, rate, depth, feedback),
    crate::EffectType::Distortion { drive, tone } => {
      apply_distortion_effect(audio_data, drive, tone)
    }
    crate::EffectType::Compressor {
      threshold,
      ratio,
      attack,
      release,
    } => apply_compressor_effect(audio_data, threshold, ratio, attack, release),
    crate::EffectType::Limiter { threshold, release } => {
      apply_limiter_effect(audio_data, threshold, release)
    }
    crate::EffectType::Gate {
      threshold,
      attack,
      release,
      hold,
    } => apply_gate_effect(audio_data, threshold, attack, release, hold),
    crate::EffectType::Pitch { semitones } => apply_pitch_effect(audio_data, semitones),
    crate::EffectType::TimeStretch { ratio } => apply_time_stretch_effect(audio_data, ratio),
  }
}

fn apply_reverb_effect(audio_data: &mut AudioData, room_size: f32, damping: f32) -> Result<()> {
  let delay_time = room_size;
  let delay_samples = (delay_time * audio_data.sample_rate as f32) as usize;
  let mut delay_buffer = vec![0.0f32; delay_samples];
  let mut delay_index = 0;

  for sample in audio_data.samples.iter_mut() {
    let delayed_sample = delay_buffer[delay_index];
    let reverb_sample = *sample * 0.7 + delayed_sample * damping;
    delay_buffer[delay_index] = reverb_sample;
    *sample = reverb_sample;
    delay_index = (delay_index + 1) % delay_samples;
  }

  Ok(())
}

fn apply_delay_effect(
  audio_data: &mut AudioData,
  time: f32,
  feedback: f32,
  mix: f32,
) -> Result<()> {
  let delay_samples = (time * audio_data.sample_rate as f32) as usize;
  let mut delay_buffer = vec![0.0f32; delay_samples];
  let mut delay_index = 0;

  for sample in audio_data.samples.iter_mut() {
    let delayed_sample = delay_buffer[delay_index];
    let feedback_sample = delayed_sample * feedback;
    let output = *sample * (1.0 - mix) + (delayed_sample + feedback_sample) * mix;
    delay_buffer[delay_index] = *sample + feedback_sample;
    *sample = output;
    delay_index = (delay_index + 1) % delay_samples;
  }

  Ok(())
}

fn apply_chorus_effect(audio_data: &mut AudioData, rate: f32, depth: f32, mix: f32) -> Result<()> {
  let max_delay_samples = (depth * audio_data.sample_rate as f32) as usize;
  let mut delay_buffer = vec![0.0f32; max_delay_samples];
  let mut delay_index = 0;
  let mut phase = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let delay_samples = ((phase.sin() * 0.5 + 0.5) * max_delay_samples as f32) as usize;
    let delayed_sample = delay_buffer[(delay_index + delay_samples) % max_delay_samples];
    let output = *sample * (1.0 - mix) + delayed_sample * mix;

    delay_buffer[delay_index] = *sample;
    *sample = output;
    delay_index = (delay_index + 1) % max_delay_samples;
    phase += rate * 2.0 * std::f32::consts::PI / audio_data.sample_rate as f32;
  }

  Ok(())
}

fn apply_flanger_effect(
  audio_data: &mut AudioData,
  rate: f32,
  depth: f32,
  feedback: f32,
) -> Result<()> {
  let max_delay_samples = (depth * audio_data.sample_rate as f32) as usize;
  let mut delay_buffer = vec![0.0f32; max_delay_samples];
  let mut delay_index = 0;
  let mut phase = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let delay_samples = ((phase.sin() * 0.5 + 0.5) * max_delay_samples as f32) as usize;
    let delayed_sample = delay_buffer[(delay_index + delay_samples) % max_delay_samples];
    let feedback_sample = delayed_sample * feedback;
    let output = *sample + delayed_sample + feedback_sample;

    delay_buffer[delay_index] = output;
    *sample = output;
    delay_index = (delay_index + 1) % max_delay_samples;
    phase += rate * 2.0 * std::f32::consts::PI / audio_data.sample_rate as f32;
  }

  Ok(())
}

fn apply_distortion_effect(audio_data: &mut AudioData, drive: f32, tone: f32) -> Result<()> {
  for sample in audio_data.samples.iter_mut() {
    let distorted = (*sample * drive).tanh();
    let filtered = distorted * (1.0 + tone);
    *sample = filtered.clamp(-1.0, 1.0);
  }

  Ok(())
}

fn apply_compressor_effect(
  audio_data: &mut AudioData,
  threshold: f32,
  ratio: f32,
  attack: f32,
  release: f32,
) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let attack_coeff = (-1.0 / (attack * sample_rate)).exp();
  let release_coeff = (-1.0 / (release * sample_rate)).exp();
  let mut envelope = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let input_level = sample.abs();
    let target = if input_level > threshold {
      threshold + (input_level - threshold) / ratio
    } else {
      input_level
    };

    let coeff = if target > envelope {
      attack_coeff
    } else {
      release_coeff
    };
    envelope = target + coeff * (envelope - target);

    let gain = if envelope > 0.0 {
      envelope / input_level
    } else {
      1.0
    };
    *sample *= gain;
  }

  Ok(())
}

fn apply_limiter_effect(audio_data: &mut AudioData, threshold: f32, release: f32) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let release_coeff = (-1.0 / (release * sample_rate)).exp();
  let mut envelope = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let input_level = sample.abs();
    let target = input_level.min(threshold);
    envelope = target + release_coeff * (envelope - target);

    let gain = if envelope > 0.0 {
      envelope / input_level
    } else {
      1.0
    };
    *sample *= gain;
  }

  Ok(())
}

fn apply_gate_effect(
  audio_data: &mut AudioData,
  threshold: f32,
  attack: f32,
  release: f32,
  hold: f32,
) -> Result<()> {
  let sample_rate = audio_data.sample_rate as f32;
  let attack_coeff = (-1.0 / (attack * sample_rate)).exp();
  let release_coeff = (-1.0 / (release * sample_rate)).exp();
  let mut envelope = 0.0f32;
  let mut hold_counter = 0.0f32;

  for sample in audio_data.samples.iter_mut() {
    let input_level = sample.abs();
    let target = if input_level > threshold {
      1.0
    } else if hold_counter > 0.0 {
      1.0
    } else {
      0.0
    };

    let coeff = if target > envelope {
      attack_coeff
    } else {
      release_coeff
    };
    envelope = target + coeff * (envelope - target);

    if target == 1.0 {
      hold_counter = hold;
    } else if hold_counter > 0.0 {
      hold_counter -= 1.0 / sample_rate;
    }

    *sample *= envelope;
  }

  Ok(())
}

fn apply_pitch_effect(audio_data: &mut AudioData, semitones: f32) -> Result<()> {
  let pitch_factor = (2.0f32).powf(semitones / 12.0);
  resample_audio(
    audio_data,
    (audio_data.sample_rate as f32 * pitch_factor) as u32,
    crate::ResampleMethod::Cubic,
  )?;
  Ok(())
}

fn apply_time_stretch_effect(audio_data: &mut AudioData, ratio: f32) -> Result<()> {
  let new_sample_count = (audio_data.samples.len() as f32 / ratio) as usize;
  let mut stretched_samples = Vec::with_capacity(new_sample_count);

  for i in 0..new_sample_count {
    let source_index = (i as f32 * ratio) as usize;
    if source_index < audio_data.samples.len() {
      stretched_samples.push(audio_data.samples[source_index]);
    }
  }

  audio_data.samples = stretched_samples;
  Ok(())
}

pub fn calculate_amplitude_envelope(audio_data: &AudioData, window_size: usize) -> Vec<f32> {
  let mut envelope = Vec::new();

  for chunk in audio_data.samples.chunks(window_size) {
    let max_amplitude = chunk.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    envelope.push(max_amplitude);
  }

  envelope
}

pub fn calculate_rms(audio_data: &AudioData) -> f32 {
  if audio_data.samples.is_empty() {
    return 0.0;
  }

  let sum_squares: f32 = audio_data
    .samples
    .iter()
    .map(|sample| sample * sample)
    .sum();

  (sum_squares / audio_data.samples.len() as f32).sqrt()
}

pub fn calculate_peak(audio_data: &AudioData) -> f32 {
  audio_data
    .samples
    .iter()
    .map(|sample| sample.abs())
    .fold(0.0f32, f32::max)
}

pub fn calculate_dynamic_range(audio_data: &AudioData) -> f32 {
  let peak = calculate_peak(audio_data);
  let rms = calculate_rms(audio_data);

  if rms > 0.0 {
    20.0 * (peak / rms).log10()
  } else {
    0.0
  }
}

pub fn calculate_zero_crossing_rate(audio_data: &AudioData) -> f64 {
  if audio_data.samples.len() < 2 {
    return 0.0;
  }

  let mut zero_crossings = 0;
  for i in 1..audio_data.samples.len() {
    if (audio_data.samples[i - 1] < 0.0 && audio_data.samples[i] >= 0.0)
      || (audio_data.samples[i - 1] > 0.0 && audio_data.samples[i] <= 0.0)
    {
      zero_crossings += 1;
    }
  }

  zero_crossings as f64 / audio_data.samples.len() as f64
}

pub fn calculate_frequency_spectrum(audio_data: &AudioData, fft_size: usize) -> Vec<f32> {
  let mut fft_input = vec![0.0f64; fft_size];
  let mut fft_output = vec![0.0f64; fft_size];

  for (i, &sample) in audio_data.samples.iter().take(fft_size).enumerate() {
    fft_input[i] = sample as f64;
  }

  simple_dft(&mut fft_input, &mut fft_output);

  fft_output
    .iter()
    .take(fft_size / 2)
    .map(|&magnitude| magnitude as f32)
    .collect()
}

fn simple_dft(input: &mut [f64], output: &mut [f64]) {
  let n = input.len();

  for k in 0..n {
    let mut real_sum = 0.0f64;
    let mut imag_sum = 0.0f64;

    for t in 0..n {
      let angle = -2.0 * std::f64::consts::PI * k as f64 * t as f64 / n as f64;
      real_sum += input[t] * angle.cos();
      imag_sum += input[t] * angle.sin();
    }

    output[k] = (real_sum * real_sum + imag_sum * imag_sum).sqrt();
  }
}

pub fn calculate_spectrogram(
  audio_data: &AudioData,
  fft_size: usize,
  hop_size: usize,
) -> Vec<Vec<f32>> {
  let mut spectrogram = Vec::new();
  let mut window_samples = Vec::with_capacity(fft_size);

  for chunk in audio_data.samples.chunks(hop_size) {
    window_samples.clear();
    window_samples.extend_from_slice(chunk);
    window_samples.resize(fft_size, 0.0);

    let spectrum = calculate_frequency_spectrum(
      &AudioData {
        sample_rate: audio_data.sample_rate,
        channels: audio_data.channels,
        samples: window_samples,
      },
      fft_size,
    );

    spectrogram.push(spectrum);
  }

  spectrogram
}
