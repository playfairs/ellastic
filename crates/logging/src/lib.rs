use ellastic_errors::{
  EllasticError,
  Result,
};
use std::path::{
  Path,
  PathBuf,
};

pub mod config;
pub mod filters;
pub mod formatters;

pub use config::*;
pub use filters::*;
pub use formatters::*;

#[derive(Debug, Clone)]
pub struct Logger {
  config: LoggingConfig,
  _guards: Vec<tracing_appender::non_blocking::WorkerGuard>,
}

impl Logger {
  pub fn new() -> Self {
    Self::with_config(LoggingConfig::default())
  }

  pub fn with_config(config: LoggingConfig) -> Self {
    let mut guards = Vec::new();

    if config.file_enabled {
      if let Ok(file_appender) = Self::create_file_appender(&config) {
        guards.push(file_appender);
      }
    }

    Self {
      config,
      _guards: guards,
    }
  }

  pub fn init(self) -> Result<()> {
    let subscriber = tracing_subscriber::registry();

    let subscriber = if self.config.console_enabled {
      subscriber.with(Self::create_console_layer(&self.config)?)
    } else {
      subscriber
    };

    let subscriber = if self.config.file_enabled {
      subscriber.with(Self::create_file_layer(&self.config)?)
    } else {
      subscriber
    };

    subscriber.with(tracing_error::ErrorLayer::default()).init();

    tracing::info!("Logger initialized with level: {}", self.config.level);
    Ok(())
  }

  pub fn config(&self) -> &LoggingConfig {
    &self.config
  }

  pub fn update_config(&mut self, config: LoggingConfig) -> Result<()> {
    self.config = config;
    self.reinit()?;
    Ok(())
  }

  fn reinit(&mut self) -> Result<()> {
    self._guards.clear();

    if self.config.file_enabled {
      if let Ok(file_appender) = Self::create_file_appender(&self.config) {
        self._guards.push(file_appender);
      }
    }

    Ok(())
  }

  fn create_file_appender(
    config: &LoggingConfig,
  ) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let log_dir = Self::get_log_directory()?;
    std::fs::create_dir_all(&log_dir)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to create log directory: {}", e)))?;

    let log_path = if let Some(ref file_path) = config.file_path {
      log_dir.join(file_path)
    } else {
      log_dir.join("ellastic.log")
    };

