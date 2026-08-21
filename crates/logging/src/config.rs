use ellastic_errors::{
  EllasticError,
  Result,
};
use std::path::{
  Path,
  PathBuf,
};

use crate::LoggingConfig;

#[derive(Debug, Clone)]
pub struct LoggingConfigBuilder {
  config: LoggingConfig,
}

impl LoggingConfigBuilder {
  pub fn new() -> Self {
    Self {
      config: LoggingConfig::default(),
    }
  }

  pub fn level(mut self, level: impl Into<String>) -> Self {
    self.config.level = level.into();
    self
  }

  pub fn file_enabled(mut self, enabled: bool) -> Self {
    self.config.file_enabled = enabled;
    self
  }

  pub fn file_path(mut self, path: Option<PathBuf>) -> Self {
    self.config.file_path = path;
    self
  }

  pub fn max_file_size_mb(mut self, size_mb: u64) -> Self {
    self.config.max_file_size_mb = size_mb;
    self
  }

  pub fn max_files(mut self, max_files: u32) -> Self {
    self.config.max_files = max_files;
    self
  }

  pub fn console_enabled(mut self, enabled: bool) -> Self {
    self.config.console_enabled = enabled;
    self
  }

  pub fn structured(mut self, structured: bool) -> Self {
    self.config.structured = structured;
    self
  }

  pub fn include_timestamps(mut self, include: bool) -> Self {
    self.config.include_timestamps = include;
    self
  }

  pub fn include_module(mut self, include: bool) -> Self {
    self.config.include_module = include;
    self
  }

  pub fn filter(mut self, filter: impl Into<String>) -> Self {
    self.config.filters.push(filter.into());
    self
  }

  pub fn filters(mut self, filters: Vec<String>) -> Self {
    self.config.filters = filters;
    self
  }

  pub fn build(self) -> Result<LoggingConfig> {
    self.validate()?;
    Ok(self.config)
  }

  fn validate(&self) -> Result<()> {
    if self.config.level.is_empty() {
      return Err(EllasticError::ConfigError(
        "Log level cannot be empty".to_string(),
      ));
    }

    let valid_levels = ["trace", "debug", "info", "warn", "error", "off"];
    if !valid_levels.contains(&self.config.level.to_lowercase().as_str()) {
      return Err(EllasticError::ConfigError(format!(
        "Invalid log level: {}. Valid levels are: {}",
        self.config.level,
        valid_levels.join(", ")
      )));
    }

    if self.config.max_file_size_mb == 0 {
      return Err(EllasticError::ConfigError(
        "max_file_size_mb must be greater than 0".to_string(),
      ));
    }

    if self.config.max_files == 0 {
      return Err(EllasticError::ConfigError(
        "max_files must be greater than 0".to_string(),
      ));
    }

    Ok(())
  }
}

impl Default for LoggingConfigBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl LoggingConfig {
  pub fn builder() -> LoggingConfigBuilder {
    LoggingConfigBuilder::new()
  }

  pub fn development() -> Self {
    LoggingConfigBuilder::new()
      .level("debug")
      .console_enabled(true)
      .file_enabled(true)
      .structured(false)
      .include_timestamps(true)
      .include_module(true)
      .build()
      .expect("Failed to create development logging config")
  }

  pub fn production() -> Self {
    LoggingConfigBuilder::new()
      .level("info")
      .console_enabled(false)
      .file_enabled(true)
      .structured(true)
      .include_timestamps(true)
      .include_module(true)
      .max_file_size_mb(50)
      .max_files(10)
      .build()
      .expect("Failed to create production logging config")
  }

  pub fn testing() -> Self {
    LoggingConfigBuilder::new()
      .level("warn")
      .console_enabled(false)
      .file_enabled(false)
      .structured(false)
      .include_timestamps(false)
      .include_module(false)
      .build()
      .expect("Failed to create testing logging config")
  }

  pub fn minimal() -> Self {
    LoggingConfigBuilder::new()
      .level("error")
      .console_enabled(true)
      .file_enabled(false)
      .structured(false)
      .include_timestamps(false)
      .include_module(false)
      .build()
      .expect("Failed to create minimal logging config")
  }

  pub fn verbose() -> Self {
    LoggingConfigBuilder::new()
      .level("trace")
      .console_enabled(true)
      .file_enabled(true)
      .structured(false)
      .include_timestamps(true)
      .include_module(true)
      .build()
      .expect("Failed to create verbose logging config")
  }

  pub fn with_file_logging<P: AsRef<Path>>(mut self, log_file: P) -> Self {
    self.config.file_path = Some(log_file.as_ref().to_path_buf());
    self
  }

  pub fn with_level(mut self, level: impl Into<String>) -> Self {
    self.config.level = level.into();
    self
  }

  pub fn enable_console(mut self, enabled: bool) -> Self {
    self.config.console_enabled = enabled;
    self
  }

  pub fn enable_file(mut self, enabled: bool) -> Self {
    self.config.file_enabled = enabled;
    self
  }

  pub fn structured(mut self, structured: bool) -> Self {
    self.config.structured = structured;
    self
  }

  pub fn validate(&self) -> Result<()> {
    let builder = LoggingConfigBuilder {
      config: self.clone(),
    };
    builder.validate()
  }

  pub fn to_toml(&self) -> Result<String> {
    toml::to_string_pretty(self).map_err(|e| {
      EllasticError::SerializationError(format!("Failed to serialize logging config: {}", e))
    })
  }

