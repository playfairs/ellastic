use crate::{
  ByteBuffer,
  ByteOrder,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};

#[derive(Debug, Clone)]
pub struct EndianConverter {
  buffer: ByteBuffer,
  source_endian: ByteOrder,
  target_endian: ByteOrder,
}

impl EndianConverter {
  pub fn new(buffer: ByteBuffer, source_endian: ByteOrder, target_endian: ByteOrder) -> Self {
    Self {
      buffer,
      source_endian,
      target_endian,
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

  pub fn source_endian(&self) -> ByteOrder {
    self.source_endian
  }

  pub fn target_endian(&self) -> ByteOrder {
    self.target_endian
  }

  pub fn set_source_endian(&mut self, endian: ByteOrder) {
    self.source_endian = endian;
  }

  pub fn set_target_endian(&mut self, endian: ByteOrder) {
    self.target_endian = endian;
  }

  pub fn swap_endian(&mut self) {
    std::mem::swap(&mut self.source_endian, &mut self.target_endian);
  }

  pub fn needs_conversion(&self) -> bool {
    self.source_endian != self.target_endian
  }

  pub fn convert_u16(&self, value: u16) -> u16 {
    if self.needs_conversion() {
      value.swap_bytes()
    } else {
      value
    }
  }

  pub fn convert_u32(&self, value: u32) -> u32 {
    if self.needs_conversion() {
      value.swap_bytes()
    } else {
      value
    }
  }

  pub fn convert_u64(&self, value: u64) -> u64 {
    if self.needs_conversion() {
      value.swap_bytes()
    } else {
      value
    }
  }

  pub fn convert_i16(&self, value: i16) -> i16 {
    if self.needs_conversion() {
      value.swap_bytes()
    } else {
      value
    }
  }

  pub fn convert_i32(&self, value: i32) -> i32 {
    if self.needs_conversion() {
      value.swap_bytes()
    } else {
      value
    }
  }

  pub fn convert_i64(&self, value: i64) -> i64 {
    if self.needs_conversion() {
      value.swap_bytes()
    } else {
      value
    }
  }

  pub fn convert_f32(&self, value: f32) -> f32 {
    if self.needs_conversion() {
      f32::from_bits(value.to_bits().swap_bytes())
    } else {
      value
    }
  }

  pub fn convert_f64(&self, value: f64) -> f64 {
    if self.needs_conversion() {
      f64::from_bits(value.to_bits().swap_bytes())
    } else {
      value
    }
  }

  pub fn convert_buffer(&mut self) -> Result<()> {
    if !self.needs_conversion() {
      return Ok(());
    }

    let data = self.buffer.data().to_vec();
    let mut converted = Vec::with_capacity(data.len());

    let mut i = 0;
    while i < data.len() {
      if i + 1 < data.len() {
        converted.push(data[i + 1]);
        converted.push(data[i]);
        i += 2;
      } else {
        converted.push(data[i]);
        i += 1;
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&converted);

    Ok(())
  }

  pub fn convert_buffer_u16(&mut self) -> Result<()> {
    if !self.needs_conversion() {
      return Ok(());
    }

    let data = self.buffer.data().to_vec();
    if data.len() % 2 != 0 {
      return Err(EllasticError::InvalidBinaryData {
        expected: data.len() + 1,
        actual: data.len(),
      });
    }

    let mut converted = Vec::with_capacity(data.len());
    for chunk in data.chunks(2) {
      if chunk.len() == 2 {
        converted.push(chunk[1]);
        converted.push(chunk[0]);
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&converted);

    Ok(())
  }

  pub fn convert_buffer_u32(&mut self) -> Result<()> {
    if !self.needs_conversion() {
      return Ok(());
    }

    let data = self.buffer.data().to_vec();
    if data.len() % 4 != 0 {
      return Err(EllasticError::InvalidBinaryData {
        expected: data.len() + (4 - data.len() % 4),
        actual: data.len(),
      });
    }

    let mut converted = Vec::with_capacity(data.len());
    for chunk in data.chunks(4) {
      if chunk.len() == 4 {
        converted.push(chunk[3]);
        converted.push(chunk[2]);
        converted.push(chunk[1]);
        converted.push(chunk[0]);
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&converted);

    Ok(())
  }

  pub fn convert_buffer_u64(&mut self) -> Result<()> {
    if !self.needs_conversion() {
      return Ok(());
    }

    let data = self.buffer.data().to_vec();
    if data.len() % 8 != 0 {
      return Err(EllasticError::InvalidBinaryData {
        expected: data.len() + (8 - data.len() % 8),
        actual: data.len(),
      });
    }

    let mut converted = Vec::with_capacity(data.len());
    for chunk in data.chunks(8) {
      if chunk.len() == 8 {
        converted.push(chunk[7]);
        converted.push(chunk[6]);
        converted.push(chunk[5]);
        converted.push(chunk[4]);
        converted.push(chunk[3]);
        converted.push(chunk[2]);
        converted.push(chunk[1]);
        converted.push(chunk[0]);
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&converted);

    Ok(())
  }

  pub fn convert_buffer_with_alignment(&mut self, alignment: usize) -> Result<()> {
    if !self.needs_conversion() {
      return Ok(());
    }

    let data = self.buffer.data().to_vec();
    if data.len() % alignment != 0 {
      return Err(EllasticError::InvalidBinaryData {
        expected: data.len() + (alignment - data.len() % alignment),
        actual: data.len(),
      });
    }

    let mut converted = Vec::with_capacity(data.len());
    for chunk in data.chunks(alignment) {
      for &byte in chunk.iter().rev() {
        converted.push(byte);
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&converted);

    Ok(())
  }

  pub fn convert_range(&mut self, start: usize, end: usize) -> Result<()> {
    if start > end || end > self.buffer.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.buffer.len(),
      });
    }

    if !self.needs_conversion() {
      return Ok(());
    }

    let data = self.buffer.data().to_vec();
    let range_data = &data[start..end];
    let mut converted_range = range_data.to_vec();

    for i in 0..converted_range.len() / 2 {
      let swap_index = converted_range.len() - 1 - i;
      converted_range.swap(i, swap_index);
    }

    let mut new_data = data;
    new_data.splice(start..end, converted_range);

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&new_data);

    Ok(())
  }

  pub fn detect_endian(&self) -> ByteOrder {
    let data = self.buffer.data();
    if data.len() < 4 {
      return ByteOrder::LittleEndian;
    }

    let first_u32 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    if first_u32 == 0x474E5089 {
      ByteOrder::LittleEndian
    } else if first_u32.swap_bytes() == 0x474E5089 {
      ByteOrder::BigEndian
    } else if first_u32 == 0x002A4949 {
      ByteOrder::LittleEndian
    } else if first_u32.swap_bytes() == 0x002A4949 {
      ByteOrder::BigEndian
    } else if first_u32 == 0x46445025 {
      ByteOrder::LittleEndian
    } else if first_u32.swap_bytes() == 0x46445025 {
      ByteOrder::BigEndian
    } else {
      ByteOrder::LittleEndian
    }
  }

  pub fn auto_detect_and_convert(&mut self) -> Result<ByteOrder> {
    let detected = self.detect_endian();
    self.source_endian = detected;
    self.convert_buffer()?;
    Ok(detected)
  }

  pub fn validate_endian_consistency(&self) -> bool {
    let data = self.buffer.data();
    if data.len() < 8 {
      return true;
    }

    let first_u32 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let second_u32 = if data.len() >= 8 {
      u32::from_le_bytes([data[4], data[5], data[6], data[7]])
    } else {
      0
    };

    match self.source_endian {
      ByteOrder::LittleEndian => {
        first_u32 != first_u32.swap_bytes() && second_u32 != second_u32.swap_bytes()
      }
      ByteOrder::BigEndian => {
        first_u32 == first_u32.swap_bytes() && second_u32 == second_u32.swap_bytes()
      }
    }
  }

  pub fn get_endian_mismatch_positions(&self) -> Vec<usize> {
    let data = self.buffer.data();
    let mut mismatches = Vec::new();

    if data.len() < 4 {
      return mismatches;
    }

    for i in (0..data.len() - 3).step_by(4) {
      let current = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

      if self.is_likely_little_endian_value(current) {
        if self.source_endian == ByteOrder::BigEndian {
          mismatches.push(i);
        }
      } else if self.is_likely_big_endian_value(current) {
        if self.source_endian == ByteOrder::LittleEndian {
          mismatches.push(i);
        }
      }
    }

    mismatches
  }

  fn is_likely_little_endian_value(&self, value: u32) -> bool {
    value <= 0xFFFFFF || (value & 0xFF000000) == 0
  }

  fn is_likely_big_endian_value(&self, value: u32) -> bool {
    value >= 0x01000000 || (value & 0x000000FF) == 0
  }

  pub fn create_endian_map(&self) -> EndianMap {
    let data = self.buffer.data();
    let mut map = EndianMap::new();

    for i in (0..data.len() - 3).step_by(4) {
      let current = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
      let swapped = current.swap_bytes();

      if self.is_likely_little_endian_value(current) {
        map.add_position(i, ByteOrder::LittleEndian);
      } else if self.is_likely_big_endian_value(current) {
        map.add_position(i, ByteOrder::BigEndian);
      } else {
        map.add_position(i, ByteOrder::LittleEndian);
        map.add_position(i, ByteOrder::BigEndian);
      }
    }

    map
  }

  pub fn apply_endian_map(&mut self, map: &EndianMap) -> Result<()> {
    let data = self.buffer.data().to_vec();
    let mut converted = data;

    for (position, endian) in map.positions() {
      if *endian != self.target_endian {
        let start = *position;
        let end = std::cmp::min(start + 4, converted.len());

        if end - start >= 2 {
          for i in 0..(end - start) / 2 {
            converted.swap(start + i, start + (end - start - 1 - i));
          }
        }
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&converted);

    Ok(())
  }

  pub fn mixed_endian_conversion(
    &mut self,
    conversions: &[(usize, usize, ByteOrder)],
  ) -> Result<()> {
    let data = self.buffer.data().to_vec();
    let mut converted = data;

    for (start, end, target_endian) in conversions {
      if *start > *end || *end > converted.len() {
        continue;
      }

      let range = &mut converted[*start..*end];
      if target_endian != &self.target_endian {
        range.reverse();
      }
    }

    self.buffer.data_mut().clear();
    self.buffer.data_mut().extend_from_slice(&converted);

    Ok(())
  }

  pub fn byte_swapped_copy(&self) -> ByteBuffer {
    let data = self.buffer.data();
    let mut swapped = Vec::with_capacity(data.len());

    for chunk in data.chunks(2) {
      if chunk.len() == 2 {
        swapped.push(chunk[1]);
        swapped.push(chunk[0]);
      } else {
        swapped.push(chunk[0]);
      }
    }

    ByteBuffer::from(swapped)
  }

  pub fn word_swapped_copy(&self) -> ByteBuffer {
    let data = self.buffer.data();
    let mut swapped = Vec::with_capacity(data.len());

    for chunk in data.chunks(4) {
      if chunk.len() == 4 {
        swapped.push(chunk[3]);
        swapped.push(chunk[2]);
        swapped.push(chunk[1]);
        swapped.push(chunk[0]);
      } else {
        swapped.extend_from_slice(chunk);
      }
    }

    ByteBuffer::from(swapped)
  }

  pub fn long_swapped_copy(&self) -> ByteBuffer {
    let data = self.buffer.data();
    let mut swapped = Vec::with_capacity(data.len());

    for chunk in data.chunks(8) {
      if chunk.len() == 8 {
        swapped.push(chunk[7]);
        swapped.push(chunk[6]);
        swapped.push(chunk[5]);
        swapped.push(chunk[4]);
        swapped.push(chunk[3]);
        swapped.push(chunk[2]);
        swapped.push(chunk[1]);
        swapped.push(chunk[0]);
      } else {
        swapped.extend_from_slice(chunk);
      }
    }

    ByteBuffer::from(swapped)
  }

  pub fn get_conversion_statistics(&self) -> ConversionStats {
    let data = self.buffer.data();
    let total_values = data.len() / 4;
    let mismatches = self.get_endian_mismatch_positions().len();

    ConversionStats {
      total_values,
      mismatched_values: mismatches,
      mismatch_percentage: if total_values > 0 {
        mismatches as f64 / total_values as f64 * 100.0
      } else {
        0.0
      },
      source_endian: self.source_endian,
      target_endian: self.target_endian,
      needs_conversion: self.needs_conversion(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct EndianMap {
  positions: Vec<(usize, ByteOrder)>,
}

impl EndianMap {
  pub fn new() -> Self {
    Self {
      positions: Vec::new(),
    }
  }

  pub fn add_position(&mut self, position: usize, endian: ByteOrder) {
    self.positions.push((position, endian));
  }

  pub fn positions(&self) -> &[(usize, ByteOrder)] {
    &self.positions
  }

  pub fn get_endian_at(&self, position: usize) -> Option<ByteOrder> {
    self
      .positions
      .iter()
      .find(|(pos, _)| *pos == position)
      .map(|(_, endian)| *endian)
  }

  pub fn get_positions_for_endian(&self, endian: ByteOrder) -> Vec<usize> {
    self
      .positions
      .iter()
      .filter(|(_, e)| *e == endian)
      .map(|(pos, _)| *pos)
      .collect()
  }

  pub fn merge(&mut self, other: &EndianMap) {
    self.positions.extend(other.positions.clone());
  }

  pub fn clear(&mut self) {
    self.positions.clear();
  }

  pub fn len(&self) -> usize {
    self.positions.len()
  }

  pub fn is_empty(&self) -> bool {
    self.positions.is_empty()
  }
}

impl Default for EndianMap {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct ConversionStats {
  pub total_values: usize,
  pub mismatched_values: usize,
  pub mismatch_percentage: f64,
  pub source_endian: ByteOrder,
  pub target_endian: ByteOrder,
  pub needs_conversion: bool,
}

pub fn create_endian_converter(
  buffer: ByteBuffer,
  source_endian: ByteOrder,
  target_endian: ByteOrder,
) -> EndianConverter {
  EndianConverter::new(buffer, source_endian, target_endian)
}

pub fn create_endian_map() -> EndianMap {
  EndianMap::new()
}

pub fn detect_buffer_endian(buffer: &ByteBuffer) -> ByteOrder {
  let converter = EndianConverter::new(
    buffer.clone(),
    ByteOrder::LittleEndian,
    ByteOrder::BigEndian,
  );
  converter.detect_endian()
}

pub fn convert_buffer_endian(
  buffer: ByteBuffer,
  from: ByteOrder,
  to: ByteOrder,
) -> Result<ByteBuffer> {
  let mut converter = create_endian_converter(buffer, from, to);
  converter.convert_buffer()?;
  Ok(converter.into_buffer())
}
