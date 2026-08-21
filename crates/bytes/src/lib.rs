use byteorder::{
  BigEndian,
  LittleEndian,
  ReadBytesExt,
  WriteBytesExt,
};
pub use ellastic_errors;
use ellastic_errors::{
  EllasticError,
  Result,
};
use std::io::{
  Cursor,
  Read,
  Seek,
  SeekFrom,
  Write,
};

pub mod analysis;
pub mod corruption;
pub mod endian;
pub mod manipulation;
pub mod patterns;

pub use analysis::*;
pub use corruption::*;
pub use endian::*;
pub use manipulation::*;
pub use patterns::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
  LittleEndian,
  BigEndian,
}

impl ByteOrder {
  pub fn read_u16(&self, bytes: &[u8]) -> u16 {
    match self {
      ByteOrder::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
      ByteOrder::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
    }
  }

  pub fn read_u32(&self, bytes: &[u8]) -> u32 {
    match self {
      ByteOrder::LittleEndian => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
      ByteOrder::BigEndian => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
    }
  }

  pub fn read_u64(&self, bytes: &[u8]) -> u64 {
    match self {
      ByteOrder::LittleEndian => u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
      ]),
      ByteOrder::BigEndian => u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
      ]),
    }
  }

  pub fn write_u16(&self, buffer: &mut [u8], value: u16) {
    let bytes = match self {
      ByteOrder::LittleEndian => value.to_le_bytes(),
      ByteOrder::BigEndian => value.to_be_bytes(),
    };
    buffer[..2].copy_from_slice(&bytes);
  }

  pub fn write_u32(&self, buffer: &mut [u8], value: u32) {
    let bytes = match self {
      ByteOrder::LittleEndian => value.to_le_bytes(),
      ByteOrder::BigEndian => value.to_be_bytes(),
    };
    buffer[..4].copy_from_slice(&bytes);
  }

  pub fn write_u64(&self, buffer: &mut [u8], value: u64) {
    let bytes = match self {
      ByteOrder::LittleEndian => value.to_le_bytes(),
      ByteOrder::BigEndian => value.to_be_bytes(),
    };
    buffer[..8].copy_from_slice(&bytes);
  }

  pub fn flip(&self) -> ByteOrder {
    match self {
      ByteOrder::LittleEndian => ByteOrder::BigEndian,
      ByteOrder::BigEndian => ByteOrder::LittleEndian,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ByteBuffer {
  data: Vec<u8>,
  position: usize,
  endian: ByteOrder,
}

impl ByteBuffer {
  pub fn new() -> Self {
    Self {
      data: Vec::new(),
      position: 0,
      endian: ByteOrder::LittleEndian,
    }
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      data: Vec::with_capacity(capacity),
      position: 0,
      endian: ByteOrder::LittleEndian,
    }
  }

  pub fn from_bytes(bytes: Vec<u8>) -> Self {
    Self {
      data: bytes,
      position: 0,
      endian: ByteOrder::LittleEndian,
    }
  }

  pub fn with_endian(mut self, endian: ByteOrder) -> Self {
    self.endian = endian;
    self
  }

  pub fn set_endian(&mut self, endian: ByteOrder) {
    self.endian = endian;
  }

  pub fn endian(&self) -> ByteOrder {
    self.endian
  }

  pub fn data(&self) -> &[u8] {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut Vec<u8> {
    &mut self.data
  }

  pub fn into_data(self) -> Vec<u8> {
    self.data
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn capacity(&self) -> usize {
    self.data.capacity()
  }

  pub fn position(&self) -> usize {
    self.position
  }

  pub fn set_position(&mut self, position: usize) {
    self.position = position.min(self.data.len());
  }

  pub fn seek(&mut self, offset: isize) -> Result<()> {
    let new_position = if offset >= 0 {
      self.position.checked_add(offset as usize)
    } else {
      self.position.checked_sub((-offset) as usize)
    };

    if let Some(new_pos) = new_position {
      if new_pos <= self.data.len() {
        self.position = new_pos;
        Ok(())
      } else {
        Err(EllasticError::OffsetOverflow(new_pos))
      }
    } else {
      Err(EllasticError::OffsetOverflow(self.position))
    }
  }

  pub fn read_byte(&mut self) -> Result<u8> {
    if self.position >= self.data.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: self.position + 1,
        actual: self.data.len(),
      });
    }

    let byte = self.data[self.position];
    self.position += 1;
    Ok(byte)
  }

  pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
    if self.position + count > self.data.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: self.position + count,
        actual: self.data.len(),
      });
    }

    let bytes = self.data[self.position..self.position + count].to_vec();
    self.position += count;
      Ok(bytes)
  }

  pub fn read_u16(&mut self) -> Result<u16> {
    let bytes = self.read_bytes(2)?;
    Ok(self.endian.read_u16(&bytes))
  }

  pub fn read_u32(&mut self) -> Result<u32> {
    let bytes = self.read_bytes(4)?;
    Ok(self.endian.read_u32(&bytes))
  }

  pub fn read_u64(&mut self) -> Result<u64> {
    let bytes = self.read_bytes(8)?;
    Ok(self.endian.read_u64(&bytes))
  }

  pub fn read_i16(&mut self) -> Result<i16> {
    Ok(self.read_u16()? as i16)
  }

  pub fn read_i32(&mut self) -> Result<i32> {
    Ok(self.read_u32()? as i32)
  }

  pub fn read_i64(&mut self) -> Result<i64> {
    Ok(self.read_u64()? as i64)
  }

  pub fn read_f32(&mut self) -> Result<f32> {
    Ok(f32::from_bits(self.read_u32()?))
  }

  pub fn read_f64(&mut self) -> Result<f64> {
    Ok(f64::from_bits(self.read_u64()?))
  }

  pub fn read_string(&mut self, length: usize) -> Result<String> {
    let bytes = self.read_bytes(length)?;
    String::from_utf8(bytes)
      .map_err(|e| EllasticError::DeserializationError(format!("Invalid UTF-8: {}", e)))
  }

  pub fn read_null_terminated_string(&mut self) -> Result<String> {
    let mut bytes = Vec::new();

    loop {
      let byte = self.read_byte()?;
      if byte == 0 {
        break;
      }
      bytes.push(byte);
    }

    String::from_utf8(bytes)
      .map_err(|e| EllasticError::DeserializationError(format!("Invalid UTF-8: {}", e)))
  }

  pub fn write_byte(&mut self, byte: u8) {
    if self.position >= self.data.len() {
      self.data.push(byte);
    } else {
      self.data[self.position] = byte;
    }
    self.position += 1;
  }

  pub fn write_bytes(&mut self, bytes: &[u8]) {
    let end_pos = self.position + bytes.len();

    if end_pos > self.data.len() {
      self.data.resize(end_pos, 0);
    }

    self.data[self.position..end_pos].copy_from_slice(bytes);
    self.position = end_pos;
    }

  pub fn write_u16(&mut self, value: u16) {
    let mut bytes = [0u8; 2];
    self.endian.write_u16(&mut bytes, value);
    self.write_bytes(&bytes);
  }

  pub fn write_u32(&mut self, value: u32) {
    let mut bytes = [0u8; 4];
    self.endian.write_u32(&mut bytes, value);
    self.write_bytes(&bytes);
  }

  pub fn write_u64(&mut self, value: u64) {
    let mut bytes = [0u8; 8];
    self.endian.write_u64(&mut bytes, value);
    self.write_bytes(&bytes);
  }

  pub fn write_i16(&mut self, value: i16) {
    self.write_u16(value as u16);
  }

  pub fn write_i32(&mut self, value: i32) {
    self.write_u32(value as u32);
  }

  pub fn write_i64(&mut self, value: i64) {
    self.write_u64(value as u64);
  }

  pub fn write_f32(&mut self, value: f32) {
    self.write_u32(value.to_bits());
  }

  pub fn write_f64(&mut self, value: f64) {
    self.write_u64(value.to_bits());
  }

  pub fn write_string(&mut self, string: &str) {
    self.write_bytes(string.as_bytes());
  }

  pub fn write_null_terminated_string(&mut self, string: &str) {
    self.write_bytes(string.as_bytes());
    self.write_byte(0);
  }

  pub fn peek_byte(&self) -> Result<u8> {
    if self.position >= self.data.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: self.position + 1,
        actual: self.data.len(),
      });
    }
    Ok(self.data[self.position])
  }

  pub fn peek_bytes(&self, count: usize) -> Result<&[u8]> {
    if self.position + count > self.data.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: self.position + count,
        actual: self.data.len(),
      });
    }
    Ok(&self.data[self.position..self.position + count])
  }

  pub fn bytes_remaining(&self) -> usize {
    self.data.len().saturating_sub(self.position)
  }

  pub fn at_end(&self) -> bool {
    self.position >= self.data.len()
  }

  pub fn reset(&mut self) {
    self.position = 0;
  }

  pub fn clear(&mut self) {
    self.data.clear();
    self.position = 0;
  }

  pub fn resize(&mut self, new_size: usize) {
    self.data.resize(new_size, 0);
    if self.position > new_size {
      self.position = new_size;
    }
  }

  pub fn reserve(&mut self, additional: usize) {
    self.data.reserve(additional);
  }

  pub fn shrink_to_fit(&mut self) {
    self.data.shrink_to_fit();
  }

  pub fn truncate(&mut self, len: usize) {
    self.data.truncate(len);
    if self.position > len {
      self.position = len;
    }
  }

  pub fn append(&mut self, other: &ByteBuffer) {
    self.data.extend_from_slice(&other.data);
  }

  pub fn split_at(&mut self, position: usize) -> ByteBuffer {
    let split_data = self.data.split_off(position);
    ByteBuffer {
      data: split_data,
      position: 0,
      endian: self.endian,
    }
  }

  pub fn slice(&self, start: usize, end: usize) -> Result<ByteBuffer> {
    if start > end || end > self.data.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.data.len(),
      });
    }

    Ok(ByteBuffer {
      data: self.data[start..end].to_vec(),
      position: 0,
      endian: self.endian,
    })
  }

  pub fn find_byte(&self, byte: u8, start: usize) -> Option<usize> {
    self.data[start..]
      .iter()
      .position(|&b| b == byte)
      .map(|pos| pos + start)
  }

  pub fn find_bytes(&self, pattern: &[u8], start: usize) -> Option<usize> {
    if pattern.is_empty() {
      return Some(start);
    }

    for i in start..=(self.data.len().saturating_sub(pattern.len())) {
      if &self.data[i..i + pattern.len()] == pattern {
        return Some(i);
      }
    }
    None
  }

  pub fn replace_byte(&mut self, old: u8, new: u8) -> usize {
    let mut count = 0;
    for byte in &mut self.data {
      if *byte == old {
        *byte = new;
        count += 1;
      }
    }
    count
  }

  pub fn replace_bytes(&mut self, old: &[u8], new: &[u8]) -> Result<usize> {
    if old.is_empty() {
      return Err(EllasticError::InvalidParameter(
        "Empty pattern for replacement".to_string(),
      ));
    }

    let mut count = 0;
    let mut i = 0;

    while i <= self.data.len().saturating_sub(old.len()) {
      if &self.data[i..i + old.len()] == old {
        self.data.splice(i..i + old.len(), new.iter().cloned());
        i += new.len();
        count += 1;
      } else {
        i += 1;
      }
    }

    Ok(count)
  }

  pub fn xor(&mut self, key: u8) {
    for byte in &mut self.data {
      *byte ^= key;
    }
  }

  pub fn xor_with_key(&mut self, key: &[u8]) {
    if key.is_empty() {
      return;
    }

    for (i, byte) in self.data.iter_mut().enumerate() {
      *byte ^= key[i % key.len()];
    }
  }

  pub fn add(&mut self, value: u8) {
    for byte in &mut self.data {
      *byte = byte.wrapping_add(value);
    }
  }

  pub fn subtract(&mut self, value: u8) {
    for byte in &mut self.data {
      *byte = byte.wrapping_sub(value);
    }
  }

  pub fn multiply(&mut self, value: u8) {
    for byte in &mut self.data {
      *byte = byte.wrapping_mul(value);
    }
  }

  pub fn divide(&mut self, value: u8) {
    if value == 0 {
      return;
    }

    for byte in &mut self.data {
      *byte = byte.wrapping_div(value);
    }
  }

  pub fn shift_left(&mut self, bits: u8) {
    for byte in &mut self.data {
      *byte <<= bits;
    }
  }

  pub fn shift_right(&mut self, bits: u8) {
    for byte in &mut self.data {
      *byte >>= bits;
    }
  }

  pub fn rotate_left(&mut self, bits: u8) {
    for byte in &mut self.data {
      *byte = byte.rotate_left(bits as u32);
    }
  }

  pub fn rotate_right(&mut self, bits: u8) {
    for byte in &mut self.data {
      *byte = byte.rotate_right(bits as u32);
    }
  }

  pub fn reverse(&mut self) {
    self.data.reverse();
  }

  pub fn reverse_bytes(&mut self) {
    for byte in &mut self.data {
      *byte = byte.reverse_bits();
    }
  }

  pub fn complement(&mut self) {
    for byte in &mut self.data {
      *byte = !*byte;
    }
  }

  pub fn swap_endian(&mut self) {
    self.endian = self.endian.flip();
  }

  pub fn copy_from(&mut self, other: &ByteBuffer) {
    self.data.clear();
    self.data.extend_from_slice(&other.data);
    self.position = 0;
  }

  pub fn clone_range(&self, start: usize, length: usize) -> Result<ByteBuffer> {
    self.slice(start, start + length)
  }

  pub fn insert_bytes(&mut self, position: usize, bytes: &[u8]) -> Result<()> {
    if position > self.data.len() {
      return Err(EllasticError::OffsetOverflow(position));
    }

    self.data.splice(position..position, bytes.iter().cloned());
    if self.position >= position {
      self.position += bytes.len();
    }
    Ok(())
  }

  pub fn delete_range(&mut self, start: usize, end: usize) -> Result<usize> {
    if start > end || end > self.data.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.data.len(),
      });
    }

    let deleted_count = end - start;
    self.data.splice(start..end, std::iter::empty());

    if self.position >= start {
      if self.position >= end {
        self.position -= deleted_count;
      } else {
        self.position = start;
      }
    }

    Ok(deleted_count)
  }

  pub fn duplicate_range(&mut self, start: usize, end: usize) -> Result<()> {
    let range_data = self.slice(start, end)?;
    self.insert_bytes(end, &range_data.data)?;
    Ok(())
  }

  pub fn get_entropy(&self) -> f64 {
    if self.data.is_empty() {
      return 0.0;
    }

    let mut frequency = [0u64; 256];
    for &byte in &self.data {
      frequency[byte as usize] += 1;
    }

    let len = self.data.len() as f64;
    let mut entropy = 0.0;

    for &count in &frequency {
      if count > 0 {
        let probability = count as f64 / len;
        entropy -= probability * probability.log2();
      }
    }

    entropy
  }

  pub fn get_histogram(&self) -> [u64; 256] {
    let mut histogram = [0u64; 256];
    for &byte in &self.data {
      histogram[byte as usize] += 1;
    }
    histogram
  }

  pub fn get_checksum(&self) -> u8 {
    self
      .data
      .iter()
      .fold(0u8, |acc, &byte| acc.wrapping_add(byte))
  }

  pub fn get_xor_checksum(&self) -> u8 {
    self.data.iter().fold(0u8, |acc, &byte| acc ^ byte)
  }

  pub fn verify_checksum(&self, expected: u8) -> bool {
    self.get_checksum() == expected
  }

  pub fn verify_xor_checksum(&self, expected: u8) -> bool {
    self.get_xor_checksum() == expected
  }
}

