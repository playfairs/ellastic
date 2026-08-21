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
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EffectCache {
  cache: HashMap<String, CacheEntry>,
  max_size: usize,
  max_memory_mb: usize,
  current_memory_mb: usize,
  access_order: Vec<String>,
  hit_count: u64,
  miss_count: u64,
  created_at: DateTime<Utc>,
  last_cleanup: DateTime<Utc>,
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
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
  LRU,
  LFU,
  FIFO,
  Random,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
  pub max_size: usize,
  pub max_memory_mb: usize,
  pub cleanup_interval_secs: u64,
  pub policy: CachePolicy,
  pub enable_compression: bool,
  pub enable_encryption: bool,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
  pub total_entries: usize,
  pub hit_count: u64,
  pub miss_count: u64,
  pub hit_rate: f64,
  pub current_memory_mb: usize,
  pub max_memory_mb: usize,
  pub oldest_entry: Option<DateTime<Utc>>,
  pub newest_entry: Option<DateTime<Utc>>,
  pub average_access_count: f64,
}

impl EffectCache {
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      cache: HashMap::new(),
      max_size: 1000,
      max_memory_mb: 100,
      current_memory_mb: 0,
      access_order: Vec::new(),
      hit_count: 0,
      miss_count: 0,
      created_at: now,
      last_cleanup: now,
    }
  }

  pub fn with_config(config: CacheConfig) -> Self {
    let now = Utc::now();
    Self {
      cache: HashMap::new(),
      max_size: config.max_size,
      max_memory_mb: config.max_memory_mb,
      current_memory_mb: 0,
      access_order: Vec::new(),
      hit_count: 0,
      miss_count: 0,
      created_at: now,
      last_cleanup: now,
    }
  }

  pub fn get(&mut self, key: &str) -> Option<MediaProcessor> {
    if let Some(entry) = self.cache.get_mut(key) {
      entry.access_count += 1;
      entry.last_access = Utc::now();
      self.hit_count += 1;

      if let Some(index) = self.access_order.iter().position(|k| k == key) {
        self.access_order.remove(index);
      }
      self.access_order.push(key.to_string());

      Some(entry.media_processor.clone())
    } else {
      self.miss_count += 1;
      None
    }
  }

  pub fn put(&mut self, key: String, media_processor: MediaProcessor) -> Result<()> {
    let now = Utc::now();
    let size_bytes = self.estimate_size(&media_processor);

    if self.cache.len() >= self.max_size
      || self.current_memory_mb + size_bytes / (1024 * 1024) > self.max_memory_mb
    {
      self.evict_entries()?;
    }

    let entry = CacheEntry {
      id: Uuid::new_v4(),
      key: key.clone(),
      media_processor,
      timestamp: now,
      access_count: 1,
      last_access: now,
      size_bytes,
      metadata: HashMap::new(),
    };

    self.current_memory_mb += size_bytes / (1024 * 1024);
    self.cache.insert(key.clone(), entry);

    self.access_order.push(key);

    Ok(())
  }

  pub fn remove(&mut self, key: &str) -> Option<MediaProcessor> {
    if let Some(entry) = self.cache.remove(key) {
      self.current_memory_mb = self
        .current_memory_mb
        .saturating_sub(entry.size_bytes / (1024 * 1024));

      if let Some(index) = self.access_order.iter().position(|k| k == key) {
        self.access_order.remove(index);
      }

      Some(entry.media_processor)
    } else {
      None
    }
  }

  pub fn clear(&mut self) {
    self.cache.clear();
    self.access_order.clear();
    self.current_memory_mb = 0;
  }

  pub fn contains_key(&self, key: &str) -> bool {
    self.cache.contains_key(key)
  }

  pub fn len(&self) -> usize {
    self.cache.len()
  }

  pub fn is_empty(&self) -> bool {
    self.cache.is_empty()
  }

  pub fn get_stats(&self) -> CacheStats {
    let total_requests = self.hit_count + self.miss_count;
    let hit_rate = if total_requests > 0 {
      self.hit_count as f64 / total_requests as f64
    } else {
      0.0
    };

    let (oldest_entry, newest_entry) = if self.cache.is_empty() {
      (None, None)
    } else {
      let entries: Vec<_> = self.cache.values().collect();
      let oldest = entries
        .iter()
        .min_by_key(|e| e.timestamp)
        .map(|e| e.timestamp);
      let newest = entries
        .iter()
        .max_by_key(|e| e.timestamp)
        .map(|e| e.timestamp);
      (oldest, newest)
    };

    let average_access_count = if self.cache.is_empty() {
      0.0
    } else {
      self.cache.values().map(|e| e.access_count).sum::<u64>() as f64 / self.cache.len() as f64
    };

    CacheStats {
      total_entries: self.cache.len(),
      hit_count: self.hit_count,
      miss_count: self.miss_count,
      hit_rate,
      current_memory_mb: self.current_memory_mb,
      max_memory_mb: self.max_memory_mb,
      oldest_entry,
      newest_entry,
      average_access_count,
    }
  }

  pub fn cleanup(&mut self) -> Result<usize> {
    let now = Utc::now();
    let mut removed_count = 0;

    let cutoff = now - chrono::Duration::hours(1);

    let keys_to_remove: Vec<String> = self
      .cache
      .iter()
      .filter(|(_, entry)| entry.timestamp < cutoff)
      .map(|(key, _)| key.clone())
      .collect();

    for key in keys_to_remove {
      if self.remove(&key).is_some() {
        removed_count += 1;
      }
    }

    self.last_cleanup = now;
    Ok(removed_count)
  }

  pub fn optimize(&mut self) -> Result<()> {
    let mut entries: Vec<_> = self.cache.iter().collect();
    entries.sort_by_key(|(_, entry)| entry.access_count);

    while (self.cache.len() > self.max_size || self.current_memory_mb > self.max_memory_mb)
      && !entries.is_empty()
    {
      if let Some((key, _)) = entries.remove(0) {
        self.remove(key);
      }
    }

    Ok(())
  }

  fn evict_entries(&mut self) -> Result<()> {
    while (self.cache.len() >= self.max_size || self.current_memory_mb >= self.max_memory_mb)
      && !self.access_order.is_empty()
    {
      if let Some(key) = self.access_order.remove(0) {
        self.remove(&key);
      }
    }
    Ok(())
  }

  fn estimate_size(&self, media_processor: &MediaProcessor) -> usize {
    match media_processor.media_type() {
      MediaType::Image => {
        let (width, height) = (media_processor.width(), media_processor.height());
        (width * height * 4) as usize
      }
      MediaType::Audio => 1_000_000,
      MediaType::Video => {
        let (width, height) = (media_processor.width(), media_processor.height());
        (width * height * 4) as usize
      }
      _ => 1_000_000,
    }
  }

  pub fn clone(&self) -> EffectCache {
    EffectCache {
      cache: self.cache.clone(),
      max_size: self.max_size,
      max_memory_mb: self.max_memory_mb,
      current_memory_mb: self.current_memory_mb,
      access_order: self.access_order.clone(),
      hit_count: self.hit_count,
      miss_count: self.miss_count,
      created_at: self.created_at,
      last_cleanup: self.last_cleanup,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DistributedCache {
  local_cache: EffectCache,
  remote_cache: Option<Box<dyn RemoteCacheBackend>>,
  sync_interval_secs: u64,
  last_sync: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CacheEntryMetadata {
  pub key: String,
  pub size_bytes: usize,
  pub timestamp: DateTime<Utc>,
  pub access_count: u64,
  pub last_access: DateTime<Utc>,
  pub ttl_secs: Option<u64>,
}

pub trait RemoteCacheBackend: Send + Sync {
  fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
  fn put(&self, key: String, data: Vec<u8>, metadata: CacheEntryMetadata) -> Result<()>;
  fn remove(&self, key: &str) -> Result<bool>;
  fn clear(&self) -> Result<()>;
  fn list_keys(&self) -> Result<Vec<String>>;
  fn get_stats(&self) -> Result<HashMap<String, String>>;
}

impl DistributedCache {
  pub fn new(local_cache: EffectCache) -> Self {
    let now = Utc::now();
    Self {
      local_cache,
      remote_cache: None,
      sync_interval_secs: 300,
      last_sync: now,
    }
  }

  pub fn with_remote_cache(mut self, remote_cache: Box<dyn RemoteCacheBackend>) -> Self {
    self.remote_cache = Some(remote_cache);
    self
  }

  pub fn with_sync_interval(mut self, interval_secs: u64) -> Self {
    self.sync_interval_secs = interval_secs;
    self
  }

  pub fn get(&mut self, key: &str) -> Option<MediaProcessor> {
    if let Some(processor) = self.local_cache.get(key) {
      return Some(processor);
    }

    if let Some(ref remote_cache) = self.remote_cache {
      if let Ok(Some(data)) = remote_cache.get(key) {
        if let Ok(processor) = self.deserialize_processor(&data) {
          let _ = self.local_cache.put(key.to_string(), processor.clone());
          return Some(processor);
        }
      }
    }

    None
  }

  pub fn put(&mut self, key: String, media_processor: MediaProcessor) -> Result<()> {
    self.local_cache.put(key.clone(), media_processor.clone())?;

    if let Some(ref remote_cache) = self.remote_cache {
      let data = self.serialize_processor(&media_processor)?;
      let metadata = CacheEntryMetadata {
        key: key.clone(),
        size_bytes: data.len(),
        timestamp: Utc::now(),
        access_count: 1,
        last_access: Utc::now(),
        ttl_secs: Some(3600),
      };
      remote_cache.put(key, data, metadata)?;
    }

    Ok(())
  }

  pub fn remove(&mut self, key: &str) -> Option<MediaProcessor> {
    let result = self.local_cache.remove(key);

    if let Some(ref remote_cache) = self.remote_cache {
      let _ = remote_cache.remove(key);
    }

    result
  }

  pub fn clear(&mut self) -> Result<()> {
    self.local_cache.clear();

    if let Some(ref remote_cache) = self.remote_cache {
      remote_cache.clear()?;
    }

    Ok(())
  }

  pub fn sync(&mut self) -> Result<()> {
    let now = Utc::now();

    if let Some(ref remote_cache) = self.remote_cache {
      for (key, entry) in &self.local_cache.cache {
        let data = self.serialize_processor(&entry.media_processor)?;
        let metadata = CacheEntryMetadata {
          key: key.clone(),
          size_bytes: data.len(),
          timestamp: entry.timestamp,
          access_count: entry.access_count,
          last_access: entry.last_access,
          ttl_secs: Some(3600),
        };
        remote_cache.put(key.clone(), data, metadata)?;
      }

      if let Ok(keys) = remote_cache.list_keys() {
        for key in keys {
          if !self.local_cache.contains_key(&key) {
            if let Ok(Some(data)) = remote_cache.get(&key) {
              if let Ok(processor) = self.deserialize_processor(&data) {
                let _ = self.local_cache.put(key, processor);
              }
            }
          }
        }
      }
    }

    self.last_sync = now;
    Ok(())
  }

  pub fn should_sync(&self) -> bool {
    let now = Utc::now();
    (now - self.last_sync).num_seconds() >= self.sync_interval_secs as i64
  }

  fn serialize_processor(&self, processor: &MediaProcessor) -> Result<Vec<u8>> {
    let data = processor.data().to_bytes();
    Ok(data)
  }

  fn deserialize_processor(&self, data: &[u8]) -> Result<MediaProcessor> {
    let media_data = MediaData::from_bytes(data)?;
    Ok(MediaProcessor::new(media_data))
  }

  pub fn clone(&self) -> DistributedCache {
    DistributedCache {
      local_cache: self.local_cache.clone(),
      remote_cache: None,
      sync_interval_secs: self.sync_interval_secs,
      last_sync: self.last_sync,
    }
  }
}

#[derive(Debug, Clone)]
pub struct CacheManager {
  caches: HashMap<String, EffectCache>,
  default_cache_name: String,
  global_cache: Option<DistributedCache>,
}

impl CacheManager {
  pub fn new() -> Self {
    Self {
      caches: HashMap::new(),
      default_cache_name: "default".to_string(),
      global_cache: None,
    }
  }

  pub fn with_default_cache_name(mut self, name: String) -> Self {
    self.default_cache_name = name;
    self
  }

  pub fn with_global_cache(mut self, global_cache: DistributedCache) -> Self {
    self.global_cache = Some(global_cache);
    self
  }

  pub fn add_cache(&mut self, name: String, cache: EffectCache) -> Result<()> {
    if self.caches.contains_key(&name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Cache '{}' already exists",
        name
      )));
    }
    self.caches.insert(name, cache);
    Ok(())
  }

  pub fn remove_cache(&mut self, name: &str) -> Option<EffectCache> {
    self.caches.remove(name)
  }

  pub fn get_cache(&self, name: &str) -> Option<&EffectCache> {
    self.caches.get(name)
  }

  pub fn get_cache_mut(&mut self, name: &str) -> Option<&mut EffectCache> {
    self.caches.get_mut(name)
  }

  pub fn get_default_cache(&self) -> Option<&EffectCache> {
    self.caches.get(&self.default_cache_name)
  }

  pub fn get_default_cache_mut(&mut self) -> Option<&mut EffectCache> {
    self.caches.get_mut(&self.default_cache_name)
  }

  pub fn list_caches(&self) -> Vec<&String> {
    self.caches.keys().collect()
  }

  pub fn get(&mut self, cache_name: &str, key: &str) -> Option<MediaProcessor> {
    if let Some(cache) = self.caches.get_mut(cache_name) {
      if let Some(processor) = cache.get(key) {
        return Some(processor);
      }
    }

    if let Some(ref mut global_cache) = self.global_cache {
      global_cache.get(key)
    } else {
      None
    }
  }

  pub fn put(
    &mut self,
    cache_name: &str,
    key: String,
    media_processor: MediaProcessor,
  ) -> Result<()> {
    if let Some(cache) = self.caches.get_mut(cache_name) {
      cache.put(key.clone(), media_processor.clone())?;
    }

    if let Some(ref mut global_cache) = self.global_cache {
      global_cache.put(key, media_processor)?;
    }

    Ok(())
  }

  pub fn remove(&mut self, cache_name: &str, key: &str) -> Option<MediaProcessor> {
    let mut result = None;

    if let Some(cache) = self.caches.get_mut(cache_name) {
      result = cache.remove(key);
    }

    if let Some(ref mut global_cache) = self.global_cache {
      global_cache.remove(key);
    }

    result
  }

  pub fn clear(&mut self, cache_name: &str) -> Result<()> {
    if let Some(cache) = self.caches.get_mut(cache_name) {
      cache.clear();
    }

    if let Some(ref mut global_cache) = self.global_cache {
      global_cache.clear()?;
    }

    Ok(())
  }

  pub fn clear_all(&mut self) -> Result<()> {
    for cache in self.caches.values_mut() {
      cache.clear();
    }

    if let Some(ref mut global_cache) = self.global_cache {
      global_cache.clear()?;
    }

    Ok(())
  }

  pub fn cleanup_all(&mut self) -> Result<usize> {
    let mut total_removed = 0;

    for cache in self.caches.values_mut() {
      total_removed += cache.cleanup().unwrap_or(0);
    }

    if let Some(ref mut global_cache) = self.global_cache {
      global_cache.cleanup()?;
    }

    Ok(total_removed)
  }

  pub fn optimize_all(&mut self) -> Result<()> {
    for cache in self.caches.values_mut() {
      cache.optimize()?;
    }

    Ok(())
  }

  pub fn get_all_stats(&self) -> HashMap<String, CacheStats> {
    let mut stats = HashMap::new();

    for (name, cache) in &self.caches {
      stats.insert(name.clone(), cache.get_stats());
    }

    stats
  }

  pub fn clone(&self) -> CacheManager {
    CacheManager {
      caches: self.caches.clone(),
      default_cache_name: self.default_cache_name.clone(),
      global_cache: self.global_cache.clone(),
    }
  }
}

pub fn create_effect_cache() -> EffectCache {
  EffectCache::new()
}

pub fn create_effect_cache_with_config(config: CacheConfig) -> EffectCache {
  EffectCache::with_config(config)
}

pub fn create_distributed_cache(local_cache: EffectCache) -> DistributedCache {
  DistributedCache::new(local_cache)
}

pub fn create_cache_manager() -> CacheManager {
  CacheManager::new()
}

pub fn create_cache_config() -> CacheConfig {
  CacheConfig {
    max_size: 1000,
    max_memory_mb: 100,
    cleanup_interval_secs: 300,
    policy: CachePolicy::LRU,
    enable_compression: false,
    enable_encryption: false,
  }
}
