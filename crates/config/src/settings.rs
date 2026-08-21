use ellastic_errors::{
  EllasticError,
  Result,
};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessingSettings {
  pub thread_count: usize,
  pub memory_limit_mb: u64,
  pub cache_size_mb: u64,
  pub chunk_size: usize,
  pub batch_size: usize,
  pub timeout_seconds: u64,
  pub auto_save: bool,
  pub auto_backup: bool,
  pub preview_quality: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UISettings {
  pub theme: String,
  pub font_size: f32,
  pub window_width: u32,
  pub window_height: u32,
  pub panel_layout: HashMap<String, PanelSettings>,
  pub shortcuts: HashMap<String, String>,
  pub auto_save_layout: bool,
  pub show_advanced_options: bool,
  pub preview_update_interval_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PanelSettings {
  pub visible: bool,
  pub width: u32,
  pub height: u32,
  pub docked: bool,
  pub position: Option<(i32, i32)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportSettings {
  pub default_format: String,
  pub default_quality: u8,
  pub output_directory: Option<String>,
  pub filename_template: String,
  pub preserve_metadata: bool,
  pub create_subdirectories: bool,
  pub overwrite_existing: bool,
  pub compression_level: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LuaSettings {
  pub script_directory: Option<String>,
  pub auto_reload: bool,
  pub sandbox_enabled: bool,
  pub max_execution_time_ms: u64,
  pub max_memory_mb: u64,
  pub allowed_modules: Vec<String>,
  pub forbidden_functions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoggingSettings {
  pub level: String,
  pub file_enabled: bool,
  pub file_path: Option<String>,
  pub max_file_size_mb: u64,
  pub max_files: u32,
  pub console_enabled: bool,
  pub structured: bool,
  pub include_timestamps: bool,
  pub include_module: bool,
}

#[derive(Debug, Clone)]
pub struct SettingsManager {
  processing: ProcessingSettings,
  ui: UISettings,
  export: ExportSettings,
  lua: LuaSettings,
  logging: LoggingSettings,
  custom_settings: HashMap<String, serde_json::Value>,
}

impl SettingsManager {
  pub fn new() -> Self {
    Self {
      processing: ProcessingSettings::default(),
      ui: UISettings::default(),
      export: ExportSettings::default(),
      lua: LuaSettings::default(),
      logging: LoggingSettings::default(),
      custom_settings: HashMap::new(),
    }
  }

  pub fn processing(&self) -> &ProcessingSettings {
    &self.processing
  }

  pub fn processing_mut(&mut self) -> &mut ProcessingSettings {
    &mut self.processing
  }

  pub fn ui(&self) -> &UISettings {
    &self.ui
  }

  pub fn ui_mut(&mut self) -> &mut UISettings {
    &mut self.ui
  }

  pub fn export(&self) -> &ExportSettings {
    &self.export
  }

  pub fn export_mut(&mut self) -> &mut ExportSettings {
    &mut self.export
  }

  pub fn lua(&self) -> &LuaSettings {
    &self.lua
  }

  pub fn lua_mut(&mut self) -> &mut LuaSettings {
    &mut self.lua
  }

  pub fn logging(&self) -> &LoggingSettings {
    &self.logging
  }

  pub fn logging_mut(&mut self) -> &mut LoggingSettings {
    &mut self.logging
  }

  pub fn get_custom_setting(&self, key: &str) -> Option<&serde_json::Value> {
    self.custom_settings.get(key)
  }

  pub fn set_custom_setting(&mut self, key: String, value: serde_json::Value) {
    self.custom_settings.insert(key, value);
  }

  pub fn remove_custom_setting(&mut self, key: &str) -> Option<serde_json::Value> {
    self.custom_settings.remove(key)
  }

  pub fn list_custom_settings(&self) -> Vec<&String> {
    self.custom_settings.keys().collect()
  }

  pub fn validate(&self) -> Result<()> {
    if self.processing.thread_count == 0 {
      return Err(EllasticError::ConfigError(
        "thread_count must be greater than 0".to_string(),
      ));
    }

    if self.processing.memory_limit_mb == 0 {
      return Err(EllasticError::ConfigError(
        "memory_limit_mb must be greater than 0".to_string(),
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

    if self.lua.max_execution_time_ms == 0 {
      return Err(EllasticError::ConfigError(
        "max_execution_time_ms must be greater than 0".to_string(),
      ));
    }

    if self.lua.max_memory_mb == 0 {
      return Err(EllasticError::ConfigError(
        "max_memory_mb must be greater than 0".to_string(),
      ));
    }

    Ok(())
  }

  pub fn reset_to_defaults(&mut self) {
    self.processing = ProcessingSettings::default();
    self.ui = UISettings::default();
    self.export = ExportSettings::default();
    self.lua = LuaSettings::default();
    self.logging = LoggingSettings::default();
    self.custom_settings.clear();
  }

  pub fn export_settings(&self) -> Result<String> {
    let settings = SettingsExport {
      processing: self.processing.clone(),
      ui: self.ui.clone(),
      export: self.export.clone(),
      lua: self.lua.clone(),
      logging: self.logging.clone(),
      custom_settings: self.custom_settings.clone(),
    };

    toml::to_string_pretty(&settings)
      .map_err(|e| EllasticError::SerializationError(format!("Failed to export settings: {}", e)))
  }

  pub fn import_settings(&mut self, settings_str: &str) -> Result<()> {
    let settings: SettingsExport = toml::from_str(settings_str).map_err(|e| {
      EllasticError::DeserializationError(format!("Failed to import settings: {}", e))
    })?;

    self.processing = settings.processing;
    self.ui = settings.ui;
    self.export = settings.export;
    self.lua = settings.lua;
    self.logging = settings.logging;
    self.custom_settings = settings.custom_settings;

    self.validate()?;
    Ok(())
  }

  pub fn merge_settings(&mut self, other: &SettingsManager) -> Result<()> {
    self.processing = other.processing.clone();
    self.ui = other.ui.clone();
    self.export = other.export.clone();
    self.lua = other.lua.clone();
    self.logging = other.logging.clone();

    for (key, value) in &other.custom_settings {
      self.custom_settings.insert(key.clone(), value.clone());
    }

    self.validate()?;
    Ok(())
  }

  pub fn get_setting_path(&self, category: &str, key: &str) -> Option<&serde_json::Value> {
    match category {
      "processing" => self.get_processing_setting(key),
      "ui" => self.get_ui_setting(key),
      "export" => self.get_export_setting(key),
      "lua" => self.get_lua_setting(key),
      "logging" => self.get_logging_setting(key),
      _ => self.get_custom_setting(key),
    }
  }

  pub fn set_setting_path(
    &mut self,
    category: &str,
    key: String,
    value: serde_json::Value,
  ) -> Result<()> {
    match category {
      "processing" => self.set_processing_setting(key, value),
      "ui" => self.set_ui_setting(key, value),
      "export" => self.set_export_setting(key, value),
      "lua" => self.set_lua_setting(key, value),
      "logging" => self.set_logging_setting(key, value),
      _ => {
        self.set_custom_setting(key, value);
        Ok(())
      }
    }
  }

  fn get_processing_setting(&self, key: &str) -> Option<&serde_json::Value> {
    match key {
      "thread_count" => Some(&serde_json::json!(self.processing.thread_count)),
      "memory_limit_mb" => Some(&serde_json::json!(self.processing.memory_limit_mb)),
      "cache_size_mb" => Some(&serde_json::json!(self.processing.cache_size_mb)),
      "chunk_size" => Some(&serde_json::json!(self.processing.chunk_size)),
      "batch_size" => Some(&serde_json::json!(self.processing.batch_size)),
      "timeout_seconds" => Some(&serde_json::json!(self.processing.timeout_seconds)),
      "auto_save" => Some(&serde_json::json!(self.processing.auto_save)),
      "auto_backup" => Some(&serde_json::json!(self.processing.auto_backup)),
      "preview_quality" => Some(&serde_json::json!(self.processing.preview_quality)),
      _ => None,
    }
  }

  fn set_processing_setting(&mut self, key: String, value: serde_json::Value) -> Result<()> {
    match key.as_str() {
      "thread_count" => {
        self.processing.thread_count = value
          .as_u64()
          .ok_or_else(|| EllasticError::ConfigError("thread_count must be a number".to_string()))?
          as usize;
      }
      "memory_limit_mb" => {
        self.processing.memory_limit_mb = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("memory_limit_mb must be a number".to_string())
        })?;
      }
      "cache_size_mb" => {
        self.processing.cache_size_mb = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("cache_size_mb must be a number".to_string())
        })?;
      }
      "chunk_size" => {
        self.processing.chunk_size = value
          .as_u64()
          .ok_or_else(|| EllasticError::ConfigError("chunk_size must be a number".to_string()))?
          as usize;
      }
      "batch_size" => {
        self.processing.batch_size = value
          .as_u64()
          .ok_or_else(|| EllasticError::ConfigError("batch_size must be a number".to_string()))?
          as usize;
      }
      "timeout_seconds" => {
        self.processing.timeout_seconds = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("timeout_seconds must be a number".to_string())
        })?;
      }
      "auto_save" => {
        self.processing.auto_save = value
          .as_bool()
          .ok_or_else(|| EllasticError::ConfigError("auto_save must be a boolean".to_string()))?;
      }
      "auto_backup" => {
        self.processing.auto_backup = value
          .as_bool()
          .ok_or_else(|| EllasticError::ConfigError("auto_backup must be a boolean".to_string()))?;
      }
      "preview_quality" => {
        self.processing.preview_quality = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("preview_quality must be a number".to_string())
        })? as u8;
      }
      _ => {
        return Err(EllasticError::ConfigError(format!(
          "Unknown processing setting: {}",
          key
        )));
      }
    }
    Ok(())
  }

  fn get_ui_setting(&self, key: &str) -> Option<&serde_json::Value> {
    match key {
      "theme" => Some(&serde_json::json!(self.ui.theme)),
      "font_size" => Some(&serde_json::json!(self.ui.font_size)),
      "window_width" => Some(&serde_json::json!(self.ui.window_width)),
      "window_height" => Some(&serde_json::json!(self.ui.window_height)),
      "auto_save_layout" => Some(&serde_json::json!(self.ui.auto_save_layout)),
      "show_advanced_options" => Some(&serde_json::json!(self.ui.show_advanced_options)),
      "preview_update_interval_ms" => Some(&serde_json::json!(self.ui.preview_update_interval_ms)),
      _ => None,
    }
  }

  fn set_ui_setting(&mut self, key: String, value: serde_json::Value) -> Result<()> {
    match key.as_str() {
      "theme" => {
        self.ui.theme = value
          .as_str()
          .ok_or_else(|| EllasticError::ConfigError("theme must be a string".to_string()))?
          .to_string();
      }
      "font_size" => {
        self.ui.font_size = value
          .as_f64()
          .ok_or_else(|| EllasticError::ConfigError("font_size must be a number".to_string()))?
          as f32;
      }
      "window_width" => {
        self.ui.window_width = value
          .as_u64()
          .ok_or_else(|| EllasticError::ConfigError("window_width must be a number".to_string()))?
          as u32;
      }
      "window_height" => {
        self.ui.window_height = value
          .as_u64()
          .ok_or_else(|| EllasticError::ConfigError("window_height must be a number".to_string()))?
          as u32;
      }
      "auto_save_layout" => {
        self.ui.auto_save_layout = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("auto_save_layout must be a boolean".to_string())
        })?;
      }
      "show_advanced_options" => {
        self.ui.show_advanced_options = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("show_advanced_options must be a boolean".to_string())
        })?;
      }
      "preview_update_interval_ms" => {
        self.ui.preview_update_interval_ms = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("preview_update_interval_ms must be a number".to_string())
        })?;
      }
      _ => {
        return Err(EllasticError::ConfigError(format!(
          "Unknown UI setting: {}",
          key
        )));
      }
    }
    Ok(())
  }

  fn get_export_setting(&self, key: &str) -> Option<&serde_json::Value> {
    match key {
      "default_format" => Some(&serde_json::json!(self.export.default_format)),
      "default_quality" => Some(&serde_json::json!(self.export.default_quality)),
      "output_directory" => Some(&serde_json::json!(self.export.output_directory)),
      "filename_template" => Some(&serde_json::json!(self.export.filename_template)),
      "preserve_metadata" => Some(&serde_json::json!(self.export.preserve_metadata)),
      "create_subdirectories" => Some(&serde_json::json!(self.export.create_subdirectories)),
      "overwrite_existing" => Some(&serde_json::json!(self.export.overwrite_existing)),
      "compression_level" => Some(&serde_json::json!(self.export.compression_level)),
      _ => None,
    }
  }

  fn set_export_setting(&mut self, key: String, value: serde_json::Value) -> Result<()> {
    match key.as_str() {
      "default_format" => {
        self.export.default_format = value
          .as_str()
          .ok_or_else(|| EllasticError::ConfigError("default_format must be a string".to_string()))?
          .to_string();
      }
      "default_quality" => {
        self.export.default_quality = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("default_quality must be a number".to_string())
        })? as u8;
      }
      "output_directory" => {
        self.export.output_directory = value.as_str().map(|s| s.to_string());
      }
      "filename_template" => {
        self.export.filename_template = value
          .as_str()
          .ok_or_else(|| {
            EllasticError::ConfigError("filename_template must be a string".to_string())
          })?
          .to_string();
      }
      "preserve_metadata" => {
        self.export.preserve_metadata = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("preserve_metadata must be a boolean".to_string())
        })?;
      }
      "create_subdirectories" => {
        self.export.create_subdirectories = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("create_subdirectories must be a boolean".to_string())
        })?;
      }
      "overwrite_existing" => {
        self.export.overwrite_existing = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("overwrite_existing must be a boolean".to_string())
        })?;
      }
      "compression_level" => {
        self.export.compression_level = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("compression_level must be a number".to_string())
        })? as u8;
      }
      _ => {
        return Err(EllasticError::ConfigError(format!(
          "Unknown export setting: {}",
          key
        )));
      }
    }
    Ok(())
  }

  fn get_lua_setting(&self, key: &str) -> Option<&serde_json::Value> {
    match key {
      "script_directory" => Some(&serde_json::json!(self.lua.script_directory)),
      "auto_reload" => Some(&serde_json::json!(self.lua.auto_reload)),
      "sandbox_enabled" => Some(&serde_json::json!(self.lua.sandbox_enabled)),
      "max_execution_time_ms" => Some(&serde_json::json!(self.lua.max_execution_time_ms)),
      "max_memory_mb" => Some(&serde_json::json!(self.lua.max_memory_mb)),
      "allowed_modules" => Some(&serde_json::json!(self.lua.allowed_modules)),
      "forbidden_functions" => Some(&serde_json::json!(self.lua.forbidden_functions)),
      _ => None,
    }
  }

  fn set_lua_setting(&mut self, key: String, value: serde_json::Value) -> Result<()> {
    match key.as_str() {
      "script_directory" => {
        self.lua.script_directory = value.as_str().map(|s| s.to_string());
      }
      "auto_reload" => {
        self.lua.auto_reload = value
          .as_bool()
          .ok_or_else(|| EllasticError::ConfigError("auto_reload must be a boolean".to_string()))?;
      }
      "sandbox_enabled" => {
        self.lua.sandbox_enabled = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("sandbox_enabled must be a boolean".to_string())
        })?;
      }
      "max_execution_time_ms" => {
        self.lua.max_execution_time_ms = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("max_execution_time_ms must be a number".to_string())
        })?;
      }
      "max_memory_mb" => {
        self.lua.max_memory_mb = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("max_memory_mb must be a number".to_string())
        })?;
      }
      "allowed_modules" => {
        self.lua.allowed_modules = value
          .as_array()
          .ok_or_else(|| {
            EllasticError::ConfigError("allowed_modules must be an array".to_string())
          })?
          .iter()
          .filter_map(|v| v.as_str().map(|s| s.to_string()))
          .collect();
      }
      "forbidden_functions" => {
        self.lua.forbidden_functions = value
          .as_array()
          .ok_or_else(|| {
            EllasticError::ConfigError("forbidden_functions must be an array".to_string())
          })?
          .iter()
          .filter_map(|v| v.as_str().map(|s| s.to_string()))
          .collect();
      }
      _ => {
        return Err(EllasticError::ConfigError(format!(
          "Unknown Lua setting: {}",
          key
        )));
      }
    }
    Ok(())
  }

  fn get_logging_setting(&self, key: &str) -> Option<&serde_json::Value> {
    match key {
      "level" => Some(&serde_json::json!(self.logging.level)),
      "file_enabled" => Some(&serde_json::json!(self.logging.file_enabled)),
      "file_path" => Some(&serde_json::json!(self.logging.file_path)),
      "max_file_size_mb" => Some(&serde_json::json!(self.logging.max_file_size_mb)),
      "max_files" => Some(&serde_json::json!(self.logging.max_files)),
      "console_enabled" => Some(&serde_json::json!(self.logging.console_enabled)),
      "structured" => Some(&serde_json::json!(self.logging.structured)),
      "include_timestamps" => Some(&serde_json::json!(self.logging.include_timestamps)),
      "include_module" => Some(&serde_json::json!(self.logging.include_module)),
      _ => None,
    }
  }

  fn set_logging_setting(&mut self, key: String, value: serde_json::Value) -> Result<()> {
    match key.as_str() {
      "level" => {
        self.logging.level = value
          .as_str()
          .ok_or_else(|| EllasticError::ConfigError("level must be a string".to_string()))?
          .to_string();
      }
      "file_enabled" => {
        self.logging.file_enabled = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("file_enabled must be a boolean".to_string())
        })?;
      }
      "file_path" => {
        self.logging.file_path = value.as_str().map(|s| s.to_string());
      }
      "max_file_size_mb" => {
        self.logging.max_file_size_mb = value.as_u64().ok_or_else(|| {
          EllasticError::ConfigError("max_file_size_mb must be a number".to_string())
        })?;
      }
      "max_files" => {
        self.logging.max_files = value
          .as_u64()
          .ok_or_else(|| EllasticError::ConfigError("max_files must be a number".to_string()))?
          as u32;
      }
      "console_enabled" => {
        self.logging.console_enabled = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("console_enabled must be a boolean".to_string())
        })?;
      }
      "structured" => {
        self.logging.structured = value
          .as_bool()
          .ok_or_else(|| EllasticError::ConfigError("structured must be a boolean".to_string()))?;
      }
      "include_timestamps" => {
        self.logging.include_timestamps = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("include_timestamps must be a boolean".to_string())
        })?;
      }
      "include_module" => {
        self.logging.include_module = value.as_bool().ok_or_else(|| {
          EllasticError::ConfigError("include_module must be a boolean".to_string())
        })?;
      }
      _ => {
        return Err(EllasticError::ConfigError(format!(
          "Unknown logging setting: {}",
          key
        )));
      }
    }
    Ok(())
  }
}

