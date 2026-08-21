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
use ellastic_pipeline::{
  PipelineGraph,
  PipelineProcessor,
};
use ellastic_utils::create_random_generator;
use parking_lot::RwLock;
use rayon::prelude::*;
use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod assets;
pub mod backup;
pub mod export;
pub mod history;
pub mod import;
pub mod metadata;
pub mod project;
pub mod session;
pub mod templates;
pub mod workspace;

pub use assets::*;
pub use backup::*;
pub use export::*;
pub use history::*;
pub use import::*;
pub use metadata::*;
pub use project::*;
pub use session::*;
pub use templates::*;
pub use workspace::*;

#[derive(Debug, Clone)]
pub struct ProjectManager {
  projects: Arc<RwLock<HashMap<Uuid, Project>>>,
  sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
  workspace_manager: Arc<RwLock<WorkspaceManager>>,
  template_manager: Arc<RwLock<TemplateManager>>,
  history_manager: Arc<RwLock<HistoryManager>>,
  backup_manager: Arc<RwLock<BackupManager>>,
  metadata_manager: Arc<RwLock<MetadataManager>>,
  asset_manager: Arc<RwLock<AssetManager>>,
  export_manager: Arc<RwLock<ExportManager>>,
  import_manager: Arc<RwLock<ImportManager>>,
  config: ProjectManagerConfig,
}

#[derive(Debug, Clone)]
pub struct ProjectManagerConfig {
  pub max_projects: usize,
  pub max_sessions: usize,
  pub auto_save_interval_seconds: u64,
  pub auto_backup_enabled: bool,
  pub backup_retention_days: u32,
  pub workspace_root: String,
  pub template_directory: String,
  pub asset_directory: String,
  pub export_directory: String,
  pub import_directory: String,
  pub enable_history: bool,
  pub history_retention_days: u32,
  pub enable_metadata_indexing: bool,
  pub enable_asset_caching: bool,
}

#[derive(Debug, Clone)]
pub struct Project {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub project_type: ProjectType,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub metadata: ProjectMetadata,
  pub workspace: Workspace,
  pub sessions: Vec<Session>,
  pub assets: AssetCollection,
  pub history: ProjectHistory,
  pub settings: ProjectSettings,
  pub status: ProjectStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
  Image,
  Audio,
  Video,
  Multimedia,
  Pipeline,
  Script,
  Mixed,
}

