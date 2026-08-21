use ellastic_errors::{Result, EllasticError};
use std::io::{Read, Write, Cursor};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DecompressionEngine {
    algorithm: DecompressionAlgorithm,
    buffer_size: usize,
    verify_checksum: bool,
}

impl DecompressionEngine {
    pub fn new(algorithm: DecompressionAlgorithm) -> Self {
        Self {
            algorithm,
            buffer_size: 8192,
            verify_checksum: false,
        }
    }

    pub fn algorithm(&self) -> DecompressionAlgorithm {
        self.algorithm
    }

    pub fn set_algorithm(&mut self, algorithm: DecompressionAlgorithm) {
        self.algorithm = algorithm;
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn set_buffer_size(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size.max(1024);
    }

    pub fn verify_checksum(&self) -> bool {
        self.verify_checksum
    }

    pub fn set_verify_checksum(&mut self, verify_checksum: bool) {
        self.verify_checksum = verify_checksum;
    }

    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.algorithm {
            DecompressionAlgorithm::Deflate => self.decompress_deflate(data),
            DecompressionAlgorithm::Gzip => self.decompress_gzip(data),
            DecompressionAlgorithm::Brotli => self.decompress_brotli(data),
            DecompressionAlgorithm::LZ4 => self.decompress_lz4(data),
            DecompressionAlgorithm::Zstd => self.decompress_zstd(data),
            DecompressionAlgorithm::LZMA => self.decompress_lzma(data),
            DecompressionAlgorithm::XZ => self.decompress_xz(data),
            DecompressionAlgorithm::Custom(ref name) => self.decompress_custom(data, name),
        }
    }

    pub fn decompress_stream(&self, reader: &mut dyn Read, writer: &mut dyn Write) -> Result<usize> {
        let mut buffer = vec![0u8; self.buffer_size];
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

    pub fn decompress_with_progress<F>(&mut self, data: &[u8], progress_callback: F) -> Result<Vec<u8>>
    where
        F: Fn(usize, usize),
    {
        let total_size = data.len();
        let chunk_size = self.buffer_size.min(total_size);
        let mut result = Vec::new();
        let mut decompressor = self.create_stream_decompressor()?;

        for chunk in data.chunks(chunk_size) {
            let decompressed_chunk = decompressor.decompress(chunk)?;
            result.extend_from_slice(&decompressed_chunk);
            progress_callback(result.len(), total_size);
        }

        let final_chunk = decompressor.finish()?;
        result.extend_from_slice(&final_chunk);
        progress_callback(result.len(), total_size);

        Ok(result)
    }

    pub fn validate_compressed_data(&self, data: &[u8]) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        if data.is_empty() {
            issues.push("Compressed data is empty".to_string());
            return Ok(ValidationResult {
                is_valid: false,
                issues,
                warnings,
            });
        }

        let estimated_size = self.estimate_decompressed_size(data);
        if estimated_size == 0 {
            issues.push("Unable to estimate decompressed size".to_string());
        }

        if self.verify_checksum {
            if let Err(checksum_issue) = self.verify_data_checksum(data) {
                issues.push(checksum_issue);
            }
        }

        let is_valid = issues.is_empty();
        Ok(ValidationResult {
            is_valid,
            issues,
            warnings,
        })
    }

    pub fn estimate_decompressed_size(&self, data: &[u8]) -> usize {
        match self.algorithm {
            DecompressionAlgorithm::Deflate => self.estimate_deflate_size(data),
            DecompressionAlgorithm::Gzip => self.estimate_gzip_size(data),
            DecompressionAlgorithm::Brotli => self.estimate_brotli_size(data),
            DecompressionAlgorithm::LZ4 => self.estimate_lz4_size(data),
            DecompressionAlgorithm::Zstd => self.estimate_zstd_size(data),
            DecompressionAlgorithm::LZMA => self.estimate_lzma_size(data),
            DecompressionAlgorithm::XZ => self.estimate_xz_size(data),
            DecompressionAlgorithm::Custom(_) => data.len(),
        }
    }

