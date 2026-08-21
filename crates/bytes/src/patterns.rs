use crate::{
  ByteBuffer,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use ellastic_utils::{
  RandomGenerator,
  create_random_generator,
  create_random_generator_with_seed,
};

#[derive(Debug, Clone)]
pub struct PatternMatcher {
  buffer: ByteBuffer,
}

impl PatternMatcher {
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

  pub fn find_pattern(&self, pattern: &[u8]) -> Vec<usize> {
    let data = self.buffer.data();
    let mut matches = Vec::new();

    if pattern.is_empty() || data.len() < pattern.len() {
      return matches;
    }

    for i in 0..=(data.len() - pattern.len()) {
        if &data[i..i + pattern.len()] == pattern {
        matches.push(i);
      }
    }

    matches
  }

  pub fn find_pattern_with_wildcards(&self, pattern: &[Option<u8>]) -> Vec<usize> {
    let data = self.buffer.data();
    let mut matches = Vec::new();

    if pattern.is_empty() || data.len() < pattern.len() {
      return matches;
    }

    for i in 0..=(data.len() - pattern.len()) {
      let mut is_match = true;
      for (j, &pattern_byte) in pattern.iter().enumerate() {
        match pattern_byte {
          Some(expected) if data[i + j] != expected => {
            is_match = false;
            break;
          }
          None => continue,
          _ => {}
        }
      }
      if is_match {
        matches.push(i);
      }
    }

    matches
  }

  pub fn find_repeating_pattern(&self, min_length: usize) -> Vec<(usize, usize, Vec<u8>)> {
    let data = self.buffer.data();
    let mut patterns = Vec::new();

    for length in min_length..=data.len() / 2 {
      for start in 0..=(data.len() - length * 2) {
        let pattern = &data[start..start + length];
        let mut end = start + length;
        let mut repetitions = 1;

        while end + length <= data.len() && &data[end..end + length] == pattern {
          repetitions += 1;
          end += length;
        }

        if repetitions >= 2 {
          patterns.push((start, repetitions, pattern.to_vec()));
        }
      }
    }

    patterns
  }

  pub fn find_sequences(&self, sequence_type: SequenceType) -> Vec<(usize, usize)> {
    let data = self.buffer.data();
    let mut sequences = Vec::new();

    match sequence_type {
      SequenceType::Increasing => {
        let mut start = 0;
        for i in 1..data.len() {
          if data[i] != data[i - 1] + 1 {
            if i - start >= 2 {
              sequences.push((start, i));
            }
            start = i;
          }
        }
        if data.len() - start >= 2 {
          sequences.push((start, data.len()));
        }
      }
      SequenceType::Decreasing => {
        let mut start = 0;
        for i in 1..data.len() {
          if data[i] != data[i - 1] - 1 {
            if i - start >= 2 {
              sequences.push((start, i));
            }
            start = i;
          }
        }
        if data.len() - start >= 2 {
          sequences.push((start, data.len()));
        }
      }
      SequenceType::Constant => {
        let mut start = 0;
        for i in 1..data.len() {
          if data[i] != data[i - 1] {
            if i - start >= 3 {
              sequences.push((start, i));
            }
            start = i;
          }
        }
        if data.len() - start >= 3 {
          sequences.push((start, data.len()));
        }
      }
      SequenceType::Alternating { pattern } => {
        let pattern_len = pattern.len();
        if pattern_len == 0 {
          return sequences;
        }

        for i in 0..=(data.len() - pattern_len * 2) {
          let mut is_match = true;
          for j in 0..pattern_len * 2 {
            let expected = if (j / pattern_len) % 2 == 0 {
              pattern[j % pattern_len]
            } else {
              pattern[pattern_len - 1 - (j % pattern_len)]
            };
            if data[i + j] != expected {
              is_match = false;
              break;
            }
          }
          if is_match {
            sequences.push((i, i + pattern_len * 2));
          }
        }
      }
    }

    sequences
  }

  pub fn find_anomalies(&self, window_size: usize, threshold: f64) -> Vec<(usize, f64)> {
    let data = self.buffer.data();
    let mut anomalies = Vec::new();

    if data.len() < window_size {
      return anomalies;
    }

    for i in 0..=(data.len() - window_size) {
      let window = &data[i..i + window_size];
      let mean = window.iter().map(|&b| b as f64).sum::<f64>() / window_size as f64;
      let variance = window
        .iter()
        .map(|&b| (b as f64 - mean).powi(2))
        .sum::<f64>()
        / window_size as f64;
      let std_dev = variance.sqrt();

      for (j, &byte) in window.iter().enumerate() {
        let z_score = (byte as f64 - mean) / std_dev;
        if z_score.abs() > threshold {
          anomalies.push((i + j, z_score));
        }
      }
    }

    anomalies
  }

  pub fn find_entropy_anomalies(&self, window_size: usize, threshold: f64) -> Vec<(usize, f64)> {
    let data = self.buffer.data();
    let mut anomalies = Vec::new();

    if data.len() < window_size {
      return anomalies;
    }

    for i in 0..=(data.len() - window_size) {
      let window = &data[i..i + window_size];
      let entropy = self.calculate_entropy(window);

      if entropy < threshold {
        anomalies.push((i, entropy));
      }
    }

    anomalies
  }

  pub fn find_byte_frequency_anomalies(&self, threshold: f64) -> Vec<(u8, f64)> {
    let data = self.buffer.data();
    let histogram = self.buffer.get_histogram();
    let total = data.len() as f64;
    let mut anomalies = Vec::new();

    for (byte, &count) in histogram.iter().enumerate() {
      let frequency = count as f64 / total;
      if frequency > threshold {
        anomalies.push((byte as u8, frequency));
      }
    }

    anomalies
  }

  pub fn replace_pattern(&mut self, pattern: &[u8], replacement: &[u8]) -> Result<usize> {
    let matches = self.find_pattern(pattern);
    let data = self.buffer.data().to_vec();
    let mut new_data = Vec::with_capacity(data.len());
    let mut last_pos = 0;
    let mut replacements = 0;

    for &pos in &matches {
      new_data.extend_from_slice(&data[last_pos..pos]);
      new_data.extend_from_slice(replacement);
      last_pos = pos + pattern.len();
      replacements += 1;
    }
    new_data.extend_from_slice(&data[last_pos..]);

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&new_data);

    Ok(replacements)
  }

  pub fn replace_pattern_with_callback<F>(
    &mut self,
    pattern: &[u8],
    mut callback: F,
  ) -> Result<usize>
  where
    F: FnMut(&[u8]) -> Vec<u8>,
  {
    let matches = self.find_pattern(pattern);
    let data = self.buffer.data().to_vec();
    let mut new_data = Vec::with_capacity(data.len());
    let mut last_pos = 0;
    let mut replacements = 0;

    for &pos in &matches {
      new_data.extend_from_slice(&data[last_pos..pos]);
      let replacement = callback(&data[pos..pos + pattern.len()]);
      new_data.extend_from_slice(&replacement);
      last_pos = pos + pattern.len();
      replacements += 1;
    }
    new_data.extend_from_slice(&data[last_pos..]);

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&new_data);

    Ok(replacements)
  }

  pub fn extract_patterns(&self, min_length: usize, max_patterns: Option<usize>) -> Vec<Pattern> {
    let data = self.buffer.data();
    let mut patterns = Vec::new();
    let mut pattern_counts = std::collections::HashMap::new();

    for length in min_length..=std::cmp::min(16, data.len()) {
      for i in 0..=(data.len() - length) {
        let pattern = data[i..i + length].to_vec();
        *pattern_counts.entry(pattern.clone()).or_insert(0) += 1;
      }
    }

    let mut sorted_patterns: Vec<_> = pattern_counts.into_iter().collect();
    sorted_patterns.sort_by(|a, b| b.1.cmp(&a.1));

    let limit = max_patterns.unwrap_or(sorted_patterns.len());
    for (pattern, count) in sorted_patterns.into_iter().take(limit) {
      if count >= 2 {
        let positions = self.find_pattern(&pattern);
        patterns.push(Pattern {
          data: pattern,
          occurrences: count,
          positions,
        });
      }
    }

    patterns
  }

  pub fn generate_signature(&self) -> PatternSignature {
    let data = self.buffer.data();
    let histogram = self.buffer.get_histogram();
    let entropy = self.calculate_entropy(data);

    let mut byte_frequencies = Vec::new();
    for (byte, &count) in histogram.iter().enumerate() {
      if count > 0 {
          byte_frequencies.push((byte as u8, count as f64 / data.len() as f64));
      }
    }
    byte_frequencies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let sequences = self.find_sequences(SequenceType::Increasing);
    let repeating_patterns = self.find_repeating_pattern(2);

    PatternSignature {
      length: data.len(),
      entropy,
        byte_frequencies: byte_frequencies.clone(),
      sequence_count: sequences.len(),
      repeating_pattern_count: repeating_patterns.len(),
      most_common_byte: byte_frequencies.first().map(|(b, _)| *b),
      least_common_byte: byte_frequencies.last().map(|(b, _)| *b),
    }
  }

  pub fn compare_signatures(&self, other: &PatternSignature) -> SimilarityScore {
    let my_signature = self.generate_signature();

    let length_similarity = 1.0
      - (my_signature.length as f64 - other.length as f64).abs()
        / (my_signature.length as f64 + other.length as f64);
    let entropy_similarity =
      1.0 - (my_signature.entropy - other.entropy).abs() / (my_signature.entropy + other.entropy);

    let mut frequency_similarity = 0.0;
    let mut common_bytes = 0;

    for (my_byte, my_freq) in &my_signature.byte_frequencies {
      for (other_byte, other_freq) in &other.byte_frequencies {
        if my_byte == other_byte {
          frequency_similarity += 1.0 - (my_freq - other_freq).abs();
          common_bytes += 1;
          break;
        }
      }
    }

    let total_unique_bytes =
      my_signature.byte_frequencies.len() + other.byte_frequencies.len() - common_bytes;
    if total_unique_bytes > 0 {
      frequency_similarity /= total_unique_bytes as f64;
    }

    SimilarityScore {
      overall: (length_similarity + entropy_similarity + frequency_similarity) / 3.0,
      length_similarity,
      entropy_similarity,
      frequency_similarity,
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
pub enum SequenceType {
  Increasing,
  Decreasing,
  Constant,
  Alternating { pattern: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct Pattern {
  pub data: Vec<u8>,
  pub occurrences: usize,
  pub positions: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct PatternSignature {
  pub length: usize,
  pub entropy: f64,
  pub byte_frequencies: Vec<(u8, f64)>,
  pub sequence_count: usize,
  pub repeating_pattern_count: usize,
  pub most_common_byte: Option<u8>,
  pub least_common_byte: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct SimilarityScore {
  pub overall: f64,
  pub length_similarity: f64,
  pub entropy_similarity: f64,
  pub frequency_similarity: f64,
}

#[derive(Debug, Clone)]
pub struct PatternGenerator {
  rng: RandomGenerator,
}

impl PatternGenerator {
  pub fn new() -> Self {
    Self {
      rng: create_random_generator(),
    }
  }

  pub fn with_seed(seed: u64) -> Self {
    Self {
      rng: create_random_generator_with_seed(seed),
    }
  }

  pub fn generate_repeating_pattern(&mut self, pattern: &[u8], repetitions: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(pattern.len() * repetitions);
    for _ in 0..repetitions {
      result.extend_from_slice(pattern);
    }
    result
  }

  pub fn generate_sequence(&mut self, sequence_type: SequenceType, length: usize) -> Vec<u8> {
    match sequence_type {
      SequenceType::Increasing => {
        let start = self.rng.gen_range(0, 256 - length as u64) as u8;
        (0..length).map(|i| start + i as u8).collect()
      }
      SequenceType::Decreasing => {
        let start = self.rng.gen_range(length as u64, 256) as u8;
        (0..length).map(|i| start - i as u8).collect()
      }
      SequenceType::Constant => {
        let value = self.rng.gen_u8();
        vec![value; length]
      }
      SequenceType::Alternating { pattern } => {
        let mut result = Vec::with_capacity(length);
        for i in 0..length {
          result.push(pattern[i % pattern.len()]);
        }
        result
      }
    }
  }

  pub fn generate_noise_pattern(&mut self, length: usize) -> Vec<u8> {
    (0..length).map(|_| self.rng.gen_u8()).collect()
  }

  pub fn generate_structured_pattern(&mut self, structure: &PatternStructure) -> Vec<u8> {
    let mut result = Vec::new();

    for segment in &structure.segments {
      match segment {
        PatternSegment::Constant { value, length } => {
          result.extend(std::iter::repeat(*value).take(*length));
        }
        PatternSegment::Random { length } => {
          result.extend((0..*length).map(|_| self.rng.gen_u8()));
        }
        PatternSegment::Sequence {
          start,
          step,
          length,
        } => {
          result.extend((0..*length).map(|i| (start + i as i8 * step) as u8));
        }
        PatternSegment::Repeating {
          pattern,
          repetitions,
        } => {
          for _ in 0..*repetitions {
            result.extend_from_slice(pattern);
          }
        }
      }
    }

    result
  }

  pub fn mutate_pattern(&mut self, pattern: &[u8], mutation_rate: f64) -> Vec<u8> {
    pattern
      .iter()
      .map(|&byte| {
        if self.rng.gen_range(0, 1000) as f64 / 1000.0 < mutation_rate {
          self.rng.gen_u8()
        } else {
          byte
        }
      })
      .collect()
  }

  pub fn cross_over(&mut self, parent1: &[u8], parent2: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let min_len = std::cmp::min(parent1.len(), parent2.len());
    let crossover_point = self.rng.gen_range(1, min_len as u64) as usize;

    let mut child1 = Vec::with_capacity(parent1.len());
    let mut child2 = Vec::with_capacity(parent2.len());

    child1.extend_from_slice(&parent1[..crossover_point]);
    child1.extend_from_slice(&parent2[crossover_point..]);

    child2.extend_from_slice(&parent2[..crossover_point]);
    child2.extend_from_slice(&parent1[crossover_point..]);

    (child1, child2)
  }
}

#[derive(Debug, Clone)]
pub struct PatternStructure {
  pub segments: Vec<PatternSegment>,
}

#[derive(Debug, Clone)]
pub enum PatternSegment {
  Constant {
    value: u8,
    length: usize,
  },
  Random {
    length: usize,
  },
  Sequence {
    start: i8,
    step: i8,
    length: usize,
  },
  Repeating {
    pattern: Vec<u8>,
    repetitions: usize,
  },
}

pub fn create_pattern_matcher(buffer: ByteBuffer) -> PatternMatcher {
  PatternMatcher::new(buffer)
}

pub fn create_pattern_generator() -> PatternGenerator {
  PatternGenerator::new()
}

pub fn create_pattern_generator_with_seed(seed: u64) -> PatternGenerator {
  PatternGenerator::with_seed(seed)
}
