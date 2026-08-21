use crate::{
  ByteBuffer,
  ByteOrder,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use ellastic_utils::{
  RandomGenerator,
  create_random_generator,
};

#[derive(Debug, Clone)]
pub struct ByteManipulator {
  buffer: ByteBuffer,
}

impl ByteManipulator {
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

  pub fn insert_byte(&mut self, position: usize, byte: u8) -> Result<()> {
    self.buffer.insert_bytes(position, &[byte])
  }

  pub fn insert_bytes(&mut self, position: usize, bytes: &[u8]) -> Result<()> {
    self.buffer.insert_bytes(position, bytes)
  }

  pub fn delete_byte(&mut self, position: usize) -> Result<()> {
    self.buffer.delete_range(position, position + 1).map(|_| ())
  }

  pub fn delete_range(&mut self, start: usize, end: usize) -> Result<()> {
    self.buffer.delete_range(start, end).map(|_| ())
  }

  pub fn replace_byte(&mut self, position: usize, new_byte: u8) -> Result<()> {
    if position >= self.buffer.len() {
      return Err(EllasticError::OffsetOverflow(position));
    }
    self.buffer.data_mut()[position] = new_byte;
    Ok(())
  }

  pub fn replace_range(&mut self, start: usize, end: usize, new_bytes: &[u8]) -> Result<()> {
    self.delete_range(start, end)?;
    self.insert_bytes(start, new_bytes)?;
      Ok(())
  }

  pub fn swap_bytes(&mut self, pos1: usize, pos2: usize) -> Result<()> {
    let len = self.buffer.len();
    if pos1 >= len || pos2 >= len {
      return Err(EllasticError::OffsetOverflow(len));
    }

    let data = self.buffer.data_mut();
    data.swap(pos1, pos2);
    Ok(())
  }

  pub fn reverse_range(&mut self, start: usize, end: usize) -> Result<()> {
    if start > end || end > self.buffer.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.buffer.len(),
      });
    }

    let data = self.buffer.data_mut();
    data[start..end].reverse();
    Ok(())
  }

  pub fn duplicate_range(&mut self, start: usize, end: usize) -> Result<()> {
    self.buffer.duplicate_range(start, end)
  }

  pub fn move_range(&mut self, start: usize, end: usize, new_position: usize) -> Result<()> {
    if start > end || end > self.buffer.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.buffer.len(),
      });
    }

    if new_position > self.buffer.len() {
      return Err(EllasticError::OffsetOverflow(new_position));
    }

    let range_data = self.buffer.clone_range(start, end)?;

    if new_position < start {
      self.delete_range(start, end)?;
      self.insert_bytes(new_position, &range_data.data())?;
    } else if new_position > end {
      self.insert_bytes(new_position, &range_data.data())?;
      self.delete_range(start, end)?;
    } else {
      return Err(EllasticError::InvalidParameter(
        "Invalid move range".to_string(),
      ));
    }

    Ok(())
  }

  pub fn shuffle_range(&mut self, start: usize, end: usize) -> Result<()> {
    if start > end || end > self.buffer.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.buffer.len(),
      });
    }

    let mut rng = create_random_generator();
    let data = self.buffer.data_mut();
    let slice = &mut data[start..end];
    rng.shuffle(slice);

    Ok(())
  }

  pub fn sort_range(&mut self, start: usize, end: usize) -> Result<()> {
    if start > end || end > self.buffer.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.buffer.len(),
      });
    }

    let data = self.buffer.data_mut();
    let slice = &mut data[start..end];
    slice.sort();

    Ok(())
  }

  pub fn rotate_range(&mut self, start: usize, end: usize, amount: isize) -> Result<()> {
    if start > end || end > self.buffer.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.buffer.len(),
      });
    }

    let range_len = end - start;
    if range_len == 0 {
      return Ok(());
    }

    let data = self.buffer.data_mut();
    let slice = &mut data[start..end];

    let rotation = if amount >= 0 {
      amount as usize % range_len
    } else {
      range_len - ((-amount) as usize % range_len)
    };

    slice.rotate_left(rotation);

    Ok(())
  }

  pub fn interleave(&mut self, other: &ByteBuffer) -> Result<()> {
    let min_len = std::cmp::min(self.buffer.len(), other.len());
    let mut result = Vec::with_capacity(min_len * 2);

    for i in 0..min_len {
      result.push(self.buffer.data()[i]);
      result.push(other.data()[i]);
    }

    result.extend_from_slice(&self.buffer.data()[min_len..]);
    result.extend_from_slice(&other.data()[min_len..]);

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&result);

    Ok(())
  }

  pub fn deinterleave(&mut self, stride: usize) -> Result<Vec<ByteBuffer>> {
    if stride == 0 {
      return Err(EllasticError::InvalidParameter(
        "Stride cannot be zero".to_string(),
      ));
    }

    let len = self.buffer.len();
    let mut results = vec![ByteBuffer::new(); stride];

    for (i, &byte) in self.buffer.data().iter().enumerate() {
      results[i % stride].write_byte(byte);
    }

    Ok(results)
  }

  pub fn chunk(&mut self, chunk_size: usize) -> Result<Vec<ByteBuffer>> {
    if chunk_size == 0 {
      return Err(EllasticError::InvalidParameter(
        "Chunk size cannot be zero".to_string(),
      ));
    }

    let data = self.buffer.data().to_vec();
    let mut chunks = Vec::new();

    for chunk in data.chunks(chunk_size) {
      chunks.push(ByteBuffer::from(chunk.to_vec()));
    }

    Ok(chunks)
  }

  pub fn merge(&mut self, chunks: &[ByteBuffer]) -> Result<()> {
    let mut total_size = 0;
    for chunk in chunks {
      total_size += chunk.len();
    }

    if total_size > self.buffer.capacity() {
      let additional = total_size - self.buffer.len();
      self
        .buffer
        .data_mut()
        .reserve(additional);
    }

    self.buffer.data_mut().clear();
    for chunk in chunks {
      self.buffer.data_mut().extend_from_slice(chunk.data());
    }

    Ok(())
  }

  pub fn pad(&mut self, padding_byte: u8, target_size: usize) -> Result<()> {
    let current_size = self.buffer.len();
    if target_size <= current_size {
      return Ok(());
    }

    let padding_needed = target_size - current_size;
    let padding = vec![padding_byte; padding_needed];
    self.buffer.data_mut().extend_from_slice(&padding);

    Ok(())
  }

  pub fn trim(&mut self, trim_byte: u8) -> Result<usize> {
    let data = self.buffer.data().to_vec();
    let start = data
      .iter()
      .position(|&b| b != trim_byte)
      .unwrap_or(data.len());
    let end = data
      .iter()
      .rposition(|&b| b != trim_byte)
      .map(|pos| pos + 1)
      .unwrap_or(start);

    if start >= end {
      self.buffer.data_mut().clear();
      return Ok(data.len());
    }

    let trimmed_count = start + (data.len() - end);
    let trimmed_data = data[start..end].to_vec();

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&trimmed_data);

    Ok(trimmed_count)
  }

  pub fn repeat(&mut self, count: usize) -> Result<()> {
    if count == 0 {
      self.buffer.data_mut().clear();
      return Ok(());
    }

    let original_data = self.buffer.data().to_vec();
    let mut result = Vec::with_capacity(original_data.len() * count);

    for _ in 0..count {
      result.extend_from_slice(&original_data);
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&result);

    Ok(())
  }

  pub fn splice(&mut self, start: usize, end: usize, replacement: &[u8]) -> Result<()> {
    self.replace_range(start, end, replacement)
  }

  pub fn extract(&mut self, start: usize, end: usize) -> Result<ByteBuffer> {
    self.buffer.slice(start, end)
  }

  pub fn extract_pattern(
    &mut self,
    pattern: &[u8],
    max_matches: Option<usize>,
  ) -> Result<Vec<ByteBuffer>> {
    let mut matches = Vec::new();
    let data = self.buffer.data();
    let pattern_len = pattern.len();

    if pattern_len == 0 {
      return Err(EllasticError::InvalidParameter("Empty pattern".to_string()));
    }

    let mut pos = 0;
    let mut found = 0;

    while pos <= data.len().saturating_sub(pattern_len) {
      if &data[pos..pos + pattern_len] == pattern {
        matches.push(ByteBuffer::from(data[pos..pos + pattern_len].to_vec()));
        pos += pattern_len;
        found += 1;

        if let Some(max) = max_matches {
          if found >= max {
            break;
          }
        }
      } else {
        pos += 1;
      }
    }

    Ok(matches)
  }

  pub fn extract_sequences(&mut self, min_length: usize) -> Result<Vec<ByteBuffer>> {
    let data = self.buffer.data();
    let mut sequences = Vec::new();
    let mut current_sequence = Vec::new();

    for &byte in data {
      if current_sequence.is_empty() || byte == current_sequence[current_sequence.len() - 1] + 1 {
        current_sequence.push(byte);
      } else {
        if current_sequence.len() >= min_length {
          sequences.push(ByteBuffer::from(current_sequence.clone()));
        }
        current_sequence.clear();
        current_sequence.push(byte);
      }
    }

    if current_sequence.len() >= min_length {
      sequences.push(ByteBuffer::from(current_sequence));
    }

    Ok(sequences)
  }

  pub fn extract_unique_bytes(&mut self) -> Result<ByteBuffer> {
    let data = self.buffer.data();
    let mut seen = [false; 256];
    let mut unique_bytes = Vec::new();

    for &byte in data {
      if !seen[byte as usize] {
        seen[byte as usize] = true;
        unique_bytes.push(byte);
      }
    }

    Ok(ByteBuffer::from(unique_bytes))
  }

  pub fn extract_frequent_bytes(&mut self, min_frequency: usize) -> Result<ByteBuffer> {
    let histogram = self.buffer.get_histogram();
    let mut frequent_bytes = Vec::new();

    for (i, &count) in histogram.iter().enumerate() {
      if count >= min_frequency as u64 {
        frequent_bytes.push(i as u8);
      }
    }

    Ok(ByteBuffer::from(frequent_bytes))
  }

  pub fn compress_runs(&mut self) -> Result<()> {
    let data = self.buffer.data().to_vec();
    let mut compressed = Vec::new();

    if data.is_empty() {
      return Ok(());
    }

    let mut current_byte = data[0];
    let mut run_length = 1;

    for &byte in data.iter().skip(1) {
      if byte == current_byte && run_length < 255 {
        run_length += 1;
      } else {
        compressed.push(current_byte);
        compressed.push(run_length as u8);
        current_byte = byte;
        run_length = 1;
      }
    }

    compressed.push(current_byte);
    compressed.push(run_length as u8);

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&compressed);

    Ok(())
  }

  pub fn decompress_runs(&mut self) -> Result<()> {
    let data = self.buffer.data().to_vec();
    let mut decompressed = Vec::new();

    if data.len() % 2 != 0 {
      return Err(EllasticError::InvalidParameter(
        "Invalid run-length encoding".to_string(),
      ));
    }

    for chunk in data.chunks(2) {
      let byte = chunk[0];
      let count = chunk[1];
      decompressed.extend(std::iter::repeat(byte).take(count as usize));
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&decompressed);

    Ok(())
  }

  pub fn apply_bitmask(&mut self, mask: u8) -> Result<()> {
    for byte in self.buffer.data_mut() {
      *byte &= mask;
    }
    Ok(())
  }

  pub fn apply_bitmask_range(&mut self, start: usize, end: usize, mask: u8) -> Result<()> {
    if start > end || end > self.buffer.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.buffer.len(),
      });
    }

    let data = self.buffer.data_mut();
    for byte in &mut data[start..end] {
      *byte &= mask;
    }

    Ok(())
  }

  pub fn set_bits(&mut self, mask: u8) -> Result<()> {
    for byte in self.buffer.data_mut() {
      *byte |= mask;
    }
    Ok(())
  }

  pub fn clear_bits(&mut self, mask: u8) -> Result<()> {
    for byte in self.buffer.data_mut() {
      *byte &= !mask;
    }
    Ok(())
  }

  pub fn toggle_bits(&mut self, mask: u8) -> Result<()> {
    for byte in self.buffer.data_mut() {
      *byte ^= mask;
    }
    Ok(())
  }

  pub fn count_set_bits(&self) -> usize {
    self
      .buffer
      .data()
      .iter()
      .map(|byte| byte.count_ones() as usize)
      .sum()
  }

  pub fn count_clear_bits(&self) -> usize {
    self
      .buffer
      .data()
      .iter()
      .map(|byte| byte.count_zeros() as usize)
      .sum()
  }

  pub fn bit_density(&self) -> f64 {
    let total_bits = self.buffer.len() * 8;
    if total_bits == 0 {
      return 0.0;
    }
    self.count_set_bits() as f64 / total_bits as f64
  }
}

