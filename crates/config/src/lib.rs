use ellastic_errors::{
  EllasticError,
  Result,
};
use std::collections::HashMap;
use std::path::{
  Path,
  PathBuf,
};

pub mod manager;
pub mod profiles;
pub mod settings;

pub use manager::*;
pub use profiles::*;
pub use settings::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
  pub application: ApplicationConfig,
  pub processing: ProcessingConfig,
  pub ui: UIConfig,
  pub export: ExportConfig,
  pub logging: LoggingConfig,
  pub lua: LuaConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationConfig {
  pub name: String,
  pub version: String,
  pub data_directory: Option<PathBuf>,
  pub temp_directory: Option<PathBuf>,
  pub max_memory_usage_mb: u64,
  pub max_concurrent_tasks: usize,
  pub auto_save_interval_seconds: u64,
  pub backup_enabled: bool,
  pub backup_retention_days: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessingConfig {
  pub default_threads: usize,
  pub chunk_size: usize,
  pub max_batch_size: usize,
  pub cache_enabled: bool,
  pub cache_size_mb: u64,
  pub preview_quality: u8,
  pub deterministic_mode: bool,
  pub default_seed: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UIConfig {
  pub theme: String,
  pub window_width: u32,
  pub window_height: u32,
  pub window_maximized: bool,
  pub font_size: f32,
  pub auto_save_layout: bool,
  pub show_advanced_options: bool,
  pub preview_update_interval_ms: u64,
  pub panel_layout: HashMap<String, PanelConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PanelConfig {
  pub visible: bool,
  pub width: u32,
  pub height: u32,
  pub docked: bool,
  pub position: Option<(i32, i32)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportConfig {
  pub default_format: String,
  pub default_quality: u8,
  pub output_directory: Option<PathBuf>,
  pub filename_template: String,
  pub preserve_metadata: bool,
  pub create_subdirectories: bool,
  pub overwrite_existing: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoggingConfig {
  pub level: String,
  pub file_enabled: bool,
  pub file_path: Option<PathBuf>,
  pub max_file_size_mb: u64,
  pub max_files: u32,
  pub console_enabled: bool,
  pub structured: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LuaConfig {
  pub script_directory: Option<PathBuf>,
  pub auto_reload_scripts: bool,
  pub sandbox_enabled: bool,
  pub max_execution_time_ms: u64,
  pub max_memory_usage_mb: u64,
  pub allowed_modules: Vec<String>,
  pub forbidden_functions: Vec<String>,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      application: ApplicationConfig::default(),
      processing: ProcessingConfig::default(),
      ui: UIConfig::default(),
      export: ExportConfig::default(),
      logging: LoggingConfig::default(),
      lua: LuaConfig::default(),
    }
  }
}

impl Default for ApplicationConfig {
  fn default() -> Self {
    Self {
      name: "Ellastic".to_string(),
      version: "0.1.0".to_string(),
      data_directory: None,
      temp_directory: None,
      max_memory_usage_mb: 2048,
      max_concurrent_tasks: 4,
      auto_save_interval_seconds: 300,
      backup_enabled: true,
      backup_retention_days: 30,
    }
  }
}

impl Default for ProcessingConfig {
  fn default() -> Self {
    Self {
      default_threads: num_cpus::get(),
      chunk_size: 1024 * 1024,
      max_batch_size: 100,
      cache_enabled: true,
      cache_size_mb: 512,
      preview_quality: 75,
      deterministic_mode: false,
      default_seed: None,
    }
  }
}

impl Default for UIConfig {
  fn default() -> Self {
    Self {
      theme: "dark".to_string(),
      window_width: 1920,
      window_height: 1080,
      window_maximized: false,
      font_size: 14.0,
      auto_save_layout: true,
      show_advanced_options: false,
      preview_update_interval_ms: 100,
      panel_layout: HashMap::new(),
    }
  }
}

impl Default for ExportConfig {
  fn default() -> Self {
    Self {
      default_format: "png".to_string(),
      default_quality: 90,
      output_directory: None,
      filename_template: "{name}_{timestamp}.{ext}".to_string(),
      preserve_metadata: true,
      create_subdirectories: false,
      overwrite_existing: false,
    }
  }
}

impl Default for LoggingConfig {
  fn default() -> Self {
    Self {
      level: "info".to_string(),
      file_enabled: true,
      file_path: None,
      max_file_size_mb: 10,
      max_files: 5,
      console_enabled: true,
      structured: false,
    }
  }
}

impl Default for LuaConfig {
  fn default() -> Self {
    Self {
      script_directory: None,
      auto_reload_scripts: true,
      sandbox_enabled: true,
      max_execution_time_ms: 5000,
      max_memory_usage_mb: 256,
      allowed_modules: vec![
        "string".to_string(),
        "math".to_string(),
        "table".to_string(),
        "os".to_string(),
      ],
      forbidden_functions: vec![
        "dofile".to_string(),
        "loadfile".to_string(),
        "require".to_string(),
        "debug".to_string(),
      ],
    }
  }
}

pub trait ConfigValidator {
  fn validate(&self) -> Result<()>;
}

impl ConfigValidator for AppConfig {
  fn validate(&self) -> Result<()> {
    if self.application.max_memory_usage_mb == 0 {
      return Err(EllasticError::ConfigError(
        "max_memory_usage_mb must be greater than 0".to_string(),
      ));
    }

    if self.application.max_concurrent_tasks == 0 {
      return Err(EllasticError::ConfigError(
        "max_concurrent_tasks must be greater than 0".to_string(),
      ));
    }

    if self.processing.default_threads == 0 {
      return Err(EllasticError::ConfigError(
        "default_threads must be greater than 0".to_string(),
      ));
    }

    if self.processing.chunk_size == 0 {
      return Err(EllasticError::ConfigError(
        "chunk_size must be greater than 0".to_string(),
      ));
    }

    if self.ui.window_width == 0 || self.ui.window_height == 0 {
      return Err(EllasticError::ConfigError(
        "window dimensions must be greater than 0".to_string(),
      ));
    }

    if self.export.default_quality > 100 {
      return Err(EllasticError::ConfigError(
        "default_quality must be between 0 and 100".to_string(),
      ));
    }

    Ok(())
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigProfile {
  pub name: String,
  pub description: Option<String>,
  pub config: AppConfig,
  pub created_at: std::time::SystemTime,
  pub modified_at: std::time::SystemTime,
}

impl ConfigProfile {
  pub fn new(name: String, config: AppConfig) -> Self {
    let now = std::time::SystemTime::now();
    Self {
      name,
      description: None,
      config,
      created_at: now,
      modified_at: now,
    }
  }

  pub fn with_description(mut self, description: String) -> Self {
    self.description = Some(description);
    self
  }

  pub fn update_config(&mut self, config: AppConfig) {
    self.config = config;
    self.modified_at = std::time::SystemTime::now();
  }
}

pub fn get_default_config_path() -> Result<PathBuf> {
  let dirs = directories::ProjectDirs::from("com", "ellastic", "ellastic")
    .ok_or_else(|| EllasticError::ConfigError("Failed to get project directories".to_string()))?;

  let config_dir = dirs.config_dir();
  std::fs::create_dir_all(config_dir)
    .map_err(|e| EllasticError::ConfigError(format!("Failed to create config directory: {}", e)))?;

  Ok(config_dir.join("config.toml"))
}

pub fn get_default_data_path() -> Result<PathBuf> {
  let dirs = directories::ProjectDirs::from("com", "ellastic", "ellastic")
    .ok_or_else(|| EllasticError::ConfigError("Failed to get project directories".to_string()))?;

  let data_dir = dirs.data_dir();
  std::fs::create_dir_all(data_dir)
    .map_err(|e| EllasticError::ConfigError(format!("Failed to create data directory: {}", e)))?;

  Ok(data_dir.to_path_buf())
}

pub fn get_default_cache_path() -> Result<PathBuf> {
  let dirs = directories::ProjectDirs::from("com", "ellastic", "ellastic")
    .ok_or_else(|| EllasticError::ConfigError("Failed to get project directories".to_string()))?;

  let cache_dir = dirs.cache_dir();
  std::fs::create_dir_all(cache_dir)
    .map_err(|e| EllasticError::ConfigError(format!("Failed to create cache directory: {}", e)))?;

  Ok(cache_dir.to_path_buf())
}
