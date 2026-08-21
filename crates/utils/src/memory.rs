use ellastic_errors::{
  EllasticError,
  Result,
};
use std::alloc::{
  GlobalAlloc,
  Layout,
  System,
};
use std::sync::atomic::{
  AtomicU64,
  Ordering,
};

static TOTAL_ALLOCATED: AtomicU64 = AtomicU64::new(0);
static TOTAL_DEALLOCATED: AtomicU64 = AtomicU64::new(0);
static PEAK_MEMORY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct MemoryTracker {
  initial_memory: u64,
  peak_usage: u64,
}

impl MemoryTracker {
  pub fn new() -> Self {
    Self {
      initial_memory: Self::get_current_memory_usage(),
      peak_usage: Self::get_current_memory_usage(),
    }
  }

  pub fn get_current_memory_usage() -> u64 {
    TOTAL_ALLOCATED.load(Ordering::Relaxed) - TOTAL_DEALLOCATED.load(Ordering::Relaxed)
  }

  pub fn get_peak_memory_usage() -> u64 {
    PEAK_MEMORY.load(Ordering::Relaxed)
  }

  pub fn get_total_allocated() -> u64 {
    TOTAL_ALLOCATED.load(Ordering::Relaxed)
  }

  pub fn get_total_deallocated() -> u64 {
    TOTAL_DEALLOCATED.load(Ordering::Relaxed)
  }

  pub fn update_peak(&mut self) {
    let current = Self::get_current_memory_usage();
    if current > self.peak_usage {
      self.peak_usage = current;
    }
  }

  pub fn reset(&mut self) {
    self.initial_memory = Self::get_current_memory_usage();
    self.peak_usage = Self::get_current_memory_usage();
  }

  pub fn memory_increase(&self) -> u64 {
    Self::get_current_memory_usage().saturating_sub(self.initial_memory)
  }

  pub fn memory_usage_mb(&self) -> f64 {
    Self::get_current_memory_usage() as f64 / (1024.0 * 1024.0)
  }

  pub fn peak_usage_mb(&self) -> f64 {
    self.peak_usage as f64 / (1024.0 * 1024.0)
  }

  pub fn memory_increase_mb(&self) -> f64 {
    self.memory_increase() as f64 / (1024.0 * 1024.0)
  }
}

impl Default for MemoryTracker {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct MemoryPool {
  buffer: Vec<u8>,
  allocated: Vec<(usize, usize)>,
  free_blocks: Vec<(usize, usize)>,
}

impl MemoryPool {
  pub fn new(size: usize) -> Self {
    Self {
      buffer: vec![0u8; size],
      allocated: Vec::new(),
      free_blocks: vec![(0, size)],
    }
  }

  pub fn allocate(&mut self, size: usize) -> Result<usize> {
    let aligned_size = (size + 7) & !7;

    for (i, &(offset, block_size)) in self.free_blocks.iter().enumerate() {
      if block_size >= aligned_size {
        let offset = offset;
        let remaining_size = block_size - aligned_size;

        self.free_blocks.remove(i);

        if remaining_size > 0 {
          self.insert_free_block(offset + aligned_size, remaining_size);
        }

        self.allocated.push((offset, aligned_size));
        return Ok(offset);
      }
    }

    Err(EllasticError::MemoryError(
      "Out of memory in pool".to_string(),
    ))
  }

  pub fn deallocate(&mut self, offset: usize) -> Result<()> {
    for (i, &(alloc_offset, size)) in self.allocated.iter().enumerate() {
      if alloc_offset == offset {
        self.allocated.remove(i);
        self.insert_free_block(offset, size);
        self.coalesce_free_blocks();
        return Ok(());
      }
    }

    Err(EllasticError::MemoryError(
      "Invalid offset for deallocation".to_string(),
    ))
  }

  pub fn get_buffer(&self) -> &[u8] {
    &self.buffer
  }

  pub fn get_buffer_mut(&mut self) -> &mut [u8] {
    &mut self.buffer
  }

  pub fn get_slice(&self, offset: usize, size: usize) -> Result<&[u8]> {
    if offset + size > self.buffer.len() {
      return Err(EllasticError::MemoryError(
        "Slice out of bounds".to_string(),
      ));
    }

    Ok(&self.buffer[offset..offset + size])
  }

  pub fn get_slice_mut(&mut self, offset: usize, size: usize) -> Result<&mut [u8]> {
    if offset + size > self.buffer.len() {
      return Err(EllasticError::MemoryError(
        "Slice out of bounds".to_string(),
      ));
    }

    Ok(&mut self.buffer[offset..offset + size])
  }