#[derive(Debug, Clone)]
pub struct ProjectMetadata {
  pub author: String,
  pub version: String,
  pub license: String,
  pub tags: Vec<String>,
  pub categories: Vec<String>,
  pub keywords: Vec<String>,
  pub custom_fields: HashMap<String, String>,
  pub thumbnail: Option<String>,
  pub preview: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectSettings {
  pub auto_save: bool,
  pub auto_backup: bool,
  pub compression_enabled: bool,
  pub encryption_enabled: bool,
  pub workspace_layout: WorkspaceLayout,
  pub performance_settings: PerformanceSettings,
  pub export_settings: ExportSettings,
  pub security_settings: SecuritySettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceLayout {
  Default,
  Compact,
  Detailed,
  Custom,
}

#[derive(Debug, Clone)]
pub struct PerformanceSettings {
  pub max_memory_mb: usize,
  pub max_cpu_cores: u8,
  pub enable_parallel_processing: bool,
  pub cache_size_mb: usize,
  pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
  None,
  Basic,
  Standard,
  Aggressive,
}

#[derive(Debug, Clone)]
pub struct ExportSettings {
  pub default_format: String,
  pub default_quality: u8,
  pub compression_level: u8,
  pub include_metadata: bool,
  pub watermark_enabled: bool,
  pub watermark_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SecuritySettings {
  pub encryption_enabled: bool,
  pub password_protected: bool,
  pub access_control: bool,
  pub audit_logging: bool,
  pub allowed_users: Vec<String>,
  pub blocked_users: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectStatus {
  Active,
  Inactive,
  Archived,
  Deleted,
  Corrupted,
}

impl ProjectManager {
  pub fn new(config: ProjectManagerConfig) -> Result<Self> {
    let mut manager = Self {
      projects: Arc::new(RwLock::new(HashMap::new())),
      sessions: Arc::new(RwLock::new(HashMap::new())),
      workspace_manager: Arc::new(RwLock::new(WorkspaceManager::new(&config))),
      template_manager: Arc::new(RwLock::new(TemplateManager::new(&config))),
      history_manager: Arc::new(RwLock::new(HistoryManager::new(&config))),
      backup_manager: Arc::new(RwLock::new(BackupManager::new(&config))),
      metadata_manager: Arc::new(RwLock::new(MetadataManager::new(&config))),
      asset_manager: Arc::new(RwLock::new(AssetManager::new(&config))),
      export_manager: Arc::new(RwLock::new(ExportManager::new(&config))),
      import_manager: Arc::new(RwLock::new(ImportManager::new(&config))),
      config,
    };

    manager.initialize()?;
    Ok(manager)
  }

  pub fn config(&self) -> &ProjectManagerConfig {
    &self.config
  }

  pub fn projects(&self) -> Arc<RwLock<HashMap<Uuid, Project>>> {
    self.projects.clone()
  }

  pub fn sessions(&self) -> Arc<RwLock<HashMap<Uuid, Session>>> {
    self.sessions.clone()
  }

  pub fn workspace_manager(&self) -> Arc<RwLock<WorkspaceManager>> {
    self.workspace_manager.clone()
  }

  pub fn template_manager(&self) -> Arc<RwLock<TemplateManager>> {
    self.template_manager.clone()
  }

  pub fn history_manager(&self) -> Arc<RwLock<HistoryManager>> {
    self.history_manager.clone()
  }

  pub fn backup_manager(&self) -> Arc<RwLock<BackupManager>> {
    self.backup_manager.clone()
  }

  pub fn metadata_manager(&self) -> Arc<RwLock<MetadataManager>> {
    self.metadata_manager.clone()
  }

  pub fn asset_manager(&self) -> Arc<RwLock<AssetManager>> {
    self.asset_manager.clone()
  }

  pub fn export_manager(&self) -> Arc<RwLock<ExportManager>> {
    self.export_manager.clone()
  }

  pub fn import_manager(&self) -> Arc<RwLock<ImportManager>> {
    self.import_manager.clone()
  }

  fn initialize(&mut self) -> Result<()> {
    std::fs::create_dir_all(&self.config.workspace_root)?;
    std::fs::create_dir_all(&self.config.template_directory)?;
    std::fs::create_dir_all(&self.config.asset_directory)?;
    std::fs::create_dir_all(&self.config.export_directory)?;
    std::fs::create_dir_all(&self.config.import_directory)?;

    self.workspace_manager.write().initialize()?;
    self.template_manager.write().initialize()?;
    self.history_manager.write().initialize()?;
    self.backup_manager.write().initialize()?;
    self.metadata_manager.write().initialize()?;
    self.asset_manager.write().initialize()?;
    self.export_manager.write().initialize()?;
    self.import_manager.write().initialize()?;

    self.load_projects()?;

    Ok(())
  }

  pub fn create_project(
    &mut self,
    name: String,
    description: String,
    project_type: ProjectType,
  ) -> Result<Uuid> {
    let project_id = Uuid::new_v4();
    let now = Utc::now();

    if self.projects.read().len() >= self.config.max_projects {
      return Err(EllasticError::LimitExceeded(
        "Maximum project limit reached".to_string(),
      ));
    }

    let workspace = self
      .workspace_manager
      .write()
      .create_workspace(project_id, &name)?;

    let project = Project {
      id: project_id,
      name: name.clone(),
      description,
      project_type,
      created_at: now,
      updated_at: now,
      metadata: ProjectMetadata::new(),
      workspace,
      sessions: Vec::new(),
      assets: AssetCollection::new(),
      history: ProjectHistory::new(),
      settings: ProjectSettings::default(),
      status: ProjectStatus::Active,
    };

    self.projects.write().insert(project_id, project);

    self.history_manager.write().record_event(
      project_id,
      HistoryEvent::ProjectCreated {
        project_id,
        name,
        timestamp: now,
      },
    )?;

    if self.config.auto_backup_enabled {
      self.backup_manager.write().create_backup(project_id)?;
    }

    Ok(project_id)
  }

  pub fn load_project(&mut self, project_id: Uuid) -> Result<Project> {
    let project = self
      .projects
      .read()
      .get(&project_id)
      .ok_or_else(|| EllasticError::InvalidParameter(format!("Project {} not found", project_id)))?
      .clone();

    self.update_project_accessed(project_id)?;

    Ok(project)
  }

  pub fn save_project(&mut self, project_id: Uuid) -> Result<()> {
    let mut projects = self.projects.write();
    if let Some(project) = projects.get_mut(&project_id) {
      project.updated_at = Utc::now();

      self.workspace_manager.write().save_workspace(project_id)?;

      self.asset_manager.write().save_assets(project_id)?;

      self.history_manager.write().save_history(project_id)?;

      if self.config.auto_backup_enabled {
        self.backup_manager.write().create_backup(project_id)?;
      }

      self.history_manager.write().record_event(
        project_id,
        HistoryEvent::ProjectSaved {
          project_id,
          timestamp: Utc::now(),
        },
      )?;
    } else {
      return Err(EllasticError::InvalidParameter(format!(
        "Project {} not found",
        project_id
      )));
    }

    Ok(())
  }

  pub fn delete_project(&mut self, project_id: Uuid) -> Result<()> {
    self.history_manager.write().record_event(
      project_id,
      HistoryEvent::ProjectDeleted {
        project_id,
        timestamp: Utc::now(),
      },
    )?;

    let project = self.projects.write().remove(&project_id).ok_or_else(|| {
      EllasticError::InvalidParameter(format!("Project {} not found", project_id))
    })?;

    self
      .workspace_manager
      .write()
      .delete_workspace(project_id)?;

    self.asset_manager.write().delete_assets(project_id)?;

    self.history_manager.write().delete_history(project_id)?;

    self.backup_manager.write().delete_backups(project_id)?;

    self.metadata_manager.write().delete_metadata(project_id)?;

    Ok(())
  }

  pub fn archive_project(&mut self, project_id: Uuid) -> Result<()> {
    let mut projects = self.projects.write();
    if let Some(project) = projects.get_mut(&project_id) {
      project.status = ProjectStatus::Archived;
      project.updated_at = Utc::now();

      self.history_manager.write().record_event(
        project_id,
        HistoryEvent::ProjectArchived {
          project_id,
          timestamp: Utc::now(),
        },
      )?;

      self.backup_manager.write().create_backup(project_id)?;

      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Project {} not found",
        project_id
      )))
    }
  }

  pub fn restore_project(&mut self, project_id: Uuid) -> Result<()> {
    let mut projects = self.projects.write();
    if let Some(project) = projects.get_mut(&project_id) {
      project.status = ProjectStatus::Active;
      project.updated_at = Utc::now();

      self.history_manager.write().record_event(
        project_id,
        HistoryEvent::ProjectRestored {
          project_id,
          timestamp: Utc::now(),
        },
      )?;

      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Project {} not found",
        project_id
      )))
    }
  }

  pub fn list_projects(&self) -> Vec<&Project> {
    self.projects.read().values().collect()
  }

  pub fn list_active_projects(&self) -> Vec<&Project> {
    self
      .projects
      .read()
      .values()
      .filter(|project| project.status == ProjectStatus::Active)
      .collect()
  }

  pub fn list_archived_projects(&self) -> Vec<&Project> {
    self
      .projects
      .read()
      .values()
      .filter(|project| project.status == ProjectStatus::Archived)
      .collect()
  }

  pub fn search_projects(&self, query: &str) -> Vec<&Project> {
    let query = query.to_lowercase();
    self
      .projects
      .read()
      .values()
      .filter(|project| {
        project.name.to_lowercase().contains(&query)
          || project.description.to_lowercase().contains(&query)
          || project
            .metadata
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query))
          || project
            .metadata
            .keywords
            .iter()
            .any(|keyword| keyword.to_lowercase().contains(&query))
      })
      .collect()
  }

  pub fn create_session(&mut self, project_id: Uuid, session_name: String) -> Result<Uuid> {
    let session_id = Uuid::new_v4();
    let now = Utc::now();

    if self.sessions.read().len() >= self.config.max_sessions {
      return Err(EllasticError::LimitExceeded(
        "Maximum session limit reached".to_string(),
      ));
    }

    if !self.projects.read().contains_key(&project_id) {
      return Err(EllasticError::InvalidParameter(format!(
        "Project {} not found",
        project_id
      )));
    }

    let session = Session {
      id: session_id,
      project_id,
      name: session_name.clone(),
      created_at: now,
      updated_at: now,
      status: SessionStatus::Active,
      metadata: SessionMetadata::new(),
      workspace_state: WorkspaceState::new(),
      asset_state: AssetState::new(),
      history: SessionHistory::new(),
    };

    self.sessions.write().insert(session_id, session);

    let mut projects = self.projects.write();
    if let Some(project) = projects.get_mut(&project_id) {
      project.sessions.push(session.clone());
      project.updated_at = now;
    }

    self.history_manager.write().record_event(
      project_id,
      HistoryEvent::SessionCreated {
        session_id,
        project_id,
        name: session_name,
        timestamp: now,
      },
    )?;

    Ok(session_id)
  }

  pub fn load_session(&mut self, session_id: Uuid) -> Result<Session> {
    let session = self
      .sessions
      .read()
      .get(&session_id)
      .ok_or_else(|| EllasticError::InvalidParameter(format!("Session {} not found", session_id)))?
      .clone();

    self.update_session_accessed(session_id)?;

    Ok(session)
  }

  pub fn save_session(&mut self, session_id: Uuid) -> Result<()> {
    let mut sessions = self.sessions.write();
    if let Some(session) = sessions.get_mut(&session_id) {
      session.updated_at = Utc::now();

      self
        .workspace_manager
        .write()
        .save_workspace_state(session_id)?;

      self.asset_manager.write().save_asset_state(session_id)?;

      self.history_manager.write().record_event(
        session.project_id,
        HistoryEvent::SessionSaved {
          session_id,
          timestamp: Utc::now(),
        },
      )?;

      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Session {} not found",
        session_id
      )))
    }
  }

  pub fn delete_session(&mut self, session_id: Uuid) -> Result<()> {
    let session = self.sessions.write().remove(&session_id).ok_or_else(|| {
      EllasticError::InvalidParameter(format!("Session {} not found", session_id))
    })?;

    let mut projects = self.projects.write();
    if let Some(project) = projects.get_mut(&session.project_id) {
      project.sessions.retain(|s| s.id != session_id);
      project.updated_at = Utc::now();
    }

    self.history_manager.write().record_event(
      session.project_id,
      HistoryEvent::SessionDeleted {
        session_id,
        project_id: session.project_id,
        timestamp: Utc::now(),
      },
    )?;

    Ok(())
  }

  pub fn list_sessions(&self) -> Vec<&Session> {
    self.sessions.read().values().collect()
  }

  pub fn list_sessions_for_project(&self, project_id: Uuid) -> Vec<&Session> {
    self
      .sessions
      .read()
      .values()
      .filter(|session| session.project_id == project_id)
      .collect()
  }

  fn load_projects(&mut self) -> Result<()> {
    Ok(())
  }

  fn update_project_accessed(&mut self, project_id: Uuid) -> Result<()> {
    let mut projects = self.projects.write();
    if let Some(project) = projects.get_mut(&project_id) {
      project.updated_at = Utc::now();
    }
    Ok(())
  }

  fn update_session_accessed(&mut self, session_id: Uuid) -> Result<()> {
    let mut sessions = self.sessions.write();
    if let Some(session) = sessions.get_mut(&session_id) {
      session.updated_at = Utc::now();
    }
    Ok(())
  }

  pub fn clone(&self) -> ProjectManager {
    ProjectManager {
      projects: self.projects.clone(),
      sessions: self.sessions.clone(),
      workspace_manager: self.workspace_manager.clone(),
      template_manager: self.template_manager.clone(),
      history_manager: self.assets.clone(),
      backup_manager: self.backup_manager.clone(),
      metadata_manager: self.metadata_manager.clone(),
      asset_manager: self.asset_manager.clone(),
      export_manager: self.export_manager.clone(),
      import_manager: self.import_manager.clone(),
      config: self.config.clone(),
    }
  }
}

