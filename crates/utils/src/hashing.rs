use ellastic_errors::{
  EllasticError,
  Result,
};
use std::io::Read;

#[derive(Debug, Clone)]
pub enum HashAlgorithm {
  Md5,
  Sha256,
  Sha512,
  Blake3,
}

impl HashAlgorithm {
  pub fn hash_bytes(&self, data: &[u8]) -> String {
    match self {
      HashAlgorithm::Md5 => {
        format!("{:x}", md5::compute(data))
      }
      HashAlgorithm::Sha256 => {
        use sha2::{
          Digest,
          Sha256,
        };
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
      }
      HashAlgorithm::Sha512 => {
        use sha2::{
          Digest,
          Sha512,
        };
        let mut hasher = Sha512::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
      }
      HashAlgorithm::Blake3 => {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
      }
    }
  }

  pub fn hash_reader<R: Read>(&self, reader: &mut R) -> Result<String> {
    match self {
      HashAlgorithm::Md5 => {
        let mut hasher = md5::Context::new();
        let mut buffer = [0u8; 8192];

        loop {
          let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| EllasticError::IoError(format!("Failed to read for hashing: {}", e)))?;

          if bytes_read == 0 {
            break;
          }

          hasher.consume(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.compute()))
      }
      HashAlgorithm::Sha256 => {
        use sha2::{
          Digest,
          Sha256,
        };
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
          let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| EllasticError::IoError(format!("Failed to read for hashing: {}", e)))?;

          if bytes_read == 0 {
            break;
          }

          hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
      }
      HashAlgorithm::Sha512 => {
        use sha2::{
          Digest,
          Sha512,
        };
        let mut hasher = Sha512::new();
        let mut buffer = [0u8; 8192];

        loop {
          let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| EllasticError::IoError(format!("Failed to read for hashing: {}", e)))?;

          if bytes_read == 0 {
            break;
          }

          hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
      }
      HashAlgorithm::Blake3 => {
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 8192];

        loop {
          let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| EllasticError::IoError(format!("Failed to read for hashing: {}", e)))?;

          if bytes_read == 0 {
            break;
          }

          hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().to_hex().to_string())
      }
    }
  }

  pub fn name(&self) -> &'static str {
    match self {
      HashAlgorithm::Md5 => "md5",
      HashAlgorithm::Sha256 => "sha256",
      HashAlgorithm::Sha512 => "sha512",
      HashAlgorithm::Blake3 => "blake3",
    }
  }

  pub fn output_length(&self) -> usize {
    match self {
      HashAlgorithm::Md5 => 32,
      HashAlgorithm::Sha256 => 64,
      HashAlgorithm::Sha512 => 128,
      HashAlgorithm::Blake3 => 64,
    }
  }
}

#[derive(Debug, Clone)]
pub struct FileHasher {
  algorithm: HashAlgorithm,
}

impl FileHasher {
  pub fn new(algorithm: HashAlgorithm) -> Self {
    Self { algorithm }
  }

  pub fn hash_file(&self, file_path: &std::path::Path) -> Result<String> {
    let mut file = std::fs::File::open(file_path).map_err(|e| {
      EllasticError::IoError(format!(
        "Failed to open file {}: {}",
        file_path.display(),
        e
      ))
    })?;

    self.algorithm.hash_reader(&mut file)
  }

  pub fn hash_bytes(&self, data: &[u8]) -> String {
    self.algorithm.hash_bytes(data)
  }

  pub fn verify_file(&self, file_path: &std::path::Path, expected_hash: &str) -> Result<bool> {
    let actual_hash = self.hash_file(file_path)?;
    Ok(actual_hash == expected_hash)
  }

  pub fn algorithm(&self) -> &HashAlgorithm {
    &self.algorithm
  }
}

#[derive(Debug, Clone)]
pub struct MultiHasher {
  algorithms: Vec<HashAlgorithm>,
}

impl MultiHasher {
  pub fn new() -> Self {
    Self {
      algorithms: vec![
        HashAlgorithm::Md5,
        HashAlgorithm::Sha256,
        HashAlgorithm::Blake3,
      ],
    }
  }

  pub fn with_algorithms(algorithms: Vec<HashAlgorithm>) -> Self {
    Self { algorithms }
  }

  pub fn add_algorithm(mut self, algorithm: HashAlgorithm) -> Self {
    self.algorithms.push(algorithm);
    self
  }

  pub fn hash_file(&self, file_path: &std::path::Path) -> Result<Vec<(String, String)>> {
    let mut results = Vec::new();

    for algorithm in &self.algorithms {
      let hasher = FileHasher::new(algorithm.clone());
      let hash = hasher.hash_file(file_path)?;
      results.push((algorithm.name().to_string(), hash));
    }

    Ok(results)
  }

  pub fn hash_bytes(&self, data: &[u8]) -> Vec<(String, String)> {
    let mut results = Vec::new();

    for algorithm in &self.algorithms {
      let hash = algorithm.hash_bytes(data);
      results.push((algorithm.name().to_string(), hash));
    }

    results
  }

