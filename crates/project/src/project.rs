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

#[derive(Debug, Clone)]
pub struct Workspace {
  pub id: Uuid,
  pub project_id: Uuid,
  pub name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub layout: WorkspaceLayout,
  pub panels: Vec<WorkspacePanel>,
  pub tabs: Vec<WorkspaceTab>,
  pub active_tab: Option<Uuid>,
  pub view_settings: ViewSettings,
}

#[derive(Debug, Clone)]
pub struct WorkspacePanel {
  pub id: Uuid,
  pub name: String,
  pub panel_type: PanelType,
  pub position: PanelPosition,
  pub size: PanelSize,
  pub visible: bool,
  pub docked: bool,
  pub content: PanelContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelType {
  MediaViewer,
  EffectControls,
  PipelineEditor,
  ScriptEditor,
  AssetBrowser,
  History,
  Properties,
  Console,
  Preview,
  Custom,
}

#[derive(Debug, Clone)]
pub struct PanelPosition {
  pub x: f64,
  pub y: f64,
  pub z: u32,
}

#[derive(Debug, Clone)]
pub struct PanelSize {
  pub width: f64,
  pub height: f64,
}

#[derive(Debug, Clone)]
pub struct PanelContent {
  pub content_type: ContentType,
  pub data: HashMap<String, String>,
  pub state: PanelState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
  Empty,
  Media,
  Text,
  Graph,
  Table,
  Custom,
}

#[derive(Debug, Clone)]
pub struct PanelState {
  pub scroll_x: f64,
  pub scroll_y: f64,
  pub zoom: f64,
  pub selected_items: Vec<String>,
  pub active_item: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceTab {
  pub id: Uuid,
  pub name: String,
  pub content_type: TabContentType,
  pub content: TabContent,
  pub created_at: DateTime<Utc>,
  pub modified: bool,
  pub saved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabContentType {
  Media,
  Script,
  Pipeline,
  Configuration,
  Custom,
}

#[derive(Debug, Clone)]
pub struct TabContent {
  pub content_type: TabContentType,
  pub data: HashMap<String, String>,
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ViewSettings {
  pub zoom_level: f64,
  pub pan_x: f64,
  pub pan_y: f64,
  pub show_grid: bool,
  pub show_rulers: bool,
  pub show_guides: bool,
  pub snap_to_grid: bool,
  pub grid_size: f64,
  pub background_color: String,
  pub foreground_color: String,
}

#[derive(Debug, Clone)]
pub struct AssetCollection {
  pub id: Uuid,
  pub project_id: Uuid,
  pub assets: HashMap<String, Asset>,
  pub folders: HashMap<String, AssetFolder>,
  pub tags: HashMap<String, AssetTag>,
  pub metadata: AssetCollectionMetadata,
}

#[derive(Debug, Clone)]
pub struct Asset {
  pub id: Uuid,
  pub name: String,
  pub asset_type: AssetType,
  pub file_path: String,
  pub size_bytes: u64,
  pub created_at: DateTime<Utc>,
  pub modified_at: DateTime<Utc>,
  pub metadata: AssetMetadata,
  pub thumbnail: Option<String>,
  pub preview: Option<String>,
  pub tags: Vec<String>,
  pub folder_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
  Image,
  Audio,
  Video,
  Script,
  Pipeline,
  Configuration,
  Other,
}

#[derive(Debug, Clone)]
pub struct AssetMetadata {
  pub width: Option<u32>,
  pub height: Option<u32>,
  pub duration: Option<f64>,
  pub format: String,
  pub codec: Option<String>,
  pub bit_rate: Option<u32>,
  pub sample_rate: Option<u32>,
  pub channels: Option<u32>,
  pub color_space: Option<String>,
  pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AssetFolder {
  pub id: String,
  pub name: String,
  pub parent_id: Option<String>,
  pub children: Vec<String>,
  pub created_at: DateTime<Utc>,
  pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AssetTag {
  pub id: String,
  pub name: String,
  pub color: String,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AssetCollectionMetadata {
  pub total_assets: usize,
  pub total_size_bytes: u64,
  pub last_modified: DateTime<Utc>,
  pub asset_types: HashMap<AssetType, usize>,
  pub tags: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct ProjectHistory {
  pub id: Uuid,
  pub project_id: Uuid,
  pub events: Vec<HistoryEvent>,
  pub snapshots: Vec<ProjectSnapshot>,
  pub bookmarks: Vec<HistoryBookmark>,
  pub settings: HistorySettings,
}

#[derive(Debug, Clone)]
pub struct HistoryEvent {
  pub id: Uuid,
  pub timestamp: DateTime<Utc>,
  pub event_type: HistoryEventType,
  pub description: String,
  pub user_id: Option<String>,
  pub session_id: Option<Uuid>,
  pub data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum HistoryEventType {
  ProjectCreated,
  ProjectSaved,
  ProjectDeleted,
  ProjectArchived,
  ProjectRestored,
  SessionCreated,
  SessionSaved,
  SessionDeleted,
  AssetAdded,
  AssetRemoved,
  AssetModified,
  WorkspaceChanged,
  SettingsChanged,
  Custom,
}

#[derive(Debug, Clone)]
pub struct ProjectSnapshot {
  pub id: Uuid,
  pub timestamp: DateTime<Utc>,
  pub name: String,
  pub description: String,
  pub project_data: String,
  pub thumbnail: Option<String>,
  pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HistoryBookmark {
  pub id: Uuid,
  pub name: String,
  pub timestamp: DateTime<Utc>,
  pub event_id: Uuid,
  pub description: String,
  pub color: String,
}

#[derive(Debug, Clone)]
pub struct HistorySettings {
  pub enabled: bool,
  pub max_events: usize,
  pub max_snapshots: usize,
  pub auto_snapshot_interval_seconds: u64,
  pub compression_enabled: bool,
  pub encryption_enabled: bool,
}

impl Project {
  pub fn new(name: String, description: String, project_type: ProjectType) -> Self {
    let project_id = Uuid::new_v4();
    let now = Utc::now();

    Self {
      id: project_id,
      name,
      description,
      project_type,
      created_at: now,
      updated_at: now,
      metadata: ProjectMetadata::new(),
      workspace: Workspace::new(project_id),
      sessions: Vec::new(),
      assets: AssetCollection::new(project_id),
      history: ProjectHistory::new(project_id),
      settings: ProjectSettings::new(),
      status: ProjectStatus::Active,
    }
  }

  pub fn with_metadata(mut self, metadata: ProjectMetadata) -> Self {
    self.metadata = metadata;
    self
  }

  pub fn with_settings(mut self, settings: ProjectSettings) -> Self {
    self.settings = settings;
    self
  }

  pub fn add_session(&mut self, session: Session) {
    self.sessions.push(session);
    self.updated_at = Utc::now();
  }

  pub fn remove_session(&mut self, session_id: Uuid) -> Option<Session> {
    let index = self.sessions.iter().position(|s| s.id == session_id)?;
    let session = self.sessions.remove(index);
    self.updated_at = Utc::now();
    Some(session)
  }

  pub fn add_asset(&mut self, asset: Asset) {
    self.assets.add_asset(asset);
    self.updated_at = Utc::now();
  }

  pub fn remove_asset(&mut self, asset_id: &str) -> Option<Asset> {
    let asset = self.assets.remove_asset(asset_id);
    if asset.is_some() {
      self.updated_at = Utc::now();
    }
    asset
  }

  pub fn get_asset(&self, asset_id: &str) -> Option<&Asset> {
    self.assets.get_asset(asset_id)
  }

  pub fn list_assets(&self) -> Vec<&Asset> {
    self.assets.list_assets()
  }

  pub fn search_assets(&self, query: &str) -> Vec<&Asset> {
    self.assets.search_assets(query)
  }

  pub fn record_event(&mut self, event: HistoryEvent) {
    self.history.add_event(event);
    self.updated_at = Utc::now();
  }

  pub fn create_snapshot(&mut self, name: String, description: String) -> Uuid {
    let snapshot_id = self.history.create_snapshot(name, description, &self);
    self.updated_at = Utc::now();
    snapshot_id
  }

  pub fn restore_snapshot(&mut self, snapshot_id: Uuid) -> Result<()> {
    self.history.restore_snapshot(snapshot_id, self)?;
    self.updated_at = Utc::now();
    Ok(())
  }

  pub fn add_bookmark(&mut self, name: String, event_id: Uuid, description: String) -> Uuid {
    let bookmark_id = self.history.add_bookmark(name, event_id, description);
    self.updated_at = Utc::now();
    bookmark_id
  }

  pub fn get_workspace(&self) -> &Workspace {
    &self.workspace
  }

  pub fn get_workspace_mut(&mut self) -> &mut Workspace {
    &mut self.workspace
  }

  pub fn get_assets(&self) -> &AssetCollection {
    &self.assets
  }

  pub fn get_assets_mut(&mut self) -> &mut AssetCollection {
    &mut self.assets
  }

  pub fn get_history(&self) -> &ProjectHistory {
    &self.history
  }

  pub fn get_history_mut(&mut self) -> &mut ProjectHistory {
    &mut self.history
  }

  pub fn update_timestamp(&mut self) {
    self.updated_at = Utc::now();
  }

  pub fn is_active(&self) -> bool {
    self.status == ProjectStatus::Active
  }

  pub fn is_archived(&self) -> bool {
    self.status == ProjectStatus::Archived
  }

  pub fn archive(&mut self) {
    self.status = ProjectStatus::Archived;
    self.updated_at = Utc::now();
  }

  pub fn restore(&mut self) {
    self.status = ProjectStatus::Active;
    self.updated_at = Utc::now();
  }

  pub fn delete(&mut self) {
    self.status = ProjectStatus::Deleted;
    self.updated_at = Utc::now();
  }

  pub fn clone(&self) -> Project {
    Project {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      project_type: self.project_type,
      created_at: self.created_at,
      updated_at: self.updated_at,
      metadata: self.metadata.clone(),
      workspace: self.workspace.clone(),
      sessions: self.sessions.clone(),
      assets: self.assets.clone(),
      history: self.history.clone(),
      settings: self.settings.clone(),
      status: self.status,
    }
  }
}

impl Workspace {
  pub fn new(project_id: Uuid) -> Self {
    let workspace_id = Uuid::new_v4();
    let now = Utc::now();

    Self {
      id: workspace_id,
      project_id,
      name: "Default Workspace".to_string(),
      created_at: now,
      updated_at: now,
      layout: WorkspaceLayout::Default,
      panels: Vec::new(),
      tabs: Vec::new(),
      active_tab: None,
      view_settings: ViewSettings::new(),
    }
  }

  pub fn with_name(mut self, name: String) -> Self {
    self.name = name;
    self
  }

  pub fn with_layout(mut self, layout: WorkspaceLayout) -> Self {
    self.layout = layout;
    self
  }

  pub fn add_panel(&mut self, panel: WorkspacePanel) {
    self.panels.push(panel);
    self.updated_at = Utc::now();
  }

  pub fn remove_panel(&mut self, panel_id: Uuid) -> Option<WorkspacePanel> {
    let index = self.panels.iter().position(|p| p.id == panel_id)?;
    let panel = self.panels.remove(index);
    self.updated_at = Utc::now();
    Some(panel)
  }

  pub fn get_panel(&self, panel_id: Uuid) -> Option<&WorkspacePanel> {
    self.panels.iter().find(|p| p.id == panel_id)
  }

  pub fn get_panel_mut(&mut self, panel_id: Uuid) -> Option<&mut WorkspacePanel> {
    self.panels.iter_mut().find(|p| p.id == panel_id)
  }

  pub fn list_panels(&self) -> &[WorkspacePanel] {
    &self.panels
  }

  pub fn add_tab(&mut self, tab: WorkspaceTab) {
    self.tabs.push(tab);
    self.updated_at = Utc::now();
  }

  pub fn remove_tab(&mut self, tab_id: Uuid) -> Option<WorkspaceTab> {
    let index = self.tabs.iter().position(|t| t.id == tab_id)?;
    let tab = self.tabs.remove(index);

    if self.active_tab == Some(tab_id) {
      self.active_tab = self.tabs.last().map(|t| t.id);
    }

    self.updated_at = Utc::now();
    Some(tab)
  }

  pub fn get_tab(&self, tab_id: Uuid) -> Option<&WorkspaceTab> {
    self.tabs.iter().find(|t| t.id == tab_id)
  }

  pub fn get_tab_mut(&mut self, tab_id: Uuid) -> Option<&mut WorkspaceTab> {
    self.tabs.iter_mut().find(|t| t.id == tab_id)
  }

  pub fn list_tabs(&self) -> &[WorkspaceTab] {
    &self.tabs
  }

  pub fn set_active_tab(&mut self, tab_id: Option<Uuid>) {
    self.active_tab = tab_id;
    self.updated_at = Utc::now();
  }

  pub fn get_active_tab(&self) -> Option<&WorkspaceTab> {
    self.active_tab.and_then(|id| self.get_tab(id))
  }

  pub fn update_view_settings(&mut self, settings: ViewSettings) {
    self.view_settings = settings;
    self.updated_at = Utc::now();
  }

  pub fn clone(&self) -> Workspace {
    Workspace {
      id: self.id,
      project_id: self.project_id,
      name: self.name.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
      layout: self.layout,
      panels: self.panels.clone(),
      tabs: self.tabs.clone(),
      active_tab: self.active_tab,
      view_settings: self.view_settings.clone(),
    }
  }
}

impl AssetCollection {
  pub fn new(project_id: Uuid) -> Self {
    Self {
      id: Uuid::new_v4(),
      project_id,
      assets: HashMap::new(),
      folders: HashMap::new(),
      tags: HashMap::new(),
      metadata: AssetCollectionMetadata::new(),
    }
  }

  pub fn add_asset(&mut self, asset: Asset) {
    self.assets.insert(asset.id.to_string(), asset.clone());
    self.update_metadata();
  }

  pub fn remove_asset(&mut self, asset_id: &str) -> Option<Asset> {
    let asset = self.assets.remove(asset_id);
    if asset.is_some() {
      self.update_metadata();
    }
    asset
  }

  pub fn get_asset(&self, asset_id: &str) -> Option<&Asset> {
    self.assets.get(asset_id)
  }

  pub fn list_assets(&self) -> Vec<&Asset> {
    self.assets.values().collect()
  }

  pub fn search_assets(&self, query: &str) -> Vec<&Asset> {
    let query = query.to_lowercase();
    self
      .assets
      .values()
      .filter(|asset| {
        asset.name.to_lowercase().contains(&query)
          || asset
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query))
          || asset
            .metadata
            .custom_fields
            .values()
            .any(|value| value.to_lowercase().contains(&query))
      })
      .collect()
  }

  pub fn add_folder(&mut self, folder: AssetFolder) {
    self.folders.insert(folder.id.clone(), folder.clone());
  }

  pub fn remove_folder(&mut self, folder_id: &str) -> Option<AssetFolder> {
    self.folders.remove(folder_id)
  }

  pub fn get_folder(&self, folder_id: &str) -> Option<&AssetFolder> {
    self.folders.get(folder_id)
  }

  pub fn list_folders(&self) -> Vec<&AssetFolder> {
    self.folders.values().collect()
  }

  pub fn add_tag(&mut self, tag: AssetTag) {
    self.tags.insert(tag.id.clone(), tag.clone());
  }

  pub fn remove_tag(&mut self, tag_id: &str) -> Option<AssetTag> {
    self.tags.remove(tag_id)
  }

  pub fn get_tag(&self, tag_id: &str) -> Option<&AssetTag> {
    self.tags.get(tag_id)
  }

  pub fn list_tags(&self) -> Vec<&AssetTag> {
    self.tags.values().collect()
  }

  fn update_metadata(&mut self) {
    self.metadata.total_assets = self.assets.len();
    self.metadata.total_size_bytes = self.assets.values().map(|a| a.size_bytes).sum();
    self.metadata.last_modified = Utc::now();

    self.metadata.asset_types.clear();
    for asset in self.assets.values() {
      *self
        .metadata
        .asset_types
        .entry(asset.asset_type)
        .or_insert(0) += 1;
    }

    self.metadata.tags.clear();
    for asset in self.assets.values() {
      for tag in &asset.tags {
        *self.metadata.tags.entry(tag.clone()).or_insert(0) += 1;
      }
    }
  }

  pub fn clone(&self) -> AssetCollection {
    AssetCollection {
      id: self.id,
      project_id: self.project_id,
      assets: self.assets.clone(),
      folders: self.folders.clone(),
      tags: self.tags.clone(),
      metadata: self.metadata.clone(),
    }
  }
}

impl ProjectHistory {
  pub fn new(project_id: Uuid) -> Self {
    Self {
      id: Uuid::new_v4(),
      project_id,
      events: Vec::new(),
      snapshots: Vec::new(),
      bookmarks: Vec::new(),
      settings: HistorySettings::new(),
    }
  }

  pub fn add_event(&mut self, event: HistoryEvent) {
    self.events.push(event);

    if self.events.len() > self.settings.max_events {
      self.events.drain(0..self.events.len() / 2);
    }
  }

  pub fn list_events(&self) -> &[HistoryEvent] {
    &self.events
  }

  pub fn get_event(&self, event_id: Uuid) -> Option<&HistoryEvent> {
    self.events.iter().find(|e| e.id == event_id)
  }

  pub fn create_snapshot(&mut self, name: String, description: String, project: &Project) -> Uuid {
    let snapshot_id = Uuid::new_v4();

    let project_data = serde_json::to_string(project).unwrap_or_default();

    let snapshot = ProjectSnapshot {
      id: snapshot_id,
      timestamp: Utc::now(),
      name,
      description,
      project_data,
      thumbnail: None,
      tags: Vec::new(),
    };

    self.snapshots.push(snapshot.clone());

    if self.snapshots.len() > self.settings.max_snapshots {
      self.snapshots.drain(0..self.snapshots.len() / 2);
    }

    snapshot_id
  }

  pub fn list_snapshots(&self) -> &[ProjectSnapshot] {
    &self.snapshots
  }

  pub fn get_snapshot(&self, snapshot_id: Uuid) -> Option<&ProjectSnapshot> {
    self.snapshots.iter().find(|s| s.id == snapshot_id)
  }

  pub fn restore_snapshot(&self, snapshot_id: Uuid, project: &mut Project) -> Result<()> {
    let snapshot = self
      .snapshots
      .iter()
      .find(|s| s.id == snapshot_id)
      .ok_or_else(|| {
        EllasticError::InvalidParameter(format!("Snapshot {} not found", snapshot_id))
      })?;

    let restored_project: Project = serde_json::from_str(&snapshot.project_data).map_err(|e| {
      EllasticError::SerializationError(format!("Failed to restore snapshot: {}", e))
    })?;

    project.name = restored_project.name;
    project.description = restored_project.description;
    project.metadata = restored_project.metadata;
    project.workspace = restored_project.workspace;
    project.settings = restored_project.settings;

    Ok(())
  }

  pub fn add_bookmark(&mut self, name: String, event_id: Uuid, description: String) -> Uuid {
    let bookmark_id = Uuid::new_v4();

    let bookmark = HistoryBookmark {
      id: bookmark_id,
      name,
      timestamp: Utc::now(),
      event_id,
      description,
      color: "#FF5722".to_string(),
    };

    self.bookmarks.push(bookmark);
    bookmark_id
  }

  pub fn list_bookmarks(&self) -> &[HistoryBookmark] {
    &self.bookmarks
  }

  pub fn get_bookmark(&self, bookmark_id: Uuid) -> Option<&HistoryBookmark> {
    self.bookmarks.iter().find(|b| b.id == bookmark_id)
  }

  pub fn remove_bookmark(&mut self, bookmark_id: Uuid) -> Option<HistoryBookmark> {
    let index = self.bookmarks.iter().position(|b| b.id == bookmark_id)?;
    Some(self.bookmarks.remove(index))
  }

  pub fn clone(&self) -> ProjectHistory {
    ProjectHistory {
      id: self.id,
      project_id: self.project_id,
      events: self.events.clone(),
      snapshots: self.snapshots.clone(),
      bookmarks: self.bookmarks.clone(),
      settings: self.settings.clone(),
    }
  }
}

impl Default for ProjectMetadata {
  fn default() -> Self {
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
}

impl Default for ProjectSettings {
  fn default() -> Self {
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
}

impl Default for PerformanceSettings {
  fn default() -> Self {
    Self {
      max_memory_mb: 1024,
      max_cpu_cores: 4,
      enable_parallel_processing: true,
      cache_size_mb: 256,
      optimization_level: OptimizationLevel::Standard,
    }
  }
}

impl Default for ExportSettings {
  fn default() -> Self {
    Self {
      default_format: "png".to_string(),
      default_quality: 90,
      compression_level: 6,
      include_metadata: true,
      watermark_enabled: false,
      watermark_text: None,
    }
  }
}

impl Default for SecuritySettings {
  fn default() -> Self {
    Self {
      encryption_enabled: false,
      password_protected: false,
      access_control: false,
      audit_logging: false,
      allowed_users: Vec::new(),
      blocked_users: Vec::new(),
    }
  }
}

impl Default for ViewSettings {
  fn default() -> Self {
    Self {
      zoom_level: 1.0,
      pan_x: 0.0,
      pan_y: 0.0,
      show_grid: false,
      show_rulers: false,
      show_guides: false,
      snap_to_grid: false,
      grid_size: 10.0,
      background_color: "#FFFFFF".to_string(),
      foreground_color: "#000000".to_string(),
    }
  }
}

impl Default for AssetCollectionMetadata {
  fn default() -> Self {
    Self {
      total_assets: 0,
      total_size_bytes: 0,
      last_modified: Utc::now(),
      asset_types: HashMap::new(),
      tags: HashMap::new(),
    }
  }
}

impl Default for HistorySettings {
  fn default() -> Self {
    Self {
      enabled: true,
      max_events: 1000,
      max_snapshots: 50,
      auto_snapshot_interval_seconds: 3600,
      compression_enabled: false,
      encryption_enabled: false,
    }
  }
}

pub fn create_project(name: String, description: String, project_type: ProjectType) -> Project {
  Project::new(name, description, project_type)
}

pub fn create_workspace(project_id: Uuid) -> Workspace {
  Workspace::new(project_id)
}

pub fn create_asset_collection(project_id: Uuid) -> AssetCollection {
  AssetCollection::new(project_id)
}

pub fn create_project_history(project_id: Uuid) -> ProjectHistory {
  ProjectHistory::new(project_id)
}