impl ProjectMetadata {
  pub fn new() -> Self {
    Self {
      author: "Ellastic User".to_string(),
      version: "1.0".to_string(),
      license: "MIT".to_string(),
      tags: Vec::new(),
      categories: Vec::new(),
      keywords: Vec::new(),
      custom_fields: HashMap::new(),
      thumbnail: None,
      preview: None,
    }
  }

  pub fn with_author(mut self, author: String) -> Self {
    self.author = author;
    self
  }

  pub fn with_version(mut self, version: String) -> Self {
    self.version = version;
    self
  }

  pub fn with_license(mut self, license: String) -> Self {
    self.license = license;
    self
  }

  pub fn with_tags(mut self, tags: Vec<String>) -> Self {
    self.tags = tags;
    self
  }

  pub fn with_categories(mut self, categories: Vec<String>) -> Self {
    self.categories = categories;
    self
  }

  pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
    self.keywords = keywords;
    self
  }

  pub fn with_custom_field(mut self, key: String, value: String) -> Self {
    self.custom_fields.insert(key, value);
    self
  }

  pub fn with_thumbnail(mut self, thumbnail: String) -> Self {
    self.thumbnail = Some(thumbnail);
    self
  }

  pub fn with_preview(mut self, preview: String) -> Self {
    self.preview = Some(preview);
    self
  }

  pub fn clone(&self) -> ProjectMetadata {
    ProjectMetadata {
      author: self.author.clone(),
      version: self.version.clone(),
      license: self.license.clone(),
      tags: self.tags.clone(),
      categories: self.categories.clone(),
      keywords: self.keywords.clone(),
      custom_fields: self.custom_fields.clone(),
      thumbnail: self.thumbnail.clone(),
      preview: self.preview.clone(),
    }
  }
}