  pub fn algorithms(&self) -> &[HashAlgorithm] {
    &self.algorithms
  }
}

impl Default for MultiHasher {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct ChecksumCalculator {
  algorithm: HashAlgorithm,
}

impl ChecksumCalculator {
  pub fn new(algorithm: HashAlgorithm) -> Self {
    Self { algorithm }
  }

  pub fn calculate_checksum(&self, data: &[u8]) -> String {
    self.algorithm.hash_bytes(data)
  }

  pub fn calculate_checksum_reader<R: Read>(&self, reader: &mut R) -> Result<String> {
    self.algorithm.hash_reader(reader)
  }

  pub fn verify_checksum(&self, data: &[u8], expected_checksum: &str) -> bool {
    let actual_checksum = self.calculate_checksum(data);
    actual_checksum == expected_checksum
  }

  pub fn verify_checksum_reader<R: Read>(
    &self,
    reader: &mut R,
    expected_checksum: &str,
  ) -> Result<bool> {
    let actual_checksum = self.calculate_checksum_reader(reader)?;
    Ok(actual_checksum == expected_checksum)
  }

  pub fn generate_checksum_file<P: AsRef<std::path::Path>>(
    &self,
    file_path: P,
    checksum_path: P,
  ) -> Result<()> {
    let file_path = file_path.as_ref();
    let checksum_path = checksum_path.as_ref();

    let checksum = self.hash_file(file_path)?;

    std::fs::write(
      checksum_path,
      format!("{}  {}", checksum, file_path.display()),
    )
    .map_err(|e| EllasticError::IoError(format!("Failed to write checksum file: {}", e)))?;

    Ok(())
  }

  pub fn verify_checksum_file<P: AsRef<std::path::Path>>(
    &self,
    file_path: P,
    checksum_path: P,
  ) -> Result<bool> {
    let file_path = file_path.as_ref();
    let checksum_path = checksum_path.as_ref();

    let checksum_content = std::fs::read_to_string(checksum_path)
      .map_err(|e| EllasticError::IoError(format!("Failed to read checksum file: {}", e)))?;

    let mut parts = checksum_content.trim().split_whitespace();
    let expected_checksum = parts
      .next()
      .ok_or_else(|| EllasticError::InvalidParameter("Empty checksum file".to_string()))?;

    let actual_checksum = self.hash_file(file_path)?;
    Ok(actual_checksum == expected_checksum)
  }

  fn hash_file(&self, file_path: &std::path::Path) -> Result<String> {
    let mut file = std::fs::File::open(file_path).map_err(|e| {
      EllasticError::IoError(format!(
        "Failed to open file {}: {}",
        file_path.display(),
        e
      ))
    })?;

    self.algorithm.hash_reader(&mut file)
  }
}

#[derive(Debug, Clone)]
pub struct RollingHash {
  window_size: usize,
  hash: u64,
  window: Vec<u8>,
  window_pos: usize,
  initialized: bool,
}

impl RollingHash {
  pub fn new(window_size: usize) -> Self {
    Self {
      window_size,
      hash: 0,
      window: vec![0; window_size],
      window_pos: 0,
      initialized: false,
    }
  }

  pub fn update(&mut self, byte: u8) -> u64 {
    if !self.initialized {
      self.window[self.window_pos] = byte;
      self.hash = self.hash.wrapping_mul(31).wrapping_add(byte as u64);
      self.window_pos = (self.window_pos + 1) % self.window_size;

      if self.window_pos == 0 {
        self.initialized = true;
      }
    } else {
      let old_byte = self.window[self.window_pos] as u64;
      let power = 31u64.pow(self.window_size as u32);

      self.hash = self
        .hash
        .wrapping_sub(old_byte.wrapping_mul(power))
        .wrapping_mul(31)
        .wrapping_add(byte as u64);

      self.window[self.window_pos] = byte;
      self.window_pos = (self.window_pos + 1) % self.window_size;
    }

    self.hash
  }

  pub fn current_hash(&self) -> u64 {
    self.hash
  }

  pub fn reset(&mut self) {
    self.hash = 0;
    self.window.fill(0);
    self.window_pos = 0;
    self.initialized = false;
  }

  pub fn is_initialized(&self) -> bool {
    self.initialized
  }
}

#[derive(Debug, Clone)]
pub struct BloomFilter {
  bit_array: Vec<bool>,
  hash_count: usize,
  size: usize,
}

impl BloomFilter {
  pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
    let size = Self::optimal_size(expected_items, false_positive_rate);
    let hash_count = Self::optimal_hash_count(size, expected_items);

    Self {
      bit_array: vec![false; size],
      hash_count,
      size,
    }
  }

  pub fn with_size_and_hashes(size: usize, hash_count: usize) -> Self {
    Self {
      bit_array: vec![false; size],
      hash_count,
      size,
    }
  }