  pub fn total_size(&self) -> usize {
    self.buffer.len()
  }

  pub fn used_size(&self) -> usize {
    self.allocated.iter().map(|(_, size)| *size).sum()
  }

  pub fn free_size(&self) -> usize {
    self.total_size() - self.used_size()
  }

  pub fn utilization(&self) -> f64 {
    self.used_size() as f64 / self.total_size() as f64
  }

  pub fn fragmentation_ratio(&self) -> f64 {
    if self.free_blocks.is_empty() {
      return 0.0;
    }

    let total_free: usize = self.free_blocks.iter().map(|(_, size)| *size).sum();
    let max_free: usize = self
      .free_blocks
      .iter()
      .map(|(_, size)| *size)
      .max()
      .unwrap_or(0);

    if total_free == 0 {
      0.0
    } else {
      1.0 - (max_free as f64 / total_free as f64)
    }
  }

  fn insert_free_block(&mut self, offset: usize, size: usize) {
    let mut insert_pos = self.free_blocks.len();

    for (i, &(existing_offset, _)) in self.free_blocks.iter().enumerate() {
      if offset < existing_offset {
        insert_pos = i;
        break;
      }
    }

    self.free_blocks.insert(insert_pos, (offset, size));
  }

  fn coalesce_free_blocks(&mut self) {
    let mut i = 0;
    while i < self.free_blocks.len() - 1 {
      let (offset1, size1) = self.free_blocks[i];
      let (offset2, _) = self.free_blocks[i + 1];

      if offset1 + size1 == offset2 {
        let merged_size = size1 + self.free_blocks[i + 1].1;
        self.free_blocks[i] = (offset1, merged_size);
        self.free_blocks.remove(i + 1);
      } else {
        i += 1;
      }
    }
  }

  pub fn reset(&mut self) {
    self.allocated.clear();
    self.free_blocks.clear();
    self.free_blocks.push((0, self.buffer.len()));
  }
}

#[derive(Debug, Clone)]
pub struct BufferPool<T> {
  buffers: Vec<Vec<T>>,
  available: Vec<usize>,
  buffer_size: usize,
}

impl<T: Clone + Default> BufferPool<T> {
  pub fn new(buffer_size: usize, initial_count: usize) -> Self {
    let mut buffers = Vec::with_capacity(initial_count);
    let available = (0..initial_count).collect();

    for _ in 0..initial_count {
      buffers.push(vec![T::default(); buffer_size]);
    }

    Self {
      buffers,
      available,
      buffer_size,
    }
  }

  pub fn acquire(&mut self) -> Result<usize> {
    if let Some(index) = self.available.pop() {
      Ok(index)
    } else {
      let new_index = self.buffers.len();
      self.buffers.push(vec![T::default(); self.buffer_size]);
      Ok(new_index)
    }
  }

  pub fn release(&mut self, index: usize) -> Result<()> {
    if index < self.buffers.len() {
      if !self.available.contains(&index) {
        self.available.push(index);
      }
      Ok(())
    } else {
      Err(EllasticError::MemoryError(
        "Invalid buffer index".to_string(),
      ))
    }
  }

  pub fn get_buffer(&self, index: usize) -> Result<&[T]> {
    self
      .buffers
      .get(index)
      .map(|buf| buf.as_slice())
      .ok_or_else(|| EllasticError::MemoryError("Invalid buffer index".to_string()))
  }

  pub fn get_buffer_mut(&mut self, index: usize) -> Result<&mut [T]> {
    self
      .buffers
      .get_mut(index)
      .map(|buf| buf.as_mut_slice())
      .ok_or_else(|| EllasticError::MemoryError("Invalid buffer index".to_string()))
  }

  pub fn buffer_size(&self) -> usize {
    self.buffer_size
  }

  pub fn total_buffers(&self) -> usize {
    self.buffers.len()
  }

  pub fn available_buffers(&self) -> usize {
    self.available.len()
  }

  pub fn used_buffers(&self) -> usize {
    self.total_buffers() - self.available_buffers()
  }

  pub fn utilization(&self) -> f64 {
    if self.total_buffers() == 0 {
      0.0
    } else {
      self.used_buffers() as f64 / self.total_buffers() as f64
    }
  }

  pub fn clear(&mut self) {
    self.available.clear();
    self.available.extend(0..self.buffers.len());
  }