impl ProjectSettings {
  pub fn new() -> Self {
    Self {
      auto_save: true,
      auto_backup: true,
      compression_enabled: false,
      encryption_enabled: false,
      workspace_layout: WorkspaceLayout::Default,
      performance_settings: PerformanceSettings::new(),
      export_settings: ExportSettings::new(),
      security_settings: SecuritySettings::new(),
    }
  }

  pub fn with_auto_save(mut self, auto_save: bool) -> Self {
    self.auto_save = auto_save;
    self
  }

  pub fn with_auto_backup(mut self, auto_backup: bool) -> Self {
    self.auto_backup = auto_backup;
    self
  }

  pub fn with_compression(mut self, compression_enabled: bool) -> Self {
    self.compression_enabled = compression_enabled;
    self
  }

  pub fn with_encryption(mut self, encryption_enabled: bool) -> Self {
    self.encryption_enabled = encryption_enabled;
    self
  }

  pub fn with_workspace_layout(mut self, layout: WorkspaceLayout) -> Self {
    self.workspace_layout = layout;
    self
  }

  pub fn with_performance_settings(mut self, settings: PerformanceSettings) -> Self {
    self.performance_settings = settings;
    self
  }

  pub fn with_export_settings(mut self, settings: ExportSettings) -> Self {
    self.export_settings = settings;
    self
  }

