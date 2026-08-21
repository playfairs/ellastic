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
};

#[derive(Debug)]
pub struct ProfileManager {
  profiles: HashMap<String, ConfigProfile>,
  active_profile: String,
  profile_directory: PathBuf,
}

impl ProfileManager {
  pub fn new<P: AsRef<Path>>(profile_directory: P) -> Result<Self> {
    let profile_directory = profile_directory.as_ref().to_path_buf();
    std::fs::create_dir_all(&profile_directory).map_err(|e| {
      EllasticError::ConfigError(format!("Failed to create profile directory: {}", e))
    })?;

    let mut manager = Self {
      profiles: HashMap::new(),
      active_profile: "default".to_string(),
      profile_directory,
    };

    manager.load_profiles()?;

    if !manager.profiles.contains_key("default") {
      manager.create_default_profile()?;
    }

    Ok(manager)
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
    self.profiles.insert(name.clone(), profile);

    self.save_profile(&name)?;
    tracing::info!("Created profile: {}", name);

    Ok(())
  }

  pub fn create_profile_from_current(&mut self, name: String) -> Result<()> {
    let current_config = self.get_active_config()?;
    self.create_profile(name, current_config)
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

    if self.active_profile == name {
      self.switch_profile("default")?;
    }

    self.profiles.remove(name);

    let profile_path = self.profile_directory.join(format!("{}.toml", name));
    std::fs::remove_file(&profile_path)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to delete profile file: {}", e)))?;

    tracing::info!("Deleted profile: {}", name);
    Ok(())
  }

  pub fn switch_profile(&mut self, name: &str) -> Result<()> {
    if !self.profiles.contains_key(name) {
      return Err(EllasticError::ConfigError(format!(
        "Profile '{}' not found",
        name
      )));
    }

    self.active_profile = name.to_string();
    tracing::info!("Switched to profile: {}", name);
    Ok(())
  }

  pub fn get_active_profile(&self) -> &str {
    &self.active_profile
  }

  pub fn get_active_config(&self) -> Result<AppConfig> {
    self
      .profiles
      .get(&self.active_profile)
      .map(|p| p.config.clone())
      .ok_or_else(|| {
        EllasticError::ConfigError(format!(
          "Active profile '{}' not found",
          self.active_profile
        ))
      })
  }

  pub fn get_profile(&self, name: &str) -> Option<&ConfigProfile> {
    self.profiles.get(name)
  }

  pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut ConfigProfile> {
    self.profiles.get_mut(name)
  }

  pub fn list_profiles(&self) -> Vec<&String> {
    self.profiles.keys().collect()
  }

  pub fn update_profile<F>(&mut self, name: &str, updater: F) -> Result<()>
  where
    F: FnOnce(&mut ConfigProfile),
  {
    let profile = self
      .profiles
      .get_mut(name)
      .ok_or_else(|| EllasticError::ConfigError(format!("Profile '{}' not found", name)))?;

    updater(profile);
    profile.config.validate()?;

    self.save_profile(name)?;
    tracing::info!("Updated profile: {}", name);

    Ok(())
  }

  pub fn duplicate_profile(&mut self, source_name: &str, target_name: String) -> Result<()> {
    let source_profile = self.profiles.get(source_name).ok_or_else(|| {
      EllasticError::ConfigError(format!("Source profile '{}' not found", source_name))
    })?;

    if self.profiles.contains_key(&target_name) {
      return Err(EllasticError::ConfigError(format!(
        "Target profile '{}' already exists",
        target_name
      )));
    }

    let mut target_profile = source_profile.clone();
    target_profile.name = target_name.clone();
    target_profile.created_at = std::time::SystemTime::now();
    target_profile.modified_at = std::time::SystemTime::now();

    self.profiles.insert(target_name.clone(), target_profile);
    self.save_profile(&target_name)?;

    tracing::info!("Duplicated profile '{}' to '{}'", source_name, target_name);
    Ok(())
  }

  pub fn export_profile(&self, name: &str, export_path: &Path) -> Result<()> {
    let profile = self
      .profiles
      .get(name)
      .ok_or_else(|| EllasticError::ConfigError(format!("Profile '{}' not found", name)))?;

    let content = toml::to_string_pretty(profile).map_err(|e| {
      EllasticError::SerializationError(format!("Failed to serialize profile: {}", e))
    })?;

    std::fs::write(export_path, content)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to write export file: {}", e)))?;

    tracing::info!("Exported profile '{}' to {}", name, export_path.display());
    Ok(())
  }

