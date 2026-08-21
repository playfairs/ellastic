use crate::graph::{
  EdgeType,
  PipelineEdge,
  PipelineGraph,
  PipelineNode,
  PipelineNodeType,
};
use chrono::{
  DateTime,
  Utc,
};
use ellastic_audio::{
  AudioData,
  AudioProcessor,
};
use ellastic_core::{
  MediaData,
  MediaType,
};
use ellastic_effects::{
  EffectProcessor,
  EffectType,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_glitch::{
  GlitchEffect,
  GlitchProcessor,
};
use ellastic_image::{
  ImageData,
  ImageProcessor,
};
use ellastic_media::MediaProcessor;
use ellastic_utils::create_random_generator;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::{
  HashMap,
  HashSet,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PipelineCache {
  node_cache: Arc<RwLock<NodeCache>>,
  graph_cache: Arc<RwLock<GraphCache>>,
  cache_config: CacheConfig,
  cache_stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
  pub enable_node_caching: bool,
  pub enable_graph_caching: bool,
  pub max_cache_size_mb: usize,
  pub cache_ttl_seconds: u64,
  pub enable_persistent_cache: bool,
  pub cache_eviction_policy: CacheEvictionPolicy,
  pub compression_enabled: bool,
  pub serialization_format: CacheSerializationFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheEvictionPolicy {
  LRU,
  LFU,
  FIFO,
  Random,
  TTL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheSerializationFormat {
  Binary,
  JSON,
  MessagePack,
}

#[derive(Debug, Clone)]
pub struct NodeCache {
  cache: HashMap<String, CacheEntry>,
  max_size: usize,
  max_memory_mb: usize,
  current_memory_mb: usize,
  eviction_policy: CacheEvictionPolicy,
  access_order: Vec<String>,
  access_counts: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
  pub id: Uuid,
  pub key: String,
  pub media_processor: MediaProcessor,
  pub timestamp: DateTime<Utc>,
  pub access_count: u64,
  pub last_access: DateTime<Utc>,
  pub size_bytes: usize,
  pub ttl_seconds: Option<u64>,
  pub metadata: HashMap<String, String>,
  pub compressed: bool,
}

#[derive(Debug, Clone)]
pub struct GraphCache {
  cache: HashMap<String, GraphCacheEntry>,
  max_size: usize,
  max_memory_mb: usize,
  current_memory_mb: usize,
  eviction_policy: CacheEvictionPolicy,
  access_order: Vec<String>,
  access_counts: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct GraphCacheEntry {
  pub id: Uuid,
  pub key: String,
  pub graph: PipelineGraph,
  pub timestamp: DateTime<Utc>,
  pub access_count: u64,
  pub last_access: DateTime<Utc>,
  pub size_bytes: usize,
  pub ttl_seconds: Option<u64>,
  pub metadata: HashMap<String, String>,
  pub compressed: bool,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
  pub node_cache_stats: NodeCacheStats,
  pub graph_cache_stats: GraphCacheStats,
  pub total_hits: u64,
  pub total_misses: u64,
  pub hit_rate: f64,
  pub memory_usage_mb: f64,
  pub eviction_count: u64,
  pub compression_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct NodeCacheStats {
  pub entries: usize,
  pub hits: u64,
  pub misses: u64,
  pub hit_rate: f64,
  pub memory_usage_mb: f64,
  pub evictions: u64,
  pub average_access_time: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct GraphCacheStats {
  pub entries: usize,
  pub hits: u64,
  pub misses: u64,
  pub hit_rate: f64,
  pub memory_usage_mb: f64,
  pub evictions: u64,
  pub average_access_time: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct CacheManager {
  caches: HashMap<String, PipelineCache>,
  default_cache_name: String,
  global_stats: Arc<RwLock<CacheStats>>,
}

impl PipelineCache {
  pub fn new(config: CacheConfig) -> Self {
    Self {
      node_cache: Arc::new(RwLock::new(NodeCache::new(&config))),
      graph_cache: Arc::new(RwLock::new(GraphCache::new(&config))),
      cache_config: config,
      cache_stats: Arc::new(RwLock::new(CacheStats::new())),
    }
  }

  pub fn config(&self) -> &CacheConfig {
    &self.cache_config
  }

  pub fn node_cache(&self) -> Arc<RwLock<NodeCache>> {
    self.node_cache.clone()
  }

  pub fn graph_cache(&self) -> Arc<RwLock<GraphCache>> {
    self.graph_cache.clone()
  }

  pub fn cache_stats(&self) -> Arc<RwLock<CacheStats>> {
    self.cache_stats.clone()
  }

  pub fn get_node(&self, key: &str) -> Option<MediaProcessor> {
    if !self.cache_config.enable_node_caching {
      return None;
    }

    let mut node_cache = self.node_cache.write();
    let start_time = std::time::Instant::now();

    if let Some(entry) = node_cache.get(key) {
      if let Some(ttl_seconds) = entry.ttl_seconds {
        let elapsed = Utc::now().signed_duration_since(entry.timestamp);
        if elapsed.num_seconds() > ttl_seconds as i64 {
          node_cache.remove(key);
          self.cache_stats.write().record_miss();
          return None;
        }
      }

      let access_time = start_time.elapsed();
      self.cache_stats.write().record_hit(access_time);
      Some(entry.media_processor.clone())
    } else {
      self.cache_stats.write().record_miss();
      None
    }
  }

  pub fn put_node(&mut self, key: String, media_processor: MediaProcessor) -> Result<()> {
    if !self.cache_config.enable_node_caching {
      return Ok(());
    }

    let mut node_cache = self.node_cache.write();
    let size_bytes = self.estimate_processor_size(&media_processor);
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    if node_cache.current_memory_mb + size_mb > self.cache_config.max_cache_size_mb as f64 {
      self.evict_node_entries(&mut node_cache, size_mb)?;
    }

    if node_cache.cache.len() >= self.cache_config.max_cache_size {
      self.evict_node_entries(&mut node_cache, size_mb)?;
    }

    let entry = CacheEntry {
      id: Uuid::new_v4(),
      key: key.clone(),
      media_processor,
      timestamp: Utc::now(),
      access_count: 1,
      last_access: Utc::now(),
      size_bytes,
      ttl_seconds: Some(self.cache_config.cache_ttl_seconds),
      metadata: HashMap::new(),
      compressed: false,
    };

    let mut compressed_processor = entry.media_processor.clone();
    let mut compressed_size = size_bytes;

    if self.cache_config.compression_enabled {
      if let Ok(compressed_data) = self.compress_processor(&entry.media_processor) {
        if let Ok(restored_processor) = self.decompress_processor(&compressed_data) {
          compressed_processor = restored_processor;
          compressed_size = compressed_data.len();
          entry.compressed = true;
        }
      }
    }

    node_cache.current_memory_mb += compressed_size as f64 / (1024.0 * 1024.0);
    node_cache.cache.insert(key.clone(), entry);
    node_cache.update_access_order(&key);
    node_cache.access_counts.insert(key, 1);

    Ok(())
  }

  pub fn remove_node(&mut self, key: &str) -> Option<MediaProcessor> {
    let mut node_cache = self.node_cache.write();

    if let Some(entry) = node_cache.remove(key) {
      node_cache.current_memory_mb = node_cache
        .current_memory_mb
        .saturating_sub(entry.size_bytes as f64 / (1024.0 * 1024.0));
      node_cache.access_order.retain(|k| k != key);
      node_cache.access_counts.remove(key);
      Some(entry.media_processor)
    } else {
      None
    }
  }

  pub fn get_graph(&self, key: &str) -> Option<PipelineGraph> {
    if !self.cache_config.enable_graph_caching {
      return None;
    }

    let mut graph_cache = self.graph_cache.write();
    let start_time = std::time::Instant::now();

    if let Some(entry) = graph_cache.get(key) {
      if let Some(ttl_seconds) = entry.ttl_seconds {
        let elapsed = Utc::now().signed_duration_since(entry.timestamp);
        if elapsed.num_seconds() > ttl_seconds as i64 {
          graph_cache.remove(key);
          self.cache_stats.write().record_miss();
          return None;
        }
      }

      let access_time = start_time.elapsed();
      self.cache_stats.write().record_hit(access_time);
      Some(entry.graph.clone())
    } else {
      self.cache_stats.write().record_miss();
      None
    }
  }

  pub fn put_graph(&mut self, key: String, graph: PipelineGraph) -> Result<()> {
    if !self.cache_config.enable_graph_caching {
      return Ok(());
    }

    let mut graph_cache = self.graph_cache.write();
    let size_bytes = self.estimate_graph_size(&graph);
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    if graph_cache.current_memory_mb + size_mb > self.cache_config.max_cache_size_mb as f64 {
      self.evict_graph_entries(&mut graph_cache, size_mb)?;
    }

    if graph_cache.cache.len() >= self.cache_config.max_cache_size {
      self.evict_graph_entries(&mut graph_cache, size_mb)?;
    }

    let entry = GraphCacheEntry {
      id: Uuid::new_v4(),
      key: key.clone(),
      graph,
      timestamp: Utc::now(),
      access_count: 1,
      last_access: Utc::now(),
      size_bytes,
      ttl_seconds: Some(self.cache_config.cache_ttl_seconds),
      metadata: HashMap::new(),
      compressed: false,
    };

    let mut compressed_graph = entry.graph.clone();
    let mut compressed_size = size_bytes;

    if self.cache_config.compression_enabled {
      if let Ok(compressed_data) = self.compress_graph(&entry.graph) {
        if let Ok(restored_graph) = self.decompress_graph(&compressed_data) {
          compressed_graph = restored_graph;
          compressed_size = compressed_data.len();
          entry.compressed = true;
        }
      }
    }

    graph_cache.current_memory_mb += compressed_size as f64 / (1024.0 * 1024.0);
    graph_cache.cache.insert(key.clone(), entry);
    graph_cache.update_access_order(&key);
    graph_cache.access_counts.insert(key, 1);

    Ok(())
  }

  pub fn remove_graph(&mut self, key: &str) -> Option<PipelineGraph> {
    let mut graph_cache = self.graph_cache.write();

    if let Some(entry) = graph_cache.remove(key) {
      graph_cache.current_memory_mb = graph_cache
        .current_memory_mb
        .saturating_sub(entry.size_bytes as f64 / (1024.0 * 1024.0));
      graph_cache.access_order.retain(|k| k != key);
      graph_cache.access_counts.remove(key);
      Some(entry.graph)
    } else {
      None
    }
  }

  pub fn clear(&mut self) {
    let mut node_cache = self.node_cache.write();
    let mut graph_cache = self.graph_cache.write();

    node_cache.clear();
    graph_cache.clear();

    self.cache_stats.write().reset();
  }

  pub fn cleanup(&mut self) -> Result<usize> {
    let mut removed_count = 0;
    let now = Utc::now();

    {
      let mut node_cache = self.node_cache.write();
      let keys_to_remove: Vec<String> = node_cache
        .cache
        .iter()
        .filter(|(_, entry)| {
          if let Some(ttl_seconds) = entry.ttl_seconds {
            let elapsed = now.signed_duration_since(entry.timestamp);
            elapsed.num_seconds() > ttl_seconds as i64
          } else {
            false
          }
        })
        .map(|(key, _)| key.clone())
        .collect();

      for key in keys_to_remove {
        node_cache.remove(&key);
        removed_count += 1;
      }
    }

    {
      let mut graph_cache = self.graph_cache.write();
      let keys_to_remove: Vec<String> = graph_cache
        .cache
        .iter()
        .filter(|(_, entry)| {
          if let Some(ttl_seconds) = entry.ttl_seconds {
            let elapsed = now.signed_duration_since(entry.timestamp);
            elapsed.num_seconds() > ttl_seconds as i64
          } else {
            false
          }
        })
        .map(|(key, _)| key.clone())
        .collect();

      for key in keys_to_remove {
        graph_cache.remove(&key);
        removed_count += 1;
      }
    }

    Ok(removed_count)
  }

  fn evict_node_entries(&self, node_cache: &mut NodeCache, required_size_mb: f64) -> Result<()> {
    while node_cache.current_memory_mb + required_size_mb
      > self.cache_config.max_cache_size_mb as f64
    {
      if let Some(key_to_remove) = self.select_victim_node(node_cache) {
        node_cache.remove(&key_to_remove);
      } else {
        break;
      }
    }
    Ok(())
  }

  fn evict_graph_entries(&self, graph_cache: &mut GraphCache, required_size_mb: f64) -> Result<()> {
    while graph_cache.current_memory_mb + required_size_mb
      > self.cache_config.max_cache_size_mb as f64
    {
      if let Some(key_to_remove) = self.select_victim_graph(graph_cache) {
        graph_cache.remove(&key_to_remove);
      } else {
        break;
      }
    }
    Ok(())
  }

  fn select_victim_node(&self, node_cache: &NodeCache) -> Option<String> {
    match node_cache.eviction_policy {
      CacheEvictionPolicy::LRU => node_cache.access_order.first().cloned(),
      CacheEvictionPolicy::LFU => node_cache
        .access_counts
        .iter()
        .min_by_key(|(_, &count)| count)
        .map(|(key, _)| key.clone()),
      CacheEvictionPolicy::FIFO => node_cache.cache.keys().next().cloned(),
      CacheEvictionPolicy::Random => {
        if node_cache.cache.is_empty() {
          None
        } else {
          let mut rng = create_random_generator();
          let index = rng.gen_range(0, node_cache.cache.len());
          node_cache.cache.keys().nth(index).cloned()
        }
      }
      CacheEvictionPolicy::TTL => node_cache
        .cache
        .iter()
        .filter(|(_, entry)| {
          if let Some(ttl_seconds) = entry.ttl_seconds {
            let elapsed = Utc::now().signed_duration_since(entry.timestamp);
            elapsed.num_seconds() > ttl_seconds as i64
          } else {
            false
          }
        })
        .min_by_key(|_, entry| entry.timestamp)
        .map(|(key, _)| key.clone()),
    }
  }

  fn select_victim_graph(&self, graph_cache: &GraphCache) -> Option<String> {
    match graph_cache.eviction_policy {
      CacheEvictionPolicy::LRU => graph_cache.access_order.first().cloned(),
      CacheEvictionPolicy::LFU => graph_cache
        .access_counts
        .iter()
        .min_by_key(|(_, &count)| count)
        .map(|(key, _)| key.clone()),
      CacheEvictionPolicy::FIFO => graph_cache.cache.keys().next().cloned(),
      CacheEvictionPolicy::Random => {
        if graph_cache.cache.is_empty() {
          None
        } else {
          let mut rng = create_random_generator();
          let index = rng.gen_range(0, graph_cache.cache.len());
          graph_cache.cache.keys().nth(index).cloned()
        }
      }
      CacheEvictionPolicy::TTL => graph_cache
        .cache
        .iter()
        .filter(|(_, entry)| {
          if let Some(ttl_seconds) = entry.ttl_seconds {
            let elapsed = Utc::now().signed_duration_since(entry.timestamp);
            elapsed.num_seconds() > ttl_seconds as i64
          } else {
            false
          }
        })
        .min_by_key(|_, entry| entry.timestamp)
        .map(|(key, _)| key.clone()),
    }
  }

  fn estimate_processor_size(&self, processor: &MediaProcessor) -> usize {
    match processor.media_type() {
      MediaType::Image => {
        let (width, height) = (processor.width(), processor.height());
        (width * height * 4) as usize
      }
      MediaType::Audio => 1_000_000,
      MediaType::Video => {
        let (width, height) = (processor.width(), processor.height());
        (width * height * 4) as usize
      }
      _ => 1_000_000,
    }
  }

  fn estimate_graph_size(&self, graph: &PipelineGraph) -> usize {
    let nodes_size = graph.nodes.len() * 1000;
    let edges_size = graph.edges.len() * 500;
    nodes_size + edges_size
  }

  fn compress_processor(&self, processor: &MediaProcessor) -> Result<Vec<u8>> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let data = processor.data().to_bytes();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
      .write_all(&data)
      .map_err(|e| EllasticError::IOError(format!("Compression failed: {}", e)))?;

    encoder
      .finish()
      .map_err(|e| EllasticError::IOError(format!("Compression finish failed: {}", e)))
  }

  fn decompress_processor(&self, compressed_data: &[u8]) -> Result<MediaProcessor> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed = Vec::new();

    decoder
      .read_to_end(&mut decompressed)
      .map_err(|e| EllasticError::IOError(format!("Decompression failed: {}", e)))?;

    let media_data = MediaData::from_bytes(&decompressed)?;
    Ok(MediaProcessor::new(media_data))
  }

  fn compress_graph(&self, graph: &PipelineGraph) -> Result<Vec<u8>> {
    let serialized = serde_json::to_vec(graph).map_err(|e| {
      EllasticError::SerializationError(format!("Graph serialization failed: {}", e))
    })?;

    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
      .write_all(&serialized)
      .map_err(|e| EllasticError::IOError(format!("Graph compression failed: {}", e)))?;

    encoder
      .finish()
      .map_err(|e| EllasticError::IOError(format!("Graph compression finish failed: {}", e)))
  }

  fn decompress_graph(&self, compressed_data: &[u8]) -> Result<PipelineGraph> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed = Vec::new();

    decoder
      .read_to_end(&mut decompressed)
      .map_err(|e| EllasticError::IOError(format!("Graph decompression failed: {}", e)))?;

    serde_json::from_slice(&decompressed).map_err(|e| {
      EllasticError::SerializationError(format!("Graph deserialization failed: {}", e))
    })
  }

  pub fn clone(&self) -> PipelineCache {
    PipelineCache {
      node_cache: self.node_cache.clone(),
      graph_cache: self.graph_cache.clone(),
      cache_config: self.cache_config.clone(),
      cache_stats: self.cache_stats.clone(),
    }
  }
}

impl NodeCache {
  fn new(config: &CacheConfig) -> Self {
    Self {
      cache: HashMap::new(),
      max_size: config.max_cache_size,
      max_memory_mb: config.max_cache_size_mb,
      current_memory_mb: 0,
      eviction_policy: config.cache_eviction_policy,
      access_order: Vec::new(),
      access_counts: HashMap::new(),
    }
  }

  fn get(&self, key: &str) -> Option<&CacheEntry> {
    self.cache.get(key)
  }

  fn remove(&mut self, key: &str) -> Option<CacheEntry> {
    if let Some(entry) = self.cache.remove(key) {
      self.current_memory_mb = self
        .current_memory_mb
        .saturating_sub(entry.size_bytes as f64 / (1024.0 * 1024.0));
      self.access_order.retain(|k| k != key);
      self.access_counts.remove(key);
      Some(entry)
    } else {
      None
    }
  }

  fn clear(&mut self) {
    self.cache.clear();
    self.access_order.clear();
    self.access_counts.clear();
    self.current_memory_mb = 0;
  }

  fn update_access_order(&mut self, key: &str) {
    self.access_order.retain(|k| k != key);
    self.access_order.push(key.to_string());
  }

  fn clone(&self) -> NodeCache {
    NodeCache {
      cache: self.cache.clone(),
      max_size: self.max_size,
      max_memory_mb: self.max_memory_mb,
      current_memory_mb: self.current_memory_mb,
      eviction_policy: self.eviction_policy,
      access_order: self.access_order.clone(),
      access_counts: self.access_counts.clone(),
    }
  }
}

impl GraphCache {
  fn new(config: &CacheConfig) -> Self {
    Self {
      cache: HashMap::new(),
      max_size: config.max_cache_size,
      max_memory_mb: config.max_cache_size_mb,
      current_memory_mb: 0,
      eviction_policy: config.cache_eviction_policy,
      access_order: Vec::new(),
      access_counts: HashMap::new(),
    }
  }

  fn get(&self, key: &str) -> Option<&GraphCacheEntry> {
    self.cache.get(key)
  }

  fn remove(&mut self, key: &str) -> Option<GraphCacheEntry> {
    if let Some(entry) = self.cache.remove(key) {
      self.current_memory_mb = self
        .current_memory_mb
        .saturating_sub(entry.size_bytes as f64 / (1024.0 * 1024.0));
      self.access_order.retain(|k| k != key);
      self.access_counts.remove(key);
      Some(entry)
    } else {
      None
    }
  }

  fn clear(&mut self) {
    self.cache.clear();
    self.access_order.clear();
    self.access_counts.clear();
    self.current_memory_mb = 0;
  }

  fn update_access_order(&mut self, key: &str) {
    self.access_order.retain(|k| k != key);
    self.access_order.push(key.to_string());
  }

  fn clone(&self) -> GraphCache {
    GraphCache {
      cache: self.cache.clone(),
      max_size: self.max_size,
      max_memory_mb: self.max_memory_mb,
      current_memory_mb: self.current_memory_mb,
      eviction_policy: self.eviction_policy,
      access_order: self.access_order.clone(),
      access_counts: self.access_counts.clone(),
    }
  }
}

impl CacheStats {
  fn new() -> Self {
    Self {
      node_cache_stats: NodeCacheStats::new(),
      graph_cache_stats: GraphCacheStats::new(),
      total_hits: 0,
      total_misses: 0,
      hit_rate: 0.0,
      memory_usage_mb: 0.0,
      eviction_count: 0,
      compression_ratio: 0.0,
    }
  }

  fn record_hit(&mut self, access_time: std::time::Duration) {
    self.total_hits += 1;
    self.update_hit_rate();

    let total_requests = self.total_hits + self.total_misses;
    if total_requests > 0 {
      let current_avg = self.node_cache_stats.average_access_time;
      let new_avg = (current_avg.as_secs_f64() * (total_requests - 1) as f64
        + access_time.as_secs_f64())
        / total_requests as f64;
      self.node_cache_stats.average_access_time = std::time::Duration::from_secs_f64(new_avg);
    }
  }

  fn record_miss(&mut self) {
    self.total_misses += 1;
    self.update_hit_rate();
  }

  fn update_hit_rate(&mut self) {
    let total_requests = self.total_hits + self.total_misses;
    if total_requests > 0 {
      self.hit_rate = self.total_hits as f64 / total_requests as f64;
    }
  }

  fn reset(&mut self) {
    *self = Self::new();
  }

  fn clone(&self) -> CacheStats {
    CacheStats {
      node_cache_stats: self.node_cache_stats.clone(),
      graph_cache_stats: self.graph_cache_stats.clone(),
      total_hits: self.total_hits,
      total_misses: self.total_misses,
      hit_rate: self.hit_rate,
      memory_usage_mb: self.memory_usage_mb,
      eviction_count: self.eviction_count,
      compression_ratio: self.compression_ratio,
    }
  }
}

impl NodeCacheStats {
  fn new() -> Self {
    Self {
      entries: 0,
      hits: 0,
      misses: 0,
      hit_rate: 0.0,
      memory_usage_mb: 0.0,
      evictions: 0,
      average_access_time: std::time::Duration::ZERO,
    }
  }

  fn clone(&self) -> NodeCacheStats {
    NodeCacheStats {
      entries: self.entries,
      hits: self.hits,
      misses: self.misses,
      hit_rate: self.hit_rate,
      memory_usage_mb: self.memory_usage_mb,
      evictions: self.evictions,
      average_access_time: self.average_access_time,
    }
  }
}

impl GraphCacheStats {
  fn new() -> Self {
    Self {
      entries: 0,
      hits: 0,
      misses: 0,
      hit_rate: 0.0,
      memory_usage_mb: 0.0,
      evictions: 0,
      average_access_time: std::time::Duration::ZERO,
    }
  }

  fn clone(&self) -> GraphCacheStats {
    GraphCacheStats {
      entries: self.entries,
      hits: self.hits,
      misses: self.misses,
      hit_rate: self.hit_rate,
      memory_usage_mb: self.memory_usage_mb,
      evictions: self.evictions,
      average_access_time: self.average_access_time,
    }
  }
}

impl CacheManager {
  pub fn new() -> Self {
    Self {
      caches: HashMap::new(),
      default_cache_name: "default".to_string(),
      global_stats: Arc::new(RwLock::new(CacheStats::new())),
    }
  }

  pub fn with_default_cache_name(mut self, name: String) -> Self {
    self.default_cache_name = name;
    self
  }

  pub fn add_cache(&mut self, name: String, cache: PipelineCache) -> Result<()> {
    if self.caches.contains_key(&name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Cache '{}' already exists",
        name
      )));
    }

    self.caches.insert(name, cache);
    Ok(())
  }

  pub fn remove_cache(&mut self, name: &str) -> Option<PipelineCache> {
    self.caches.remove(name)
  }

  pub fn get_cache(&self, name: &str) -> Option<&PipelineCache> {
    self.caches.get(name)
  }

  pub fn get_cache_mut(&mut self, name: &str) -> Option<&mut PipelineCache> {
    self.caches.get_mut(name)
  }

  pub fn get_default_cache(&self) -> Option<&PipelineCache> {
    self.caches.get(&self.default_cache_name)
  }

  pub fn get_default_cache_mut(&mut self) -> Option<&mut PipelineCache> {
    self.caches.get_mut(&self.default_cache_name)
  }

  pub fn list_caches(&self) -> Vec<&String> {
    self.caches.keys().collect()
  }

  pub fn clear_all_caches(&mut self) {
    for cache in self.caches.values_mut() {
      cache.clear();
    }
    self.global_stats.write().reset();
  }

  pub fn cleanup_all_caches(&mut self) -> Result<usize> {
    let mut total_removed = 0;

    for cache in self.caches.values_mut() {
      total_removed += cache.cleanup()?;
    }

    Ok(total_removed)
  }

  pub fn get_global_stats(&self) -> Arc<RwLock<CacheStats>> {
    self.global_stats.clone()
  }

  pub fn clone(&self) -> CacheManager {
    CacheManager {
      caches: self.caches.clone(),
      default_cache_name: self.default_cache_name.clone(),
      global_stats: self.global_stats.clone(),
    }
  }
}

impl Default for CacheConfig {
  fn default() -> Self {
    Self {
      enable_node_caching: true,
      enable_graph_caching: false,
      max_cache_size_mb: 100,
      max_cache_size: 1000,
      cache_ttl_seconds: 3600,
      enable_persistent_cache: false,
      cache_eviction_policy: CacheEvictionPolicy::LRU,
      compression_enabled: false,
      serialization_format: CacheSerializationFormat::Binary,
    }
  }
}

pub fn create_pipeline_cache(config: CacheConfig) -> PipelineCache {
  PipelineCache::new(config)
}

pub fn create_cache_config() -> CacheConfig {
  CacheConfig::default()
}

pub fn create_cache_manager() -> CacheManager {
  CacheManager::new()
}
