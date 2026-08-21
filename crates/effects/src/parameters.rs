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
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_image::{
  ImageData,
  ImageProcessor,
};
use ellastic_media::MediaProcessor;
use ellastic_utils::create_random_generator;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EffectParameter {
  pub id: Uuid,
  pub name: String,
  pub display_name: String,
  pub description: String,
  pub parameter_type: ParameterType,
  pub default_value: ParameterValue,
  pub min_value: Option<ParameterValue>,
  pub max_value: Option<ParameterValue>,
  pub step: Option<ParameterValue>,
  pub options: Vec<String>,
  pub category: ParameterCategory,
  pub group: String,
  pub advanced: bool,
  pub required: bool,
  pub visible: bool,
  pub enabled: bool,
  pub validation_rules: Vec<ValidationRule>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
  Integer,
  Float,
  Boolean,
  String,
  Color,
  Vector2,
  Vector3,
  Vector4,
  Matrix3,
  Matrix4,
  Enum,
  File,
  Directory,
  Image,
  Audio,
  Video,
  Custom(String),
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
  Integer(i64),
  Float(f64),
  Boolean(bool),
  String(String),
  Color([u8; 4]),
  Vector2([f32; 2]),
  Vector3([f32; 3]),
  Vector4([f32; 4]),
  Matrix3([[f32; 3]; 3]),
  Matrix4([[f32; 4]; 4]),
  Enum(String),
  File(String),
  Directory(String),
  Image(Vec<u8>),
  Audio(Vec<u8>),
  Video(Vec<u8>),
  Custom(String, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterCategory {
  Basic,
  Advanced,
  Expert,
  Debug,
  Custom,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
  Range {
    min: ParameterValue,
    max: ParameterValue,
  },
  MinLength {
    min: usize,
  },
  MaxLength {
    max: usize,
  },
  Pattern {
    pattern: String,
  },
  Custom {
    validation_function: Box<dyn Fn(&ParameterValue) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone)]
pub struct ParameterSet {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub parameters: HashMap<String, EffectParameter>,
  pub groups: HashMap<String, ParameterGroup>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ParameterGroup {
  pub id: Uuid,
  pub name: String,
  pub display_name: String,
  pub description: String,
  pub collapsed: bool,
  pub enabled: bool,
  pub order: u32,
}

#[derive(Debug, Clone)]
pub struct ParameterController {
  parameter_set: ParameterSet,
  values: HashMap<String, ParameterValue>,
  presets: HashMap<String, HashMap<String, ParameterValue>>,
  history: Vec<ParameterSnapshot>,
  max_history_size: usize,
  validation_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ParameterSnapshot {
  pub timestamp: DateTime<Utc>,
  pub values: HashMap<String, ParameterValue>,
  pub description: String,
}

impl EffectParameter {
  pub fn new(name: String, parameter_type: ParameterType, default_value: ParameterValue) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      display_name: name.clone(),
      description: String::new(),
      name,
      parameter_type,
      default_value,
      min_value: None,
      max_value: None,
      step: None,
      options: Vec::new(),
      category: ParameterCategory::Basic,
      group: "General".to_string(),
      advanced: false,
      required: false,
      visible: true,
      enabled: true,
      validation_rules: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  pub fn with_display_name(mut self, display_name: String) -> Self {
    self.display_name = display_name;
    self
  }

  pub fn with_description(mut self, description: String) -> Self {
    self.description = description;
    self
  }

  pub fn with_range(mut self, min: ParameterValue, max: ParameterValue) -> Self {
    self.min_value = Some(min);
    self.max_value = Some(max);
    self
  }

  pub fn with_step(mut self, step: ParameterValue) -> Self {
    self.step = Some(step);
    self
  }

  pub fn with_options(mut self, options: Vec<String>) -> Self {
    self.options = options;
    self
  }

  pub fn with_category(mut self, category: ParameterCategory) -> Self {
    self.category = category;
    self
  }

  pub fn with_group(mut self, group: String) -> Self {
    self.group = group;
    self
  }

  pub fn advanced(mut self) -> Self {
    self.advanced = true;
    self
  }

  pub fn required(mut self) -> Self {
    self.required = true;
    self
  }

  pub fn hidden(mut self) -> Self {
    self.visible = false;
    self
  }

  pub fn disabled(mut self) -> Self {
    self.enabled = false;
    self
  }

  pub fn with_validation_rules(mut self, rules: Vec<ValidationRule>) -> Self {
    self.validation_rules = rules;
    self
  }

  pub fn validate_value(&self, value: &ParameterValue) -> Result<()> {
    if !self.is_compatible_type(value) {
      return Err(EllasticError::InvalidParameter(format!(
        "Parameter '{}' type mismatch: expected {:?}, got {:?}",
        self.name, self.parameter_type, value
      )));
    }

    if let (Some(min), Some(max)) = (&self.min_value, &self.max_value) {
      if !self.is_in_range(value, min, max) {
        return Err(EllasticError::InvalidParameter(format!(
          "Parameter '{}' value out of range: {:?} not in [{:?}, {:?}]",
          self.name, value, min, max
        )));
      }
    }

    for rule in &self.validation_rules {
      match rule {
        ValidationRule::Range { min, max } => {
          if !self.is_in_range(value, min, max) {
            return Err(EllasticError::InvalidParameter(format!(
              "Parameter '{}' value out of range: {:?} not in [{:?}, {:?}]",
              self.name, value, min, max
            )));
          }
        }
        ValidationRule::MinLength { min } => {
          if let ParameterValue::String(s) = value {
            if s.len() < *min {
              return Err(EllasticError::InvalidParameter(format!(
                "Parameter '{}' string too short: {} < {}",
                self.name,
                s.len(),
                min
              )));
            }
          }
        }
        ValidationRule::MaxLength { max } => {
          if let ParameterValue::String(s) = value {
            if s.len() > *max {
              return Err(EllasticError::InvalidParameter(format!(
                "Parameter '{}' string too long: {} > {}",
                self.name,
                s.len(),
                max
              )));
            }
          }
        }
        ValidationRule::Pattern { pattern } => {
          if let ParameterValue::String(s) = value {
            let regex = regex::Regex::new(pattern)
              .map_err(|_| EllasticError::InvalidParameter("Invalid regex pattern".to_string()))?;
            if !regex.is_match(s) {
              return Err(EllasticError::InvalidParameter(format!(
                "Parameter '{}' doesn't match pattern: {}",
                self.name, pattern
              )));
            }
          }
        }
        ValidationRule::Custom {
          validation_function,
        } => {
          validation_function(value)?;
        }
      }
    }

    Ok(())
  }

  fn is_compatible_type(&self, value: &ParameterValue) -> bool {
    match (&self.parameter_type, value) {
      (ParameterType::Integer, ParameterValue::Integer(_)) => true,
      (ParameterType::Float, ParameterValue::Float(_)) => true,
      (ParameterType::Boolean, ParameterValue::Boolean(_)) => true,
      (ParameterType::String, ParameterValue::String(_)) => true,
      (ParameterType::Color, ParameterValue::Color(_)) => true,
      (ParameterType::Vector2, ParameterValue::Vector2(_)) => true,
      (ParameterType::Vector3, ParameterValue::Vector3(_)) => true,
      (ParameterType::Vector4, ParameterValue::Vector4(_)) => true,
      (ParameterType::Matrix3, ParameterValue::Matrix3(_)) => true,
      (ParameterType::Matrix4, ParameterValue::Matrix4(_)) => true,
      (ParameterType::Enum, ParameterValue::Enum(_)) => true,
      (ParameterType::File, ParameterValue::File(_)) => true,
      (ParameterType::Directory, ParameterValue::Directory(_)) => true,
      (ParameterType::Image, ParameterValue::Image(_)) => true,
      (ParameterType::Audio, ParameterValue::Audio(_)) => true,
      (ParameterType::Video, ParameterValue::Video(_)) => true,
      (ParameterType::Custom(_), ParameterValue::Custom(..)) => true,
      _ => false,
    }
  }

  fn is_in_range(
    &self,
    value: &ParameterValue,
    min: &ParameterValue,
    max: &ParameterValue,
  ) -> bool {
    match (value, min, max) {
      (
        ParameterValue::Integer(v),
        ParameterValue::Integer(min_v),
        ParameterValue::Integer(max_v),
      ) => v >= *min_v && v <= *max_v,
      (ParameterValue::Float(v), ParameterValue::Float(min_v), ParameterValue::Float(max_v)) => {
        v >= *min_v && v <= *max_v
      }
      (ParameterValue::String(v), ParameterValue::String(min_v), ParameterValue::String(max_v)) => {
        v >= min_v && v <= max_v
      }
      _ => true,
    }
  }

  pub fn clone(&self) -> EffectParameter {
    EffectParameter {
      id: self.id,
      name: self.name.clone(),
      display_name: self.display_name.clone(),
      description: self.description.clone(),
      parameter_type: self.parameter_type,
      default_value: self.default_value.clone(),
      min_value: self.min_value.clone(),
      max_value: self.max_value.clone(),
      step: self.step.clone(),
      options: self.options.clone(),
      category: self.category,
      group: self.group.clone(),
      advanced: self.advanced,
      required: self.required,
      visible: self.visible,
      enabled: self.enabled,
      validation_rules: self.validation_rules.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
    }
  }
}

impl ParameterSet {
  pub fn new(name: String, description: String) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name,
      description,
      parameters: HashMap::new(),
      groups: HashMap::new(),
      created_at: now,
      updated_at: now,
    }
  }

  pub fn add_parameter(&mut self, parameter: EffectParameter) -> Result<()> {
    if self.parameters.contains_key(&parameter.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Parameter '{}' already exists in set '{}'",
        parameter.name, self.name
      )));
    }

    self.parameters.insert(parameter.name.clone(), parameter);
    self.update_timestamp();
    Ok(())
  }

  pub fn remove_parameter(&mut self, name: &str) -> Option<EffectParameter> {
    let parameter = self.parameters.remove(name);
    if parameter.is_some() {
      self.update_timestamp();
    }
    parameter
  }

  pub fn get_parameter(&self, name: &str) -> Option<&EffectParameter> {
    self.parameters.get(name)
  }

  pub fn get_parameter_mut(&mut self, name: &str) -> Option<&mut EffectParameter> {
    self.parameters.get_mut(name)
  }

  pub fn list_parameters(&self) -> Vec<&EffectParameter> {
    self.parameters.values().collect()
  }

  pub fn list_parameters_by_category(&self, category: ParameterCategory) -> Vec<&EffectParameter> {
    self
      .parameters
      .values()
      .filter(|p| p.category == category)
      .collect()
  }

  pub fn list_parameters_by_group(&self, group: &str) -> Vec<&EffectParameter> {
    self
      .parameters
      .values()
      .filter(|p| p.group == group)
      .collect()
  }

  pub fn add_group(&mut self, group: ParameterGroup) -> Result<()> {
    if self.groups.contains_key(&group.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Group '{}' already exists in set '{}'",
        group.name, self.name
      )));
    }

    self.groups.insert(group.name.clone(), group);
    self.update_timestamp();
    Ok(())
  }

  pub fn remove_group(&mut self, name: &str) -> Option<ParameterGroup> {
    let group = self.groups.remove(name);
    if group.is_some() {
      self.update_timestamp();
    }
    group
  }

  pub fn get_group(&self, name: &str) -> Option<&ParameterGroup> {
    self.groups.get(name)
  }

  pub fn get_group_mut(&mut self, name: &str) -> Option<&mut ParameterGroup> {
    self.groups.get_mut(name)
  }

  pub fn list_groups(&self) -> Vec<&ParameterGroup> {
    self.groups.values().collect()
  }

  pub fn validate_all_values(&self, values: &HashMap<String, ParameterValue>) -> Result<()> {
    for (name, value) in values {
      if let Some(parameter) = self.parameters.get(name) {
        parameter.validate_value(value)?;
      } else {
        return Err(EllasticError::InvalidParameter(format!(
          "Unknown parameter: {}",
          name
        )));
      }
    }
    Ok(())
  }

  pub fn get_default_values(&self) -> HashMap<String, ParameterValue> {
    self
      .parameters
      .iter()
      .map(|(name, param)| (name.clone(), param.default_value.clone()))
      .collect()
  }

  pub fn update_timestamp(&mut self) {
    self.updated_at = Utc::now();
  }

  pub fn clone(&self) -> ParameterSet {
    ParameterSet {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      parameters: self.parameters.clone(),
      groups: self.groups.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
    }
  }
}