  pub fn import_profile(&mut self, import_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(import_path)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to read import file: {}", e)))?;

    let mut profile: ConfigProfile = toml::from_str(&content).map_err(|e| {
      EllasticError::DeserializationError(format!("Failed to parse profile: {}", e))
    })?;

    profile.config.validate()?;

    if self.profiles.contains_key(&profile.name) {
      return Err(EllasticError::ConfigError(format!(
        "Profile '{}' already exists",
        profile.name
      )));
    }

    profile.created_at = std::time::SystemTime::now();
    profile.modified_at = std::time::SystemTime::now();

    self.profiles.insert(profile.name.clone(), profile.clone());
    self.save_profile(&profile.name)?;

    tracing::info!(
      "Imported profile '{}' from {}",
      profile.name,
      import_path.display()
    );
    Ok(())
  }

  pub fn reset_profile(&mut self, name: &str) -> Result<()> {
    let default_config = AppConfig::default();
    self.update_profile(name, |profile| {
      profile.config = default_config.clone();
    })
  }

  pub fn get_profile_info(&self, name: &str) -> Option<ProfileInfo> {
    self.profiles.get(name).map(|profile| ProfileInfo {
      name: profile.name.clone(),
      description: profile.description.clone(),
      created_at: profile.created_at,
      modified_at: profile.modified_at,
      is_active: name == self.active_profile,
    })
  }

  pub fn get_all_profiles_info(&self) -> Vec<ProfileInfo> {
    self
      .profiles
      .iter()
      .map(|(name, profile)| ProfileInfo {
        name: name.clone(),
        description: profile.description.clone(),
        created_at: profile.created_at,
        modified_at: profile.modified_at,
        is_active: name == &self.active_profile,
      })
      .collect()
  }

  fn load_profiles(&mut self) -> Result<()> {
    if !self.profile_directory.exists() {
      return Ok(());
    }

    let entries = std::fs::read_dir(&self.profile_directory).map_err(|e| {
      EllasticError::ConfigError(format!("Failed to read profile directory: {}", e))
    })?;

    for entry in entries {
      let entry = entry.map_err(|e| {
        EllasticError::ConfigError(format!("Failed to read directory entry: {}", e))
      })?;

      let path = entry.path();
      if path.extension().and_then(|s| s.to_str()) == Some("toml") {
        self.load_single_profile(&path)?;
      }
    }

    tracing::info!("Loaded {} profiles", self.profiles.len());
    Ok(())
  }

  fn load_single_profile(&mut self, path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to read profile file: {}", e)))?;

    let profile: ConfigProfile = toml::from_str(&content).map_err(|e| {
      EllasticError::DeserializationError(format!("Failed to parse profile: {}", e))
    })?;

    profile.config.validate()?;
    self.profiles.insert(profile.name.clone(), profile);
    Ok(())
  }

  fn save_profile(&self, name: &str) -> Result<()> {
    let profile = self
      .profiles
      .get(name)
      .ok_or_else(|| EllasticError::ConfigError(format!("Profile '{}' not found", name)))?;

    let content = toml::to_string_pretty(profile).map_err(|e| {
      EllasticError::SerializationError(format!("Failed to serialize profile: {}", e))
    })?;

    let profile_path = self.profile_directory.join(format!("{}.toml", name));
    std::fs::write(&profile_path, content)
      .map_err(|e| EllasticError::ConfigError(format!("Failed to write profile file: {}", e)))?;

    Ok(())
  }

  fn create_default_profile(&mut self) -> Result<()> {
    let default_profile = ConfigProfile::new("default".to_string(), AppConfig::default());
    self.profiles.insert("default".to_string(), default_profile);
    self.save_profile("default")?;
    Ok(())
  }

  pub fn cleanup_orphaned_files(&self) -> Result<usize> {
    let mut removed_count = 0;

    let entries = std::fs::read_dir(&self.profile_directory).map_err(|e| {
      EllasticError::ConfigError(format!("Failed to read profile directory: {}", e))
    })?;

    for entry in entries {
      let entry = entry.map_err(|e| {
        EllasticError::ConfigError(format!("Failed to read directory entry: {}", e))
      })?;

      let path = entry.path();
      if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
        if !self.profiles.contains_key(name)
          && path.extension().and_then(|s| s.to_str()) == Some("toml")
        {
          std::fs::remove_file(&path).map_err(|e| {
            EllasticError::ConfigError(format!("Failed to remove orphaned file: {}", e))
          })?;
          removed_count += 1;
        }
      }
    }

    if removed_count > 0 {
      tracing::info!("Cleaned up {} orphaned profile files", removed_count);
    }

    Ok(removed_count)
  }
}

#[derive(Debug, Clone)]
pub struct ProfileInfo {
  pub name: String,
  pub description: Option<String>,
  pub created_at: std::time::SystemTime,
  pub modified_at: std::time::SystemTime,
  pub is_active: bool,
}
