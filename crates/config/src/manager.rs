use ellastic_errors::{
  EllasticError,
  Result,
};
use std::collections::HashMap;
use std::path::{
  Path,
  PathBuf,
};

use crate::{
  AppConfig,
  ConfigProfile,
  ConfigValidator,
  get_default_config_path,
};

#[derive(Debug)]
pub struct ConfigManager {
  config_path: PathBuf,
  current_config: AppConfig,
  profiles: HashMap<String, ConfigProfile>,
  current_profile: String,
}

impl ConfigManager {
  pub fn new() -> Result<Self> {
    let config_path = get_default_config_path()?;
    Self::with_path(config_path)
  }

  pub fn with_path<P: AsRef<Path>>(config_path: P) -> Result<Self> {
    let config_path = config_path.as_ref().to_path_buf();
    let mut manager = Self {
      config_path: config_path.clone(),
      current_config: AppConfig::default(),
      profiles: HashMap::new(),
      current_profile: "default".to_string(),
    };

    if config_path.exists() {
      manager.load()?;
    } else {
      manager.create_default_profile()?;
      manager.save()?;
    }

    Ok(manager)
  }

  pub fn load(&mut self) -> Result<()> {
    let content = std::fs::read_to_string(&self.config_path)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to read config file: {}", e)))?;

    let config_data: ConfigData = toml::from_str(&content)
      .map_err(|e| EllasticError::DeserializationError(format!("Failed to parse config: {}", e)))?;

    self.current_config = config_data.config;
    self.profiles = config_data.profiles.into_iter().collect();
    self.current_profile = config_data.current_profile;

    self.current_config.validate()?;

    tracing::info!("Configuration loaded from {}", self.config_path.display());
    Ok(())
  }

  pub fn save(&self) -> Result<()> {
    let config_data = ConfigData {
      config: self.current_config.clone(),
      profiles: self.profiles.clone().into_iter().collect(),
      current_profile: self.current_profile.clone(),
    };

    let content = toml::to_string_pretty(&config_data).map_err(|e| {
      EllasticError::SerializationError(format!("Failed to serialize config: {}", e))
    })?;

    std::fs::write(&self.config_path, content)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to write config file: {}", e)))?;

    tracing::info!("Configuration saved to {}", self.config_path.display());
    Ok(())
  }

  pub fn get_config(&self) -> &AppConfig {
    &self.current_config
  }

  pub fn get_config_mut(&mut self) -> &mut AppConfig {
    &mut self.current_config
  }

  pub fn update_config<F>(&mut self, updater: F) -> Result<()>
  where
    F: FnOnce(&mut AppConfig),
  {
    updater(&mut self.current_config);
    self.current_config.validate()?;
    self.save()
  }

  pub fn create_profile(&mut self, name: String, config: AppConfig) -> Result<()> {
    if self.profiles.contains_key(&name) {
      return Err(EllasticError::ConfigError(format!(
        "Profile '{}' already exists",
        name
      )));
    }

    config.validate()?;
    let profile = ConfigProfile::new(name.clone(), config);
    self.profiles.insert(name, profile);
    self.save()
  }

  pub fn create_profile_from_current(&mut self, name: String) -> Result<()> {
    self.create_profile(name, self.current_config.clone())
  }

  pub fn switch_profile(&mut self, name: &str) -> Result<()> {
    if !self.profiles.contains_key(name) {
      return Err(EllasticError::ConfigError(format!(
        "Profile '{}' not found",
        name
      )));
    }

    self.current_profile = name.to_string();
    let profile = self.profiles.get(name).unwrap();
    self.current_config = profile.config.clone();
    self.save()
  }

  pub fn delete_profile(&mut self, name: &str) -> Result<()> {
    if name == "default" {
      return Err(EllasticError::ConfigError(
        "Cannot delete default profile".to_string(),
      ));
    }

    if !self.profiles.contains_key(name) {
      return Err(EllasticError::ConfigError(format!(
        "Profile '{}' not found",
        name
      )));
    }

    if self.current_profile == name {
      self.switch_profile("default")?;
    }

    self.profiles.remove(name);
    self.save()
  }