    pub fn get_decompression_info(&self) -> DecompressionInfo {
        DecompressionInfo {
            algorithm: self.algorithm,
            supports_streaming: self.algorithm.supports_streaming(),
            supports_multiple_files: self.algorithm.supports_multiple_files(),
            supports_checksums: self.algorithm.supports_checksums(),
            supports_dictionary: self.algorithm.supports_dictionary(),
            max_chunk_size: self.algorithm.max_chunk_size(),
            default_buffer_size: 8192,
            memory_usage: self.algorithm.estimated_memory_usage(),
        }
    }

    fn decompress_deflate(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;
        
        let mut decoder = ZlibDecoder::new(Cursor::new(data))?;
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)
            .map_err(|e| EllasticError::DecompressionError(format!("Deflate decompression failed: {}", e)))?;
        Ok(result)
    }

    fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        
        let mut decoder = GzDecoder::new(Cursor::new(data))?;
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)
            .map_err(|e| EllasticError::DecompressionError(format!("Gzip decompression failed: {}", e)))?;
        Ok(result)
    }

    fn decompress_brotli(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decompress_lz4(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decompress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decompress_lzma(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decompress_xz(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decompress_custom(&self, data: &[u8], name: &str) -> Result<Vec<u8>> {
        Err(EllasticError::UnsupportedOperation(format!("Custom decompression algorithm {} not implemented", name)))
    }

    fn create_stream_decompressor(&self) -> Result<Box<dyn StreamDecompressor>> {
        match self.algorithm {
            DecompressionAlgorithm::Deflate => Ok(Box::new(DeflateDecompressor::new())),
            DecompressionAlgorithm::Gzip => Ok(Box::new(GzipDecompressor::new())),
            _ => Err(EllasticError::UnsupportedOperation("Streaming decompression not supported for this algorithm".to_string())),
        }
    }

    fn estimate_deflate_size(&self, data: &[u8]) -> usize {
        if data.len() < 6 {
            return 0;
        }

        let mut size = 0;
        let mut pos = 0;
        
        while pos < data.len() {
            if pos + 6 > data.len() {
                break;
            }

            let header = &data[pos..pos + 6];
            let final_bit = (header[0] & 0x01) != 0;
            let type_bits = (header[0] >> 1) & 0x03;
            
            if type_bits == 0x03 {
                let len = ((header[2] as u32) << 8) | (header[3] as u32);
                pos += 4 + len + 4;
                size += len as usize;
            } else if type_bits == 0x01 {
                let len = ((header[2] as u32) << 8) | (header[3] as u32);
                pos += 4 + len + 4;
                size += len as usize;
            } else {
                pos += 1;
            }

            if final_bit {
                break;
            }
        }

        size
    }

    fn estimate_gzip_size(&self, data: &[u8]) -> usize {
        if data.len() < 10 {
            return 0;
        }

        let mut size = 0;
        let mut pos = 10;
        
        while pos < data.len() {
            if pos + 8 > data.len() {
                break;
            }

            let compressed_len = ((data[pos + 4] as u32) << 24) |
                                 ((data[pos + 5] as u32) << 16) |
                                 ((data[pos + 6] as u32) << 8) |
                                 (data[pos + 7] as u32);
            
            pos += 8 + compressed_len as usize;
            size += compressed_len as usize;
        }

        let uncompressed_size = ((data[data.len() - 4] as u32) << 24) |
                              ((data[data.len() - 3] as u32) << 16) |
                              ((data[data.len() - 2] as u32) << 8) |
                              (data[data.len() - 1] as u32);

        uncompressed_size as usize
    }

    fn estimate_brotli_size(&self, data: &[u8]) -> usize {
        data.len() * 3
    }

    fn estimate_lz4_size(&self, data: &[u8]) -> usize {
        data.len() * 2
    }

    fn estimate_zstd_size(&self, data: &[u8]) -> usize {
        data.len() * 2
    }

    fn estimate_lzma_size(&self, data: &[u8]) -> usize {
        data.len() * 2
    }

    fn estimate_xz_size(&self, data: &[u8]) -> usize {
        data.len() * 2
    }

    fn verify_data_checksum(&self, data: &[u8]) -> Result<String> {
        match self.algorithm {
            DecompressionAlgorithm::Gzip => self.verify_gzip_checksum(data),
            _ => Ok("Checksum verification not supported".to_string()),
        }
    }

    fn verify_gzip_checksum(&self, data: &[u8]) -> Result<String> {
        if data.len() < 8 {
            return Ok("Data too short for checksum verification".to_string());
        }

        let stored_crc = ((data[data.len() - 8] as u32) << 24) |
                        ((data[data.len() - 7] as u32) << 16) |
                        ((data[data.len() - 6] as u32) << 8) |
                        (data[data.len() - 5] as u32);

        let mut crc = 0xFFFFFFFFu32;
        let compressed_data = &data[..data.len() - 8];
        
        for &byte in compressed_data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }

        crc ^= 0xFFFFFFFF;

        if crc != stored_crc {
            return Ok("CRC32 checksum mismatch".to_string());
        }

        Ok("Checksum verified".to_string())
    }

    pub fn clone(&self) -> DecompressionEngine {
        Self {
            algorithm: self.algorithm,
            buffer_size: self.buffer_size,
            verify_checksum: self.verify_checksum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecompressionAlgorithm {
    Deflate,
    Gzip,
    Brotli,
    LZ4,
    Zstd,
    LZMA,
    XZ,
    Custom(String),
}

impl DecompressionAlgorithm {
    pub fn supports_streaming(&self) -> bool {
        match self {
            DecompressionAlgorithm::Deflate => true,
            DecompressionAlgorithm::Gzip => true,
            DecompressionAlgorithm::Brotli => true,
            DecompressionAlgorithm::LZ4 => true,
            DecompressionAlgorithm::Zstd => true,
            DecompressionAlgorithm::LZMA => false,
            DecompressionAlgorithm::XZ => false,
            DecompressionAlgorithm::Custom(_) => false,
        }
    }

    pub fn supports_multiple_files(&self) -> bool {
        match self {
            DecompressionAlgorithm::Gzip => true,
            DecompressionAlgorithm::Brotli => true,
            DecompressionAlgorithm::LZMA => true,
            DecompressionAlgorithm::XZ => true,
            _ => false,
        }
    }

    pub fn supports_checksums(&self) -> bool {
        match self {
            DecompressionAlgorithm::Gzip => true,
            DecompressionAlgorithm::Brotli => true,
            _ => false,
        }
    }

    pub fn supports_dictionary(&self) -> bool {
        match self {
            DecompressionAlgorithm::Deflate => false,
            DecompressionAlgorithm::Gzip => false,
            DecompressionAlgorithm::Brotli => true,
            DecompressionAlgorithm::LZ4 => true,
            DecompressionAlgorithm::Zstd => true,
            DecompressionAlgorithm::LZMA => true,
            DecompressionAlgorithm::XZ => true,
            DecompressionAlgorithm::Custom(_) => false,
        }
    }

    pub fn max_chunk_size(&self) -> usize {
        match self {
            DecompressionAlgorithm::Deflate => 65535,
            DecompressionAlgorithm::Gzip => 65535,
            DecompressionAlgorithm::Brotli => 16777215,
            DecompressionAlgorithm::LZ4 => 2147483647,
            DecompressionAlgorithm::Zstd => 2147483647,
            DecompressionAlgorithm::LZMA => 2147483647,
            DecompressionAlgorithm::XZ => 2147483647,
            DecompressionAlgorithm::Custom(_) => 65535,
        }
    }

    pub fn estimated_memory_usage(&self) -> usize {
        match self {
            DecompressionAlgorithm::Deflate => 1024 * 1024,
            DecompressionAlgorithm::Gzip => 1024 * 1024,
            DecompressionAlgorithm::Brotli => 2 * 1024 * 1024,
            DecompressionAlgorithm::LZ4 => 512 * 1024,
            DecompressionAlgorithm::Zstd => 2 * 1024 * 1024,
            DecompressionAlgorithm::LZMA => 4 * 1024 * 1024,
            DecompressionAlgorithm::XZ => 4 * 1024 * 1024,
            DecompressionAlgorithm::Custom(_) => 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecompressionInfo {
    pub algorithm: DecompressionAlgorithm,
    pub supports_streaming: bool,
    pub supports_multiple_files: bool,
    pub supports_checksums: bool,
    pub supports_dictionary: bool,
    pub max_chunk_size: usize,
    pub default_buffer_size: usize,
    pub memory_usage: usize,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

pub trait StreamDecompressor {
    fn decompress(&mut self, data: &[u8]) -> Result<Vec<u8>>;
    fn finish(&mut self) -> Result<Vec<u8>>;
    fn reset(&mut self);
}

struct DeflateDecompressor {
    decoder: Option<flate2::read::ZlibDecoder<std::io::Cursor<Vec<u8>>>>,
}

impl DeflateDecompressor {
    fn new() -> Self {
        Self {
            decoder: None,
        }
    }
}

impl StreamDecompressor for DeflateDecompressor {
    fn decompress(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if self.decoder.is_none() {
            self.decoder = Some(flate2::read::ZlibDecoder::new(std::io::Cursor::new(Vec::new()))?);
        }
        
        self.decoder.as_mut().unwrap().read_to_end(&mut Vec::new())
            .map_err(|e| EllasticError::DecompressionError(format!("Deflate decompression failed: {}", e)))
    }

    fn finish(&mut self) -> Result<Vec<u8>> {
        if let Some(decoder) = self.decoder.take() {
            decoder.finish().into_inner()
                .map_err(|e| EllasticError::DecompressionError(format!("Deflate decompression failed: {}", e)))
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
        Self {
            decoder: None,
        }
    }
}

impl StreamDecompressor for GzipDecompressor {
    fn decompress(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if self.decoder.is_none() {
            self.decoder = Some(flate2::read::GzDecoder::new(std::io::Cursor::new(Vec::new()))?);
        }
        
        self.decoder.as_mut().unwrap().read_to_end(&mut Vec::new())
            .map_err(|e| EllasticError::DecompressionError(format!("Gzip decompression failed: {}", e)))
    }

    fn finish(&mut self) -> Result<Vec<u8>> {
        if let Some(decoder) = self.decoder.take() {
            decoder.finish().into_inner()
                .map_err(|e| EllasticError::DecompressionError(format!("Gzip decompression failed: {}", e)))
        } else {
            Ok(Vec::new())
        }
    }

    fn reset(&mut self) {
        self.decoder = None;
    }
}

#[derive(Debug, Clone)]
pub struct DecompressionBenchmark {
    algorithms: Vec<DecompressionAlgorithm>,
    test_data: Vec<u8>,
}

impl DecompressionBenchmark {
    pub fn new(test_data: Vec<u8>) -> Self {
        Self {
            algorithms: vec![
                DecompressionAlgorithm::Deflate,
                DecompressionAlgorithm::Gzip,
                DecompressionAlgorithm::LZ4,
                DecompressionAlgorithm::Zstd,
            ],
            test_data,
        }
    }

    pub fn add_algorithm(&mut self, algorithm: DecompressionAlgorithm) {
        self.algorithms.push(algorithm);
    }

    pub fn run_benchmark(&self) -> Result<Vec<BenchmarkResult>> {
        let mut results = Vec::new();
        
        for algorithm in &self.algorithms {
            let mut total_time = std::time::Duration::from_millis(0);
            let mut total_decompressed_size = 0;
            let mut total_original_size = 0;
            
            let engine = DecompressionEngine::new(algorithm.clone());
            let compressed_data = self.create_compressed_data(algorithm)?;
            
            let start_time = std::time::Instant::now();
            let decompressed = engine.decompress(&compressed_data)?;
            let decompression_time = start_time.elapsed();
            
            total_time += decompression_time;
            total_decompressed_size += decompressed.len();
            total_original_size += compressed_data.len();
            
            let compression_ratio = if total_original_size > 0 {
                total_original_size as f32 / total_decompressed_size as f32
            } else {
                1.0
            };
            
            results.push(BenchmarkResult {
                algorithm: algorithm.clone(),
                avg_decompression_time: decompression_time,
                compression_ratio,
                min_compression_ratio: compression_ratio,
                max_compression_ratio: compression_ratio,
            });
        }
        
        Ok(results)
    }

    pub fn find_fastest_algorithm(&self) -> Result<Option<&DecompressionAlgorithm>> {
        let results = self.run_benchmark()?;
        
        if results.is_empty() {
            return Ok(None);
        }
        
        let fastest = results.iter()
            .min_by(|a, b| {
                let score_a = a.compression_ratio / (a.avg_decompression_time.as_millis() as f32 + 1.0);
                let score_b = b.compression_ratio / (b.avg_decompression_time.as_millis() as f32 + 1.0);
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
        
        Ok(fastest.map(|result| &result.algorithm))
    }

    fn create_compressed_data(&self, algorithm: &DecompressionAlgorithm) -> Result<Vec<u8>> {
        match algorithm {
            DecompressionAlgorithm::Deflate => self.create_deflate_data(),
            DecompressionAlgorithm::Gzip => self.create_gzip_data(),
            _ => Ok(self.test_data.clone()),
        }
    }

    fn create_deflate_data(&self) -> Result<Vec<u8>> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6))?;
        encoder.write_all(&self.test_data)?;
        encoder.finish()
            .map_err(|e| EllasticError::CompressionError(format!("Deflate compression failed: {}", e)))
    }

    fn create_gzip_data(&self) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(6))?;
        encoder.write_all(&self.test_data)?;
        encoder.finish()
            .map_err(|e| EllasticError::CompressionError(format!("Gzip compression failed: {}", e)))
    }

    pub fn clone(&self) -> DecompressionBenchmark {
        Self {
            algorithms: self.algorithms.clone(),
            test_data: self.test_data.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub algorithm: DecompressionAlgorithm,
    pub avg_decompression_time: std::time::Duration,
    pub compression_ratio: f32,
    pub min_compression_ratio: f32,
    pub max_compression_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct DecompressionCache {
    cache: HashMap<String, CacheEntry>,
    max_size: usize,
    max_age: std::time::Duration,
}

impl DecompressionCache {
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
        
        self.cache.insert(key, CacheEntry {
            data,
            created: std::time::SystemTime::now(),
            age: std::time::Duration::from_secs(0),
        });
    }

    pub fn cleanup(&mut self) {
        let now = std::time::SystemTime::now();
        
        self.cache.retain(|_, entry| {
            entry.age = now.duration_since(entry.created).unwrap_or_default();
            entry.age <= self.max_age
        });
    }

    fn find_oldest_key(&self) -> Option<String> {
        self.cache.iter()
            .min_by_key(|_, entry)| entry.created)
            .map(|(key, _)| key.clone())
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    created: std::time::SystemTime,
    age: std::time::Duration,
}

pub fn create_decompression_engine(algorithm: DecompressionAlgorithm) -> DecompressionEngine {
    DecompressionEngine::new(algorithm)
}

pub fn create_decompression_benchmark(test_data: Vec<u8>) -> DecompressionBenchmark {
    DecompressionBenchmark::new(test_data)
}

pub fn create_decompression_cache(max_size: usize, max_age: std::time::Duration) -> DecompressionCache {
    DecompressionCache::new(max_size, max_age)
}