  pub fn with_security_settings(mut self, settings: SecuritySettings) -> Self {
    self.security_settings = settings;
    self
  }

  pub fn clone(&self) -> ProjectSettings {
    ProjectSettings {
      auto_save: self.auto_save,
      auto_backup: self.auto_backup,
      compression_enabled: self.compression_enabled,
      encryption_enabled: self.encryption_enabled,
      workspace_layout: self.workspace_layout,
      performance_settings: self.performance_settings.clone(),
      export_settings: self.export_settings.clone(),
      security_settings: self.security_settings.clone(),
    }
  }
}

impl PerformanceSettings {
  pub fn new() -> Self {
    Self {
      max_memory_mb: 1024,
      max_cpu_cores: 4,
      enable_parallel_processing: true,
      cache_size_mb: 256,
      optimization_level: OptimizationLevel::Standard,
    }
  }

  pub fn with_max_memory(mut self, max_memory_mb: usize) -> Self {
    self.max_memory_mb = max_memory_mb;
    self
  }

  pub fn with_max_cpu_cores(mut self, max_cpu_cores: u8) -> Self {
    self.max_cpu_cores = max_cpu_cores;
    self
  }

  pub fn with_parallel_processing(mut self, enable: bool) -> Self {
    self.enable_parallel_processing = enable;
    self
  }

  pub fn with_cache_size(mut self, cache_size_mb: usize) -> Self {
    self.cache_size_mb = cache_size_mb;
    self
  }

  pub fn with_optimization_level(mut self, level: OptimizationLevel) -> Self {
    self.optimization_level = level;
    self
  }