impl Default for SettingsManager {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SettingsExport {
  processing: ProcessingSettings,
  ui: UISettings,
  export: ExportSettings,
  lua: LuaSettings,
  logging: LoggingSettings,
  custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for ProcessingSettings {
  fn default() -> Self {
    Self {
      thread_count: num_cpus::get(),
      memory_limit_mb: 2048,
      cache_size_mb: 512,
      chunk_size: 1024 * 1024,
      batch_size: 100,
      timeout_seconds: 300,
      auto_save: true,
      auto_backup: true,
      preview_quality: 75,
    }
  }
}

impl Default for UISettings {
  fn default() -> Self {
    Self {
      theme: "dark".to_string(),
      font_size: 14.0,
      window_width: 1920,
      window_height: 1080,
      panel_layout: HashMap::new(),
      shortcuts: HashMap::new(),
      auto_save_layout: true,
      show_advanced_options: false,
      preview_update_interval_ms: 100,
    }
  }
}

impl Default for ExportSettings {
  fn default() -> Self {
    Self {
      default_format: "png".to_string(),
      default_quality: 90,
      output_directory: None,
      filename_template: "{name}_{timestamp}.{ext}".to_string(),
      preserve_metadata: true,
      create_subdirectories: false,
      overwrite_existing: false,
      compression_level: 6,
    }
  }
}

impl Default for LuaSettings {
  fn default() -> Self {
    Self {
      script_directory: None,
      auto_reload: true,
      sandbox_enabled: true,
      max_execution_time_ms: 5000,
      max_memory_mb: 256,
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

impl Default for LoggingSettings {
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
    }
  }
}
