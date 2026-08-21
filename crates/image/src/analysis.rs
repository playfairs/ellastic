use crate::{
  ImageData,
  ImageProcessor,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ImageAnalyzer {
  image: ImageProcessor,
}

impl ImageAnalyzer {
  pub fn new(image: ImageProcessor) -> Self {
    Self { image }
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

  pub fn analyze(&self) -> ImageAnalysisReport {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();
    let pixel_count = self.image.pixel_count();
    let byte_size = self.image.byte_size();

    let histogram = self.image.histogram();
    let channel_histograms = self.analyze_channels();
    let statistics = self.calculate_statistics();
    let color_analysis = self.analyze_colors();
    let texture_analysis = self.analyze_texture();
    let edge_analysis = self.analyze_edges();
    let frequency_analysis = self.analyze_frequency();
    let quality_metrics = self.calculate_quality_metrics();

    ImageAnalysisReport {
      width,
      height,
      channels,
      pixel_count,
      byte_size,
      histogram,
      channel_histograms,
      statistics,
      color_analysis,
      texture_analysis,
      edge_analysis,
      frequency_analysis,
      quality_metrics,
    }
  }

  pub fn calculate_statistics(&self) -> ImageStatistics {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();
    let data = self.image.data().data;

    let mut min = 255u8;
    let mut max = 0u8;
    let mut sum = 0u64;

    for &byte in data {
      min = min.min(byte);
      max = max.max(byte);
      sum += byte as u64;
    }

    let mean = sum as f64 / data.len() as f64;

    let mut variance = 0.0f64;
    for &byte in data {
      variance += (byte as f64 - mean).powi(2);
    }
    variance /= data.len() as f64;
    let std_dev = variance.sqrt();

    let mut sorted_data = data.to_vec();
    sorted_data.sort();
    let median = if sorted_data.is_empty() {
      0.0
    } else {
      let mid = sorted_data.len() / 2;
      if sorted_data.len() % 2 == 0 {
        (sorted_data[mid - 1] + sorted_data[mid]) as f64 / 2.0
      } else {
        sorted_data[mid] as f64
      }
    };

    let mode = self.calculate_mode(&sorted_data);
    let entropy = self.calculate_entropy(data);
    let dynamic_range = max - min;
    let contrast = if max > min {
      (max as f64 - min as f64) / (max as f64 + min as f64)
    } else {
      0.0
    };

    ImageStatistics {
      width,
      height,
      channels,
      min_value: min,
      max_value: max,
      mean_value: mean,
      median_value: median,
      mode_value: mode,
      standard_deviation: std_dev,
      variance,
      entropy,
      dynamic_range,
      contrast,
    }
  }

  pub fn analyze_channels(&self) -> Vec<ChannelAnalysis> {
    let mut channel_analyses = Vec::new();

    for channel in 0..self.image.channels() {
      if let Ok(channel_data) = self.image.get_channel(channel) {
        let mut min = 255u8;
        let mut max = 0u8;
        let mut sum = 0u64;

        for &byte in &channel_data {
          min = min.min(byte);
          max = max.max(byte);
          sum += byte as u64;
        }

        let mean = sum as f64 / channel_data.len() as f64;

        let mut variance = 0.0f64;
        for &byte in &channel_data {
          variance += (byte as f64 - mean).powi(2);
        }
        variance /= channel_data.len() as f64;
        let std_dev = variance.sqrt();

        let histogram = self.calculate_channel_histogram(&channel_data);
        let entropy = self.calculate_entropy(&channel_data);

        channel_analyses.push(ChannelAnalysis {
          channel,
          min_value: min,
          max_value: max,
          mean_value: mean,
          standard_deviation: std_dev,
          histogram,
          entropy,
        });
      }
    }

    channel_analyses
  }

  pub fn analyze_colors(&self) -> ColorAnalysis {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    if channels < 3 {
      return ColorAnalysis::default();
    }

    let mut color_counts = HashMap::new();
    let mut total_r = 0u32;
    let mut total_g = 0u32;
    let mut total_b = 0u32;
    let mut pixel_count = 0u32;

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, _)) = self.image.get_rgba_pixel(x, y) {
          let color = (r, g, b);
          *color_counts.entry(color).or_insert(0) += 1;
          total_r += r as u32;
          total_g += g as u32;
          total_b += b as u32;
          pixel_count += 1;
        }
      }
    }

    let average_color = if pixel_count > 0 {
      (
        (total_r / pixel_count) as u8,
        (total_g / pixel_count) as u8,
        (total_b / pixel_count) as u8,
      )
    } else {
      (0, 0, 0)
    };

    let dominant_color = color_counts
      .iter()
      .max_by_key(|(_, &count)| count)
      .map(|(&(r, g, b), _)| (r, g, b))
      .unwrap_or((0, 0, 0));

    let unique_colors = color_counts.len();
    let color_diversity = if pixel_count > 0 {
      unique_colors as f64 / pixel_count as f64
    } else {
      0.0
    };

    let color_palette = color_counts.into_iter().collect::<Vec<_>>();
    color_palette.sort_by(|a, b| b.1.cmp(&a.1));
    let top_colors: Vec<_> = color_palette.into_iter().take(16).collect();

    ColorAnalysis {
      average_color,
      dominant_color,
      unique_colors,
      color_diversity,
      top_colors,
    }
  }

  pub fn analyze_texture(&self) -> TextureAnalysis {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    let mut contrast_sum = 0.0f64;
    let mut contrast_count = 0;
    let mut directionality = [0.0f64; 4];
    let mut roughness = 0.0f64;
    let mut regularity = 0.0f64;

    for y in 1..height - 1 {
      for x in 1..width - 1 {
        for c in 0..channels {
          let center = self.image.get_pixel_value(x, y, c).unwrap_or(0) as f64;
          let top = self.image.get_pixel_value(x, y - 1, c).unwrap_or(0) as f64;
          let bottom = self.image.get_pixel_value(x, y + 1, c).unwrap_or(0) as f64;
          let left = self.image.get_pixel_value(x - 1, y, c).unwrap_or(0) as f64;
          let right = self.image.get_pixel_value(x + 1, y, c).unwrap_or(0) as f64;

          let horizontal_contrast = (right - left).abs();
          let vertical_contrast = (bottom - top).abs();
          let local_contrast = horizontal_contrast.max(vertical_contrast);

          contrast_sum += local_contrast;
          contrast_count += 1;

          if horizontal_contrast > vertical_contrast {
            directionality[0] += horizontal_contrast;
          } else {
            directionality[1] += vertical_contrast;
          }

          roughness += local_contrast;
        }
      }
    }

    let average_contrast = if contrast_count > 0 {
      contrast_sum / contrast_count as f64
    } else {
      0.0
    };

    let dominant_direction = directionality
      .iter()
      .enumerate()
      .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
      .map(|(i, _)| i)
      .unwrap_or(0);

    regularity = if contrast_count > 0 {
      let direction_variance = directionality
        .iter()
        .map(|&d| (d - average_contrast / 4.0).powi(2))
        .sum::<f64>()
        / 4.0;
      1.0 / (1.0 + direction_variance)
    } else {
      0.0
    };

    TextureAnalysis {
      average_contrast,
      dominant_direction,
      roughness: if contrast_count > 0 {
        roughness / contrast_count as f64
      } else {
        0.0
      },
      regularity,
    }
  }

  pub fn analyze_edges(&self) -> EdgeAnalysis {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    let mut edge_count = 0u32;
    let mut edge_strength_sum = 0.0f64;
    let mut edge_directions = [0u32; 8];
    let mut edge_lengths = Vec::new();

    for y in 1..height - 1 {
      for x in 1..width - 1 {
        for c in 0..channels {
          let center = self.image.get_pixel_value(x, y, c).unwrap_or(0) as f64;
          let gx = (self.image.get_pixel_value(x + 1, y, c).unwrap_or(0) as f64
            - self.image.get_pixel_value(x - 1, y, c).unwrap_or(0) as f64)
            / 2.0;
          let gy = (self.image.get_pixel_value(x, y + 1, c).unwrap_or(0) as f64
            - self.image.get_pixel_value(x, y - 1, c).unwrap_or(0) as f64)
            / 2.0;

          let magnitude = (gx * gx + gy * gy).sqrt();
          let angle = gy.atan2(gx);

          if magnitude > 10.0 {
            edge_count += 1;
            edge_strength_sum += magnitude;

            let direction_index =
              ((angle + std::f64::consts::PI) / (std::f64::consts::PI / 4.0)) as usize % 8;
            edge_directions[direction_index] += 1;
          }
        }
      }
    }

    let total_pixels = (width * height * channels as u32) as f64;
    let edge_density = edge_count as f64 / total_pixels;
    let average_edge_strength = if edge_count > 0 {
      edge_strength_sum / edge_count as f64
    } else {
      0.0
    };

    let dominant_edge_direction = edge_directions
      .iter()
      .enumerate()
      .max_by_key(|(_, &count)| count)
      .map(|(i, _)| i)
      .unwrap_or(0);

    EdgeAnalysis {
      edge_count,
      edge_density,
      average_edge_strength,
      dominant_edge_direction,
      edge_directions,
    }
  }

  pub fn analyze_frequency(&self) -> FrequencyAnalysis {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    let mut low_freq_energy = 0.0f64;
    let mut mid_freq_energy = 0.0f64;
    let mut high_freq_energy = 0.0f64;
    let mut total_energy = 0.0f64;

    for y in 1..height - 1 {
      for x in 1..width - 1 {
        for c in 0..channels {
          let center = self.image.get_pixel_value(x, y, c).unwrap_or(0) as f64;
          let neighbors = [
            self.image.get_pixel_value(x - 1, y, c).unwrap_or(0) as f64,
            self.image.get_pixel_value(x + 1, y, c).unwrap_or(0) as f64,
            self.image.get_pixel_value(x, y - 1, c).unwrap_or(0) as f64,
            self.image.get_pixel_value(x, y + 1, c).unwrap_or(0) as f64,
          ];

          let avg_neighbor = neighbors.iter().sum::<f64>() / 4.0;
          let low_freq = avg_neighbor;
          let high_freq = center - avg_neighbor;
          let mid_freq = (center + avg_neighbor) / 2.0 - low_freq;

          low_freq_energy += low_freq * low_freq;
          mid_freq_energy += mid_freq * mid_freq;
          high_freq_energy += high_freq * high_freq;
          total_energy += center * center;
        }
      }
    }

    let total_samples = ((width - 2) * (height - 2) * channels as u32) as f64;

    if total_samples > 0.0 {
      low_freq_energy /= total_samples;
      mid_freq_energy /= total_samples;
      high_freq_energy /= total_samples;
      total_energy /= total_samples;
    }

    let spectral_centroid = if total_energy > 0.0 {
      (low_freq_energy + 2.0 * mid_freq_energy + 3.0 * high_freq_energy) / total_energy
    } else {
      0.0
    };

    let spectral_rolloff = if total_energy > 0.0 {
      (high_freq_energy) / total_energy
    } else {
      0.0
    };

    FrequencyAnalysis {
      low_frequency_energy: low_freq_energy,
      mid_frequency_energy: mid_freq_energy,
      high_frequency_energy: high_freq_energy,
      total_energy,
      spectral_centroid,
      spectral_rolloff,
    }
  }

  pub fn calculate_quality_metrics(&self) -> QualityMetrics {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    let sharpness = self.calculate_sharpness();
    let noise_level = self.calculate_noise_level();
    let brightness = self.calculate_average_brightness();
    let contrast = self.calculate_overall_contrast();
    let saturation = self.calculate_average_saturation();
    let dynamic_range = self.calculate_dynamic_range();

    let overall_quality = (sharpness * 0.3
      + (1.0 - noise_level) * 0.2
      + contrast * 0.2
      + saturation * 0.15
      + (1.0 - (brightness - 0.5).abs()) * 0.15)
      .clamp(0.0, 1.0);

    QualityMetrics {
      sharpness,
      noise_level,
      brightness,
      contrast,
      saturation,
      dynamic_range,
      overall_quality,
    }
  }

  fn calculate_mode(&self, sorted_data: &[u8]) -> u8 {
    if sorted_data.is_empty() {
      return 0;
    }

    let mut mode = sorted_data[0];
    let mut max_count = 1;
    let mut current_count = 1;
    let mut current_value = sorted_data[0];

    for &value in sorted_data.iter().skip(1) {
      if value == current_value {
        current_count += 1;
      } else {
        if current_count > max_count {
          max_count = current_count;
          mode = current_value;
        }
        current_value = value;
        current_count = 1;
      }
    }

    if current_count > max_count {
      mode = current_value;
    }

    mode
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

  fn calculate_channel_histogram(&self, channel_data: &[u8]) -> Vec<u32> {
    let mut histogram = vec![0u32; 256];
    for &byte in channel_data {
      histogram[byte as usize] += 1;
    }
    histogram
  }

  fn calculate_sharpness(&self) -> f64 {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    let mut total_variance = 0.0f64;
    let mut sample_count = 0;

    for y in 1..height - 1 {
      for x in 1..width - 1 {
        for c in 0..channels {
          let center = self.image.get_pixel_value(x, y, c).unwrap_or(0) as f64;
          let laplacian = (self.image.get_pixel_value(x + 1, y, c).unwrap_or(0) as f64
            + self.image.get_pixel_value(x - 1, y, c).unwrap_or(0) as f64
            + self.image.get_pixel_value(x, y + 1, c).unwrap_or(0) as f64
            + self.image.get_pixel_value(x, y - 1, c).unwrap_or(0) as f64
            - 4.0 * center)
            .abs();

          total_variance += laplacian * laplacian;
          sample_count += 1;
        }
      }
    }

    if sample_count > 0 {
      (total_variance / sample_count as f64).sqrt() / 255.0
    } else {
      0.0
    }
  }

  fn calculate_noise_level(&self) -> f64 {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    let mut total_noise = 0.0f64;
    let mut sample_count = 0;

    for y in 1..height - 1 {
      for x in 1..width - 1 {
        for c in 0..channels {
          let center = self.image.get_pixel_value(x, y, c).unwrap_or(0) as f64;
          let neighbors = [
            self.image.get_pixel_value(x - 1, y, c).unwrap_or(0) as f64,
            self.image.get_pixel_value(x + 1, y, c).unwrap_or(0) as f64,
            self.image.get_pixel_value(x, y - 1, c).unwrap_or(0) as f64,
            self.image.get_pixel_value(x, y + 1, c).unwrap_or(0) as f64,
          ];

          let avg_neighbor = neighbors.iter().sum::<f64>() / 4.0;
          let noise = (center - avg_neighbor).abs();

          total_noise += noise;
          sample_count += 1;
        }
      }
    }

    if sample_count > 0 {
      (total_noise / sample_count as f64) / 255.0
    } else {
      0.0
    }
  }

  fn calculate_average_brightness(&self) -> f64 {
    let data = self.image.data().data;
    let sum: u64 = data.iter().map(|&byte| byte as u64).sum();
    sum as f64 / data.len() as f64 / 255.0
  }

  fn calculate_overall_contrast(&self) -> f64 {
    let data = self.image.data().data;
    if data.is_empty() {
      return 0.0;
    }

    let min = data.iter().min().copied().unwrap_or(0);
    let max = data.iter().max().copied().unwrap_or(0);

    if max > min {
      (max - min) as f64 / 255.0
    } else {
      0.0
    }
  }

  fn calculate_average_saturation(&self) -> f64 {
    if self.image.channels() < 3 {
      return 0.0;
    }

    let width = self.image.width();
    let height = self.image.height();
    let mut total_saturation = 0.0f64;
    let mut pixel_count = 0;

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, _)) = self.image.get_rgba_pixel(x, y) {
          let max = r.max(g).max(b) as f64;
          let min = r.min(g).min(b) as f64;
          let sum = r as f64 + g as f64 + b as f64;

          let saturation = if sum > 0.0 { (max - min) / max } else { 0.0 };

          total_saturation += saturation;
          pixel_count += 1;
        }
      }
    }

    if pixel_count > 0 {
      total_saturation / pixel_count as f64
    } else {
      0.0
    }
  }

  fn calculate_dynamic_range(&self) -> f64 {
    let data = self.image.data().data;
    if data.is_empty() {
      return 0.0;
    }

    let min = data.iter().min().copied().unwrap_or(0);
    let max = data.iter().max().copied().unwrap_or(0);

    if max > min {
      ((max - min) as f64).log2() / 8.0
    } else {
      0.0
    }
  }

  pub fn compare_with(&self, other: &ImageProcessor) -> ImageComparisonReport {
    let my_analysis = self.analyze();
    let other_analyzer = ImageAnalyzer::new(other.clone());
    let other_analysis = other_analyzer.analyze();

    let structural_similarity = self.calculate_ssim(other);
    let histogram_correlation = self.calculate_histogram_correlation(other);
    let mean_squared_error = self.calculate_mse(other);
    let peak_signal_to_noise_ratio = if mean_squared_error > 0.0 {
      20.0 * (255.0_f64).log10() - 10.0 * mean_squared_error.log10()
    } else {
      f64::INFINITY
    };

    let overall_similarity = (structural_similarity * 0.4
      + histogram_correlation * 0.3
      + (1.0 - mean_squared_error / (255.0 * 255.0)) * 0.3)
      .clamp(0.0, 1.0);

    ImageComparisonReport {
      structural_similarity,
      histogram_correlation,
      mean_squared_error,
      peak_signal_to_noise_ratio,
      overall_similarity,
    }
  }

  fn calculate_ssim(&self, other: &ImageProcessor) -> f64 {
    if self.image.width() != other.width() || self.image.height() != other.height() {
      return 0.0;
    }

    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    let mut total_ssim = 0.0f64;
    let mut window_count = 0;

    let window_size = 8;
    let k1 = 0.01;
    let k2 = 0.03;
    let l = 255.0;
    let c1 = (k1 * l).powi(2);
    let c2 = (k2 * l).powi(2);

    for y in (0..height - window_size).step_by(window_size as usize) {
      for x in (0..width - window_size).step_by(window_size as usize) {
        for c in 0..channels {
          let mut mu1 = 0.0f64;
          let mut mu2 = 0.0f64;
          let mut sigma1_sq = 0.0f64;
          let mut sigma2_sq = 0.0f64;
          let mut sigma12 = 0.0f64;
          let pixel_count = (window_size * window_size) as f64;

          for dy in 0..window_size {
            for dx in 0..window_size {
              let px = x + dx;
              let py = y + dy;

              let val1 = self.image.get_pixel_value(px, py, c).unwrap_or(0) as f64;
              let val2 = other.get_pixel_value(px, py, c).unwrap_or(0) as f64;

              mu1 += val1;
              mu2 += val2;
            }
          }

          mu1 /= pixel_count;
          mu2 /= pixel_count;

          for dy in 0..window_size {
            for dx in 0..window_size {
              let px = x + dx;
              let py = y + dy;

              let val1 = self.image.get_pixel_value(px, py, c).unwrap_or(0) as f64;
              let val2 = other.get_pixel_value(px, py, c).unwrap_or(0) as f64;

              sigma1_sq += (val1 - mu1).powi(2);
              sigma2_sq += (val2 - mu2).powi(2);
              sigma12 += (val1 - mu1) * (val2 - mu2);
            }
          }

          sigma1_sq /= pixel_count;
          sigma2_sq /= pixel_count;
          sigma12 /= pixel_count;

          let numerator = (2.0 * mu1 * mu2 + c1) * (2.0 * sigma12 + c2);
          let denominator = (mu1 * mu1 + mu2 * mu2 + c1) * (sigma1_sq + sigma2_sq + c2);

          if denominator > 0.0 {
            total_ssim += numerator / denominator;
            window_count += 1;
          }
        }
      }
    }

    if window_count > 0 {
      total_ssim / window_count as f64
    } else {
      0.0
    }
  }

  fn calculate_histogram_correlation(&self, other: &ImageProcessor) -> f64 {
    let hist1 = self.image.histogram();
    let hist2 = other.histogram();

    let mut correlation = 0.0f64;
    let mut sum1 = 0u64;
    let mut sum2 = 0u64;
    let mut sum1_sq = 0u64;
    let mut sum2_sq = 0u64;
    let mut sum12 = 0u64;
    let n = hist1.len();

    for i in 0..n {
      sum1 += hist1[i];
      sum2 += hist2[i];
      sum1_sq += hist1[i] * hist1[i];
      sum2_sq += hist2[i] * hist2[i];
      sum12 += hist1[i] * hist2[i];
    }

    let numerator = (n as u64 * sum12 - sum1 * sum2) as f64;
    let denominator =
      ((n as u64 * sum1_sq - sum1 * sum1) * (n as u64 * sum2_sq - sum2 * sum2)) as f64;

    if denominator > 0.0 {
      numerator / denominator.sqrt()
    } else {
      0.0
    }
  }

  fn calculate_mse(&self, other: &ImageProcessor) -> f64 {
    if self.image.width() != other.width() || self.image.height() != other.height() {
      return f64::INFINITY;
    }

    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    let mut total_error = 0.0f64;
    let mut pixel_count = 0;

    for y in 0..height {
      for x in 0..width {
        for c in 0..channels {
          let val1 = self.image.get_pixel_value(x, y, c).unwrap_or(0) as f64;
          let val2 = other.get_pixel_value(x, y, c).unwrap_or(0) as f64;
          total_error += (val1 - val2).powi(2);
          pixel_count += 1;
        }
      }
    }

    if pixel_count > 0 {
      total_error / pixel_count as f64
    } else {
      0.0
    }
  }
}