impl Default for ByteBuffer {
  fn default() -> Self {
    Self::new()
  }
}

impl Read for ByteBuffer {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    let remaining = self.bytes_remaining();
    let to_read = std::cmp::min(buf.len(), remaining);

    if to_read > 0 {
      buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
      self.position += to_read;
    }

    Ok(to_read)
  }
}

impl Write for ByteBuffer {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.write_bytes(buf);
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

impl Seek for ByteBuffer {
  fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
    let new_position = match pos {
      SeekFrom::Start(offset) => offset as usize,
      SeekFrom::End(offset) => {
        if offset >= 0 {
          self.data.len().checked_add(offset as usize)
        } else {
          self.data.len().checked_sub((-offset) as usize)
        }
      }.ok_or(std::io::Error::new(std::io::ErrorKind::InvalidInput, "seek position overflow"))?,
      SeekFrom::Current(offset) => {
        if offset >= 0 {
          self.position.checked_add(offset as usize)
        } else {
          self.position.checked_sub((-offset) as usize)
        }
      }.ok_or(std::io::Error::new(std::io::ErrorKind::InvalidInput, "seek position overflow"))?,
    };

    if new_position <= self.data.len() {
      self.position = new_position;
      Ok(self.position as u64)
    } else {
      Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Seek position out of bounds",
      ))
    }
  }
}

