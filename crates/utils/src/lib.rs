pub mod filesystem;
pub mod hashing;
pub mod memory;
pub mod path;
pub mod random;
pub mod timing;

pub use filesystem::*;
pub use hashing::*;
pub use memory::*;
pub use path::*;
pub use random::*;
pub use timing::*;

use ellastic_errors::{
  EllasticError,
  Result,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SystemInfo {
  pub cpu_count: usize,
  pub total_memory_mb: u64,
  pub available_memory_mb: u64,
  pub disk_space_gb: u64,
  pub available_disk_space_gb: u64,
}

impl SystemInfo {
  pub fn gather() -> Result<Self> {
    let cpu_count = num_cpus::get();

    let total_memory_mb = Self::get_total_memory_mb()?;
    let available_memory_mb = Self::get_available_memory_mb()?;

    let disk_space_gb = Self::get_disk_space_gb()?;
    let available_disk_space_gb = Self::get_available_disk_space_gb()?;

    Ok(Self {
      cpu_count,
      total_memory_mb,
      available_memory_mb,
      disk_space_gb,
      available_disk_space_gb,
    })
  }

  fn get_total_memory_mb() -> Result<u64> {
    #[cfg(target_os = "linux")]
    {
      let mut info = libc::sysinfo {
        uptime: 0,
        loads: [0; 3],
        totalram: 0,
        freeram: 0,
        sharedram: 0,
        bufferram: 0,
        totalswap: 0,
        freeswap: 0,
        procs: 0,
        totalhigh: 0,
        freehigh: 0,
        mem_unit: 0,
        _f: [0; 0],
      };

      unsafe {
        if libc::sysinfo(&mut info) != 0 {
          return Err(EllasticError::SystemError(
            "Failed to get system info".to_string(),
          ));
        }
        Ok((info.totalram * info.mem_unit) / (1024 * 1024))
      }
    }

    #[cfg(target_os = "macos")]
    {
      let mut memory = 0u64;
      let mut size = std::mem::size_of::<u64>();
      let name = std::ffi::CString::new("hw.memsize").unwrap();
      let result = unsafe {
        libc::sysctlbyname(
          name.as_ptr(),
          (&mut memory as *mut u64).cast(),
          &mut size,
          std::ptr::null_mut(),
          0,
        )
      };
      if result != 0 {
        return Err(EllasticError::SystemError("Failed to get system info".to_string()));
      }
      Ok(memory / (1024 * 1024))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
      Ok(8192)
    }
  }

  fn get_available_memory_mb() -> Result<u64> {
    #[cfg(target_os = "linux")]
    {
      let mut info = libc::sysinfo {
        uptime: 0,
        loads: [0; 3],
        totalram: 0,
        freeram: 0,
        sharedram: 0,
        bufferram: 0,
        totalswap: 0,
        freeswap: 0,
        procs: 0,
        totalhigh: 0,
        freehigh: 0,
        mem_unit: 0,
        _f: [0; 0],
      };

      unsafe {
        if libc::sysinfo(&mut info) != 0 {
          return Err(EllasticError::SystemError(
            "Failed to get system info".to_string(),
          ));
        }
        Ok((info.freeram * info.mem_unit) / (1024 * 1024))
      }
    }

    #[cfg(target_os = "macos")]
    {
      Self::get_total_memory_mb()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
      Ok(4096)
    }
  }

  fn get_disk_space_gb() -> Result<u64> {
    let current_dir = std::env::current_dir().map_err(|e| EllasticError::IoError(e.to_string()))?;

    let metadata =
      std::fs::metadata(&current_dir).map_err(|e| EllasticError::IoError(e.to_string()))?;

    let total_space = metadata.len() / (1024 * 1024 * 1024);
    Ok(total_space.max(1))
  }

  fn get_available_disk_space_gb() -> Result<u64> {
    let current_dir = std::env::current_dir().map_err(|e| EllasticError::IoError(e.to_string()))?;

    let statvfs = unsafe {
      let path_c = std::ffi::CString::new(current_dir.to_string_lossy().as_bytes())
        .map_err(|_| EllasticError::IoError("Failed to create CString".to_string()))?;

      let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
      if libc::statvfs(path_c.as_ptr(), stat.as_mut_ptr()) != 0 {
        return Err(EllasticError::IoError(
          "Failed to get disk info".to_string(),
        ));
      }
      stat.assume_init()
    };

    let available_space = (statvfs.f_bavail as u64 * statvfs.f_frsize) / (1024 * 1024 * 1024);
    Ok(available_space as u64)
  }

  pub fn can_allocate_memory(&self, required_mb: u64) -> bool {
    self.available_memory_mb >= required_mb
  }

  pub fn can_store_data(&self, required_gb: u64) -> bool {
    self.available_disk_space_gb >= required_gb
  }

  pub fn get_optimal_thread_count(&self, memory_per_thread_mb: u64) -> usize {
    let memory_limited_threads = (self.available_memory_mb / memory_per_thread_mb).max(1);
    std::cmp::min(self.cpu_count, memory_limited_threads as usize)
  }

  pub fn get_processing_recommendations(&self) -> ProcessingRecommendations {
    let recommended_threads = self.get_optimal_thread_count(512);
    let recommended_chunk_size = if self.total_memory_mb > 8192 {
      2 * 1024 * 1024
    } else if self.total_memory_mb > 4096 {
      1024 * 1024
    } else {
      512 * 1024
    };

    let recommended_batch_size = if self.total_memory_mb > 8192 {
      50
    } else if self.total_memory_mb > 4096 {
      25
    } else {
      10
    };

    ProcessingRecommendations {
      recommended_threads,
      recommended_chunk_size,
      recommended_batch_size,
      max_concurrent_operations: self.cpu_count,
      memory_pressure: self.available_memory_mb < (self.total_memory_mb / 4),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProcessingRecommendations {
  pub recommended_threads: usize,
  pub recommended_chunk_size: usize,
  pub recommended_batch_size: usize,
  pub max_concurrent_operations: usize,
  pub memory_pressure: bool,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
  pub operation_count: u64,
  pub total_duration_ms: u64,
  pub average_duration_ms: f64,
  pub min_duration_ms: u64,
  pub max_duration_ms: u64,
  pub success_count: u64,
  pub error_count: u64,
  pub throughput_ops_per_second: f64,
}

impl PerformanceMetrics {
  pub fn new() -> Self {
    Self {
      operation_count: 0,
      total_duration_ms: 0,
      average_duration_ms: 0.0,
      min_duration_ms: u64::MAX,
      max_duration_ms: 0,
      success_count: 0,
      error_count: 0,
      throughput_ops_per_second: 0.0,
    }
  }

  pub fn record_operation(&mut self, duration_ms: u64, success: bool) {
    self.operation_count += 1;
    self.total_duration_ms += duration_ms;
    self.min_duration_ms = self.min_duration_ms.min(duration_ms);
    self.max_duration_ms = self.max_duration_ms.max(duration_ms);

    if success {
      self.success_count += 1;
    } else {
      self.error_count += 1;
    }

    self.average_duration_ms = self.total_duration_ms as f64 / self.operation_count as f64;

    if self.total_duration_ms > 0 {
      self.throughput_ops_per_second =
        (self.operation_count as f64 * 1000.0) / self.total_duration_ms as f64;
    }
  }

  pub fn success_rate(&self) -> f64 {
    if self.operation_count == 0 {
      0.0
    } else {
      self.success_count as f64 / self.operation_count as f64
    }
  }

  pub fn error_rate(&self) -> f64 {
    if self.operation_count == 0 {
      0.0
    } else {
      self.error_count as f64 / self.operation_count as f64
    }
  }

  pub fn reset(&mut self) {
    *self = Self::new();
  }

  pub fn merge(&mut self, other: &PerformanceMetrics) {
    self.operation_count += other.operation_count;
    self.total_duration_ms += other.total_duration_ms;
    self.min_duration_ms = self.min_duration_ms.min(other.min_duration_ms);
    self.max_duration_ms = self.max_duration_ms.max(other.max_duration_ms);
    self.success_count += other.success_count;
    self.error_count += other.error_count;

    if self.operation_count > 0 {
      self.average_duration_ms = self.total_duration_ms as f64 / self.operation_count as f64;
      self.throughput_ops_per_second =
        (self.operation_count as f64 * 1000.0) / self.total_duration_ms as f64;
    }
  }
}

impl Default for PerformanceMetrics {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct ResourceMonitor {
  initial_system_info: SystemInfo,
  metrics: PerformanceMetrics,
  start_time: std::time::Instant,
}

impl ResourceMonitor {
  pub fn new() -> Result<Self> {
    let initial_system_info = SystemInfo::gather()?;
    Ok(Self {
      initial_system_info,
      metrics: PerformanceMetrics::new(),
      start_time: std::time::Instant::now(),
    })
  }

  pub fn record_operation(&mut self, duration_ms: u64, success: bool) {
    self.metrics.record_operation(duration_ms, success);
  }

  pub fn get_current_system_info(&self) -> Result<SystemInfo> {
    SystemInfo::gather()
  }

  pub fn get_resource_usage(&self) -> Result<ResourceUsage> {
    let current_info = self.get_current_system_info()?;

    Ok(ResourceUsage {
      memory_used_mb: current_info.total_memory_mb - current_info.available_memory_mb,
      memory_available_mb: current_info.available_memory_mb,
      disk_used_gb: current_info.disk_space_gb - current_info.available_disk_space_gb,
      disk_available_gb: current_info.available_disk_space_gb,
      cpu_utilization: self.estimate_cpu_utilization(),
      uptime_seconds: self.start_time.elapsed().as_secs(),
    })
  }

  pub fn get_metrics(&self) -> &PerformanceMetrics {
    &self.metrics
  }

  pub fn get_initial_system_info(&self) -> &SystemInfo {
    &self.initial_system_info
  }

  fn estimate_cpu_utilization(&self) -> f64 {
    let elapsed_ms = self.start_time.elapsed().as_millis() as u64;
    if elapsed_ms == 0 {
      return 0.0;
    }

    let total_processing_time_ms = self.metrics.total_duration_ms;
    let cpu_cores = self.initial_system_info.cpu_count as u64;

    (total_processing_time_ms as f64 / (elapsed_ms as f64 * cpu_cores as f64)).min(1.0)
  }

  pub fn get_performance_report(&self) -> Result<PerformanceReport> {
    let current_usage = self.get_resource_usage()?;

    Ok(PerformanceReport {
      metrics: self.metrics.clone(),
      resource_usage: current_usage,
      system_info: self.get_current_system_info()?,
      uptime_seconds: self.start_time.elapsed().as_secs(),
      recommendations: self
        .get_current_system_info()?
        .get_processing_recommendations(),
    })
  }
}

impl Default for ResourceMonitor {
  fn default() -> Self {
    Self::new().expect("Failed to create resource monitor")
  }
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
  pub memory_used_mb: u64,
  pub memory_available_mb: u64,
  pub disk_used_gb: u64,
  pub disk_available_gb: u64,
  pub cpu_utilization: f64,
  pub uptime_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
  pub metrics: PerformanceMetrics,
  pub resource_usage: ResourceUsage,
  pub system_info: SystemInfo,
  pub uptime_seconds: u64,
  pub recommendations: ProcessingRecommendations,
}

pub fn get_system_info() -> Result<SystemInfo> {
  SystemInfo::gather()
}

pub fn create_resource_monitor() -> Result<ResourceMonitor> {
  ResourceMonitor::new()
}

pub fn estimate_memory_usage(data_size_bytes: u64, overhead_factor: f64) -> u64 {
  ((data_size_bytes as f64 * overhead_factor) / (1024.0 * 1024.0)) as u64
}

pub fn estimate_processing_time(
  data_size_bytes: u64,
  throughput_bytes_per_second: u64,
) -> std::time::Duration {
  let seconds = data_size_bytes as f64 / throughput_bytes_per_second as f64;
  std::time::Duration::from_secs_f64(seconds)
}

pub fn calculate_optimal_chunk_size(
  total_size_bytes: u64,
  available_memory_mb: u64,
  min_chunk_size: usize,
  max_chunk_size: usize,
) -> usize {
  let available_memory_bytes = available_memory_mb * 1024 * 1024;
  let max_safe_chunk = (available_memory_bytes / 4) as usize;

  let memory_limited_chunk = max_safe_chunk.clamp(min_chunk_size, max_chunk_size);
  let size_optimal_chunk =
    (total_size_bytes / 100).clamp(min_chunk_size as u64, max_chunk_size as u64) as usize;

  std::cmp::min(memory_limited_chunk, size_optimal_chunk)
}

pub fn validate_system_requirements(min_memory_mb: u64, min_disk_gb: u64) -> Result<()> {
  let system_info = get_system_info()?;

  if system_info.total_memory_mb < min_memory_mb {
    return Err(EllasticError::InsufficientMemory);
  }

  if system_info.disk_space_gb < min_disk_gb {
    return Err(EllasticError::InsufficientDiskSpace);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_performance_metrics() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record_operation(100, true);
    metrics.record_operation(200, false);
    metrics.record_operation(150, true);

    assert_eq!(metrics.operation_count, 3);
    assert_eq!(metrics.success_count, 2);
    assert_eq!(metrics.error_count, 1);
    assert_eq!(metrics.min_duration_ms, 100);
    assert_eq!(metrics.max_duration_ms, 200);
    assert_eq!(metrics.total_duration_ms, 450);
    assert!((metrics.average_duration_ms - 150.0).abs() < 0.01);
    assert!((metrics.success_rate() - 0.6667).abs() < 0.01);
    assert!((metrics.error_rate() - 0.3333).abs() < 0.01);
  }

  #[test]
  fn test_calculate_optimal_chunk_size() {
    let chunk_size = calculate_optimal_chunk_size(1_000_000_000, 1024, 1024, 10_485_760);

    assert!(chunk_size >= 1024);
    assert!(chunk_size <= 10_485_760);
  }
}