#[derive(Debug, Clone)]
pub struct ImageAnalysisReport {
  pub width: u32,
  pub height: u32,
  pub channels: u8,
  pub pixel_count: usize,
  pub byte_size: usize,
  pub histogram: Vec<u32>,
  pub channel_histograms: Vec<ChannelAnalysis>,
  pub statistics: ImageStatistics,
  pub color_analysis: ColorAnalysis,
  pub texture_analysis: TextureAnalysis,
  pub edge_analysis: EdgeAnalysis,
  pub frequency_analysis: FrequencyAnalysis,
  pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone)]
pub struct ImageStatistics {
  pub width: u32,
  pub height: u32,
  pub channels: u8,
  pub min_value: u8,
  pub max_value: u8,
  pub mean_value: f64,
  pub median_value: f64,
  pub mode_value: u8,
  pub standard_deviation: f64,
  pub variance: f64,
  pub entropy: f64,
  pub dynamic_range: u8,
  pub contrast: f64,
}

#[derive(Debug, Clone)]
pub struct ChannelAnalysis {
  pub channel: u8,
  pub min_value: u8,
  pub max_value: u8,
  pub mean_value: f64,
  pub standard_deviation: f64,
  pub histogram: Vec<u32>,
  pub entropy: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ColorAnalysis {
  pub average_color: (u8, u8, u8),
  pub dominant_color: (u8, u8, u8),
  pub unique_colors: usize,
  pub color_diversity: f64,
  pub top_colors: Vec<((u8, u8, u8), u32)>,
}

#[derive(Debug, Clone)]
pub struct TextureAnalysis {
  pub average_contrast: f64,
  pub dominant_direction: usize,
  pub roughness: f64,
  pub regularity: f64,
}

#[derive(Debug, Clone)]
pub struct EdgeAnalysis {
  pub edge_count: u32,
  pub edge_density: f64,
  pub average_edge_strength: f64,
  pub dominant_edge_direction: usize,
  pub edge_directions: [u32; 8],
}

#[derive(Debug, Clone)]
pub struct FrequencyAnalysis {
  pub low_frequency_energy: f64,
  pub mid_frequency_energy: f64,
  pub high_frequency_energy: f64,
  pub total_energy: f64,
  pub spectral_centroid: f64,
  pub spectral_rolloff: f64,
}

#[derive(Debug, Clone)]
pub struct QualityMetrics {
  pub sharpness: f64,
  pub noise_level: f64,
  pub brightness: f64,
  pub contrast: f64,
  pub saturation: f64,
  pub dynamic_range: f64,
  pub overall_quality: f64,
}

#[derive(Debug, Clone)]
pub struct ImageComparisonReport {
  pub structural_similarity: f64,
  pub histogram_correlation: f64,
  pub mean_squared_error: f64,
  pub peak_signal_to_noise_ratio: f64,
  pub overall_similarity: f64,
}

pub fn create_analyzer(image: ImageProcessor) -> ImageAnalyzer {
  ImageAnalyzer::new(image)
}

pub fn analyze_image(image: &ImageProcessor) -> ImageAnalysisReport {
  let analyzer = create_analyzer(image.clone());
  analyzer.analyze()
}

pub fn compare_images(image1: &ImageProcessor, image2: &ImageProcessor) -> ImageComparisonReport {
  let analyzer = create_analyzer(image1.clone());
  analyzer.compare_with(image2)
}