  pub fn clone(&self) -> PerformanceSettings {
    PerformanceSettings {
      max_memory_mb: self.max_memory_mb,
      max_cpu_cores: self.max_cpu_cores,
      enable_parallel_processing: self.enable_parallel_processing,
      cache_size_mb: self.cache_size_mb,
      optimization_level: self.optimization_level,
    }
  }
}

impl ExportSettings {
  pub fn new() -> Self {
    Self {
      default_format: "png".to_string(),
      default_quality: 90,
      compression_level: 6,
      include_metadata: true,
      watermark_enabled: false,
      watermark_text: None,
    }
  }

  pub fn with_default_format(mut self, format: String) -> Self {
    self.default_format = format;
    self
  }

  pub fn with_default_quality(mut self, quality: u8) -> Self {
    self.default_quality = quality;
    self
  }

  pub fn with_compression_level(mut self, level: u8) -> Self {
    self.compression_level = level;
    self
  }

  pub fn with_metadata(mut self, include: bool) -> Self {
    self.include_metadata = include;
    self
  }

  pub fn with_watermark(mut self, enabled: bool, text: Option<String>) -> Self {
    self.watermark_enabled = enabled;
    self.watermark_text = text;
    self
  }

  pub fn clone(&self) -> ExportSettings {
    ExportSettings {
      default_format: self.default_format.clone(),
      default_quality: self.default_quality,
      compression_level: self.compression_level,
      include_metadata: self.include_metadata,
      watermark_enabled: self.watermark_enabled,
      watermark_text: self.watermark_text.clone(),
    }
  }
}

impl SecuritySettings {
  pub fn new() -> Self {
    Self {
      encryption_enabled: false,
      password_protected: false,
      access_control: false,
      audit_logging: false,
      allowed_users: Vec::new(),
      blocked_users: Vec::new(),
    }
  }

  pub fn with_encryption(mut self, enabled: bool) -> Self {
    self.encryption_enabled = enabled;
    self
  }

  pub fn with_password_protection(mut self, enabled: bool) -> Self {
    self.password_protected = enabled;
    self
  }

  pub fn with_access_control(mut self, enabled: bool) -> Self {
    self.access_control = enabled;
    self
  }

  pub fn with_audit_logging(mut self, enabled: bool) -> Self {
    self.audit_logging = enabled;
    self
  }

  pub fn with_allowed_users(mut self, users: Vec<String>) -> Self {
    self.allowed_users = users;
    self
  }

  pub fn with_blocked_users(mut self, users: Vec<String>) -> Self {
    self.blocked_users = users;
    self
  }

  pub fn clone(&self) -> SecuritySettings {
    SecuritySettings {
      encryption_enabled: self.encryption_enabled,
      password_protected: self.password_protected,
      access_control: self.access_control,
      audit_logging: self.audit_logging,
      allowed_users: self.allowed_users.clone(),
      blocked_users: self.blocked_users.clone(),
    }
  }
}

impl Default for ProjectManagerConfig {
  fn default() -> Self {
    Self {
      max_projects: 100,
      max_sessions: 50,
      auto_save_interval_seconds: 300,
      auto_backup_enabled: true,
      backup_retention_days: 30,
      workspace_root: "./workspaces".to_string(),
      template_directory: "./templates".to_string(),
      asset_directory: "./assets".to_string(),
      export_directory: "./exports".to_string(),
      import_directory: "./imports".to_string(),
      enable_history: true,
      history_retention_days: 90,
      enable_metadata_indexing: true,
      enable_asset_caching: true,
    }
  }
}

pub fn create_project_manager(config: ProjectManagerConfig) -> Result<ProjectManager> {
  ProjectManager::new(config)
}

pub fn create_project_manager_config() -> ProjectManagerConfig {
  ProjectManagerConfig::default()
}

pub fn create_project(name: String, description: String, project_type: ProjectType) -> Project {
  let project_id = Uuid::new_v4();
  let now = Utc::now();

  Project {
    id: project_id,
    name,
    description,
    project_type,
    created_at: now,
    updated_at: now,
    metadata: ProjectMetadata::new(),
    workspace: Workspace::new(project_id),
    sessions: Vec::new(),
    assets: AssetCollection::new(),
    history: ProjectHistory::new(),
    settings: ProjectSettings::new(),
    status: ProjectStatus::Active,
  }
}
