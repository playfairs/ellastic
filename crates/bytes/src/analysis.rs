use crate::{
  ByteBuffer,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ByteAnalyzer {
  buffer: ByteBuffer,
}

impl ByteAnalyzer {
  pub fn new(buffer: ByteBuffer) -> Self {
    Self { buffer }
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

  pub fn analyze(&self) -> AnalysisReport {
    let data = self.buffer.data();
    let histogram = self.buffer.get_histogram();
    let entropy = self.calculate_entropy(data);

    let byte_frequency = self.calculate_byte_frequency(&histogram);
    let byte_distribution = self.analyze_byte_distribution(&histogram);
    let patterns = self.detect_patterns(data);
    let anomalies = self.detect_anomalies(data);
    let structure = self.analyze_structure(data);
    let compression = self.analyze_compressibility(data);

    AnalysisReport {
      length: data.len(),
      entropy,
      histogram,
      byte_frequency,
      byte_distribution,
      patterns,
      anomalies,
      structure,
      compression,
    }
  }

  pub fn calculate_entropy(&self, data: &[u8]) -> f64 {
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

  pub fn calculate_byte_frequency(&self, histogram: &[u64; 256]) -> Vec<(u8, f64)> {
    let total: u64 = histogram.iter().sum();
    if total == 0 {
      return Vec::new();
    }

    let mut frequencies = Vec::new();
    for (i, &count) in histogram.iter().enumerate() {
      if count > 0 {
        frequencies.push((i as u8, count as f64 / total as f64));
      }
    }

    frequencies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    frequencies
  }

  pub fn analyze_byte_distribution(&self, histogram: &[u64; 256]) -> ByteDistribution {
    let total: u64 = histogram.iter().sum();
    if total == 0 {
      return ByteDistribution::default();
    }

    let mut zero_bytes = 0;
    let mut max_bytes = 0;
    let mut ascii_printable = 0;
    let mut ascii_control = 0;
    let mut high_ascii = 0;
    let mut unique_bytes = 0;

    for (i, &count) in histogram.iter().enumerate() {
      if count > 0 {
        unique_bytes += 1;
      }

      match i {
        0 => zero_bytes = count,
        255 => max_bytes = count,
        32..=126 => ascii_printable += count,
        0..=31 | 127 => ascii_control += count,
        128..=255 => high_ascii += count,
        _ => {}
      }
    }

    ByteDistribution {
      zero_bytes,
      max_bytes,
      ascii_printable,
      ascii_control,
      high_ascii,
      unique_bytes,
      total_bytes: total,
      unique_ratio: unique_bytes as f64 / 256.0,
    }
  }

  pub fn detect_patterns(&self, data: &[u8]) -> PatternAnalysis {
    let mut repeating_sequences = Vec::new();
    let mut constant_runs = Vec::new();
    let mut increasing_sequences = Vec::new();
    let mut decreasing_sequences = Vec::new();

    let mut current_run = 1;
    let mut current_value = data.get(0).copied();

    for i in 1..data.len() {
      let current_byte = data[i];

      if let Some(prev_value) = current_value {
        if current_byte == prev_value {
          current_run += 1;
        } else {
          if current_run >= 3 {
            constant_runs.push((i - current_run, i, prev_value));
          }
          current_run = 1;
          current_value = Some(current_byte);
        }
      } else {
        current_value = Some(current_byte);
        current_run = 1;
      }
    }

    if current_run >= 3 {
      if let Some(prev_value) = current_value {
        constant_runs.push((data.len() - current_run, data.len(), prev_value));
      }
    }

    for length in 2..=std::cmp::min(16, data.len() / 2) {
      let mut pattern_counts = HashMap::new();

      for i in 0..=(data.len() - length * 2) {
        let pattern = &data[i..i + length];
        let next_pattern = &data[i + length..i + length * 2];

        if pattern == next_pattern {
          let count = pattern_counts.entry(pattern.to_vec()).or_insert(0);
          *count += 1;
        }
      }

      for (pattern, count) in pattern_counts {
        if count >= 2 {
          repeating_sequences.push((pattern.clone(), count));
        }
      }
    }

    for i in 2..data.len() {
      if data[i] == data[i - 1] + 1 && data[i - 1] == data[i - 2] + 1 {
        let start = i - 2;
        let mut end = i + 1;

        while end < data.len() && data[end] == data[end - 1] + 1 {
          end += 1;
        }

        if end - start >= 3 {
          increasing_sequences.push((start, end));
        }
      }
    }

    for i in 2..data.len() {
      if data[i] == data[i - 1] - 1 && data[i - 1] == data[i - 2] - 1 {
        let start = i - 2;
        let mut end = i + 1;

        while end < data.len() && data[end] == data[end - 1] - 1 {
          end += 1;
        }

        if end - start >= 3 {
          decreasing_sequences.push((start, end));
        }
      }
    }

    PatternAnalysis {
      repeating_sequences,
      constant_runs,
      increasing_sequences,
      decreasing_sequences,
    }
  }

  pub fn detect_anomalies(&self, data: &[u8]) -> AnomalyAnalysis {
    let mut outliers = Vec::new();
    let mut statistical_anomalies = Vec::new();
    let mut entropy_anomalies = Vec::new();

    let mean = data.iter().map(|&b| b as f64).sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>() / data.len() as f64;
    let std_dev = variance.sqrt();

    for (i, &byte) in data.iter().enumerate() {
      let z_score = (byte as f64 - mean) / std_dev;
      if z_score.abs() > 3.0 {
        outliers.push((i, byte, z_score));
      }
    }

    let window_size = std::cmp::min(64, data.len() / 4);
    if window_size > 0 {
      for i in 0..=(data.len() - window_size) {
        let window = &data[i..i + window_size];
        let window_entropy = self.calculate_entropy(window);

        if window_entropy < 2.0 {
          entropy_anomalies.push((i, i + window_size, window_entropy));
        }
      }
    }

    let histogram = self.buffer.get_histogram();
    let total = data.len() as f64;

    for (i, &count) in histogram.iter().enumerate() {
      let frequency = count as f64 / total;
      if frequency > 0.1 {
        statistical_anomalies.push((i as u8, frequency));
      }
    }

    AnomalyAnalysis {
      outliers,
      statistical_anomalies,
      entropy_anomalies,
    }
  }

  pub fn analyze_structure(&self, data: &[u8]) -> StructureAnalysis {
    let mut headers = Vec::new();
    let mut footers = Vec::new();
    let mut chunk_boundaries = Vec::new();

    let common_signatures: &[&[u8]] = &[
      b"\x89PNG\r\n\x1a\n",
      b"\xff\xd8\xff",
      b"GIF87a",
      b"GIF89a",
      b"RIFF",
      b"ftyp",
      b"\x00\x00\x01\x00",
      b"ID3",
      b"OggS",
      b"fLaC",
      b"WAVE",
      b"BM",
    ];

    for (i, &signature) in common_signatures.iter().enumerate() {
      if data.starts_with(signature) {
        headers.push((0, signature.to_vec(), format!("Header type {}", i)));
      }

      for j in 0..=(data.len().saturating_sub(signature.len())) {
        if &data[j..j + signature.len()] == signature {
          if j == 0 {
            headers.push((j, signature.to_vec(), format!("Header type {}", i)));
          } else if j + signature.len() == data.len() {
            footers.push((j, signature.to_vec(), format!("Footer type {}", i)));
          } else {
            chunk_boundaries.push((j, signature.to_vec(), format!("Boundary type {}", i)));
          }
        }
      }
    }

    let mut repeated_chunks = HashMap::new();
    let chunk_size = std::cmp::min(1024, data.len() / 10);

    for i in (0..data.len()).step_by(chunk_size) {
      let end = std::cmp::min(i + chunk_size, data.len());
      let chunk = &data[i..end];

      let count = repeated_chunks.entry(chunk.to_vec()).or_insert(0);
      *count += 1;
    }

    let repeated_chunks: Vec<_> = repeated_chunks
      .into_iter()
      .filter(|(_, count)| *count > 1)
      .collect();

    StructureAnalysis {
      headers,
      footers,
      chunk_boundaries,
      repeated_chunks,
    }
  }

  pub fn analyze_compressibility(&self, data: &[u8]) -> CompressionAnalysis {
    let run_length_encoding = self.analyze_run_length_encoding(data);
    let entropy_compression = self.analyze_entropy_compression(data);
    let pattern_compression = self.analyze_pattern_compression(data);

    CompressionAnalysis {
      run_length_encoding,
      entropy_compression,
      pattern_compression,
    }
  }

  fn analyze_run_length_encoding(&self, data: &[u8]) -> RunLengthAnalysis {
    let mut runs = Vec::new();
    let mut current_run = 1;
    let mut current_value = data.get(0).copied();

    for i in 1..data.len() {
      let current_byte = data[i];

      if let Some(prev_value) = current_value {
        if current_byte == prev_value && current_run < 255 {
          current_run += 1;
        } else {
          runs.push((prev_value, current_run));
          current_run = 1;
          current_value = Some(current_byte);
        }
      } else {
        current_value = Some(current_byte);
        current_run = 1;
      }
    }

    if let Some(prev_value) = current_value {
      runs.push((prev_value, current_run));
    }

    let original_size = data.len();
    let compressed_size = runs.len() * 2;
    let compression_ratio = compressed_size as f64 / original_size as f64;

    RunLengthAnalysis {
      runs,
      original_size,
      compressed_size,
      compression_ratio,
    }
  }

  fn analyze_entropy_compression(&self, data: &[u8]) -> EntropyAnalysis {
    let entropy = self.calculate_entropy(data);
    let theoretical_min_bits_per_byte = entropy;
    let theoretical_compression_ratio = theoretical_min_bits_per_byte / 8.0;

    let histogram = self.buffer.get_histogram();
    let mut symbol_count = 0;
    for &count in &histogram {
      if count > 0 {
        symbol_count += 1;
      }
    }

    EntropyAnalysis {
      entropy,
      theoretical_min_bits_per_byte,
      theoretical_compression_ratio,
      symbol_count,
    }
  }

  fn analyze_pattern_compression(&self, data: &[u8]) -> PatternAnalysis {
    let mut patterns = HashMap::new();
    let max_pattern_length = std::cmp::min(16, data.len() / 4);

    for length in 2..=max_pattern_length {
      for i in 0..=(data.len() - length) {
        let pattern = &data[i..i + length];
        let count = patterns.entry(pattern.to_vec()).or_insert(0);
        *count += 1;
      }
    }

    let mut sorted_patterns: Vec<_> = patterns.into_iter().collect();
    sorted_patterns.sort_by(|a, b| b.1.cmp(&a.1));

    let total_patterns = sorted_patterns.len();
    let repeating_patterns = sorted_patterns
      .iter()
      .filter(|(_, count)| *count > 1)
      .count();

    PatternAnalysis {
      repeating_sequences: sorted_patterns,
      constant_runs: Vec::new(),
      increasing_sequences: Vec::new(),
      decreasing_sequences: Vec::new(),
    }
  }

  pub fn compare_with(&self, other: &ByteBuffer) -> ComparisonReport {
    let my_data = self.buffer.data();
    let other_data = other.data();

    let similarity = self.calculate_similarity(my_data, other_data);
    let differences = self.find_differences(my_data, other_data);
    let correlation = self.calculate_correlation(my_data, other_data);

    ComparisonReport {
      similarity,
      differences,
      correlation,
    }
  }

  fn calculate_similarity(&self, data1: &[u8], data2: &[u8]) -> f64 {
    let min_len = std::cmp::min(data1.len(), data2.len());
    if min_len == 0 {
      return 1.0;
    }

    let mut matching_bytes = 0;
    for i in 0..min_len {
      if data1[i] == data2[i] {
        matching_bytes += 1;
      }
    }

    matching_bytes as f64 / min_len as f64
  }

  fn find_differences(&self, data1: &[u8], data2: &[u8]) -> Vec<(usize, u8, u8)> {
    let min_len = std::cmp::min(data1.len(), data2.len());
    let mut differences = Vec::new();

    for i in 0..min_len {
      if data1[i] != data2[i] {
        differences.push((i, data1[i], data2[i]));
      }
    }

    differences
  }

  fn calculate_correlation(&self, data1: &[u8], data2: &[u8]) -> f64 {
    let min_len = std::cmp::min(data1.len(), data2.len());
    if min_len == 0 {
      return 1.0;
    }

    let mean1: f64 = data1[..min_len].iter().map(|&b| b as f64).sum::<f64>() / min_len as f64;
    let mean2: f64 = data2[..min_len].iter().map(|&b| b as f64).sum::<f64>() / min_len as f64;

    let mut numerator = 0.0;
    let mut variance1 = 0.0;
    let mut variance2 = 0.0;

    for i in 0..min_len {
      let diff1 = data1[i] as f64 - mean1;
      let diff2 = data2[i] as f64 - mean2;

      numerator += diff1 * diff2;
      variance1 += diff1 * diff1;
      variance2 += diff2 * diff2;
    }

    if variance1 == 0.0 || variance2 == 0.0 {
      return 1.0;
    }

    numerator / (variance1.sqrt() * variance2.sqrt())
  }

  pub fn get_byte_statistics(&self) -> ByteStatistics {
    let data = self.buffer.data();
    let histogram = self.buffer.get_histogram();

    let min = data.iter().min().copied().unwrap_or(0);
    let max = data.iter().max().copied().unwrap_or(0);
    let mean = data.iter().map(|&b| b as f64).sum::<f64>() / data.len() as f64;

    let variance = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>() / data.len() as f64;
    let std_dev = variance.sqrt();

    let median = if data.is_empty() {
      0.0
    } else {
      let mut sorted = data.to_vec();
      sorted.sort();
      if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) as f64 / 2.0
      } else {
        sorted[sorted.len() / 2] as f64
      }
    };

    let mode = histogram
      .iter()
      .enumerate()
      .filter(|(_, &count)| count > 0)
      .max_by_key(|(_, &count)| count)
      .map(|(byte, _)| byte as u8)
      .unwrap_or(0);

    ByteStatistics {
      min,
      max,
      mean,
      median,
      mode,
      std_dev,
      variance,
    }
  }
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
  pub length: usize,
  pub entropy: f64,
  pub histogram: [u64; 256],
  pub byte_frequency: Vec<(u8, f64)>,
  pub byte_distribution: ByteDistribution,
  pub patterns: PatternAnalysis,
  pub anomalies: AnomalyAnalysis,
  pub structure: StructureAnalysis,
  pub compression: CompressionAnalysis,
}

#[derive(Debug, Clone)]
pub struct ByteDistribution {
  pub zero_bytes: u64,
  pub max_bytes: u64,
  pub ascii_printable: u64,
  pub ascii_control: u64,
  pub high_ascii: u64,
  pub unique_bytes: usize,
  pub total_bytes: u64,
  pub unique_ratio: f64,
}

impl Default for ByteDistribution {
  fn default() -> Self {
    Self {
      zero_bytes: 0,
      max_bytes: 0,
      ascii_printable: 0,
      ascii_control: 0,
      high_ascii: 0,
      unique_bytes: 0,
      total_bytes: 0,
      unique_ratio: 0.0,
    }
  }
}

#[derive(Debug, Clone)]
pub struct PatternAnalysis {
  pub repeating_sequences: Vec<(Vec<u8>, usize)>,
  pub constant_runs: Vec<(usize, usize, u8)>,
  pub increasing_sequences: Vec<(usize, usize)>,
  pub decreasing_sequences: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct AnomalyAnalysis {
  pub outliers: Vec<(usize, u8, f64)>,
  pub statistical_anomalies: Vec<(u8, f64)>,
  pub entropy_anomalies: Vec<(usize, usize, f64)>,
}

#[derive(Debug, Clone)]
pub struct StructureAnalysis {
  pub headers: Vec<(usize, Vec<u8>, String)>,
  pub footers: Vec<(usize, Vec<u8>, String)>,
  pub chunk_boundaries: Vec<(usize, Vec<u8>, String)>,
  pub repeated_chunks: Vec<(Vec<u8>, usize)>,
}

#[derive(Debug, Clone)]
pub struct CompressionAnalysis {
  pub run_length_encoding: RunLengthAnalysis,
  pub entropy_compression: EntropyAnalysis,
  pub pattern_compression: PatternAnalysis,
}

#[derive(Debug, Clone)]
pub struct RunLengthAnalysis {
  pub runs: Vec<(u8, usize)>,
  pub original_size: usize,
  pub compressed_size: usize,
  pub compression_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct EntropyAnalysis {
  pub entropy: f64,
  pub theoretical_min_bits_per_byte: f64,
  pub theoretical_compression_ratio: f64,
  pub symbol_count: usize,
}

#[derive(Debug, Clone)]
pub struct ComparisonReport {
  pub similarity: f64,
  pub differences: Vec<(usize, u8, u8)>,
  pub correlation: f64,
}

#[derive(Debug, Clone)]
pub struct ByteStatistics {
  pub min: u8,
  pub max: u8,
  pub mean: f64,
  pub median: f64,
  pub mode: u8,
  pub std_dev: f64,
  pub variance: f64,
}

pub fn create_analyzer(buffer: ByteBuffer) -> ByteAnalyzer {
  ByteAnalyzer::new(buffer)
}

pub fn analyze_buffer(buffer: &ByteBuffer) -> AnalysisReport {
  let analyzer = create_analyzer(buffer.clone());
  analyzer.analyze()
}

pub fn compare_buffers(buffer1: &ByteBuffer, buffer2: &ByteBuffer) -> ComparisonReport {
  let analyzer = create_analyzer(buffer1.clone());
  analyzer.compare_with(buffer2)
}

pub fn calculate_entropy(data: &[u8]) -> f64 {
  let analyzer = create_analyzer(ByteBuffer::from(data.to_vec()));
  analyzer.calculate_entropy(data)
}
