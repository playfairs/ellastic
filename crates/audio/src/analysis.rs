use crate::{
  AudioData,
  AudioProcessor,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AudioAnalyzer {
  audio: AudioProcessor,
}

impl AudioAnalyzer {
  pub fn new(audio: AudioProcessor) -> Self {
    Self { audio }
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

  pub fn analyze(&self) -> AudioAnalysisReport {
    let sample_rate = self.audio.sample_rate();
    let channels = self.audio.channels();
    let sample_count = self.audio.sample_count();
    let duration = self.audio.duration_seconds();

    let waveform = self.audio.data().samples.clone();
    let statistics = self.calculate_statistics(&waveform);
    let frequency_analysis = self.analyze_frequency(&waveform, sample_rate);
    let temporal_analysis = self.analyze_temporal(&waveform, sample_rate);
    let spectral_analysis = self.analyze_spectral(&waveform, sample_rate);
    let quality_metrics = self.calculate_quality_metrics(&waveform, sample_rate);
    let channel_analysis = self.analyze_channels(&waveform, channels);

    AudioAnalysisReport {
      sample_rate,
      channels,
      sample_count,
      duration,
      statistics,
      frequency_analysis,
      temporal_analysis,
      spectral_analysis,
      quality_metrics,
      channel_analysis,
    }
  }

  pub fn calculate_statistics(&self, waveform: &[f32]) -> AudioStatistics {
    if waveform.is_empty() {
      return AudioStatistics::default();
    }

    let min = waveform.iter().fold(f32::INFINITY, |a, &b| a.min(*b));
    let max = waveform.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(*b));
    let sum: f32 = waveform.iter().sum();
    let mean = sum / waveform.len() as f32;

    let mut variance = 0.0f32;
    for &sample in waveform {
      variance += (sample - mean).powi(2);
    }
    variance /= waveform.len() as f32;
    let std_dev = variance.sqrt();

    let mut sorted = waveform.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if sorted.len() % 2 == 0 {
      (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
      sorted[sorted.len() / 2]
    };

    let rms = (variance).sqrt();
    let peak = max.abs().max(min.abs());
    let dynamic_range = if peak > 0.0 {
      20.0 * (peak / rms).log10()
    } else {
      0.0
    };

    let zero_crossings = waveform
      .windows(2)
      .filter(|window| (window[0] * window[1] < 0.0) && (window[0] != 0.0 || window[1] != 0.0))
      .count() as f32;

    let zero_crossing_rate = zero_crossings / waveform.len() as f32;

    AudioStatistics {
      min,
      max,
      mean,
      median,
      std_dev,
      rms,
      peak,
      dynamic_range,
      zero_crossing_rate,
    }
  }

  pub fn analyze_frequency(&self, waveform: &[f32], sample_rate: u32) -> FrequencyAnalysis {
    let fft_size = self.next_power_of_two(waveform.len().min(4096));
    let mut fft_input = vec![0.0f64; fft_size];
    let mut fft_output = vec![0.0f64; fft_size];

    for (i, &sample) in waveform.iter().take(fft_size).enumerate() {
      fft_input[i] = sample as f64;
    }

    self.simple_dft(&mut fft_input, &mut fft_output);

    let magnitude_spectrum: Vec<f32> = fft_output
      .iter()
      .take(fft_size / 2)
      .map(|&mag| mag as f32)
      .collect();

    let spectral_centroid = self.calculate_spectral_centroid(&magnitude_spectrum, sample_rate);
    let spectral_bandwidth =
      self.calculate_spectral_bandwidth(&magnitude_spectrum, sample_rate, spectral_centroid);
    let spectral_rolloff = self.calculate_spectral_rolloff(&magnitude_spectrum, sample_rate);
    let spectral_flux = self.calculate_spectral_flux(&magnitude_spectrum);
    let spectral_flatness = self.calculate_spectral_flatness(&magnitude_spectrum);

    let fundamental_frequency =
      self.estimate_fundamental_frequency(&magnitude_spectrum, sample_rate);
    let harmonic_content = self.analyze_harmonic_content(&magnitude_spectrum, sample_rate);

    FrequencyAnalysis {
      spectral_centroid,
      spectral_bandwidth,
      spectral_rolloff,
      spectral_flux,
      spectral_flatness,
      fundamental_frequency,
      harmonic_content,
      magnitude_spectrum,
    }
  }

  pub fn analyze_temporal(&self, waveform: &[f32], sample_rate: u32) -> TemporalAnalysis {
    let attack_time = self.calculate_attack_time(waveform, sample_rate);
    let decay_time = self.calculate_decay_time(waveform, sample_rate);
    let sustain_level = self.calculate_sustain_level(waveform);
    let release_time = self.calculate_release_time(waveform, sample_rate);
    let tempo = self.estimate_tempo(waveform, sample_rate);
    let onset_detection = self.detect_onsets(waveform, sample_rate);
    let transient_content = self.analyze_transient_content(waveform);

    TemporalAnalysis {
      attack_time,
      decay_time,
      sustain_level,
      release_time,
      tempo,
      onset_detection,
      transient_content,
    }
  }

  pub fn analyze_spectral(&self, waveform: &[f32], sample_rate: u32) -> SpectralAnalysis {
    let fft_size = self.next_power_of_two(waveform.len().min(8192));
    let hop_size = fft_size / 4;
    let spectrogram = self.calculate_spectrogram(waveform, fft_size, hop_size);

    let spectral_centroid_evolution =
      self.calculate_spectral_centroid_evolution(&spectrogram, sample_rate);
    let spectral_flux_evolution = self.calculate_spectral_flux_evolution(&spectrogram);
    let mfcc = self.calculate_mfcc(&spectrogram, sample_rate);
    let chroma = self.calculate_chroma(&spectrogram, sample_rate);
    let tonality = self.analyze_tonality(&chroma);

    SpectralAnalysis {
      spectrogram,
      spectral_centroid_evolution,
      spectral_flux_evolution,
      mfcc,
      chroma,
      tonality,
    }
  }

  pub fn calculate_quality_metrics(&self, waveform: &[f32], sample_rate: u32) -> QualityMetrics {
    let snr = self.calculate_snr(waveform);
    let thd = self.calculate_thd(waveform, sample_rate);
    let dynamic_range = self.calculate_dynamic_range_db(waveform);
    let peak_level = self.calculate_peak_level_db(waveform);
    let rms_level = self.calculate_rms_level_db(waveform);
    let crest_factor = peak_level - rms_level;

    let frequency_response = self.calculate_frequency_response(waveform, sample_rate);
    let phase_coherence = self.calculate_phase_coherence(waveform);
    let harmonic_distortion = self.calculate_harmonic_distortion(waveform, sample_rate);

    let overall_quality = self.calculate_overall_quality(
      snr,
      thd,
      dynamic_range,
      crest_factor,
      frequency_response,
      phase_coherence,
      harmonic_distortion,
    );

    QualityMetrics {
      snr,
      thd,
      dynamic_range,
      peak_level,
      rms_level,
      crest_factor,
      frequency_response,
      phase_coherence,
      harmonic_distortion,
      overall_quality,
    }
  }

  pub fn analyze_channels(&self, waveform: &[f32], channels: u8) -> Vec<ChannelAnalysis> {
    let mut channel_analyses = Vec::new();

    for channel in 0..channels {
      let channel_samples: Vec<f32> = waveform
        .iter()
        .skip(channel as usize)
        .step_by(channels as usize)
        .copied()
        .collect();

      let statistics = self.calculate_statistics(&channel_samples);
      let pan_position = if channels == 2 {
        let left_samples: Vec<f32> = waveform.iter().step_by(2).copied().collect();
        let right_samples: Vec<f32> = waveform.iter().skip(1).step_by(2).copied().collect();

        let left_rms = self.calculate_rms(&left_samples);
        let right_rms = self.calculate_rms(&right_samples);

        if left_rms + right_rms > 0.0 {
          (right_rms - left_rms) / (left_rms + right_rms)
        } else {
          0.0
        }
      } else {
        0.0
      };

      let correlation = if channels == 2 {
        self.calculate_stereo_correlation(waveform)
      } else {
        0.0
      };

      channel_analyses.push(ChannelAnalysis {
        channel,
        statistics,
        pan_position,
        correlation,
      });
    }

    channel_analyses
  }

  pub fn compare_with(&self, other: &AudioProcessor) -> AudioComparisonReport {
    let my_waveform = self.audio.data().samples.clone();
    let other_waveform = other.data().samples.clone();

    let similarity = self.calculate_similarity(&my_waveform, &other_waveform);
    let correlation = self.calculate_correlation(&my_waveform, &other_waveform);
    let mse = self.calculate_mse(&my_waveform, &other_waveform);
    let psnr = if mse > 0.0 {
      20.0 * (1.0 / mse).log10()
    } else {
      f32::INFINITY
    };

    let spectral_distance = self.calculate_spectral_distance(&my_waveform, &other_waveform);
    let temporal_distance = self.calculate_temporal_distance(&my_waveform, &other_waveform);

    AudioComparisonReport {
      similarity,
      correlation,
      mean_squared_error: mse,
      peak_signal_to_noise_ratio: psnr,
      spectral_distance,
      temporal_distance,
    }
  }

  fn next_power_of_two(&self, n: usize) -> usize {
    if n <= 1 {
      return 1;
    }
    1 << (32 - (n - 1).leading_zeros()) as usize
  }

  fn simple_dft(&self, input: &mut [f64], output: &mut [f64]) {
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

  fn calculate_spectral_centroid(&self, magnitude_spectrum: &[f32], sample_rate: u32) -> f32 {
    let mut weighted_sum = 0.0f32;
    let mut magnitude_sum = 0.0f32;

    for (i, &magnitude) in magnitude_spectrum.iter().enumerate() {
      let frequency = (i as f32 * sample_rate as f32) / magnitude_spectrum.len() as f32;
      weighted_sum += frequency * magnitude;
      magnitude_sum += magnitude;
    }

    if magnitude_sum > 0.0 {
      weighted_sum / magnitude_sum
    } else {
      0.0
    }
  }

  fn calculate_spectral_bandwidth(
    &self,
    magnitude_spectrum: &[f32],
    sample_rate: u32,
    centroid: f32,
  ) -> f32 {
    let mut weighted_sum = 0.0f32;
    let mut magnitude_sum = 0.0f32;

    for (i, &magnitude) in magnitude_spectrum.iter().enumerate() {
      let frequency = (i as f32 * sample_rate as f32) / magnitude_spectrum.len() as f32;
      let deviation = frequency - centroid;
      weighted_sum += deviation * deviation * magnitude;
      magnitude_sum += magnitude;
    }

    if magnitude_sum > 0.0 {
      (weighted_sum / magnitude_sum).sqrt()
    } else {
      0.0
    }
  }

  fn calculate_spectral_rolloff(&self, magnitude_spectrum: &[f32], sample_rate: u32) -> f32 {
    let total_energy: f32 = magnitude_spectrum.iter().sum();
    let mut cumulative_energy = 0.0f32;

    for (i, &magnitude) in magnitude_spectrum.iter().enumerate() {
      cumulative_energy += magnitude;
      if cumulative_energy >= 0.85 * total_energy {
        return (i as f32 * sample_rate as f32) / magnitude_spectrum.len() as f32;
      }
    }

    sample_rate as f32
  }

  fn calculate_spectral_flux(&self, magnitude_spectrum: &[f32]) -> f32 {
    if magnitude_spectrum.len() < 2 {
      return 0.0;
    }

    let mut flux = 0.0f32;
    for i in 1..magnitude_spectrum.len() {
      let diff = magnitude_spectrum[i] - magnitude_spectrum[i - 1];
      flux += diff.max(0.0);
    }

    flux
  }

  fn calculate_spectral_flatness(&self, magnitude_spectrum: &[f32]) -> f32 {
    let geometric_mean = magnitude_spectrum
      .iter()
      .map(|&x| x as f64)
      .product::<f64>()
      .powf(1.0 / magnitude_spectrum.len() as f64) as f32;

    let arithmetic_mean: f32 =
      magnitude_spectrum.iter().sum::<f32>() / magnitude_spectrum.len() as f32;

    if arithmetic_mean > 0.0 {
      geometric_mean / arithmetic_mean
    } else {
      0.0
    }
  }

  fn estimate_fundamental_frequency(&self, magnitude_spectrum: &[f32], sample_rate: u32) -> f32 {
    let mut max_magnitude = 0.0f32;
    let mut max_index = 0;

    for (i, &magnitude) in magnitude_spectrum.iter().enumerate() {
      if magnitude > max_magnitude {
        max_magnitude = magnitude;
        max_index = i;
      }
    }

    let min_frequency = 20.0f32;
    let max_frequency = 2000.0f32;

    let min_index = (min_frequency * magnitude_spectrum.len() as f32 / sample_rate as f32) as usize;
    let max_index = (max_frequency * magnitude_spectrum.len() as f32 / sample_rate as f32) as usize;

    let search_start = min_index.max(1);
    let search_end = max_index.min(magnitude_spectrum.len() - 1);

    let mut best_harmonic_score = 0.0f32;
    let mut best_frequency = 0.0f32;

    for i in search_start..search_end {
      let frequency = (i as f32 * sample_rate as f32) / magnitude_spectrum.len() as f32;
      let mut harmonic_score = magnitude_spectrum[i];

      for harmonic in 2..=8 {
        let harmonic_index = (i * harmonic) as usize;
        if harmonic_index < magnitude_spectrum.len() {
          harmonic_score += magnitude_spectrum[harmonic_index] / harmonic as f32;
        }
      }

      if harmonic_score > best_harmonic_score {
        best_harmonic_score = harmonic_score;
        best_frequency = frequency;
      }
    }

    best_frequency
  }

  fn analyze_harmonic_content(
    &self,
    magnitude_spectrum: &[f32],
    sample_rate: u32,
  ) -> HarmonicAnalysis {
    let fundamental = self.estimate_fundamental_frequency(magnitude_spectrum, sample_rate);
    let fundamental_index =
      (fundamental * magnitude_spectrum.len() as f32 / sample_rate as f32) as usize;

    let mut harmonics = Vec::new();
    let mut harmonic_amplitudes = Vec::new();

    for harmonic in 1..=16 {
      let harmonic_index = (fundamental_index * harmonic) as usize;
      if harmonic_index < magnitude_spectrum.len() {
        let amplitude = magnitude_spectrum[harmonic_index];
        harmonics.push(harmonic as u32);
        harmonic_amplitudes.push(amplitude);
      }
    }

    let inharmonicity = self.calculate_inharmonicity(&harmonic_amplitudes);
    let harmonic_to_noise_ratio = self.calculate_hnr(&harmonic_amplitudes, magnitude_spectrum);

    HarmonicAnalysis {
      fundamental,
      harmonics,
      harmonic_amplitudes,
      inharmonicity,
      harmonic_to_noise_ratio,
    }
  }

  fn calculate_inharmonicity(&self, amplitudes: &[f32]) -> f32 {
    if amplitudes.len() < 2 {
      return 0.0;
    }

    let mut inharmonicity = 0.0f32;
    let fundamental = amplitudes[0];

    for (i, &amplitude) in amplitudes.iter().enumerate().skip(1) {
      let expected_amplitude = fundamental / (i + 1) as f32;
      if expected_amplitude > 0.0 {
        let deviation = (amplitude - expected_amplitude).abs() / expected_amplitude;
        inharmonicity += deviation;
      }
    }

    inharmonicity / amplitudes.len() as f32
  }

  fn calculate_hnr(&self, harmonic_amplitudes: &[f32], magnitude_spectrum: &[f32]) -> f32 {
    let harmonic_energy: f32 = harmonic_amplitudes.iter().sum();
    let total_energy: f32 = magnitude_spectrum.iter().sum();

    if total_energy > 0.0 {
      harmonic_energy / total_energy
    } else {
      0.0
    }
  }

  fn calculate_attack_time(&self, waveform: &[f32], sample_rate: u32) -> f32 {
    let envelope = self.calculate_envelope(waveform);
    let threshold = envelope.iter().fold(0.0f32, |a, &b| a.max(b)) * 0.1;

    let mut attack_start = None;
    let mut attack_end = None;

    for (i, &value) in envelope.iter().enumerate() {
      if attack_start.is_none() && value > threshold {
        attack_start = Some(i);
      }
      if attack_start.is_some() && value > threshold * 0.9 {
        attack_end = Some(i);
        break;
      }
    }

    if let (Some(start), Some(end)) = (attack_start, attack_end) {
      (end - start) as f32 / sample_rate as f32
    } else {
      0.0
    }
  }

  fn calculate_decay_time(&self, waveform: &[f32], sample_rate: u32) -> f32 {
    let envelope = self.calculate_envelope(waveform);
    let peak = envelope.iter().fold(0.0f32, |a, &b| a.max(b));
    let threshold = peak * 0.1;

    let mut decay_start = None;
    let mut decay_end = None;

    for (i, &value) in envelope.iter().enumerate() {
      if value > peak * 0.9 && decay_start.is_none() {
        decay_start = Some(i);
      }
      if decay_start.is_some() && value < threshold {
        decay_end = Some(i);
        break;
      }
    }

    if let (Some(start), Some(end)) = (decay_start, decay_end) {
      (end - start) as f32 / sample_rate as f32
    } else {
      0.0
    }
  }

  fn calculate_sustain_level(&self, waveform: &[f32]) -> f32 {
    let envelope = self.calculate_envelope(waveform);
    let peak = envelope.iter().fold(0.0f32, |a, &b| a.max(b));

    let attack_end = envelope.iter().position(|&x| x >= peak * 0.9).unwrap_or(0);

    let decay_start = envelope
      .iter()
      .skip(attack_end)
      .position(|&x| x <= peak * 0.8)
      .map(|i| i + attack_end)
      .unwrap_or(envelope.len());

    if decay_start > attack_end {
      let sustain_segment = &envelope[attack_end..decay_start];
      sustain_segment.iter().sum::<f32>() / sustain_segment.len() as f32
    } else {
      peak
    }
  }

  fn calculate_release_time(&self, waveform: &[f32], sample_rate: u32) -> f32 {
    let envelope = self.calculate_envelope(waveform);
    let peak = envelope.iter().fold(0.0f32, |a, &b| a.max(b));
    let threshold = peak * 0.1;

    let mut release_start = None;
    let mut release_end = None;

    for (i, &value) in envelope.iter().enumerate().rev() {
      if release_end.is_none() && value > threshold {
        release_end = Some(i);
      }
      if release_end.is_some() && value > peak * 0.8 {
        release_start = Some(i);
        break;
      }
    }

    if let (Some(start), Some(end)) = (release_start, release_end) {
      (end - start) as f32 / sample_rate as f32
    } else {
      0.0
    }
  }

  fn estimate_tempo(&self, waveform: &[f32], sample_rate: u32) -> f32 {
    let onset_detection = self.detect_onsets(waveform, sample_rate);
    let mut intervals = Vec::new();

    for i in 1..onset_detection.len() {
      if onset_detection[i] {
        for j in (0..i).rev() {
          if onset_detection[j] {
            let interval = i - j;
            intervals.push(interval);
            break;
          }
        }
      }
    }

    if intervals.is_empty() {
      return 0.0;
    }

    intervals.sort();
    let median_interval = intervals[intervals.len() / 2];
    60.0 / (median_interval as f32 / sample_rate as f32)
  }

  fn detect_onsets(&self, waveform: &[f32], sample_rate: u32) -> Vec<bool> {
    let window_size = 1024;
    let hop_size = 512;
    let mut onsets = vec![false; waveform.len()];

    for i in (0..waveform.len() - window_size).step_by(hop_size) {
      let window = &waveform[i..i + window_size];
      let energy = window.iter().map(|&x| x * x).sum::<f32>();

      if i > 0 {
        let prev_energy = waveform[i - hop_size..i + hop_size - window_size]
          .iter()
          .map(|&x| x * x)
          .sum::<f32>();

        let energy_diff = energy - prev_energy;
        if energy_diff > 0.0 {
          onsets[i + window_size / 2] = true;
        }
      }
    }

    onsets
  }

  fn analyze_transient_content(&self, waveform: &[f32]) -> f32 {
    let window_size = 64;
    let mut transient_measure = 0.0f32;

    for i in 0..waveform.len() - window_size {
      let window = &waveform[i..i + window_size];
      let mean = window.iter().sum::<f32>() / window_size as f32;
      let variance = window.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / window_size as f32;

      transient_measure += variance.sqrt();
    }

    transient_measure / (waveform.len() - window_size) as f32
  }

  fn calculate_envelope(&self, waveform: &[f32]) -> Vec<f32> {
    let mut envelope = Vec::with_capacity(waveform.len());
    let mut current_value = 0.0f32;
    let attack_coeff = 0.3f32;
    let release_coeff = 0.0001f32;

    for &sample in waveform {
      let target = sample.abs();
      let coeff = if target > current_value {
        attack_coeff
      } else {
        release_coeff
      };
      current_value = target + coeff * (current_value - target);
      envelope.push(current_value);
    }

    envelope
  }

  fn calculate_spectrogram(
    &self,
    waveform: &[f32],
    fft_size: usize,
    hop_size: usize,
  ) -> Vec<Vec<f32>> {
    let mut spectrogram = Vec::new();

    for i in (0..waveform.len() - fft_size).step_by(hop_size) {
      let window = &waveform[i..i + fft_size];
      let mut fft_input = vec![0.0f64; fft_size];
      let mut fft_output = vec![0.0f64; fft_size];

      for (j, &sample) in window.iter().enumerate() {
        fft_input[j] = sample as f64;
      }

      self.simple_dft(&mut fft_input, &mut fft_output);

      let magnitude_spectrum: Vec<f32> = fft_output
        .iter()
        .take(fft_size / 2)
        .map(|&mag| mag as f32)
        .collect();

      spectrogram.push(magnitude_spectrum);
    }

    spectrogram
  }

  fn calculate_spectral_centroid_evolution(
    &self,
    spectrogram: &[Vec<f32>],
    sample_rate: u32,
  ) -> Vec<f32> {
    spectrogram
      .iter()
      .map(|spectrum| self.calculate_spectral_centroid(spectrum, sample_rate))
      .collect()
  }

  fn calculate_spectral_flux_evolution(&self, spectrogram: &[Vec<f32>]) -> Vec<f32> {
    if spectrogram.len() < 2 {
      return vec![0.0f32];
    }

    let mut flux_evolution = Vec::new();
    flux_evolution.push(0.0f32);

    for i in 1..spectrogram.len() {
      let flux = self.calculate_spectral_flux(&spectrogram[i]);
      flux_evolution.push(flux);
    }

    flux_evolution
  }

  fn calculate_mfcc(&self, spectrogram: &[Vec<f32>], sample_rate: u32) -> Vec<Vec<f32>> {
    let num_mfcc = 13;
    let mut mfccs = Vec::new();

    for spectrum in spectrogram {
      let mel_spectrum = self.frequency_to_mel(spectrum, sample_rate);
      let log_mel_spectrum: Vec<f32> = mel_spectrum
        .iter()
        .map(|&x| if x > 0.0 { x.ln() } else { -100.0 })
        .collect();

      let dct = self.dct(&log_mel_spectrum);
      let mfcc = dct.iter().take(num_mfcc).copied().collect();
      mfccs.push(mfcc);
    }

    mfccs
  }

  fn calculate_chroma(&self, spectrogram: &[Vec<f32>], sample_rate: u32) -> Vec<Vec<f32>> {
    let num_chroma = 12;
    let mut chromagrams = Vec::new();

    for spectrum in spectrogram {
      let mut chroma = vec![0.0f32; num_chroma];

      for (i, &magnitude) in spectrum.iter().enumerate() {
        let frequency = (i as f32 * sample_rate as f32) / spectrum.len() as f32;
        if frequency > 0.0 {
          let chroma_index = ((12.0 * (frequency / 440.0).log2()) as i32) % 12;
          let positive_index = if chroma_index < 0 {
            chroma_index + 12
          } else {
            chroma_index
          } as usize;
          chroma[positive_index] += magnitude;
        }
      }

      chromagrams.push(chroma);
    }

    chromagrams
  }

  fn analyze_tonality(&self, chroma_vectors: &[Vec<f32>]) -> TonalityAnalysis {
    let key_profiles = self.create_key_profiles();
    let mut best_key = 0;
    let mut best_correlation = -1.0f32;

    for (key_index, key_profile) in key_profiles.iter().enumerate() {
      let correlation =
        self.calculate_correlation_between_chroma_vectors(chroma_vectors, key_profile);
      if correlation > best_correlation {
        best_correlation = correlation;
        best_key = key_index;
      }
    }

    let key_names = [
      "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let mode_names = ["Major", "Minor"];

    let key_index = best_key % 12;
    let mode_index = best_key / 12;

    TonalityAnalysis {
      key: key_names[key_index].to_string(),
      mode: mode_names[mode_index].to_string(),
      correlation: best_correlation,
    }
  }

  fn create_key_profiles(&self) -> Vec<Vec<f32>> {
    vec![
      vec![1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
      vec![1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    ]
  }

  fn calculate_correlation_between_chroma_vectors(
    &self,
    chroma_vectors: &[Vec<f32>],
    key_profile: &[f32],
  ) -> f32 {
    if chroma_vectors.is_empty() {
      return 0.0;
    }

    let mut correlation = 0.0f32;
    let mut count = 0;

    for chroma_vector in chroma_vectors {
      for i in 0..12.min(chroma_vector.len()).min(key_profile.len()) {
        correlation += chroma_vector[i] * key_profile[i];
      }
      count += 1;
    }

    if count > 0 {
      correlation / count as f32
    } else {
      0.0
    }
  }

  fn frequency_to_mel(&self, spectrum: &[f32], sample_rate: u32) -> Vec<f32> {
    let mut mel_spectrum = Vec::with_capacity(spectrum.len());

    for (i, &magnitude) in spectrum.iter().enumerate() {
      let frequency = (i as f32 * sample_rate as f32) / spectrum.len() as f32;
      let mel = 2595.0 * (1.0 + frequency / 700.0).log10();
      mel_spectrum.push(mel);
    }

    mel_spectrum
  }

  fn dct(&self, input: &[f32]) -> Vec<f32> {
    let n = input.len();
    let mut output = vec![0.0f32; n];

    for k in 0..n {
      let mut sum = 0.0f32;
      for i in 0..n {
        sum += input[i] * (std::f32::consts::PI * (i as f32 + 0.5) * k as f32 / n as f32).cos();
      }
      output[k] = sum;
    }

    output
  }

  fn calculate_rms(&self, samples: &[f32]) -> f32 {
    if samples.is_empty() {
      return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();

    (sum_squares / samples.len() as f32).sqrt()
  }

  fn calculate_snr(&self, waveform: &[f32]) -> f32 {
    let signal_power = waveform.iter().map(|&x| x * x).sum::<f32>() / waveform.len() as f32;

    let noise_estimate = self.estimate_noise_level(waveform);
    let noise_power = noise_estimate * noise_estimate;

    if noise_power > 0.0 {
      10.0 * (signal_power / noise_power).log10()
    } else {
      f32::INFINITY
    }
  }

  fn estimate_noise_level(&self, waveform: &[f32]) -> f32 {
    let sorted_samples = {
      let mut sorted = waveform.to_vec();
      sorted.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap());
      sorted
    };

    let noise_samples = &sorted_samples[..sorted_samples.len() / 10];
    noise_samples.iter().map(|&x| x.abs()).sum::<f32>() / noise_samples.len() as f32
  }

  fn calculate_thd(&self, waveform: &[f32], sample_rate: u32) -> f32 {
    let fft_size = self.next_power_of_two(waveform.len().min(4096));
    let mut fft_input = vec![0.0f64; fft_size];
    let mut fft_output = vec![0.0f64; fft_size];

    for (i, &sample) in waveform.iter().take(fft_size).enumerate() {
      fft_input[i] = sample as f64;
    }

    self.simple_dft(&mut fft_input, &mut fft_output);

    let magnitude_spectrum: Vec<f32> = fft_output
      .iter()
      .take(fft_size / 2)
      .map(|&mag| mag as f32)
      .collect();

    let fundamental = self.estimate_fundamental_frequency(&magnitude_spectrum, sample_rate);
    let fundamental_index =
      (fundamental * magnitude_spectrum.len() as f32 / sample_rate as f32) as usize;

    let mut harmonic_power = 0.0f32;
    let mut total_power = 0.0f32;

    for (i, &magnitude) in magnitude_spectrum.iter().enumerate() {
      total_power += magnitude * magnitude;

      for harmonic in 1..=10 {
        let harmonic_index = (fundamental_index * harmonic) as usize;
        if harmonic_index == i {
          harmonic_power += magnitude * magnitude;
          break;
        }
      }
    }

    if total_power > 0.0 && harmonic_power < total_power {
      (total_power - harmonic_power).sqrt() / harmonic_power.sqrt()
    } else {
      0.0
    }
  }

  fn calculate_dynamic_range_db(&self, waveform: &[f32]) -> f32 {
    let peak = waveform.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    let rms = self.calculate_rms(waveform);

    if rms > 0.0 {
      20.0 * (peak / rms).log10()
    } else {
      0.0
    }
  }

  fn calculate_peak_level_db(&self, waveform: &[f32]) -> f32 {
    let peak = waveform.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    if peak > 0.0 {
      20.0 * peak.log10()
    } else {
      -f32::INFINITY
    }
  }

  fn calculate_rms_level_db(&self, waveform: &[f32]) -> f32 {
    let rms = self.calculate_rms(waveform);
    if rms > 0.0 {
      20.0 * rms.log10()
    } else {
      -f32::INFINITY
    }
  }

  fn calculate_frequency_response(&self, waveform: &[f32], sample_rate: u32) -> f32 {
    let fft_size = self.next_power_of_two(waveform.len().min(4096));
    let mut fft_input = vec![0.0f64; fft_size];
    let mut fft_output = vec![0.0f64; fft_size];

    for (i, &sample) in waveform.iter().take(fft_size).enumerate() {
      fft_input[i] = sample as f64;
    }

    self.simple_dft(&mut fft_input, &mut fft_output);

    let magnitude_spectrum: Vec<f32> = fft_output
      .iter()
      .take(fft_size / 2)
      .map(|&mag| mag as f32)
      .collect();

    let spectral_flatness = self.calculate_spectral_flatness(&magnitude_spectrum);
    spectral_flatness
  }

  fn calculate_phase_coherence(&self, waveform: &[f32]) -> f32 {
    if waveform.len() < 2 {
      return 1.0;
    }

    let mut coherence_sum = 0.0f32;
    for i in 1..waveform.len() {
      let phase_diff = (waveform[i] - waveform[i - 1]).atan2(1.0);
      coherence_sum += phase_diff.cos();
    }

    coherence_sum / (waveform.len() - 1) as f32
  }

  fn calculate_harmonic_distortion(&self, waveform: &[f32], sample_rate: u32) -> f32 {
    let thd = self.calculate_thd(waveform, sample_rate);
    thd
  }

  fn calculate_overall_quality(
    &self,
    snr: f32,
    thd: f32,
    dynamic_range: f32,
    crest_factor: f32,
    frequency_response: f32,
    phase_coherence: f32,
    harmonic_distortion: f32,
  ) -> f32 {
    let snr_score = (snr / 60.0).clamp(0.0, 1.0);
    let thd_score = (1.0 - (thd / 100.0).clamp(0.0, 1.0));
    let dynamic_range_score = (dynamic_range / 60.0).clamp(0.0, 1.0);
    let crest_score = (1.0 - (crest_factor / 20.0).clamp(0.0, 1.0));
    let frequency_score = (1.0 - frequency_response).clamp(0.0, 1.0);
    let phase_score = phase_coherence.clamp(0.0, 1.0);
    let harmonic_score = (1.0 - (harmonic_distortion / 100.0).clamp(0.0, 1.0));

    (snr_score * 0.2
      + thd_score * 0.2
      + dynamic_range_score * 0.15
      + crest_score * 0.1
      + frequency_score * 0.15
      + phase_score * 0.1
      + harmonic_score * 0.1)
  }

  fn calculate_similarity(&self, waveform1: &[f32], waveform2: &[f32]) -> f32 {
    let min_len = waveform1.len().min(waveform2.len());
    if min_len == 0 {
      return 1.0;
    }

    let mut dot_product = 0.0f32;
    let mut norm1 = 0.0f32;
    let mut norm2 = 0.0f32;

    for i in 0..min_len {
      dot_product += waveform1[i] * waveform2[i];
      norm1 += waveform1[i] * waveform1[i];
      norm2 += waveform2[i] * waveform2[i];
    }

    if norm1 > 0.0 && norm2 > 0.0 {
      dot_product / (norm1.sqrt() * norm2.sqrt())
    } else {
      0.0
    }
  }

  fn calculate_correlation(&self, waveform1: &[f32], waveform2: &[f32]) -> f32 {
    let min_len = waveform1.len().min(waveform2.len());
    if min_len == 0 {
      return 1.0;
    }

    let mean1: f32 = waveform1[..min_len].iter().sum::<f32>() / min_len as f32;
    let mean2: f32 = waveform2[..min_len].iter().sum::<f32>() / min_len as f32;

    let mut covariance = 0.0f32;
    let mut variance1 = 0.0f32;
    let mut variance2 = 0.0f32;

    for i in 0..min_len {
      let diff1 = waveform1[i] - mean1;
      let diff2 = waveform2[i] - mean2;
      covariance += diff1 * diff2;
      variance1 += diff1 * diff1;
      variance2 += diff2 * diff2;
    }

    if variance1 > 0.0 && variance2 > 0.0 {
      covariance / (variance1.sqrt() * variance2.sqrt())
    } else {
      0.0
    }
  }

  fn calculate_mse(&self, waveform1: &[f32], waveform2: &[f32]) -> f32 {
    let min_len = waveform1.len().min(waveform2.len());
    if min_len == 0 {
      return 0.0;
    }

    let mut mse = 0.0f32;
    for i in 0..min_len {
      let diff = waveform1[i] - waveform2[i];
      mse += diff * diff;
    }

    mse / min_len as f32
  }

  fn calculate_spectral_distance(&self, waveform1: &[f32], waveform2: &[f32]) -> f32 {
    let fft_size = self.next_power_of_two(waveform1.len().min(waveform2.len()).min(4096));

    let spectrum1 = self.get_magnitude_spectrum(waveform1, fft_size);
    let spectrum2 = self.get_magnitude_spectrum(waveform2, fft_size);

    let mut distance = 0.0f32;
    for i in 0..spectrum1.len().min(spectrum2.len()) {
      let diff = spectrum1[i] - spectrum2[i];
      distance += diff * diff;
    }

    (distance / spectrum1.len().min(spectrum2.len()) as f32).sqrt()
  }

  fn calculate_temporal_distance(&self, waveform1: &[f32], waveform2: &[f32]) -> f32 {
    let envelope1 = self.calculate_envelope(waveform1);
    let envelope2 = self.calculate_envelope(waveform2);

    let mut distance = 0.0f32;
    for i in 0..envelope1.len().min(envelope2.len()) {
      let diff = envelope1[i] - envelope2[i];
      distance += diff * diff;
    }

    (distance / envelope1.len().min(envelope2.len()) as f32).sqrt()
  }

  fn get_magnitude_spectrum(&self, waveform: &[f32], fft_size: usize) -> Vec<f32> {
    let mut fft_input = vec![0.0f64; fft_size];
    let mut fft_output = vec![0.0f64; fft_size];

    for (i, &sample) in waveform.iter().take(fft_size).enumerate() {
      fft_input[i] = sample as f64;
    }

    self.simple_dft(&mut fft_input, &mut fft_output);

    fft_output
      .iter()
      .take(fft_size / 2)
      .map(|&mag| mag as f32)
      .collect()
  }

  fn calculate_stereo_correlation(&self, waveform: &[f32]) -> f32 {
    if waveform.len() < 2 {
      return 0.0;
    }

    let left_samples: Vec<f32> = waveform.iter().step_by(2).copied().collect();
    let right_samples: Vec<f32> = waveform.iter().skip(1).step_by(2).copied().collect();

    self.calculate_correlation(&left_samples, &right_samples)
  }
}

#[derive(Debug, Clone)]
pub struct AudioAnalysisReport {
  pub sample_rate: u32,
  pub channels: u8,
  pub sample_count: usize,
  pub duration: f64,
  pub statistics: AudioStatistics,
  pub frequency_analysis: FrequencyAnalysis,
  pub temporal_analysis: TemporalAnalysis,
  pub spectral_analysis: SpectralAnalysis,
  pub quality_metrics: QualityMetrics,
  pub channel_analysis: Vec<ChannelAnalysis>,
}

#[derive(Debug, Clone, Default)]
pub struct AudioStatistics {
  pub min: f32,
  pub max: f32,
  pub mean: f32,
  pub median: f32,
  pub std_dev: f32,
  pub rms: f32,
  pub peak: f32,
  pub dynamic_range: f32,
  pub zero_crossing_rate: f32,
}

#[derive(Debug, Clone)]
pub struct FrequencyAnalysis {
  pub spectral_centroid: f32,
  pub spectral_bandwidth: f32,
  pub spectral_rolloff: f32,
  pub spectral_flux: f32,
  pub spectral_flatness: f32,
  pub fundamental_frequency: f32,
  pub harmonic_content: HarmonicAnalysis,
  pub magnitude_spectrum: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct HarmonicAnalysis {
  pub fundamental: f32,
  pub harmonics: Vec<u32>,
  pub harmonic_amplitudes: Vec<f32>,
  pub inharmonicity: f32,
  pub harmonic_to_noise_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct TemporalAnalysis {
  pub attack_time: f32,
  pub decay_time: f32,
  pub sustain_level: f32,
  pub release_time: f32,
  pub tempo: f32,
  pub onset_detection: Vec<bool>,
  pub transient_content: f32,
}

#[derive(Debug, Clone)]
pub struct SpectralAnalysis {
  pub spectrogram: Vec<Vec<f32>>,
  pub spectral_centroid_evolution: Vec<f32>,
  pub spectral_flux_evolution: Vec<f32>,
  pub mfcc: Vec<Vec<f32>>,
  pub chroma: Vec<Vec<f32>>,
  pub tonality: TonalityAnalysis,
}

#[derive(Debug, Clone)]
pub struct TonalityAnalysis {
  pub key: String,
  pub mode: String,
  pub correlation: f32,
}

#[derive(Debug, Clone)]
pub struct QualityMetrics {
  pub snr: f32,
  pub thd: f32,
  pub dynamic_range: f32,
  pub peak_level: f32,
  pub rms_level: f32,
  pub crest_factor: f32,
  pub frequency_response: f32,
  pub phase_coherence: f32,
  pub harmonic_distortion: f32,
  pub overall_quality: f32,
}

#[derive(Debug, Clone)]
pub struct ChannelAnalysis {
  pub channel: u8,
  pub statistics: AudioStatistics,
  pub pan_position: f32,
  pub correlation: f32,
}

#[derive(Debug, Clone)]
pub struct AudioComparisonReport {
  pub similarity: f32,
  pub correlation: f32,
  pub mean_squared_error: f32,
  pub peak_signal_to_noise_ratio: f32,
  pub spectral_distance: f32,
  pub temporal_distance: f32,
}

pub fn create_analyzer(audio: AudioProcessor) -> AudioAnalyzer {
  AudioAnalyzer::new(audio)
}

pub fn analyze_audio(audio: &AudioProcessor) -> AudioAnalysisReport {
  let analyzer = create_analyzer(audio.clone());
  analyzer.analyze()
}

pub fn compare_audio(audio1: &AudioProcessor, audio2: &AudioProcessor) -> AudioComparisonReport {
  let analyzer = create_analyzer(audio1.clone());
  analyzer.compare_with(audio2)
}