  fn optimal_size(expected_items: usize, false_positive_rate: f64) -> usize {
    ((expected_items as f64) * (-2.0 * false_positive_rate.ln()) / (2.0_f64.ln().powi(2))) as usize
  }

  fn optimal_hash_count(size: usize, expected_items: usize) -> usize {
    ((size as f64) / (expected_items as f64) * 2.0_f64.ln()) as usize
  }

  fn hash_functions(&self, item: &[u8]) -> Vec<usize> {
    let mut hashes = Vec::with_capacity(self.hash_count);

    for i in 0..self.hash_count {
      use sha2::{
        Digest,
        Sha256,
      };
      let mut hasher = Sha256::new();
      hasher.update(item);
      hasher.update(&(i as u64).to_le_bytes());
      let hash = hasher.finalize();

      let hash_value = u64::from_le_bytes([
        hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
      ]);

      hashes.push((hash_value as usize) % self.size);
    }

    hashes
  }

  pub fn insert(&mut self, item: &[u8]) {
    for hash in self.hash_functions(item) {
      self.bit_array[hash] = true;
    }
  }

  pub fn contains(&self, item: &[u8]) -> bool {
    self
      .hash_functions(item)
      .iter()
      .all(|&hash| self.bit_array[hash])
  }

  pub fn clear(&mut self) {
    self.bit_array.fill(false);
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn hash_count(&self) -> usize {
    self.hash_count
  }

  pub fn bit_count(&self) -> usize {
    self.bit_array.iter().filter(|&&bit| bit).count()
  }

  pub fn load_factor(&self) -> f64 {
    self.bit_count() as f64 / self.size as f64
  }
}

pub fn md5_hash(data: &[u8]) -> String {
  HashAlgorithm::Md5.hash_bytes(data)
}

pub fn sha256_hash(data: &[u8]) -> String {
  HashAlgorithm::Sha256.hash_bytes(data)
}

pub fn sha512_hash(data: &[u8]) -> String {
  HashAlgorithm::Sha512.hash_bytes(data)
}

pub fn blake3_hash(data: &[u8]) -> String {
  HashAlgorithm::Blake3.hash_bytes(data)
}

pub fn hash_file<P: AsRef<std::path::Path>>(
  file_path: P,
  algorithm: HashAlgorithm,
) -> Result<String> {
  let hasher = FileHasher::new(algorithm);
  hasher.hash_file(file_path.as_ref())
}

pub fn verify_file_hash<P: AsRef<std::path::Path>>(
  file_path: P,
  expected_hash: &str,
  algorithm: HashAlgorithm,
) -> Result<bool> {
  let hasher = FileHasher::new(algorithm);
  hasher.verify_file(file_path.as_ref(), expected_hash)
}

pub fn create_rolling_hash(window_size: usize) -> RollingHash {
  RollingHash::new(window_size)
}

pub fn create_bloom_filter(expected_items: usize, false_positive_rate: f64) -> BloomFilter {
  BloomFilter::new(expected_items, false_positive_rate)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_hash_algorithms() {
    let data = b"hello world";

    let md5 = md5_hash(data);
    let sha256 = sha256_hash(data);
    let blake3 = blake3_hash(data);

    assert_eq!(md5.len(), 32);
    assert_eq!(sha256.len(), 64);
    assert_eq!(blake3.len(), 64);

    assert_ne!(md5, sha256);
    assert_ne!(sha256, blake3);
    assert_ne!(md5, blake3);
  }

  #[test]
  fn test_rolling_hash() {
    let mut rolling_hash = create_rolling_hash(4);

    let hash1 = rolling_hash.update(b'a');
    let hash2 = rolling_hash.update(b'b');
    let hash3 = rolling_hash.update(b'c');
    let hash4 = rolling_hash.update(b'd');

    assert_ne!(hash1, hash2);
    assert_ne!(hash2, hash3);
    assert_ne!(hash3, hash4);

    assert!(!rolling_hash.is_initialized());

    let hash5 = rolling_hash.update(b'e');
    assert!(rolling_hash.is_initialized());
  }

  #[test]
  fn test_bloom_filter() {
    let mut bloom = create_bloom_filter(100, 0.01);

    bloom.insert(b"hello");
    bloom.insert(b"world");

    assert!(bloom.contains(b"hello"));
    assert!(bloom.contains(b"world"));
    assert!(!bloom.contains(b"test"));
  }

  #[test]
  fn test_multi_hasher() {
    let multi_hasher = MultiHasher::new();
    let data = b"test data";

    let hashes = multi_hasher.hash_bytes(data);

    assert_eq!(hashes.len(), 3);

    let mut hash_map = std::collections::HashMap::new();
    for (algorithm, hash) in hashes {
      hash_map.insert(algorithm, hash);
    }

    assert!(hash_map.contains_key("md5"));
    assert!(hash_map.contains_key("sha256"));
    assert!(hash_map.contains_key("blake3"));
  }
}