  pub fn from_toml(toml_str: &str) -> Result<Self> {
    toml::from_str(toml_str).map_err(|e| {
      EllasticError::DeserializationError(format!("Failed to deserialize logging config: {}", e))
    })
  }

  pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    let content = self.to_toml()?;
    std::fs::write(path, content).map_err(|e| {
      EllasticError::ConfigError(format!("Failed to write logging config file: {}", e))
    })
  }

  pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    let content = std::fs::read_to_string(path).map_err(|e| {
      EllasticError::ConfigError(format!("Failed to read logging config file: {}", e))
    })?;
    Self::from_toml(&content)
  }

  pub fn merge_with(&mut self, other: &LoggingConfig) {
    if other.level != "info" {
      self.config.level = other.level.clone();
    }
    if other.file_enabled {
      self.config.file_enabled = other.file_enabled;
    }
    if other.file_path.is_some() {
      self.config.file_path = other.file_path.clone();
    }
    if other.max_file_size_mb != 10 {
      self.config.max_file_size_mb = other.max_file_size_mb;
    }
    if other.max_files != 5 {
      self.config.max_files = other.max_files;
    }
    if !other.console_enabled {
      self.config.console_enabled = other.console_enabled;
    }
    if other.structured {
      self.config.structured = other.structured;
    }
    if !other.include_timestamps {
      self.config.include_timestamps = other.include_timestamps;
    }
    if !other.include_module {
      self.config.include_module = other.include_module;
    }
    if !other.filters.is_empty() {
      self.config.filters = other.filters.clone();
    }
  }

  pub fn is_development(&self) -> bool {
    self.config.level.to_lowercase() == "debug" || self.config.level.to_lowercase() == "trace"
  }

  pub fn is_production(&self) -> bool {
    self.config.structured && self.config.file_enabled && !self.config.console_enabled
  }

  pub fn is_testing(&self) -> bool {
    !self.config.file_enabled && !self.config.console_enabled
  }

  pub fn is_verbose(&self) -> bool {
    self.config.level.to_lowercase() == "trace"
  }

  pub fn get_effective_level(&self) -> tracing_subscriber::filter::LevelFilter {
    match self.config.level.to_lowercase().as_str() {
      "trace" => tracing_subscriber::filter::LevelFilter::TRACE,
      "debug" => tracing_subscriber::filter::LevelFilter::DEBUG,
      "info" => tracing_subscriber::filter::LevelFilter::INFO,
      "warn" => tracing_subscriber::filter::LevelFilter::WARN,
      "error" => tracing_subscriber::filter::LevelFilter::ERROR,
      "off" => tracing_subscriber::filter::LevelFilter::OFF,
      _ => tracing_subscriber::filter::LevelFilter::INFO,
    }
  }

  pub fn has_any_output(&self) -> bool {
    self.config.console_enabled || self.config.file_enabled
  }

  pub fn requires_file_logging(&self) -> bool {
    self.config.file_enabled
  }

  pub fn requires_console_logging(&self) -> bool {
    self.config.console_enabled
  }

  pub fn get_log_file_path(&self) -> Option<PathBuf> {
    self.config.file_path.clone()
  }

  pub fn get_max_file_size_bytes(&self) -> u64 {
    self.config.max_file_size_mb * 1024 * 1024
  }

  pub fn get_filters(&self) -> &[String] {
    &self.config.filters
  }

  pub fn add_filter(&mut self, filter: impl Into<String>) {
    self.config.filters.push(filter.into());
  }

  pub fn remove_filter(&mut self, filter: &str) -> bool {
    self.config.filters.retain(|f| f != filter);
    self.config.filters.len() < self.config.filters.len() + 1
  }

  pub fn clear_filters(&mut self) {
    self.config.filters.clear();
  }

  pub fn has_filters(&self) -> bool {
    !self.config.filters.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_logging_config_builder() {
    let config = LoggingConfigBuilder::new()
      .level("debug")
      .file_enabled(true)
      .console_enabled(true)
      .structured(false)
      .build()
      .expect("Failed to build logging config");

    assert_eq!(config.config.level, "debug");
    assert!(config.config.file_enabled);
    assert!(config.config.console_enabled);
    assert!(!config.config.structured);
  }

  #[test]
  fn test_logging_config_presets() {
    let dev_config = LoggingConfig::development();
    assert_eq!(dev_config.config.level, "debug");
    assert!(dev_config.config.console_enabled);
    assert!(dev_config.config.file_enabled);

    let prod_config = LoggingConfig::production();
    assert_eq!(prod_config.config.level, "info");
    assert!(!prod_config.config.console_enabled);
    assert!(prod_config.config.file_enabled);
    assert!(prod_config.config.structured);
  }

  #[test]
  fn test_invalid_log_level() {
    let result = LoggingConfigBuilder::new().level("invalid").build();

    assert!(result.is_err());
  }

  #[test]
  fn test_config_serialization() {
    let config = LoggingConfig::development();
    let toml_str = config.to_toml().expect("Failed to serialize");
    let restored = LoggingConfig::from_toml(&toml_str).expect("Failed to deserialize");

    assert_eq!(config.config.level, restored.config.level);
    assert_eq!(
      config.config.console_enabled,
      restored.config.console_enabled
    );
    assert_eq!(config.config.file_enabled, restored.config.file_enabled);
  }
}