impl From<Vec<u8>> for ByteBuffer {
  fn from(data: Vec<u8>) -> Self {
    ByteBuffer::from_bytes(data)
  }
}

impl From<&[u8]> for ByteBuffer {
  fn from(data: &[u8]) -> Self {
    ByteBuffer::from_bytes(data.to_vec())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_byte_buffer_basic() {
    let mut buffer = ByteBuffer::new();

    buffer.write_u32(0x12345678);
    buffer.write_u16(0x1234);
    buffer.write_byte(0x56);

    buffer.reset();

    assert_eq!(buffer.read_u32().unwrap(), 0x12345678);
    assert_eq!(buffer.read_u16().unwrap(), 0x1234);
    assert_eq!(buffer.read_byte().unwrap(), 0x56);
  }

  #[test]
  fn test_endian_conversion() {
    let little = ByteOrder::LittleEndian;
    let big = ByteOrder::BigEndian;

    let mut bytes = [0u8; 4];
    little.write_u32(&mut bytes, 0x12345678);

    assert_eq!(little.read_u32(&bytes), 0x12345678);
    assert_eq!(big.read_u32(&bytes), 0x78563412);
  }

  #[test]
  fn test_byte_operations() {
    let mut buffer = ByteBuffer::from(vec![1, 2, 3, 4]);

    buffer.xor(0xFF);
    assert_eq!(buffer.data(), &[254, 253, 252, 251]);

    buffer.add(1);
    assert_eq!(buffer.data(), &[255, 254, 253, 252]);

    buffer.reverse();
    assert_eq!(buffer.data(), &[252, 253, 254, 255]);
  }

  #[test]
  fn test_find_and_replace() {
    let mut buffer = ByteBuffer::from(vec![1, 2, 3, 2, 3, 4]);

    assert_eq!(buffer.find_byte(3, 0), Some(2));
    assert_eq!(buffer.find_byte(3, 3), Some(4));

    let pattern = [2, 3];
    assert_eq!(buffer.find_bytes(&pattern, 0), Some(1));

    buffer.replace_byte(3, 9);
    assert_eq!(buffer.data(), &[1, 2, 9, 2, 9, 4]);
  }

  #[test]
  fn test_entropy() {
    let buffer1 = ByteBuffer::from(vec![0; 100]);
    let buffer2 = ByteBuffer::from((0..=255).cycle().take(100).collect::<Vec<_>>());

    assert_eq!(buffer1.get_entropy(), 0.0);
    assert!(buffer2.get_entropy() > 7.0);
  }

  #[test]
  fn test_io_traits() {
    let mut buffer = ByteBuffer::new();

    use std::io::{
      Read,
      Seek,
      SeekFrom,
      Write,
    };

    buffer.write_all(&[1, 2, 3, 4]).unwrap();
    buffer.seek(SeekFrom::Start(2)).unwrap();

    let mut read_buf = [0u8; 2];
    buffer.read_exact(&mut read_buf).unwrap();

    assert_eq!(read_buf, [3, 4]);
  }
}
