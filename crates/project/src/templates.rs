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
pub struct TemplateManager {
  templates: Arc<RwLock<HashMap<Uuid, Template>>>,
  template_categories: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
  template_tags: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
  search_index: Arc<RwLock<TemplateSearchIndex>>,
  config: TemplateManagerConfig,
}

#[derive(Debug, Clone)]
pub struct TemplateManagerConfig {
  pub max_templates: usize,
  pub template_directory: String,
  pub auto_load_enabled: bool,
  pub auto_save_enabled: bool,
  pub backup_enabled: bool,
  pub backup_retention_days: u32,
  pub validation_enabled: bool,
  pub compression_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct Template {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub template_type: TemplateType,
  pub category: String,
  pub tags: Vec<String>,
  pub author: String,
  pub version: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub metadata: TemplateMetadata,
  pub content: TemplateContent,
  pub preview: Option<TemplatePreview>,
  pub dependencies: Vec<TemplateDependency>,
  pub compatibility: TemplateCompatibility,
  pub validation: TemplateValidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
  Project,
  Workspace,
  Session,
  Pipeline,
  Script,
  Effect,
  Asset,
  Configuration,
  Custom,
}

#[derive(Debug, Clone)]
pub struct TemplateMetadata {
  pub license: String,
  pub homepage: Option<String>,
  pub repository: Option<String>,
  pub documentation: Option<String>,
  pub keywords: Vec<String>,
  pub requirements: Vec<String>,
  pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TemplateContent {
  pub content_type: TemplateContentType,
  pub data: HashMap<String, String>,
  pub files: Vec<TemplateFile>,
  pub resources: Vec<TemplateResource>,
  pub parameters: Vec<TemplateParameter>,
  pub variables: HashMap<String, TemplateVariable>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateContentType {
  JSON,
  YAML,
  XML,
  TOML,
  Binary,
  Text,
  Mixed,
}

#[derive(Debug, Clone)]
pub struct TemplateFile {
  pub name: String,
  pub path: String,
  pub content_type: String,
  pub content: Vec<u8>,
  pub size_bytes: usize,
  pub checksum: String,
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TemplateResource {
  pub name: String,
  pub resource_type: ResourceType,
  pub path: String,
  pub data: Vec<u8>,
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
  Image,
  Audio,
  Video,
  Font,
  Icon,
  Script,
  Configuration,
  Other,
}

#[derive(Debug, Clone)]
pub struct TemplateParameter {
  pub name: String,
  pub parameter_type: ParameterType,
  pub default_value: ParameterValue,
  pub description: String,
  pub required: bool,
  pub validation_rules: Vec<ValidationRule>,
  pub options: Vec<ParameterOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
  String,
  Number,
  Boolean,
  Integer,
  Float,
  Array,
  Object,
  Enum,
  File,
  Directory,
  Color,
  Font,
  Custom,
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
  String(String),
  Number(f64),
  Boolean(bool),
  Integer(i64),
  Float(f64),
  Array(Vec<ParameterValue>),
  Object(HashMap<String, ParameterValue>),
  Enum(String),
  File(String),
  Directory(String),
  Color(String),
  Font(String),
  Custom(String),
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
  pub rule_type: ValidationRuleType,
  pub parameters: HashMap<String, String>,
  pub error_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationRuleType {
  Required,
  Min,
  Max,
  Range,
  Pattern,
  Length,
  Enum,
  Custom,
}

#[derive(Debug, Clone)]
pub struct ParameterOption {
  pub value: ParameterValue,
  pub label: String,
  pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateVariable {
  pub name: String,
  pub value: ParameterValue,
  pub description: String,
  pub scope: VariableScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableScope {
  Global,
  Local,
  Template,
  Instance,
}

#[derive(Debug, Clone)]
pub struct TemplatePreview {
  pub id: Uuid,
  pub image_data: Vec<u8>,
  pub format: String,
  pub width: u32,
  pub height: u32,
  pub generated_at: DateTime<Utc>,
  pub template_version: String,
}

#[derive(Debug, Clone)]
pub struct TemplateDependency {
  pub name: String,
  pub version_requirement: String,
  pub optional: bool,
  pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
  Template,
  Library,
  Plugin,
  System,
  External,
}

#[derive(Debug, Clone)]
pub struct TemplateCompatibility {
  pub ellastic_versions: Vec<String>,
  pub platform_requirements: Vec<String>,
  pub system_requirements: Vec<String>,
  pub incompatible_versions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateValidation {
  pub validation_rules: Vec<ValidationRule>,
  pub validation_results: Vec<ValidationResult>,
  pub last_validated: DateTime<Utc>,
  pub validation_status: ValidationStatus,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
  pub rule_name: String,
  pub passed: bool,
  pub message: String,
  pub severity: ValidationSeverity,
  pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
  Info,
  Warning,
  Error,
  Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStatus {
  Valid,
  Invalid,
  Warning,
  Unknown,
}

#[derive(Debug, Clone)]
pub struct TemplateSearchIndex {
  pub index: HashMap<String, Vec<Uuid>>,
  pub keywords: HashMap<String, Vec<Uuid>>,
  pub descriptions: HashMap<String, Vec<Uuid>>,
  pub categories: HashMap<String, Vec<Uuid>>,
  pub tags: HashMap<String, Vec<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct TemplateInstance {
  pub id: Uuid,
  pub template_id: Uuid,
  pub name: String,
  pub parameters: HashMap<String, ParameterValue>,
  pub variables: HashMap<String, TemplateVariable>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub instance_data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TemplateRenderer {
  pub id: Uuid,
  pub template_id: Uuid,
  pub renderer_type: RendererType,
  pub config: RendererConfig,
  pub context: RenderContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererType {
  Handlebars,
  Jinja2,
  Mustache,
  Liquid,
  Custom,
}

#[derive(Debug, Clone)]
pub struct RendererConfig {
  pub strict_mode: bool,
  pub auto_escape: bool,
  pub trim_blocks: bool,
  pub lstrip_blocks: bool,
  pub keep_trailing_newline: bool,
  pub custom_filters: HashMap<String, String>,
  pub custom_functions: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RenderContext {
  pub variables: HashMap<String, ParameterValue>,
  pub functions: HashMap<String, String>,
  pub filters: HashMap<String, String>,
  pub globals: HashMap<String, ParameterValue>,
  pub locale: String,
  pub timezone: String,
}

impl TemplateManager {
  pub fn new(config: TemplateManagerConfig) -> Result<Self> {
    let mut manager = Self {
      templates: Arc::new(RwLock::new(HashMap::new())),
      template_categories: Arc::new(RwLock::new(HashMap::new())),
      template_tags: Arc::new(RwLock::new(HashMap::new())),
      search_index: Arc::new(RwLock::new(TemplateSearchIndex::new())),
      config,
    };

    manager.initialize()?;
    Ok(manager)
  }

  pub fn config(&self) -> &TemplateManagerConfig {
    &self.config
  }

  pub fn templates(&self) -> Arc<RwLock<HashMap<Uuid, Template>>> {
    self.templates.clone()
  }

  pub fn template_categories(&self) -> Arc<RwLock<HashMap<String, Vec<Uuid>>>> {
    self.template_categories.clone()
  }

  pub fn template_tags(&self) -> Arc<RwLock<HashMap<String, Vec<Uuid>>>> {
    self.template_tags.clone()
  }

  pub fn search_index(&self) -> Arc<RwLock<TemplateSearchIndex>> {
    self.search_index.clone()
  }

  fn initialize(&mut self) -> Result<()> {
    std::fs::create_dir_all(&self.config.template_directory)?;

    self.load_default_templates()?;

    if self.config.auto_load_enabled {
      self.load_templates()?;
    }

    Ok(())
  }

  pub fn register_template(&mut self, template: Template) -> Result<Uuid> {
    let template_id = template.id;

    if self.templates.read().len() >= self.config.max_templates {
      return Err(EllasticError::LimitExceeded(
        "Maximum template limit reached".to_string(),
      ));
    }

    if self.config.validation_enabled {
      self.validate_template(&template)?;
    }

    self.templates.write().insert(template_id, template.clone());

    self.update_search_index(&template);
    self.update_category_index(&template);
    self.update_tag_index(&template);

    if self.config.auto_save_enabled {
      self.save_template_to_disk(&template)?;
    }

    Ok(template_id)
  }

  pub fn unregister_template(&mut self, template_id: Uuid) -> Option<Template> {
    let template = self.templates.write().remove(&template_id);

    if let Some(ref template) = template {
      self.remove_from_search_index(template);
      self.remove_from_category_index(template);
      self.remove_from_tag_index(template);

      self.delete_template_from_disk(template_id)?;
    }

    template
  }

  pub fn get_template(&self, template_id: Uuid) -> Option<&Template> {
    self.templates.read().get(&template_id)
  }

  pub fn list_templates(&self) -> Vec<&Template> {
    self.templates.read().values().collect()
  }

  pub fn list_templates_by_category(&self, category: &str) -> Vec<&Template> {
    if let Some(template_ids) = self.template_categories.read().get(category) {
      template_ids
        .iter()
        .filter_map(|id| self.templates.read().get(id))
        .collect()
    } else {
      Vec::new()
    }
  }

  pub fn list_templates_by_tag(&self, tag: &str) -> Vec<&Template> {
    if let Some(template_ids) = self.template_tags.read().get(tag) {
      template_ids
        .iter()
        .filter_map(|id| self.templates.read().get(id))
        .collect()
    } else {
      Vec::new()
    }
  }

  pub fn search_templates(&self, query: &str) -> Vec<&Template> {
    let search_index = self.search_index.read();
    let templates = self.templates.read();

    let query = query.to_lowercase();
    let mut results = HashSet::new();

    if let Some(matches) = search_index.index.get(&query) {
      for template_id in matches {
        results.insert(*template_id);
      }
    }

    for word in query.split_whitespace() {
      if let Some(matches) = search_index.keywords.get(word) {
        for template_id in matches {
          results.insert(*template_id);
        }
      }
    }

    for word in query.split_whitespace() {
      if let Some(matches) = search_index.descriptions.get(word) {
        for template_id in matches {
          results.insert(*template_id);
        }
      }
    }

    results.iter().filter_map(|id| templates.get(id)).collect()
  }

  pub fn create_instance(
    &self,
    template_id: Uuid,
    name: String,
    parameters: HashMap<String, ParameterValue>,
  ) -> Result<TemplateInstance> {
    let template = self.get_template(template_id).ok_or_else(|| {
      EllasticError::InvalidParameter(format!("Template {} not found", template_id))
    })?;

    self.validate_parameters(&template, &parameters)?;

    let instance = TemplateInstance {
      id: Uuid::new_v4(),
      template_id,
      name,
      parameters,
      variables: HashMap::new(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      instance_data: HashMap::new(),
    };

    Ok(instance)
  }

  pub fn render_template(&self, instance: &TemplateInstance) -> Result<String> {
    let template = self.get_template(instance.template_id).ok_or_else(|| {
      EllasticError::InvalidParameter(format!("Template {} not found", instance.template_id))
    })?;

    let renderer = TemplateRenderer::new(template.id);

    let context = RenderContext {
      variables: instance.parameters.clone(),
      functions: HashMap::new(),
      filters: HashMap::new(),
      globals: HashMap::new(),
      locale: "en".to_string(),
      timezone: "UTC".to_string(),
    };

    renderer.render(&template.content, &context)
  }

  pub fn validate_template(&self, template: &Template) -> Result<()> {
    let mut validation_results = Vec::new();

    if template.name.is_empty() {
      validation_results.push(ValidationResult {
        rule_name: "name_required".to_string(),
        passed: false,
        message: "Template name is required".to_string(),
        severity: ValidationSeverity::Error,
        timestamp: Utc::now(),
      });
    }

    if template.description.is_empty() {
      validation_results.push(ValidationResult {
        rule_name: "description_required".to_string(),
        passed: false,
        message: "Template description is required".to_string(),
        severity: ValidationSeverity::Warning,
        timestamp: Utc::now(),
      });
    }

    if template.content.files.is_empty() && template.content.data.is_empty() {
      validation_results.push(ValidationResult {
        rule_name: "content_required".to_string(),
        passed: false,
        message: "Template must have content".to_string(),
        severity: ValidationSeverity::Error,
        timestamp: Utc::now(),
      });
    }

    for dependency in &template.dependencies {
      if dependency.name.is_empty() {
        validation_results.push(ValidationResult {
          rule_name: "dependency_name_required".to_string(),
          passed: false,
          message: "Dependency name is required".to_string(),
          severity: ValidationSeverity::Error,
          timestamp: Utc::now(),
        });
      }
    }

    let has_errors = validation_results.iter().any(|r| {
      matches!(
        r.severity,
        ValidationSeverity::Error | ValidationSeverity::Critical
      )
    });
    if has_errors {
      return Err(EllasticError::ValidationError(
        "Template validation failed".to_string(),
      ));
    }

    Ok(())
  }

  fn validate_parameters(
    &self,
    template: &Template,
    parameters: &HashMap<String, ParameterValue>,
  ) -> Result<()> {
    for param in &template.content.parameters {
      if param.required && !parameters.contains_key(&param.name) {
        return Err(EllasticError::InvalidParameter(format!(
          "Required parameter '{}' not provided",
          param.name
        )));
      }

      if let Some(value) = parameters.get(&param.name) {
        self.validate_parameter_value(param, value)?;
      }
    }

    Ok(())
  }

  fn validate_parameter_value(
    &self,
    param: &TemplateParameter,
    value: &ParameterValue,
  ) -> Result<()> {
    if !self.is_compatible_type(&param.parameter_type, value) {
      return Err(EllasticError::InvalidParameter(format!(
        "Parameter '{}' type mismatch: expected {:?}, got {:?}",
        param.name, param.parameter_type, value
      )));
    }

    for rule in &param.validation_rules {
      self.apply_validation_rule(rule, value)?;
    }

    Ok(())
  }

  fn is_compatible_type(&self, param_type: &ParameterType, value: &ParameterValue) -> bool {
    match (param_type, value) {
      (ParameterType::String, ParameterValue::String(_)) => true,
      (ParameterType::Number, ParameterValue::Number(_)) => true,
      (ParameterType::Boolean, ParameterValue::Boolean(_)) => true,
      (ParameterType::Integer, ParameterValue::Integer(_)) => true,
      (ParameterType::Float, ParameterValue::Float(_)) => true,
      (ParameterType::Array, ParameterValue::Array(_)) => true,
      (ParameterType::Object, ParameterValue::Object(_)) => true,
      (ParameterType::Enum, ParameterValue::Enum(_)) => true,
      (ParameterType::File, ParameterValue::File(_)) => true,
      (ParameterType::Directory, ParameterValue::Directory(_)) => true,
      (ParameterType::Color, ParameterValue::Color(_)) => true,
      (ParameterType::Font, ParameterValue::Font(_)) => true,
      (ParameterType::Custom, _) => true,
      _ => false,
    }
  }

  fn apply_validation_rule(&self, rule: &ValidationRule, value: &ParameterValue) -> Result<()> {
    match rule.rule_type {
      ValidationRuleType::Required => {}
      ValidationRuleType::Min => {
        if let Some(min_str) = rule.parameters.get("value") {
          if let ParameterValue::Number(n) = value {
            let min_val: f64 = min_str.parse().unwrap_or(0.0);
            if *n < min_val {
              return Err(EllasticError::InvalidParameter(format!(
                "Value {} is less than minimum {}",
                n, min_val
              )));
            }
          }
        }
      }
      ValidationRuleType::Max => {
        if let Some(max_str) = rule.parameters.get("value") {
          if let ParameterValue::Number(n) = value {
            let max_val: f64 = max_str.parse().unwrap_or(f64::MAX);
            if *n > max_val {
              return Err(EllasticError::InvalidParameter(format!(
                "Value {} is greater than maximum {}",
                n, max_val
              )));
            }
          }
        }
      }
      ValidationRuleType::Range => {
        if let (Some(min_str), Some(max_str)) =
          (rule.parameters.get("min"), rule.parameters.get("max"))
        {
          if let ParameterValue::Number(n) = value {
            let min_val: f64 = min_str.parse().unwrap_or(0.0);
            let max_val: f64 = max_str.parse().unwrap_or(f64::MAX);
            if *n < min_val || *n > max_val {
              return Err(EllasticError::InvalidParameter(format!(
                "Value {} is not in range [{}, {}]",
                n, min_val, max_val
              )));
            }
          }
        }
      }
      ValidationRuleType::Pattern => {
        if let Some(pattern) = rule.parameters.get("pattern") {
          if let ParameterValue::String(s) = value {
            let regex = regex::Regex::new(pattern)
              .map_err(|_| EllasticError::InvalidParameter("Invalid regex pattern".to_string()))?;
            if !regex.is_match(s) {
              return Err(EllasticError::InvalidParameter(format!(
                "Value '{}' doesn't match pattern '{}'",
                s, pattern
              )));
            }
          }
        }
      }
      ValidationRuleType::Length => {
        if let Some(length_str) = rule.parameters.get("length") {
          if let ParameterValue::String(s) = value {
            let required_length: usize = length_str.parse().unwrap_or(0);
            if s.len() != required_length {
              return Err(EllasticError::InvalidParameter(format!(
                "String length {} doesn't match required {}",
                s.len(),
                required_length
              )));
            }
          }
        }
      }
      ValidationRuleType::Enum => {
        if let Some(options_str) = rule.parameters.get("options") {
          let options: Vec<&str> = options_str.split(',').collect();
          if let ParameterValue::String(s) = value {
            if !options.contains(&s.as_str()) {
              return Err(EllasticError::InvalidParameter(format!(
                "Value '{}' is not in enum: {:?}",
                s, options
              )));
            }
          }
        }
      }
      ValidationRuleType::Custom => {}
    }

    Ok(())
  }

  fn update_search_index(&mut self, template: &Template) {
    let mut search_index = self.search_index.write();

    for word in template.name.split_whitespace() {
      search_index
        .index
        .entry(word.to_lowercase())
        .or_insert_with(Vec::new)
        .push(template.id);
    }

    for keyword in &template.metadata.keywords {
      search_index
        .keywords
        .entry(keyword.to_lowercase())
        .or_insert_with(Vec::new)
        .push(template.id);
    }

    for word in template.description.split_whitespace() {
      search_index
        .descriptions
        .entry(word.to_lowercase())
        .or_insert_with(Vec::new)
        .push(template.id);
    }

    search_index
      .categories
      .entry(template.category.to_lowercase())
      .or_insert_with(Vec::new)
      .push(template.id);

    for tag in &template.tags {
      search_index
        .tags
        .entry(tag.to_lowercase())
        .or_insert_with(Vec::new)
        .push(template.id);
    }
  }

  fn update_category_index(&mut self, template: &Template) {
    let mut categories = self.template_categories.write();
    categories
      .entry(template.category.clone())
      .or_insert_with(Vec::new)
      .push(template.id);
  }

  fn update_tag_index(&mut self, template: &Template) {
    let mut tags = self.template_tags.write();
    for tag in &template.tags {
      tags
        .entry(tag.clone())
        .or_insert_with(Vec::new)
        .push(template.id);
    }
  }

  fn remove_from_search_index(&mut self, template: &Template) {
    let mut search_index = self.search_index.write();

    for word in template.name.split_whitespace() {
      if let Some(templates) = search_index.index.get_mut(&word.to_lowercase()) {
        templates.retain(|&id| id != template.id);
      }
    }

    for keyword in &template.metadata.keywords {
      if let Some(templates) = search_index.keywords.get_mut(&keyword.to_lowercase()) {
        templates.retain(|&id| id != template.id);
      }
    }

    for word in template.description.split_whitespace() {
      if let Some(templates) = search_index.descriptions.get_mut(&word.to_lowercase()) {
        templates.retain(|&id| id != template.id);
      }
    }

    if let Some(templates) = search_index
      .categories
      .get_mut(&template.category.to_lowercase())
    {
      templates.retain(|&id| id != template.id);
    }

    for tag in &template.tags {
      if let Some(templates) = search_index.tags.get_mut(&tag.to_lowercase()) {
        templates.retain(|&id| id != template.id);
      }
    }
  }

  fn remove_from_category_index(&mut self, template: &Template) {
    let mut categories = self.template_categories.write();
    if let Some(templates) = categories.get_mut(&template.category) {
      templates.retain(|&id| id != template.id);
    }
  }

  fn remove_from_tag_index(&mut self, template: &Template) {
    let mut tags = self.template_tags.write();
    for tag in &template.tags {
      if let Some(templates) = tags.get_mut(tag) {
        templates.retain(|&id| id != template.id);
      }
    }
  }

  fn load_default_templates(&mut self) -> Result<()> {
    let project_template = Template {
      id: Uuid::new_v4(),
      name: "Default Project".to_string(),
      description: "Default project template with basic setup".to_string(),
      template_type: TemplateType::Project,
      category: "Project".to_string(),
      tags: vec!["default".to_string(), "basic".to_string()],
      author: "Ellastic Team".to_string(),
      version: "1.0".to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      metadata: TemplateMetadata::new(),
      content: TemplateContent::new(),
      preview: None,
      dependencies: Vec::new(),
      compatibility: TemplateCompatibility::new(),
      validation: TemplateValidation::new(),
    };

    self.register_template(project_template)?;

    let workspace_template = Template {
      id: Uuid::new_v4(),
      name: "Default Workspace".to_string(),
      description: "Default workspace layout".to_string(),
      template_type: TemplateType::Workspace,
      category: "Workspace".to_string(),
      tags: vec!["default".to_string(), "layout".to_string()],
      author: "Ellastic Team".to_string(),
      version: "1.0".to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      metadata: TemplateMetadata::new(),
      content: TemplateContent::new(),
      preview: None,
      dependencies: Vec::new(),
      compatibility: TemplateCompatibility::new(),
      validation: TemplateValidation::new(),
    };

    self.register_template(workspace_template)?;

    Ok(())
  }

  fn load_templates(&mut self) -> Result<()> {
    Ok(())
  }

  fn save_template_to_disk(&self, template: &Template) -> Result<()> {
    let template_path = format!("{}/{}.json", self.config.template_directory, template.id);
    let template_json = serde_json::to_string_pretty(template).map_err(|e| {
      EllasticError::SerializationError(format!("Failed to serialize template: {}", e))
    })?;

    std::fs::write(template_path, template_json)
      .map_err(|e| EllasticError::IOError(format!("Failed to save template: {}", e)))?;

    Ok(())
  }

  fn delete_template_from_disk(&self, template_id: Uuid) -> Result<()> {
    let template_path = format!("{}/{}.json", self.config.template_directory, template_id);

    if std::path::Path::new(&template_path).exists() {
      std::fs::remove_file(template_path)
        .map_err(|e| EllasticError::IOError(format!("Failed to delete template file: {}", e)))?;
    }

    Ok(())
  }

  pub fn clone(&self) -> TemplateManager {
    TemplateManager {
      templates: self.templates.clone(),
      template_categories: self.template_categories.clone(),
      template_tags: self.template_tags.clone(),
      search_index: self.search_index.clone(),
      config: self.config.clone(),
    }
  }
}

impl TemplateRenderer {
  pub fn new(template_id: Uuid) -> Self {
    Self {
      id: Uuid::new_v4(),
      template_id,
      renderer_type: RendererType::Handlebars,
      config: RendererConfig::new(),
      context: RenderContext::new(),
    }
  }

  pub fn render(&self, content: &TemplateContent, context: &RenderContext) -> Result<String> {
    match content.content_type {
      TemplateContentType::JSON => serde_json::to_string_pretty(&content.data)
        .map_err(|e| EllasticError::SerializationError(format!("Failed to render JSON: {}", e))),
      TemplateContentType::Text => self.render_text_template(&content.data, context),
      _ => Ok("Template rendered".to_string()),
    }
  }

  fn render_text_template(
    &self,
    data: &HashMap<String, String>,
    context: &RenderContext,
  ) -> Result<String> {
    let mut result = String::new();

    for (key, value) in data {
      result.push_str(&format!("{}: {}\n", key, value));
    }

    Ok(result)
  }
}

impl Default for TemplateManagerConfig {
  fn default() -> Self {
    Self {
      max_templates: 1000,
      template_directory: "./templates".to_string(),
      auto_load_enabled: true,
      auto_save_enabled: true,
      backup_enabled: true,
      backup_retention_days: 30,
      validation_enabled: true,
      compression_enabled: false,
    }
  }
}

impl Default for TemplateMetadata {
  fn default() -> Self {
    Self {
      license: "MIT".to_string(),
      homepage: None,
      repository: None,
      documentation: None,
      keywords: Vec::new(),
      requirements: Vec::new(),
      custom_fields: HashMap::new(),
    }
  }
}

impl Default for TemplateContent {
  fn default() -> Self {
    Self {
      content_type: TemplateContentType::JSON,
      data: HashMap::new(),
      files: Vec::new(),
      resources: Vec::new(),
      parameters: Vec::new(),
      variables: HashMap::new(),
    }
  }
}

impl Default for TemplateCompatibility {
  fn default() -> Self {
    Self {
      ellastic_versions: vec!["0.1.0".to_string()],
      platform_requirements: Vec::new(),
      system_requirements: Vec::new(),
      incompatible_versions: Vec::new(),
    }
  }
}

impl Default for TemplateValidation {
  fn default() -> Self {
    Self {
      validation_rules: Vec::new(),
      validation_results: Vec::new(),
      last_validated: Utc::now(),
      validation_status: ValidationStatus::Unknown,
    }
  }
}

impl Default for TemplateSearchIndex {
  fn default() -> Self {
    Self {
      index: HashMap::new(),
      keywords: HashMap::new(),
      descriptions: HashMap::new(),
      categories: HashMap::new(),
      tags: HashMap::new(),
    }
  }
}

impl Default for RendererConfig {
  fn default() -> Self {
    Self {
      strict_mode: false,
      auto_escape: true,
      trim_blocks: false,
      lstrip_blocks: false,
      keep_trailing_newline: false,
      custom_filters: HashMap::new(),
      custom_functions: HashMap::new(),
    }
  }
}

impl Default for RenderContext {
  fn default() -> Self {
    Self {
      variables: HashMap::new(),
      functions: HashMap::new(),
      filters: HashMap::new(),
      globals: HashMap::new(),
      locale: "en".to_string(),
      timezone: "UTC".to_string(),
    }
  }
}

pub fn create_template_manager(config: TemplateManagerConfig) -> Result<TemplateManager> {
  TemplateManager::new(config)
}

pub fn create_template_manager_config() -> TemplateManagerConfig {
  TemplateManagerConfig::default()
}

pub fn create_template(name: String, description: String, template_type: TemplateType) -> Template {
  Template {
    id: Uuid::new_v4(),
    name,
    description,
    template_type,
    category: "General".to_string(),
    tags: Vec::new(),
    author: "Ellastic User".to_string(),
    version: "1.0".to_string(),
    created_at: Utc::now(),
    updated_at: Utc::now(),
    metadata: TemplateMetadata::new(),
    content: TemplateContent::new(),
    preview: None,
    dependencies: Vec::new(),
    compatibility: TemplateCompatibility::new(),
    validation: TemplateValidation::new(),
  }
}

pub fn create_template_parameter(
  name: String,
  parameter_type: ParameterType,
  default_value: ParameterValue,
) -> TemplateParameter {
  TemplateParameter {
    name,
    parameter_type,
    default_value,
    description: String::new(),
    required: false,
    validation_rules: Vec::new(),
    options: Vec::new(),
  }
}