impl ParameterGroup {
  pub fn new(name: String, display_name: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      name,
      display_name,
      description: String::new(),
      collapsed: false,
      enabled: true,
      order: 0,
    }
  }

  pub fn with_description(mut self, description: String) -> Self {
    self.description = description;
    self
  }

  pub fn with_order(mut self, order: u32) -> Self {
    self.order = order;
    self
  }

  pub fn collapsed(mut self) -> Self {
    self.collapsed = true;
    self
  }

  pub fn disabled(mut self) -> Self {
    self.enabled = false;
    self
  }

  pub fn clone(&self) -> ParameterGroup {
    ParameterGroup {
      id: self.id,
      name: self.name.clone(),
      display_name: self.display_name.clone(),
      description: self.description.clone(),
      collapsed: self.collapsed,
      enabled: self.enabled,
      order: self.order,
    }
  }
}

impl ParameterController {
  pub fn new(parameter_set: ParameterSet) -> Self {
    Self {
      values: parameter_set.get_default_values(),
      presets: HashMap::new(),
      history: Vec::new(),
      max_history_size: 50,
      validation_enabled: true,
      parameter_set,
    }
  }

  pub fn with_max_history_size(mut self, max_size: usize) -> Self {
    self.max_history_size = max_size;
    self
  }

