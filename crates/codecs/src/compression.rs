use ellastic_errors::{
  EllasticError,
  Result,
};
use std::collections::HashMap;
use std::io::{
  Cursor,
  Read,
  Write,
};

#[derive(Debug, Clone)]
pub struct CompressionEngine {
  algorithm: CompressionAlgorithm,
  level: u8,
  dictionary: Option<Vec<u8>>,
  chunk_size: usize,
}

impl CompressionEngine {
  pub fn new(algorithm: CompressionAlgorithm, level: u8) -> Self {
    Self {
      algorithm,
      level,
      dictionary: None,
      chunk_size: 4096,
    }
  }

  pub fn algorithm(&self) -> CompressionAlgorithm {
    self.algorithm
  }

  pub fn set_algorithm(&mut self, algorithm: CompressionAlgorithm) {
    self.algorithm = algorithm;
  }

  pub fn level(&self) -> u8 {
    self.level
  }

  pub fn set_level(&mut self, level: u8) {
    self.level = level.clamp(0, 9);
  }

  pub fn dictionary(&self) -> Option<&[u8]> {
    self.dictionary.as_deref()
  }

  pub fn set_dictionary(&mut self, dictionary: Vec<u8>) {
    self.dictionary = Some(dictionary);
  }

  pub fn chunk_size(&self) -> usize {
    self.chunk_size
  }

  pub fn set_chunk_size(&mut self, chunk_size: usize) {
    self.chunk_size = chunk_size.max(1);
  }

  pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
    match self.algorithm {
      CompressionAlgorithm::Deflate => self.compress_deflate(data),
      CompressionAlgorithm::Gzip => self.compress_gzip(data),
      CompressionAlgorithm::Brotli => self.compress_brotli(data),
      CompressionAlgorithm::LZ4 => self.compress_lz4(data),
      CompressionAlgorithm::Zstd => self.compress_zstd(data),
      CompressionAlgorithm::LZMA => self.compress_lzma(data),
      CompressionAlgorithm::XZ => self.compress_xz(data),
      CompressionAlgorithm::Custom(ref name) => self.compress_custom(data, name),
    }
  }

  pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
    match self.algorithm {
      CompressionAlgorithm::Deflate => self.decompress_deflate(data),
      CompressionAlgorithm::Gzip => self.decompress_gzip(data),
      CompressionAlgorithm::Brotli => self.decompress_brotli(data),
      CompressionAlgorithm::LZ4 => self.decompress_lz4(data),
      CompressionAlgorithm::Zstd => self.decompress_zstd(data),
      CompressionAlgorithm::LZMA => self.decompress_lzma(data),
      CompressionAlgorithm::XZ => self.decompress_xz(data),
      CompressionAlgorithm::Custom(ref name) => self.decompress_custom(data, name),
    }
  }

  pub fn compress_stream(&self, reader: &mut dyn Read, writer: &mut dyn Write) -> Result<usize> {
    let mut buffer = vec![0u8; self.chunk_size];
    let mut total_compressed = 0;
    let mut compressor = self.create_stream_compressor()?;

    loop {
      let bytes_read = reader.read(&mut buffer)?;
      if bytes_read == 0 {
        break;
      }

      let compressed_chunk = compressor.compress(&buffer[..bytes_read])?;
      writer.write_all(&compressed_chunk)?;
      total_compressed += compressed_chunk.len();
    }

    let final_chunk = compressor.finish()?;
    writer.write_all(&final_chunk)?;
    total_compressed += final_chunk.len();

    Ok(total_compressed)
  }

  pub fn decompress_stream(&self, reader: &mut dyn Read, writer: &mut dyn Write) -> Result<usize> {
    let mut buffer = vec![0u8; self.chunk_size];
    let mut total_decompressed = 0;
    let mut decompressor = self.create_stream_decompressor()?;

    loop {
      let bytes_read = reader.read(&mut buffer)?;
      if bytes_read == 0 {
        break;
      }

      let decompressed_chunk = decompressor.decompress(&buffer[..bytes_read])?;
      writer.write_all(&decompressed_chunk)?;
      total_decompressed += decompressed_chunk.len();
    }

    let final_chunk = decompressor.finish()?;
    writer.write_all(&final_chunk)?;
    total_decompressed += final_chunk.len();

    Ok(total_decompressed)
  }

  pub fn estimate_compression_ratio(&self, data: &[u8]) -> f32 {
    let sample_size = data.len().min(1024);
    let sample = &data[..sample_size];

    match self.compress(sample) {
      Ok(compressed) => sample_size as f32 / compressed.len() as f32,
      Err(_) => 1.0,
    }
  }

  pub fn get_compression_info(&self) -> CompressionInfo {
    CompressionInfo {
      algorithm: self.algorithm,
      level: self.level,
      supports_streaming: true,
      supports_dictionary: self.algorithm.supports_dictionary(),
      supports_multiple_files: self.algorithm.supports_multiple_files(),
      max_chunk_size: self.algorithm.max_chunk_size(),
      default_level: self.algorithm.default_level(),
      min_level: self.algorithm.min_level(),
      max_level: self.algorithm.max_level(),
    }
  }

  fn compress_deflate(&self, data: &[u8]) -> Result<Vec<u8>> {
    use flate2::Compression;
    use flate2::write::ZlibEncoder;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(self.level as u32))?;
    encoder.write_all(data)?;
    encoder
      .finish()
      .map_err(|e| EllasticError::CompressionError(format!("Deflate compression failed: {}", e)))
  }

  fn decompress_deflate(&self, data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;

    let mut decoder = ZlibDecoder::new(Cursor::new(data))?;
    let mut result = Vec::new();
    decoder.read_to_end(&mut result).map_err(|e| {
      EllasticError::CompressionError(format!("Deflate decompression failed: {}", e))
    })?;
    Ok(result)
  }

  fn compress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.level as u32))?;
    encoder.write_all(data)?;
    encoder
      .finish()
      .map_err(|e| EllasticError::CompressionError(format!("Gzip compression failed: {}", e)))
  }

  fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;

    let mut decoder = GzDecoder::new(Cursor::new(data))?;
    let mut result = Vec::new();
    decoder
      .read_to_end(&mut result)
      .map_err(|e| EllasticError::CompressionError(format!("Gzip decompression failed: {}", e)))?;
    Ok(result)
  }

  fn compress_brotli(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decompress_brotli(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn compress_lz4(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decompress_lz4(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn compress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decompress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn compress_lzma(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decompress_lzma(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn compress_xz(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn decompress_xz(&self, data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
  }

  fn compress_custom(&self, data: &[u8], name: &str) -> Result<Vec<u8>> {
    Err(EllasticError::UnsupportedOperation(format!(
      "Custom compression algorithm {} not implemented",
      name
    )))
  }

  fn decompress_custom(&self, data: &[u8], name: &str) -> Result<Vec<u8>> {
    Err(EllasticError::UnsupportedOperation(format!(
      "Custom decompression algorithm {} not implemented",
      name
    )))
  }

  fn create_stream_compressor(&self) -> Result<Box<dyn StreamCompressor>> {
    match self.algorithm {
      CompressionAlgorithm::Deflate => Ok(Box::new(DeflateCompressor::new(self.level))),
      CompressionAlgorithm::Gzip => Ok(Box::new(GzipCompressor::new(self.level))),
      _ => Err(EllasticError::UnsupportedOperation(
        "Streaming compression not supported for this algorithm".to_string(),
      )),
    }
  }

  fn create_stream_decompressor(&self) -> Result<Box<dyn StreamDecompressor>> {
    match self.algorithm {
      CompressionAlgorithm::Deflate => Ok(Box::new(DeflateDecompressor::new())),
      CompressionAlgorithm::Gzip => Ok(Box::new(GzipDecompressor::new())),
      _ => Err(EllasticError::UnsupportedOperation(
        "Streaming decompression not supported for this algorithm".to_string(),
      )),
    }
  }

  pub fn clone(&self) -> CompressionEngine {
    Self {
      algorithm: self.algorithm,
      level: self.level,
      dictionary: self.dictionary.clone(),
      chunk_size: self.chunk_size,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionAlgorithm {
  Deflate,
  Gzip,
  Brotli,
  LZ4,
  Zstd,
  LZMA,
  XZ,
  Custom(String),
}

impl CompressionAlgorithm {
  pub fn supports_dictionary(&self) -> bool {
    match self {
      CompressionAlgorithm::Deflate => false,
      CompressionAlgorithm::Gzip => false,
      CompressionAlgorithm::Brotli => true,
      CompressionAlgorithm::LZ4 => true,
      CompressionAlgorithm::Zstd => true,
      CompressionAlgorithm::LZMA => true,
      CompressionAlgorithm::XZ => true,
      CompressionAlgorithm::Custom(_) => false,
    }
  }

  pub fn supports_multiple_files(&self) -> bool {
    match self {
      CompressionAlgorithm::Gzip => true,
      CompressionAlgorithm::Brotli => true,
      CompressionAlgorithm::Zstd => true,
      CompressionAlgorithm::LZMA => true,
      CompressionAlgorithm::XZ => true,
      _ => false,
    }
  }

  pub fn max_chunk_size(&self) -> usize {
    match self {
      CompressionAlgorithm::Deflate => 65535,
      CompressionAlgorithm::Gzip => 65535,
      CompressionAlgorithm::Brotli => 16777215,
      CompressionAlgorithm::LZ4 => 2147483647,
      CompressionAlgorithm::Zstd => 2147483647,
      CompressionAlgorithm::LZMA => 2147483647,
      CompressionAlgorithm::XZ => 2147483647,
      CompressionAlgorithm::Custom(_) => 65535,
    }
  }

  pub fn default_level(&self) -> u8 {
    match self {
      CompressionAlgorithm::Deflate => 6,
      CompressionAlgorithm::Gzip => 6,
      CompressionAlgorithm::Brotli => 4,
      CompressionAlgorithm::LZ4 => 1,
      CompressionAlgorithm::Zstd => 3,
      CompressionAlgorithm::LZMA => 5,
      CompressionAlgorithm::XZ => 5,
      CompressionAlgorithm::Custom(_) => 5,
    }
  }

  pub fn min_level(&self) -> u8 {
    match self {
      CompressionAlgorithm::Deflate => 0,
      CompressionAlgorithm::Gzip => 0,
      CompressionAlgorithm::Brotli => 0,
      CompressionAlgorithm::LZ4 => 1,
      CompressionAlgorithm::Zstd => 1,
      CompressionAlgorithm::LZMA => 0,
      CompressionAlgorithm::XZ => 0,
      CompressionAlgorithm::Custom(_) => 0,
    }
  }

  pub fn max_level(&self) -> u8 {
    match self {
      CompressionAlgorithm::Deflate => 9,
      CompressionAlgorithm::Gzip => 9,
      CompressionAlgorithm::Brotli => 11,
      CompressionAlgorithm::LZ4 => 16,
      CompressionAlgorithm::Zstd => 22,
      CompressionAlgorithm::LZMA => 9,
      CompressionAlgorithm::XZ => 9,
      CompressionAlgorithm::Custom(_) => 9,
    }
  }
}

#[derive(Debug, Clone)]
pub struct CompressionInfo {
  pub algorithm: CompressionAlgorithm,
  pub level: u8,
  pub supports_streaming: bool,
  pub supports_dictionary: bool,
  pub supports_multiple_files: bool,
  pub max_chunk_size: usize,
  pub default_level: u8,
  pub min_level: u8,
  pub max_level: u8,
}

pub trait StreamCompressor {
  fn compress(&mut self, data: &[u8]) -> Result<Vec<u8>>;
  fn finish(&mut self) -> Result<Vec<u8>>;
  fn reset(&mut self);
}

pub trait StreamDecompressor {
  fn decompress(&mut self, data: &[u8]) -> Result<Vec<u8>>;
  fn finish(&mut self) -> Result<Vec<u8>>;
  fn reset(&mut self);
}

struct DeflateCompressor {
  level: u8,
  encoder: Option<flate2::write::ZlibEncoder<Vec<u8>>>,
}

impl DeflateCompressor {
  fn new(level: u8) -> Self {
    Self {
      level,
      encoder: None,
    }
  }
}

impl StreamCompressor for DeflateCompressor {
  fn compress(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    if self.encoder.is_none() {
      self.encoder = Some(flate2::write::ZlibEncoder::new(
        Vec::new(),
        flate2::Compression::new(self.level as u32),
      )?);
    }

    self.encoder.as_mut().unwrap().write_all(data)?;
    Ok(Vec::new())
  }

  fn finish(&mut self) -> Result<Vec<u8>> {
    if let Some(encoder) = self.encoder.take() {
      encoder
        .finish()
        .map_err(|e| EllasticError::CompressionError(format!("Deflate compression failed: {}", e)))
    } else {
      Ok(Vec::new())
    }
  }

  fn reset(&mut self) {
    self.encoder = None;
  }
}

struct GzipCompressor {
  level: u8,
  encoder: Option<flate2::write::GzEncoder<Vec<u8>>>,
}

impl GzipCompressor {
  fn new(level: u8) -> Self {
    Self {
      level,
      encoder: None,
    }
  }
}

impl StreamCompressor for GzipCompressor {
  fn compress(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    if self.encoder.is_none() {
      self.encoder = Some(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::new(self.level as u32),
      )?);
    }

    self.encoder.as_mut().unwrap().write_all(data)?;
    Ok(Vec::new())
  }

  fn finish(&mut self) -> Result<Vec<u8>> {
    if let Some(encoder) = self.encoder.take() {
      encoder
        .finish()
        .map_err(|e| EllasticError::CompressionError(format!("Gzip compression failed: {}", e)))
    } else {
      Ok(Vec::new())
    }
  }

  fn reset(&mut self) {
    self.encoder = None;
  }
}

struct DeflateDecompressor {
  decoder: Option<flate2::read::ZlibDecoder<std::io::Cursor<Vec<u8>>>>,
}

impl DeflateDecompressor {
  fn new() -> Self {
    Self { decoder: None }
  }
}

impl StreamDecompressor for DeflateDecompressor {
  fn decompress(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    if self.decoder.is_none() {
      self.decoder = Some(flate2::read::ZlibDecoder::new(std::io::Cursor::new(
        Vec::new(),
      ))?);
    }

    let cursor = std::io::Cursor::new(data.to_vec());
    self
      .decoder
      .as_mut()
      .unwrap()
      .read_to_end(&mut Vec::new())
      .map_err(|e| {
        EllasticError::CompressionError(format!("Deflate decompression failed: {}", e))
      })?;
    Ok(Vec::new())
  }

  fn finish(&mut self) -> Result<Vec<u8>> {
    if let Some(decoder) = self.decoder.take() {
      let mut result = Vec::new();
      decoder.read_to_end(&mut result).map_err(|e| {
        EllasticError::CompressionError(format!("Deflate decompression failed: {}", e))
      })?;
      Ok(result)
    } else {
      Ok(Vec::new())
    }
  }

  fn reset(&mut self) {
    self.decoder = None;
  }
}

struct GzipDecompressor {
  decoder: Option<flate2::read::GzDecoder<std::io::Cursor<Vec<u8>>>>,
}

impl GzipDecompressor {
  fn new() -> Self {
    Self { decoder: None }
  }
}

impl StreamDecompressor for GzipDecompressor {
  fn decompress(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    if self.decoder.is_none() {
      self.decoder = Some(flate2::read::GzDecoder::new(std::io::Cursor::new(
        Vec::new(),
      ))?);
    }

    let cursor = std::io::Cursor::new(data.to_vec());
    self
      .decoder
      .as_mut()
      .unwrap()
      .read_to_end(&mut Vec::new())
      .map_err(|e| EllasticError::CompressionError(format!("Gzip decompression failed: {}", e)))?;
    Ok(Vec::new())
  }

  fn finish(&mut self) -> Result<Vec<u8>> {
    if let Some(decoder) = self.decoder.take() {
      let mut result = Vec::new();
      decoder.read_to_end(&mut result).map_err(|e| {
        EllasticError::CompressionError(format!("Gzip decompression failed: {}", e))
      })?;
      Ok(result)
    } else {
      Ok(Vec::new())
    }
  }

  fn reset(&mut self) {
    self.decoder = None;
  }
}

#[derive(Debug, Clone)]
pub struct CompressionBenchmark {
  algorithms: Vec<CompressionAlgorithm>,
  test_data: Vec<u8>,
}

impl CompressionBenchmark {
  pub fn new(test_data: Vec<u8>) -> Self {
    Self {
      algorithms: vec![
        CompressionAlgorithm::Deflate,
        CompressionAlgorithm::Gzip,
        CompressionAlgorithm::Brotli,
        CompressionAlgorithm::LZ4,
        CompressionAlgorithm::Zstd,
      ],
      test_data,
    }
  }

  pub fn add_algorithm(&mut self, algorithm: CompressionAlgorithm) {
    self.algorithms.push(algorithm);
  }

  pub fn run_benchmark(&self) -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    for algorithm in &self.algorithms {
      let mut total_time = std::time::Duration::from_millis(0);
      let mut total_compressed_size = 0;
      let mut total_original_size = 0;

      for level in algorithm.min_level()..=algorithm.max_level() {
        let engine = CompressionEngine::new(algorithm.clone(), level);

        let start_time = std::time::Instant::now();
        let compressed = engine.compress(&self.test_data)?;
        let compression_time = start_time.elapsed();

        total_time += compression_time;
        total_compressed_size += compressed.len();
        total_original_size += self.test_data.len();
      }

      let avg_compression_ratio = if total_original_size > 0 {
        total_original_size as f32 / total_compressed_size as f32
      } else {
        1.0
      };

      results.push(BenchmarkResult {
        algorithm: algorithm.clone(),
        avg_compression_time: total_time
          / (algorithm.max_level() - algorithm.min_level() + 1) as u32,
        avg_compression_ratio,
        min_compression_ratio: avg_compression_ratio,
        max_compression_ratio: avg_compression_ratio,
      });
    }

    Ok(results)
  }

  pub fn find_best_algorithm(&self) -> Result<Option<&CompressionAlgorithm>> {
    let results = self.run_benchmark()?;

    if results.is_empty() {
      return Ok(None);
    }

    let best = results.iter().min_by(|a, b| {
      let score_a = a.avg_compression_ratio / (a.avg_compression_time.as_millis() as f32 + 1.0);
      let score_b = b.avg_compression_ratio / (b.avg_compression_time.as_millis() as f32 + 1.0);
      score_b
        .partial_cmp(&score_a)
        .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(best.map(|result| &result.algorithm))
  }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
  pub algorithm: CompressionAlgorithm,
  pub avg_compression_time: std::time::Duration,
  pub avg_compression_ratio: f32,
  pub min_compression_ratio: f32,
  pub max_compression_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct CompressionCache {
  cache: HashMap<String, CacheEntry>,
  max_size: usize,
  max_age: std::time::Duration,
}

impl CompressionCache {
  pub fn new(max_size: usize, max_age: std::time::Duration) -> Self {
    Self {
      cache: HashMap::new(),
      max_size,
      max_age,
    }
  }

  pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
    if let Some(entry) = self.cache.get(key) {
      if entry.age > self.max_age {
        self.cache.remove(key);
        return None;
      }
      Some(entry.data.clone())
    } else {
      None
    }
  }

  pub fn put(&mut self, key: String, data: Vec<u8>) {
    if self.cache.len() >= self.max_size {
      if let Some(oldest_key) = self.find_oldest_key() {
        self.cache.remove(&oldest_key);
      }
    }

    self.cache.insert(
      key,
      CacheEntry {
        data,
        created: std::time::SystemTime::now(),
        age: std::time::Duration::from_secs(0),
      },
    );
  }

  pub fn cleanup(&mut self) {
    let now = std::time::SystemTime::now();

    self.cache.retain(|_, entry| {
      entry.age = now.duration_since(entry.created).unwrap_or_default();
      entry.age <= self.max_age
    });
  }

  fn find_oldest_key(&self) -> Option<String> {
    self
      .cache
      .iter()
      .min_by_key(|(_, entry)| entry.created)
      .map(|(key, _)| key.clone())
  }
}

#[derive(Debug, Clone)]
struct CacheEntry {
  data: Vec<u8>,
  created: std::time::SystemTime,
  age: std::time::Duration,
}

pub fn create_compression_engine(algorithm: CompressionAlgorithm, level: u8) -> CompressionEngine {
  CompressionEngine::new(algorithm, level)
}

pub fn create_compression_benchmark(test_data: Vec<u8>) -> CompressionBenchmark {
  CompressionBenchmark::new(test_data)
}

pub fn create_compression_cache(max_size: usize, max_age: std::time::Duration) -> CompressionCache {
  CompressionCache::new(max_size, max_age)
}