impl From<ByteBuffer> for ByteManipulator {
  fn from(buffer: ByteBuffer) -> Self {
    Self::new(buffer)
  }
}

#[derive(Debug, Clone)]
pub struct ChunkProcessor {
  chunk_size: usize,
  overlap: usize,
}

impl ChunkProcessor {
  pub fn new(chunk_size: usize, overlap: usize) -> Self {
    Self {
      chunk_size,
      overlap,
    }
  }

  pub fn process_chunks<F>(&self, buffer: &ByteBuffer, mut processor: F) -> Result<Vec<ByteBuffer>>
  where
    F: FnMut(&ByteBuffer) -> Result<ByteBuffer>,
  {
    let data = buffer.data();
    let mut chunks = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
      let end = std::cmp::min(pos + self.chunk_size, data.len());
      let chunk = ByteBuffer::from(data[pos..end].to_vec());
      let processed = processor(&chunk)?;
      chunks.push(processed);

      if end >= data.len() {
        break;
      }

      pos = end - self.overlap;
    }

    Ok(chunks)
  }

  pub fn merge_chunks(&self, chunks: &[ByteBuffer]) -> Result<ByteBuffer> {
    if chunks.is_empty() {
      return Ok(ByteBuffer::new());
    }

    let mut result = ByteBuffer::new();
    result.write_bytes(chunks[0].data());

    for chunk in chunks.iter().skip(1) {
      let chunk_data = chunk.data();
      let overlap_start = std::cmp::max(0, chunk_data.len() - self.overlap);
      result.write_bytes(&chunk_data[overlap_start..]);
    }

    Ok(result)
  }
}

pub fn create_manipulator(buffer: ByteBuffer) -> ByteManipulator {
  ByteManipulator::new(buffer)
}

pub fn create_chunk_processor(chunk_size: usize, overlap: usize) -> ChunkProcessor {
  ChunkProcessor::new(chunk_size, overlap)
}