  pub fn get_profile(&self, name: &str) -> Option<&ConfigProfile> {
    self.profiles.get(name)
  }

  pub fn get_current_profile(&self) -> &str {
    &self.current_profile
  }

  pub fn list_profiles(&self) -> Vec<&str> {
    self.profiles.keys().map(|s| s.as_str()).collect()
  }

  pub fn update_profile<F>(&mut self, name: &str, updater: F) -> Result<()>
  where
    F: FnOnce(&mut AppConfig),
  {
    let profile = self
      .profiles
      .get_mut(name)
      .ok_or_else(|| EllasticError::ConfigError(format!("Profile '{}' not found", name)))?;

    updater(&mut profile.config);
    profile.config.validate()?;

    if name == self.current_profile {
      self.current_config = profile.config.clone();
    }

    self.save()
  }

  pub fn reset_to_defaults(&mut self) -> Result<()> {
    self.current_config = AppConfig::default();
    self.save()
  }

  pub fn reset_profile_to_defaults(&mut self, name: &str) -> Result<()> {
    let default_config = AppConfig::default();
    self.update_profile(name, move |config| *config = default_config)
  }

  pub fn export_profile(&self, name: &str, path: &Path) -> Result<()> {
    let profile = self
      .profiles
      .get(name)
      .ok_or_else(|| EllasticError::ConfigError(format!("Profile '{}' not found", name)))?;

    let content = toml::to_string_pretty(profile).map_err(|e| {
      EllasticError::SerializationError(format!("Failed to serialize profile: {}", e))
    })?;

    std::fs::write(path, content)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to write profile file: {}", e)))?;

    tracing::info!("Profile '{}' exported to {}", name, path.display());
    Ok(())
  }

  pub fn import_profile(&mut self, path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to read profile file: {}", e)))?;

    let profile: ConfigProfile = toml::from_str(&content).map_err(|e| {
      EllasticError::DeserializationError(format!("Failed to parse profile: {}", e))
    })?;

    profile.config.validate()?;

    if self.profiles.contains_key(&profile.name) {
      return Err(EllasticError::ConfigError(format!(
        "Profile '{}' already exists",
        profile.name
      )));
    }

    self.profiles.insert(profile.name.clone(), profile);
    self.save()?;

    tracing::info!("Profile imported from {}", path.display());
    Ok(())
  }

  pub fn backup_config(&self, backup_path: &Path) -> Result<()> {
    let backup_data = ConfigBackup {
      timestamp: std::time::SystemTime::now(),
      config: self.current_config.clone(),
      profiles: self.profiles.clone().into_iter().collect(),
      current_profile: self.current_profile.clone(),
    };

    let content = toml::to_string_pretty(&backup_data).map_err(|e| {
      EllasticError::SerializationError(format!("Failed to serialize backup: {}", e))
    })?;

    std::fs::write(backup_path, content)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to write backup file: {}", e)))?;

    tracing::info!("Configuration backed up to {}", backup_path.display());
    Ok(())
  }

  pub fn restore_config(&mut self, backup_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(backup_path)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to read backup file: {}", e)))?;

    let backup: ConfigBackup = toml::from_str(&content)
      .map_err(|e| EllasticError::DeserializationError(format!("Failed to parse backup: {}", e)))?;

    backup.config.validate()?;

    self.current_config = backup.config;
    self.profiles = backup.profiles.into_iter().collect();
    self.current_profile = backup.current_profile;

    self.save()?;

    tracing::info!("Configuration restored from {}", backup_path.display());
    Ok(())
  }

  fn create_default_profile(&mut self) -> Result<()> {
    let default_profile = ConfigProfile::new("default".to_string(), AppConfig::default());
    self.profiles.insert("default".to_string(), default_profile);
    Ok(())
  }
}

impl Default for ConfigManager {
  fn default() -> Self {
    Self::new().expect("Failed to create default config manager")
  }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ConfigData {
  config: AppConfig,
  profiles: Vec<(String, ConfigProfile)>,
  current_profile: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ConfigBackup {
  timestamp: std::time::SystemTime,
  config: AppConfig,
  profiles: Vec<(String, ConfigProfile)>,
  current_profile: String,
}