    let file_appender = tracing_appender::rolling::daily(&log_dir, "ellastic");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing::info!("File logging enabled: {}", log_path.display());
    Ok(guard)
  }

  fn create_console_layer(
    config: &LoggingConfig,
  ) -> Result<tracing_subscriber::fmt::Layer<tracing_subscriber::Registry>> {
    let level = Self::parse_level(&config.level)?;

    let layer = tracing_subscriber::fmt::layer()
      .with_target(config.include_module)
      .with_thread_ids(true)
      .with_thread_names(true);

    let layer = if config.structured {
      layer.json()
    } else {
      layer.pretty()
    };

    let layer = if config.include_timestamps {
      layer.with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
    } else {
      layer.without_time()
    };

    Ok(layer.with_filter(level))
  }

  fn create_file_layer(
    config: &LoggingConfig,
  ) -> Result<tracing_subscriber::fmt::Layer<tracing_subscriber::Registry>> {
    let level = Self::parse_level(&config.level)?;

    let layer = tracing_subscriber::fmt::layer()
      .with_target(config.include_module)
      .with_thread_ids(true)
      .with_thread_names(true);

    let layer = if config.structured {
      layer.json()
    } else {
      layer.compact()
    };

    let layer = if config.include_timestamps {
      layer.with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
    } else {
      layer.without_time()
    };

    Ok(layer.with_filter(level))
  }

  fn parse_level(level_str: &str) -> Result<tracing::level::LevelFilter> {
    match level_str.to_lowercase().as_str() {
      "trace" => Ok(tracing::level::LevelFilter::TRACE),
      "debug" => Ok(tracing::level::LevelFilter::DEBUG),
      "info" => Ok(tracing::level::LevelFilter::INFO),
      "warn" => Ok(tracing::level::LevelFilter::WARN),
      "error" => Ok(tracing::level::LevelFilter::ERROR),
      "off" => Ok(tracing::level::LevelFilter::OFF),
      _ => Err(EllasticError::ConfigError(format!(
        "Invalid log level: {}",
        level_str
      ))),
    }
  }

  fn get_log_directory() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "ellastic", "ellastic")
      .ok_or_else(|| EllasticError::ConfigError("Failed to get project directories".to_string()))?;

    let log_dir = dirs.data_dir().join("logs");
    std::fs::create_dir_all(&log_dir)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to create log directory: {}", e)))?;

    Ok(log_dir)
  }

  pub fn set_global_level(level: &str) -> Result<()> {
    let level_filter = Self::parse_level(level)?;
    tracing::subscriber::set_global_default(
      tracing_subscriber::registry()
        .with(level_filter)
        .with(tracing_error::ErrorLayer::default()),
    )
    .map_err(|e| EllasticError::ConfigError(format!("Failed to set global log level: {}", e)))?;
    Ok(())
  }

  pub fn create_span<T: Into<tracing::Id>>(id: T) -> tracing::Span {
    tracing::info_span!("operation", id = tracing::field::display(id.into()))
  }

  pub fn create_media_span(media_id: &str, operation: &str) -> tracing::Span {
    tracing::info_span!("media_operation", media_id, operation)
  }

  pub fn create_processing_span(pipeline_id: &str, task_id: &str) -> tracing::Span {
    tracing::info_span!("processing", pipeline_id, task_id)
  }

  pub fn create_lua_span(script_name: &str) -> tracing::Span {
    tracing::info_span!("lua_execution", script_name)
  }

  pub fn create_export_span(format: &str, output_path: &str) -> tracing::Span {
    tracing::info_span!("export", format, output_path)
  }
}

impl Default for Logger {
  fn default() -> Self {
    Self::new()
  }
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
  pub include_timestamps: bool,
  pub include_module: bool,
  pub filters: Vec<String>,
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
      include_timestamps: true,
      include_module: true,
      filters: Vec::new(),
    }
  }
}

pub fn init_logger() -> Result<()> {
  let logger = Logger::new();
  logger.init()
}

pub fn init_logger_with_config(config: LoggingConfig) -> Result<()> {
  let logger = Logger::with_config(config);
  logger.init()
}

pub fn get_default_log_config() -> LoggingConfig {
  LoggingConfig::default()
}

#[macro_export]
macro_rules! log_media_operation {
  ($media_id:expr, $operation:expr, $result:expr) => {
    let _span = $crate::Logger::create_media_span($media_id, $operation);
    match $result {
      Ok(_) => tracing::info!("Media operation completed successfully"),
      Err(e) => tracing::error!("Media operation failed: {}", e),
    }
  };
}

#[macro_export]
macro_rules! log_processing_operation {
  ($pipeline_id:expr, $task_id:expr, $result:expr) => {
    let _span = $crate::Logger::create_processing_span($pipeline_id, $task_id);
    match $result {
      Ok(_) => tracing::info!("Processing completed successfully"),
      Err(e) => tracing::error!("Processing failed: {}", e),
    }
  };
}

#[macro_export]
macro_rules! log_lua_execution {
  ($script_name:expr, $result:expr) => {
    let _span = $crate::Logger::create_lua_span($script_name);
    match $result {
      Ok(_) => tracing::info!("Lua script executed successfully"),
      Err(e) => tracing::error!("Lua script execution failed: {}", e),
    }
  };
}

#[macro_export]
macro_rules! log_export_operation {
  ($format:expr, $output_path:expr, $result:expr) => {
    let _span = $crate::Logger::create_export_span($format, $output_path);
    match $result {
      Ok(_) => tracing::info!("Export completed successfully"),
      Err(e) => tracing::error!("Export failed: {}", e),
    }
  };
}