  pub fn with_validation(mut self, enabled: bool) -> Self {
    self.validation_enabled = enabled;
    self
  }

  pub fn parameter_set(&self) -> &ParameterSet {
    &self.parameter_set
  }

  pub fn parameter_set_mut(&mut self) -> &mut ParameterSet {
    &mut self.parameter_set
  }

  pub fn values(&self) -> &HashMap<String, ParameterValue> {
    &self.values
  }

  pub fn presets(&self) -> &HashMap<String, HashMap<String, ParameterValue>> {
    &self.presets
  }

  pub fn history(&self) -> &Vec<ParameterSnapshot> {
    &self.history
  }

  pub fn set_value(&mut self, name: String, value: ParameterValue) -> Result<()> {
    if let Some(parameter) = self.parameter_set.get_parameter(&name) {
      if self.validation_enabled {
        parameter.validate_value(&value)?;
      }

      self.values.insert(name.clone(), value);
      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Unknown parameter: {}",
        name
      )))
    }
  }

  pub fn get_value(&self, name: &str) -> Option<&ParameterValue> {
    self.values.get(name)
  }

  pub fn get_value_or_default(&self, name: &str) -> ParameterValue {
    self
      .values
      .get(name)
      .or_else(|| {
        self
          .parameter_set
          .get_parameter(name)
          .map(|p| &p.default_value)
      })
      .cloned()
      .unwrap_or(ParameterValue::String(String::new()))
  }

  pub fn reset_value(&mut self, name: &str) -> Result<()> {
    if let Some(parameter) = self.parameter_set.get_parameter(name) {
      self
        .values
        .insert(name.to_string(), parameter.default_value.clone());
      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Unknown parameter: {}",
        name
      )))
    }
  }

  pub fn reset_all_values(&mut self) {
    self.values = self.parameter_set.get_default_values();
  }

  pub fn create_snapshot(&mut self, description: String) -> Result<Uuid> {
    let snapshot = ParameterSnapshot {
      timestamp: Utc::now(),
      values: self.values.clone(),
      description,
    };

    let snapshot_id = snapshot.timestamp.timestamp_nanos_opt().unwrap_or(0) as u64;

    self.history.push(snapshot);

    if self.history.len() > self.max_history_size {
      self.history.remove(0);
    }

    Ok(Uuid::from_u64(snapshot_id))
  }

  pub fn restore_snapshot(&mut self, snapshot_id: Uuid) -> Result<()> {
    let timestamp = snapshot_id.as_u64();

    if let Some(snapshot) = self
      .history
      .iter()
      .find(|s| s.timestamp.timestamp_nanos_opt().unwrap_or(0) as u64 == timestamp)
    {
      self.values = snapshot.values.clone();
      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(
        "Snapshot not found".to_string(),
      ))
    }
  }

  pub fn save_preset(&mut self, name: String, description: String) -> Result<()> {
    if self.presets.contains_key(&name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Preset '{}' already exists",
        name
      )));
    }

    self.presets.insert(name, self.values.clone());
    Ok(())
  }

  pub fn load_preset(&mut self, name: &str) -> Result<()> {
    if let Some(preset_values) = self.presets.get(name) {
      if self.validation_enabled {
        self.parameter_set.validate_all_values(preset_values)?;
      }

      self.values = preset_values.clone();
      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Preset '{}' not found",
        name
      )))
    }
  }

  pub fn delete_preset(&mut self, name: &str) -> Result<()> {
    if self.presets.remove(name).is_none() {
      Err(EllasticError::InvalidParameter(format!(
        "Preset '{}' not found",
        name
      )))
    } else {
      Ok(())
    }
  }

  pub fn list_presets(&self) -> Vec<&String> {
    self.presets.keys().collect()
  }

  pub fn validate_current_values(&self) -> Result<()> {
    self.parameter_set.validate_all_values(&self.values)
  }

  pub fn get_changed_parameters(&self) -> Vec<&String> {
    let default_values = self.parameter_set.get_default_values();

    self
      .values
      .iter()
      .filter(|(name, value)| {
        default_values
          .get(name)
          .map_or(true, |default| value != default)
      })
      .map(|(name, _)| name)
      .collect()
  }

  pub fn reset_changed_parameters(&mut self) {
    let default_values = self.parameter_set.get_default_values();

    for (name, default_value) in default_values {
      if self.values.get(name) != Some(&default_value) {
        self.values.insert(name.clone(), default_value);
      }
    }
  }

  pub fn export_values(&self) -> HashMap<String, String> {
    self
      .values
      .iter()
      .map(|(name, value)| (name.clone(), self.value_to_string(value)))
      .collect()
  }

  pub fn import_values(&mut self, exported_values: HashMap<String, String>) -> Result<()> {
    let mut new_values = HashMap::new();

    for (name, string_value) in exported_values {
      if let Some(parameter) = self.parameter_set.get_parameter(&name) {
        let value = self.string_to_value(&string_value, &parameter.parameter_type)?;

        if self.validation_enabled {
          parameter.validate_value(&value)?;
        }

        new_values.insert(name, value);
      }
    }

    self.values = new_values;
    Ok(())
  }

  fn value_to_string(&self, value: &ParameterValue) -> String {
    match value {
      ParameterValue::Integer(v) => v.to_string(),
      ParameterValue::Float(v) => v.to_string(),
      ParameterValue::Boolean(v) => v.to_string(),
      ParameterValue::String(v) => v.clone(),
      ParameterValue::Color(v) => format!("#{:02X}{:02X}{:02X}{:02X}", v[0], v[1], v[2], v[3]),
      ParameterValue::Vector2(v) => format!("{},{}", v[0], v[1]),
      ParameterValue::Vector3(v) => format!("{},{},{}", v[0], v[1], v[2]),
      ParameterValue::Vector4(v) => format!("{},{},{},{}", v[0], v[1], v[2], v[3]),
      ParameterValue::Enum(v) => v.clone(),
      ParameterValue::File(v) => v.clone(),
      ParameterValue::Directory(v) => v.clone(),
      ParameterValue::Image(_) => "[Image Data]".to_string(),
      ParameterValue::Audio(_) => "[Audio Data]".to_string(),
      ParameterValue::Video(_) => "[Video Data]".to_string(),
      ParameterValue::Custom(t, v) => format!("{}:{}", t, v),
    }
  }

  fn string_to_value(
    &self,
    string_value: &str,
    parameter_type: &ParameterType,
  ) -> Result<ParameterValue> {
    match parameter_type {
      ParameterType::Integer => {
        let value = string_value
          .parse::<i64>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid integer value".to_string()))?;
        Ok(ParameterValue::Integer(value))
      }
      ParameterType::Float => {
        let value = string_value
          .parse::<f64>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid float value".to_string()))?;
        Ok(ParameterValue::Float(value))
      }
      ParameterType::Boolean => {
        let value = string_value
          .parse::<bool>()
          .map_err(|_| EllasticError::InvalidParameter("Invalid boolean value".to_string()))?;
        Ok(ParameterValue::Boolean(value))
      }
      ParameterType::String => Ok(ParameterValue::String(string_value.to_string())),
      ParameterType::Color => {
        if string_value.starts_with('#') && string_value.len() >= 7 {
          let hex = &string_value[1..];
          let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| EllasticError::InvalidParameter("Invalid color format".to_string()))?;
          let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| EllasticError::InvalidParameter("Invalid color format".to_string()))?;
          let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| EllasticError::InvalidParameter("Invalid color format".to_string()))?;
          let a = if hex.len() >= 8 {
            u8::from_str_radix(&hex[6..8], 16)
              .map_err(|_| EllasticError::InvalidParameter("Invalid color format".to_string()))?
          } else {
            255
          };
          Ok(ParameterValue::Color([r, g, b, a]))
        } else {
          Err(EllasticError::InvalidParameter(
            "Invalid color format".to_string(),
          ))
        }
      }
      ParameterType::Vector2 => {
        let parts: Vec<f32> = string_value
          .split(',')
          .map(|s| s.trim().parse().unwrap())
          .collect();
        if parts.len() == 2 {
          Ok(ParameterValue::Vector2([parts[0], parts[1]]))
        } else {
          Err(EllasticError::InvalidParameter(
            "Invalid vector2 format".to_string(),
          ))
        }
      }
      ParameterType::Vector3 => {
        let parts: Vec<f32> = string_value
          .split(',')
          .map(|s| s.trim().parse().unwrap())
          .collect();
        if parts.len() == 3 {
          Ok(ParameterValue::Vector3([parts[0], parts[1], parts[2]]))
        } else {
          Err(EllasticError::InvalidParameter(
            "Invalid vector3 format".to_string(),
          ))
        }
      }
      ParameterType::Vector4 => {
        let parts: Vec<f32> = string_value
          .split(',')
          .map(|s| s.trim().parse().unwrap())
          .collect();
        if parts.len() == 4 {
          Ok(ParameterValue::Vector4([
            parts[0], parts[1], parts[2], parts[3],
          ]))
        } else {
          Err(EllasticError::InvalidParameter(
            "Invalid vector4 format".to_string(),
          ))
        }
      }
      ParameterType::Enum => Ok(ParameterValue::Enum(string_value.to_string())),
      ParameterType::File => Ok(ParameterValue::File(string_value.to_string())),
      ParameterType::Directory => Ok(ParameterValue::Directory(string_value.to_string())),
      ParameterType::Image | ParameterType::Audio | ParameterType::Video => Err(
        EllasticError::UnsupportedOperation("Cannot import binary data from string".to_string()),
      ),
      ParameterType::Custom(_) => Ok(ParameterValue::Custom(
        "custom".to_string(),
        string_value.to_string(),
      )),
    }
  }

  pub fn clone(&self) -> ParameterController {
    ParameterController {
      parameter_set: self.parameter_set.clone(),
      values: self.values.clone(),
      presets: self.presets.clone(),
      history: self.history.clone(),
      max_history_size: self.max_history_size,
      validation_enabled: self.validation_enabled,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ParameterRegistry {
  parameter_sets: HashMap<Uuid, ParameterSet>,
  parameter_sets_by_name: HashMap<String, Uuid>,
}

impl ParameterRegistry {
  pub fn new() -> Self {
    Self {
      parameter_sets: HashMap::new(),
      parameter_sets_by_name: HashMap::new(),
    }
  }

  pub fn register_parameter_set(&mut self, parameter_set: ParameterSet) -> Result<()> {
    if self
      .parameter_sets_by_name
      .contains_key(&parameter_set.name)
    {
      return Err(EllasticError::AlreadyExists(format!(
        "Parameter set '{}' already registered",
        parameter_set.name
      )));
    }

    let id = parameter_set.id;
    self
      .parameter_sets_by_name
      .insert(parameter_set.name.clone(), id);
    self.parameter_sets.insert(id, parameter_set);

    Ok(())
  }

  pub fn unregister_parameter_set(&mut self, id: Uuid) -> Option<ParameterSet> {
    if let Some(parameter_set) = self.parameter_sets.remove(&id) {
      self.parameter_sets_by_name.remove(&parameter_set.name);
      Some(parameter_set)
    } else {
      None
    }
  }

  pub fn get_parameter_set(&self, id: Uuid) -> Option<&ParameterSet> {
    self.parameter_sets.get(&id)
  }

  pub fn get_parameter_set_by_name(&self, name: &str) -> Option<&ParameterSet> {
    self
      .parameter_sets_by_name
      .get(name)
      .and_then(|id| self.parameter_sets.get(id))
  }

  pub fn list_parameter_sets(&self) -> Vec<&ParameterSet> {
    self.parameter_sets.values().collect()
  }

  pub fn search_parameter_sets(&self, query: &str) -> Vec<&ParameterSet> {
    let query = query.to_lowercase();
    self
      .parameter_sets
      .values()
      .filter(|set| {
        set.name.to_lowercase().contains(&query) || set.description.to_lowercase().contains(&query)
      })
      .collect()
  }

  pub fn clear(&mut self) {
    self.parameter_sets.clear();
    self.parameter_sets_by_name.clear();
  }

  pub fn len(&self) -> usize {
    self.parameter_sets.len()
  }

  pub fn is_empty(&self) -> bool {
    self.parameter_sets.is_empty()
  }

  pub fn clone(&self) -> ParameterRegistry {
    ParameterRegistry {
      parameter_sets: self.parameter_sets.clone(),
      parameter_sets_by_name: self.parameter_sets_by_name.clone(),
    }
  }

  pub fn load_default_parameter_sets(&mut self) -> Result<()> {
    self.register_basic_parameter_set()?;
    self.register_image_parameter_set()?;
    self.register_audio_parameter_set()?;
    self.register_video_parameter_set()?;
    self.register_glitch_parameter_set()?;
    self.register_advanced_parameter_set()?;

    Ok(())
  }

  fn register_basic_parameter_set(&mut self) -> Result<()> {
    let mut parameter_set =
      ParameterSet::new("basic".to_string(), "Basic effect parameters".to_string());

    parameter_set.add_group(
      ParameterGroup::new("adjustments".to_string(), "Adjustments".to_string())
        .with_description("Basic image and audio adjustments".to_string())
        .with_order(1),
    )?;

    parameter_set.add_group(
      ParameterGroup::new("transforms".to_string(), "Transforms".to_string())
        .with_description("Basic transformation parameters".to_string())
        .with_order(2),
    )?;

    let parameters = vec![
      EffectParameter::new(
        "brightness".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.0),
      )
      .with_display_name("Brightness".to_string())
      .with_description("Adjust the brightness of the media".to_string())
      .with_range(ParameterValue::Float(-1.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("adjustments".to_string()),
      EffectParameter::new(
        "contrast".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.0),
      )
      .with_display_name("Contrast".to_string())
      .with_description("Adjust the contrast of the media".to_string())
      .with_range(ParameterValue::Float(-1.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("adjustments".to_string()),
      EffectParameter::new(
        "saturation".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.0),
      )
      .with_display_name("Saturation".to_string())
      .with_description("Adjust the saturation of the media".to_string())
      .with_range(ParameterValue::Float(-1.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("adjustments".to_string()),
      EffectParameter::new(
        "gamma".to_string(),
        ParameterType::Float,
        ParameterValue::Float(1.0),
      )
      .with_display_name("Gamma".to_string())
      .with_description("Adjust the gamma correction of the media".to_string())
      .with_range(ParameterValue::Float(0.1), ParameterValue::Float(3.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("adjustments".to_string()),
      EffectParameter::new(
        "rotate".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.0),
      )
      .with_display_name("Rotation".to_string())
      .with_description("Rotate the media by the specified angle".to_string())
      .with_range(ParameterValue::Float(-360.0), ParameterValue::Float(360.0))
      .with_step(ParameterValue::Float(1.0))
      .with_group("transforms".to_string()),
      EffectParameter::new(
        "scale".to_string(),
        ParameterType::Vector2,
        ParameterValue::Vector2([1.0, 1.0]),
      )
      .with_display_name("Scale".to_string())
      .with_description("Scale the media by the specified factors".to_string())
      .with_range(
        ParameterValue::Vector2([0.1, 0.1]),
        ParameterValue::Vector2([10.0, 10.0]),
      )
      .with_step(ParameterValue::Vector2([0.01, 0.01]))
      .with_group("transforms".to_string()),
    ];

    for parameter in parameters {
      parameter_set.add_parameter(parameter)?;
    }

    self.register_parameter_set(parameter_set)
  }

  fn register_image_parameter_set(&mut self) -> Result<()> {
    let mut parameter_set =
      ParameterSet::new("image".to_string(), "Image-specific parameters".to_string());

    parameter_set.add_group(
      ParameterGroup::new("filters".to_string(), "Filters".to_string())
        .with_description("Image filter parameters".to_string())
        .with_order(1),
    )?;

    parameter_set.add_group(
      ParameterGroup::new("effects".to_string(), "Effects".to_string())
        .with_description("Image effect parameters".to_string())
        .with_order(2),
    )?;

    let parameters = vec![
      EffectParameter::new(
        "blur_radius".to_string(),
        ParameterType::Float,
        ParameterValue::Float(1.0),
      )
      .with_display_name("Blur Radius".to_string())
      .with_description("Radius of the blur effect".to_string())
      .with_range(ParameterValue::Float(0.1), ParameterValue::Float(10.0))
      .with_step(ParameterValue::Float(0.1))
      .with_group("filters".to_string()),
      EffectParameter::new(
        "sharpen_amount".to_string(),
        ParameterType::Float,
        ParameterValue::Float(1.0),
      )
      .with_display_name("Sharpen Amount".to_string())
      .with_description("Amount of sharpening to apply".to_string())
      .with_range(ParameterValue::Float(0.0), ParameterValue::Float(5.0))
      .with_step(ParameterValue::Float(0.1))
      .with_group("filters".to_string()),
      EffectParameter::new(
        "pixel_sort_threshold".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.5),
      )
      .with_display_name("Pixel Sort Threshold".to_string())
      .with_description("Threshold for pixel sorting effect".to_string())
      .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("effects".to_string()),
      EffectParameter::new(
        "pixel_sort_mode".to_string(),
        ParameterType::Enum,
        ParameterValue::Enum("brightness".to_string()),
      )
      .with_display_name("Pixel Sort Mode".to_string())
      .with_description("Mode for pixel sorting".to_string())
      .with_options(vec![
        "brightness".to_string(),
        "hue".to_string(),
        "saturation".to_string(),
        "random".to_string(),
      ])
      .with_group("effects".to_string()),
      EffectParameter::new(
        "data_mosh_intensity".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.3),
      )
      .with_display_name("Data Mosh Intensity".to_string())
      .with_description("Intensity of data moshing effect".to_string())
      .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("effects".to_string()),
    ];

    for parameter in parameters {
      parameter_set.add_parameter(parameter)?;
    }

    self.register_parameter_set(parameter_set)
  }

  fn register_audio_parameter_set(&mut self) -> Result<()> {
    let mut parameter_set =
      ParameterSet::new("audio".to_string(), "Audio-specific parameters".to_string());

    parameter_set.add_group(
      ParameterGroup::new("effects".to_string(), "Effects".to_string())
        .with_description("Audio effect parameters".to_string())
        .with_order(1),
    )?;

    parameter_set.add_group(
      ParameterGroup::new("filters".to_string(), "Filters".to_string())
        .with_description("Audio filter parameters".to_string())
        .with_order(2),
    )?;

    let parameters = vec![
      EffectParameter::new(
        "reverb_room_size".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.5),
      )
      .with_display_name("Reverb Room Size".to_string())
      .with_description("Size of the reverb room".to_string())
      .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("effects".to_string()),
      EffectParameter::new(
        "echo_delay".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.3),
      )
      .with_display_name("Echo Delay".to_string())
      .with_description("Delay time for echo effect".to_string())
      .with_range(ParameterValue::Float(0.0), ParameterValue::Float(2.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("effects".to_string()),
      EffectParameter::new(
        "distortion_amount".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.5),
      )
      .with_display_name("Distortion Amount".to_string())
      .with_description("Amount of distortion to apply".to_string())
      .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("effects".to_string()),
      EffectParameter::new(
        "bit_crush_depth".to_string(),
        ParameterType::Integer,
        ParameterValue::Integer(8),
      )
      .with_display_name("Bit Crush Depth".to_string())
      .with_description("Bit depth for bit crushing".to_string())
      .with_range(ParameterValue::Integer(1), ParameterValue::Integer(32))
      .with_step(ParameterValue::Integer(1))
      .with_group("effects".to_string()),
      EffectParameter::new(
        "low_pass_cutoff".to_string(),
        ParameterType::Float,
        ParameterValue::Float(1000.0),
      )
      .with_display_name("Low Pass Cutoff".to_string())
      .with_description("Cutoff frequency for low pass filter".to_string())
      .with_range(ParameterValue::Float(20.0), ParameterValue::Float(20000.0))
      .with_step(ParameterValue::Float(1.0))
      .with_group("filters".to_string()),
    ];

    for parameter in parameters {
      parameter_set.add_parameter(parameter)?;
    }

    self.register_parameter_set(parameter_set)
  }

  fn register_video_parameter_set(&mut self) -> Result<()> {
    let mut parameter_set =
      ParameterSet::new("video".to_string(), "Video-specific parameters".to_string());

    parameter_set.add_group(
      ParameterGroup::new("effects".to_string(), "Effects".to_string())
        .with_description("Video effect parameters".to_string())
        .with_order(1),
    )?;

    let parameters = vec![
      EffectParameter::new(
        "frame_duplication_count".to_string(),
        ParameterType::Integer,
        ParameterValue::Integer(1),
      )
      .with_display_name("Frame Duplication Count".to_string())
      .with_description("Number of times to duplicate each frame".to_string())
      .with_range(ParameterValue::Integer(1), ParameterValue::Integer(10))
      .with_step(ParameterValue::Integer(1))
      .with_group("effects".to_string()),
      EffectParameter::new(
        "frame_dropping_count".to_string(),
        ParameterType::Integer,
        ParameterValue::Integer(1),
      )
      .with_display_name("Frame Dropping Count".to_string())
      .with_description("Number of frames to drop".to_string())
      .with_range(ParameterValue::Integer(1), ParameterValue::Integer(10))
      .with_step(ParameterValue::Integer(1))
      .with_group("effects".to_string()),
      EffectParameter::new(
        "time_stretch_ratio".to_string(),
        ParameterType::Float,
        ParameterValue::Float(1.0),
      )
      .with_display_name("Time Stretch Ratio".to_string())
      .with_description("Ratio for time stretching".to_string())
      .with_range(ParameterValue::Float(0.1), ParameterValue::Float(10.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("effects".to_string()),
    ];

    for parameter in parameters {
      parameter_set.add_parameter(parameter)?;
    }

    self.register_parameter_set(parameter_set)
  }

  fn register_glitch_parameter_set(&mut self) -> Result<()> {
    let mut parameter_set =
      ParameterSet::new("glitch".to_string(), "Glitch effect parameters".to_string());

    parameter_set.add_group(
      ParameterGroup::new("glitches".to_string(), "Glitches".to_string())
        .with_description("Glitch effect parameters".to_string())
        .with_order(1),
    )?;

    let parameters = vec![
      EffectParameter::new(
        "glitch_intensity".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.5),
      )
      .with_display_name("Glitch Intensity".to_string())
      .with_description("Intensity of glitch effects".to_string())
      .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("glitches".to_string()),
      EffectParameter::new(
        "glitch_frequency".to_string(),
        ParameterType::Float,
        ParameterValue::Float(0.1),
      )
      .with_display_name("Glitch Frequency".to_string())
      .with_description("Frequency of glitch occurrences".to_string())
      .with_range(ParameterValue::Float(0.0), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("glitches".to_string()),
      EffectParameter::new(
        "glitch_type".to_string(),
        ParameterType::Enum,
        ParameterValue::Enum("random".to_string()),
      )
      .with_display_name("Glitch Type".to_string())
      .with_description("Type of glitch effect".to_string())
      .with_options(vec![
        "random".to_string(),
        "digital".to_string(),
        "analog".to_string(),
        "compression".to_string(),
      ])
      .with_group("glitches".to_string()),
    ];

    for parameter in parameters {
      parameter_set.add_parameter(parameter)?;
    }

    self.register_parameter_set(parameter_set)
  }

  fn register_advanced_parameter_set(&mut self) -> Result<()> {
    let mut parameter_set = ParameterSet::new(
      "advanced".to_string(),
      "Advanced effect parameters".to_string(),
    );

    parameter_set.add_group(
      ParameterGroup::new("advanced".to_string(), "Advanced".to_string())
        .with_description("Advanced parameters".to_string())
        .with_order(1),
    )?;

    let parameters = vec![
      EffectParameter::new(
        "quality".to_string(),
        ParameterType::Float,
        ParameterValue::Float(1.0),
      )
      .with_display_name("Quality".to_string())
      .with_description("Quality setting for effects".to_string())
      .with_range(ParameterValue::Float(0.1), ParameterValue::Float(1.0))
      .with_step(ParameterValue::Float(0.01))
      .with_group("advanced".to_string())
      .advanced(),
      EffectParameter::new(
        "random_seed".to_string(),
        ParameterType::Integer,
        ParameterValue::Integer(0),
      )
      .with_display_name("Random Seed".to_string())
      .with_description("Seed for random number generation".to_string())
      .with_range(
        ParameterValue::Integer(0),
        ParameterValue::Integer(u64::MAX as i64),
      )
      .with_step(ParameterValue::Integer(1))
      .with_group("advanced".to_string())
      .advanced(),
      EffectParameter::new(
        "debug_mode".to_string(),
        ParameterType::Boolean,
        ParameterValue::Boolean(false),
      )
      .with_display_name("Debug Mode".to_string())
      .with_description("Enable debug mode for troubleshooting".to_string())
      .with_group("advanced".to_string())
      .advanced(),
    ];

    for parameter in parameters {
      parameter_set.add_parameter(parameter)?;
    }

    self.register_parameter_set(parameter_set)
  }
}

pub fn create_effect_parameter(
  name: String,
  parameter_type: ParameterType,
  default_value: ParameterValue,
) -> EffectParameter {
  EffectParameter::new(name, parameter_type, default_value)
}

pub fn create_parameter_set(name: String, description: String) -> ParameterSet {
  ParameterSet::new(name, description)
}

pub fn create_parameter_group(name: String, display_name: String) -> ParameterGroup {
  ParameterGroup::new(name, display_name)
}

pub fn create_parameter_controller(parameter_set: ParameterSet) -> ParameterController {
  ParameterController::new(parameter_set)
}

pub fn create_parameter_registry() -> ParameterRegistry {
  ParameterRegistry::new()
}