  pub fn shrink_to_fit(&mut self) {
    let max_used = self.buffers.len() - self.available.len();

    for i in (max_used..self.buffers.len()).rev() {
      if self.available.contains(&i) {
        self.available.retain(|&index| index != i);
        self.buffers.remove(i);
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct MemoryLimiter {
  limit_bytes: u64,
  current_usage: u64,
}

impl MemoryLimiter {
  pub fn new(limit_bytes: u64) -> Self {
    Self {
      limit_bytes,
      current_usage: 0,
    }
  }

  pub fn try_allocate(&mut self, bytes: u64) -> Result<()> {
    if self.current_usage + bytes <= self.limit_bytes {
      self.current_usage += bytes;
      Ok(())
    } else {
      Err(EllasticError::MemoryError(format!(
        "Memory limit exceeded: {} + {} > {}",
        self.current_usage, bytes, self.limit_bytes
      )))
    }
  }

  pub fn deallocate(&mut self, bytes: u64) {
    self.current_usage = self.current_usage.saturating_sub(bytes);
  }

  pub fn current_usage(&self) -> u64 {
    self.current_usage
  }

  pub fn limit(&self) -> u64 {
    self.limit_bytes
  }

  pub fn available(&self) -> u64 {
    self.limit_bytes.saturating_sub(self.current_usage)
  }

  pub fn utilization(&self) -> f64 {
    if self.limit_bytes == 0 {
      0.0
    } else {
      self.current_usage as f64 / self.limit_bytes as f64
    }
  }

  pub fn is_near_limit(&self, threshold: f64) -> bool {
    self.utilization() >= threshold
  }

  pub fn reset(&mut self) {
    self.current_usage = 0;
  }

  pub fn set_limit(&mut self, new_limit: u64) {
    self.limit_bytes = new_limit;
  }
}

#[derive(Debug, Clone)]
pub struct MemoryGuard {
  limiter: std::sync::Arc<std::sync::Mutex<MemoryLimiter>>,
  allocated_bytes: u64,
}

impl MemoryGuard {
  pub fn new(
    limiter: std::sync::Arc<std::sync::Mutex<MemoryLimiter>>,
    allocated_bytes: u64,
  ) -> Self {
    Self {
      limiter,
      allocated_bytes,
    }
  }

  pub fn allocated_bytes(&self) -> u64 {
    self.allocated_bytes
  }
}

impl Drop for MemoryGuard {
  fn drop(&mut self) {
    if let Ok(mut limiter) = self.limiter.lock() {
      limiter.deallocate(self.allocated_bytes);
    }
  }
}

pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let ptr = System.alloc(layout);
    if !ptr.is_null() {
      let size = layout.size();
      TOTAL_ALLOCATED.fetch_add(size as u64, Ordering::Relaxed);

      let current =
        TOTAL_ALLOCATED.load(Ordering::Relaxed) - TOTAL_DEALLOCATED.load(Ordering::Relaxed);
      let peak = PEAK_MEMORY.load(Ordering::Relaxed);
      if current > peak {
        PEAK_MEMORY.store(current, Ordering::Relaxed);
      }
    }
    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    let size = layout.size();
    System.dealloc(ptr, layout);
    TOTAL_DEALLOCATED.fetch_add(size as u64, Ordering::Relaxed);
  }

  unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    let old_size = layout.size();
    let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
    let new_ptr = System.realloc(ptr, layout, new_size);

    if !new_ptr.is_null() {
      let size_diff = new_size as i64 - old_size as i64;
      if size_diff > 0 {
        TOTAL_ALLOCATED.fetch_add(size_diff as u64, Ordering::Relaxed);
      } else {
        TOTAL_DEALLOCATED.fetch_add((-size_diff) as u64, Ordering::Relaxed);
      }

      let current =
        TOTAL_ALLOCATED.load(Ordering::Relaxed) - TOTAL_DEALLOCATED.load(Ordering::Relaxed);
      let peak = PEAK_MEMORY.load(Ordering::Relaxed);
      if current > peak {
        PEAK_MEMORY.store(current, Ordering::Relaxed);
      }
    }

    new_ptr
  }
}

pub fn create_memory_tracker() -> MemoryTracker {
  MemoryTracker::new()
}

pub fn create_memory_pool(size: usize) -> MemoryPool {
  MemoryPool::new(size)
}

pub fn create_buffer_pool<T: Clone + Default>(
  buffer_size: usize,
  initial_count: usize,
) -> BufferPool<T> {
  BufferPool::new(buffer_size, initial_count)
}

pub fn create_memory_limiter(limit_bytes: u64) -> MemoryLimiter {
  MemoryLimiter::new(limit_bytes)
}

pub fn create_memory_guard(
  limiter: std::sync::Arc<std::sync::Mutex<MemoryLimiter>>,
  allocated_bytes: u64,
) -> MemoryGuard {
  MemoryGuard::new(limiter, allocated_bytes)
}

pub fn get_memory_usage() -> u64 {
  MemoryTracker::get_current_memory_usage()
}

pub fn get_peak_memory_usage() -> u64 {
  MemoryTracker::get_peak_memory_usage()
}

pub fn get_memory_usage_mb() -> f64 {
  get_memory_usage() as f64 / (1024.0 * 1024.0)
}

pub fn get_peak_memory_usage_mb() -> f64 {
  get_peak_memory_usage() as f64 / (1024.0 * 1024.0)
}

pub fn format_bytes(bytes: u64) -> String {
  const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
  let mut size = bytes as f64;
  let mut unit_index = 0;

  while size >= 1024.0 && unit_index < UNITS.len() - 1 {
    size /= 1024.0;
    unit_index += 1;
  }

  if unit_index == 0 {
    format!("{} {}", bytes, UNITS[unit_index])
  } else {
    format!("{:.2} {}", size, UNITS[unit_index])
  }
}

pub fn estimate_memory_usage(
  element_count: usize,
  element_size: usize,
  overhead_factor: f64,
) -> usize {
  let base_size = element_count * element_size;
  (base_size as f64 * overhead_factor) as usize
}

pub fn align_to_page_size(size: usize) -> usize {
  let page_size = 4096;
  ((size + page_size - 1) / page_size) * page_size
}

pub fn is_power_of_two(n: usize) -> bool {
  n != 0 && (n & (n - 1)) == 0
}

pub fn next_power_of_two(n: usize) -> usize {
  if n == 0 {
    return 1;
  }

  let mut result = n - 1;
  result |= result >> 1;
  result |= result >> 2;
  result |= result >> 4;
  result |= result >> 8;
  result |= result >> 16;
  result |= result >> 32;
  result + 1
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_memory_tracker() {
    let mut tracker = create_memory_tracker();
    let initial = tracker.get_current_memory_usage();

    let _vec = vec![0u8; 1024];
    tracker.update_peak();

    assert!(tracker.memory_increase() >= 1024);
  }

  #[test]
  fn test_memory_pool() {
    let mut pool = create_memory_pool(1024);

    let offset1 = pool.allocate(100).unwrap();
    let offset2 = pool.allocate(200).unwrap();

    assert!(offset1 != offset2);
    assert_eq!(pool.used_size(), 304);

    pool.deallocate(offset1).unwrap();
    assert_eq!(pool.used_size(), 200);

    pool.reset();
    assert_eq!(pool.used_size(), 0);
  }

  #[test]
  fn test_buffer_pool() {
    let mut pool: BufferPool<u8> = create_buffer_pool(1024, 2);

    assert_eq!(pool.total_buffers(), 2);
    assert_eq!(pool.available_buffers(), 2);

    let index1 = pool.acquire().unwrap();
    let index2 = pool.acquire().unwrap();
    let index3 = pool.acquire().unwrap();

    assert_eq!(pool.total_buffers(), 3);
    assert_eq!(pool.available_buffers(), 0);

    pool.release(index1).unwrap();
    assert_eq!(pool.available_buffers(), 1);
  }

  #[test]
  fn test_memory_limiter() {
    let mut limiter = create_memory_limiter(1000);

    assert!(limiter.try_allocate(500).is_ok());
    assert!(limiter.try_allocate(600).is_err());

    assert_eq!(limiter.current_usage(), 500);
    assert_eq!(limiter.available(), 500);
    assert_eq!(limiter.utilization(), 0.5);
  }

  #[test]
  fn test_format_bytes() {
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1536), "1.50 KB");
    assert_eq!(format_bytes(1048576), "1.00 MB");
    assert_eq!(format_bytes(1073741824), "1.00 GB");
  }

  #[test]
  fn test_power_of_two() {
    assert!(is_power_of_two(1));
    assert!(is_power_of_two(2));
    assert!(is_power_of_two(4));
    assert!(is_power_of_two(8));
    assert!(!is_power_of_two(3));
    assert!(!is_power_of_two(5));

    assert_eq!(next_power_of_two(1), 1);
    assert_eq!(next_power_of_two(3), 4);
    assert_eq!(next_power_of_two(5), 8);
    assert_eq!(next_power_of_two(17), 32);
  }
}
