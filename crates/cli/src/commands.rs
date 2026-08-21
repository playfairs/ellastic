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
use ellastic_export::{
  ExportManager,
  ExportRequest,
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
use ellastic_project::{
  Project,
  ProjectManager,
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
pub struct CommandRegistry {
  pub commands: Arc<RwLock<HashMap<String, Command>>>,
  pub aliases: Arc<RwLock<HashMap<String, String>>>,
  pub categories: Arc<RwLock<HashMap<String, CommandCategory>>>,
}

#[derive(Debug, Clone)]
pub struct Command {
  pub name: String,
  pub description: String,
  pub usage: String,
  pub long_description: Option<String>,
  pub examples: Vec<String>,
  pub aliases: Vec<String>,
  pub subcommands: HashMap<String, Command>,
  pub arguments: Vec<Argument>,
  pub options: Vec<Option>,
  pub flags: Vec<Flag>,
  pub handler: CommandHandler,
  pub category: CommandCategory,
  pub visibility: CommandVisibility,
  pub enabled: bool,
  pub deprecated: bool,
  pub deprecation_message: Option<String>,
  pub version: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Argument {
  pub name: String,
  pub description: String,
  pub help_text: Option<String>,
  pub required: bool,
  pub multiple: bool,
  pub default_value: Option<String>,
  pub possible_values: Vec<String>,
  pub value_type: ValueType,
  pub validator: Option<ArgumentValidator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
  String,
  Number,
  Integer,
  Float,
  Boolean,
  File,
  Directory,
  Url,
  Email,
  Date,
  Time,
  DateTime,
  Json,
  Yaml,
  Toml,
  Xml,
  Csv,
  Binary,
  Custom,
}

#[derive(Debug, Clone)]
pub struct ArgumentValidator {
  pub validator_type: ValidatorType,
  pub pattern: Option<String>,
  pub min_length: Option<usize>,
  pub max_length: Option<usize>,
  pub min_value: Option<f64>,
  pub max_value: Option<f64>,
  pub custom_validator: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorType {
  Required,
  Pattern,
  Length,
  Range,
  Custom,
}

#[derive(Debug, Clone)]
pub struct Option {
  pub name: String,
  pub short_name: Option<char>,
  pub long_name: Option<String>,
  pub description: String,
  pub help_text: Option<String>,
  pub required: bool,
  pub multiple: bool,
  pub default_value: Option<String>,
  pub possible_values: Vec<String>,
  pub value_type: ValueType,
  pub validator: Option<ArgumentValidator>,
  pub global: bool,
  pub deprecated: bool,
  pub deprecation_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Flag {
  pub name: String,
  pub short_name: Option<char>,
  pub long_name: Option<String>,
  pub description: String,
  pub help_text: Option<String>,
  pub default_value: bool,
  pub global: bool,
  pub deprecated: bool,
  pub deprecation_message: Option<String>,
}

pub type CommandHandler = Box<dyn Fn(&CommandContext) -> Result<CommandResult> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
  Project,
  Media,
  Effects,
  Pipeline,
  Export,
  Config,
  Help,
  System,
  Development,
  Testing,
  Debug,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandVisibility {
  Public,
  Hidden,
  Internal,
  Experimental,
}

#[derive(Debug, Clone)]
pub struct CommandContext {
  pub command_name: String,
  pub arguments: Vec<String>,
  pub options: HashMap<String, String>,
  pub flags: HashMap<String, bool>,
  pub working_directory: String,
  pub environment: HashMap<String, String>,
  pub config: CommandConfig,
  pub session: CommandSession,
}

#[derive(Debug, Clone)]
pub struct CommandConfig {
  pub global_options: HashMap<String, String>,
  pub global_flags: HashMap<String, bool>,
  pub verbosity: VerbosityLevel,
  pub output_format: OutputFormat,
  pub color_mode: ColorMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbosityLevel {
  Silent,
  Error,
  Warning,
  Info,
  Debug,
  Trace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  Text,
  Json,
  Yaml,
  Toml,
  Table,
  List,
  Tree,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
  Auto,
  Always,
  Never,
}

#[derive(Debug, Clone)]
pub struct CommandSession {
  pub session_id: String,
  pub user_id: Option<String>,
  pub start_time: DateTime<Utc>,
  pub variables: HashMap<String, String>,
  pub history: Vec<CommandHistoryEntry>,
}

#[derive(Debug, Clone)]
pub struct CommandHistoryEntry {
  pub command: String,
  pub timestamp: DateTime<Utc>,
  pub exit_code: i32,
  pub duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
  pub success: bool,
  pub message: String,
  pub data: Option<serde_json::Value>,
  pub exit_code: i32,
  pub duration: std::time::Duration,
  pub warnings: Vec<String>,
  pub errors: Vec<String>,
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CommandTemplate {
  pub name: String,
  pub description: String,
  pub template: String,
  pub variables: Vec<TemplateVariable>,
  pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateVariable {
  pub name: String,
  pub description: String,
  pub variable_type: ValueType,
  pub default_value: Option<String>,
  pub required: bool,
  pub possible_values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommandPlugin {
  pub name: String,
  pub version: String,
  pub description: String,
  pub author: String,
  pub license: String,
  pub commands: HashMap<String, Command>,
  pub dependencies: Vec<String>,
  pub enabled: bool,
  pub loaded: bool,
  pub load_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CommandMiddleware {
  pub name: String,
  pub description: String,
  pub priority: MiddlewarePriority,
  pub handler: MiddlewareHandler,
  pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MiddlewarePriority {
  Lowest = 0,
  Low = 1,
  Normal = 2,
  High = 3,
  Highest = 4,
}

pub type MiddlewareHandler = Box<dyn Fn(&mut CommandContext) -> Result<()> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct CommandInterceptor {
  pub name: String,
  pub description: String,
  pub interceptor_type: InterceptorType,
  pub handler: InterceptorHandler,
  pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterceptorType {
  Before,
  After,
  Around,
  Error,
}

pub type InterceptorHandler =
  Box<dyn Fn(&CommandContext, &CommandResult) -> Result<CommandResult> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct CommandValidator {
  pub name: String,
  pub description: String,
  pub validator_type: ValidatorType,
  pub handler: ValidatorHandler,
  pub enabled: bool,
}

pub type ValidatorHandler = Box<dyn Fn(&CommandContext) -> Result<ValidationResult> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct ValidationResult {
  pub valid: bool,
  pub errors: Vec<String>,
  pub warnings: Vec<String>,
}

impl CommandRegistry {
  pub fn new() -> Self {
    Self {
      commands: Arc::new(RwLock::new(HashMap::new())),
      aliases: Arc::new(RwLock::new(HashMap::new())),
      categories: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub fn register_command(&mut self, command: Command) -> Result<()> {
    let name = command.name.clone();

    {
      let mut commands = self.commands.write();
      commands.insert(name.clone(), command.clone());
    }

    {
      let mut aliases = self.aliases.write();
      for alias in &command.aliases {
        aliases.insert(alias.clone(), name.clone());
      }
    }

    {
      let mut categories = self.categories.write();
      categories.insert(name.clone(), command.category);
    }

    Ok(())
  }

  pub fn unregister_command(&mut self, name: &str) -> Result<()> {
    let command = {
      let commands = self.commands.read();
      commands.get(name).cloned()
    };

    if let Some(command) = command {
      {
        let mut aliases = self.aliases.write();
        for alias in &command.aliases {
          aliases.remove(alias);
        }
      }

      {
        let mut categories = self.categories.write();
        categories.remove(name);
      }

      {
        let mut commands = self.commands.write();
        commands.remove(name);
      }
    }

    Ok(())
  }

  pub fn get_command(&self, name: &str) -> Option<Command> {
    let commands = self.commands.read();

    if let Some(command) = commands.get(name) {
      return Some(command.clone());
    }

    let aliases = self.aliases.read();
    if let Some(actual_name) = aliases.get(name) {
      return commands.get(actual_name).cloned();
    }

    None
  }

  pub fn list_commands(&self) -> Vec<Command> {
    let commands = self.commands.read();
    commands.values().cloned().collect()
  }

  pub fn list_commands_by_category(&self, category: CommandCategory) -> Vec<Command> {
    let commands = self.commands.read();
    commands
      .values()
      .filter(|c| c.category == category)
      .cloned()
      .collect()
  }

  pub fn list_categories(&self) -> Vec<CommandCategory> {
    let categories = self.categories.read();
    categories.values().cloned().collect()
  }

  pub fn search_commands(&self, query: &str) -> Vec<Command> {
    let commands = self.commands.read();
    let query_lower = query.to_lowercase();

    commands
      .values()
      .filter(|c| {
        c.name.to_lowercase().contains(&query_lower)
          || c.description.to_lowercase().contains(&query_lower)
          || c
            .aliases
            .iter()
            .any(|a| a.to_lowercase().contains(&query_lower))
      })
      .cloned()
      .collect()
  }

  pub fn validate_command(
    &self,
    name: &str,
    args: &[String],
    options: &HashMap<String, String>,
    flags: &HashMap<String, bool>,
  ) -> Result<ValidationResult> {
    let command = self
      .get_command(name)
      .ok_or_else(|| EllasticError::InvalidParameter(format!("Command '{}' not found", name)))?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for (i, arg) in command.arguments.iter().enumerate() {
      if arg.required && i >= args.len() {
        errors.push(format!("Required argument '{}' is missing", arg.name));
      }
    }

    for option in &command.options {
      if option.required && !options.contains_key(&option.name) {
        errors.push(format!("Required option '--{}' is missing", option.name));
      }
    }

    for (i, arg) in command.arguments.iter().enumerate() {
      if i < args.len() {
        if let Some(validator) = &arg.validator {
          let value = &args[i];
          let result = self.validate_value(value, validator);
          if !result.valid {
            errors.extend(result.errors);
          }
          warnings.extend(result.warnings);
        }
      }
    }

    for option in &command.options {
      if let Some(value) = options.get(&option.name) {
        if let Some(validator) = &option.validator {
          let result = self.validate_value(value, validator);
          if !result.valid {
            errors.extend(result.errors);
          }
          warnings.extend(result.warnings);
        }
      }
    }

    for option in &command.options {
      if option.deprecated && options.contains_key(&option.name) {
        let message = option
          .deprecation_message
          .as_ref()
          .map(|m| format!("Option '--{}' is deprecated: {}", option.name, m))
          .unwrap_or_else(|| format!("Option '--{}' is deprecated", option.name));
        warnings.push(message);
      }
    }

    for option_name in options.keys() {
      if !command.options.iter().any(|o| o.name == *option_name) {
        warnings.push(format!("Unknown option '--{}'", option_name));
      }
    }

    for flag_name in flags.keys() {
      if !command.flags.iter().any(|f| f.name == *flag_name) {
        warnings.push(format!("Unknown flag '--{}'", flag_name));
      }
    }

    Ok(ValidationResult {
      valid: errors.is_empty(),
      errors,
      warnings,
    })
  }

  fn validate_value(&self, value: &str, validator: &ArgumentValidator) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    match validator.validator_type {
      ValidatorType::Required => {
        if value.is_empty() {
          errors.push("Value is required".to_string());
        }
      }
      ValidatorType::Pattern => {
        if let Some(pattern) = &validator.pattern {
          if pattern.is_empty() {
            warnings.push("Empty pattern".to_string());
          }
        }
      }
      ValidatorType::Length => {
        let length = value.len();
        if let Some(min_length) = validator.min_length {
          if length < min_length {
            errors.push(format!(
              "Value is too short (minimum {} characters)",
              min_length
            ));
          }
        }
        if let Some(max_length) = validator.max_length {
          if length > max_length {
            errors.push(format!(
              "Value is too long (maximum {} characters)",
              max_length
            ));
          }
        }
      }
      ValidatorType::Range => {
        if let Ok(num_value) = value.parse::<f64>() {
          if let Some(min_value) = validator.min_value {
            if num_value < min_value {
              errors.push(format!("Value is too small (minimum {})", min_value));
            }
          }
          if let Some(max_value) = validator.max_value {
            if num_value > max_value {
              errors.push(format!("Value is too large (maximum {})", max_value));
            }
          }
        } else {
          errors.push("Value must be a number".to_string());
        }
      }
      ValidatorType::Custom => {
        warnings.push("Custom validation not implemented".to_string());
      }
    }

    ValidationResult {
      valid: errors.is_empty(),
      errors,
      warnings,
    }
  }

  pub fn execute_command(
    &self,
    name: &str,
    args: Vec<String>,
    options: HashMap<String, String>,
    flags: HashMap<String, bool>,
  ) -> Result<CommandResult> {
    let command = self
      .get_command(name)
      .ok_or_else(|| EllasticError::InvalidParameter(format!("Command '{}' not found", name)))?;

    if !command.enabled {
      return Err(EllasticError::InvalidOperation(format!(
        "Command '{}' is disabled",
        name
      )));
    }

    if command.deprecated {
      let message = command
        .deprecation_message
        .as_ref()
        .map(|m| format!("Command '{}' is deprecated: {}", name, m))
        .unwrap_or_else(|| format!("Command '{}' is deprecated", name));
    }

    let context = CommandContext {
      command_name: name.to_string(),
      arguments: args,
      options,
      flags,
      working_directory: std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .to_string_lossy()
        .to_string(),
      environment: std::env::vars().collect(),
      config: CommandConfig::new(),
      session: CommandSession::new(),
    };

    let start_time = std::time::Instant::now();
    let result = (command.handler)(&context);
    let duration = start_time.elapsed();

    match result {
      Ok(mut command_result) => {
        command_result.duration = duration;
        Ok(command_result)
      }
      Err(e) => Ok(CommandResult {
        success: false,
        message: e.to_string(),
        data: None,
        exit_code: 1,
        duration,
        warnings: Vec::new(),
        errors: vec![e.to_string()],
        metadata: HashMap::new(),
      }),
    }
  }

  pub fn clone(&self) -> CommandRegistry {
    CommandRegistry {
      commands: self.commands.clone(),
      aliases: self.aliases.clone(),
      categories: self.categories.clone(),
    }
  }
}

impl Command {
  pub fn new(name: String, description: String, handler: CommandHandler) -> Self {
    let now = Utc::now();
    Self {
      name,
      description,
      usage: String::new(),
      long_description: None,
      examples: Vec::new(),
      aliases: Vec::new(),
      subcommands: HashMap::new(),
      arguments: Vec::new(),
      options: Vec::new(),
      flags: Vec::new(),
      handler,
      category: CommandCategory::Custom,
      visibility: CommandVisibility::Public,
      enabled: true,
      deprecated: false,
      deprecation_message: None,
      version: "1.0.0".to_string(),
      created_at: now,
      updated_at: now,
    }
  }

  pub fn with_usage(mut self, usage: String) -> Self {
    self.usage = usage;
    self
  }

  pub fn with_description(mut self, description: String) -> Self {
    self.long_description = Some(description);
    self
  }

  pub fn with_example(mut self, example: String) -> Self {
    self.examples.push(example);
    self
  }

  pub fn with_alias(mut self, alias: String) -> Self {
    self.aliases.push(alias);
    self
  }

  pub fn with_category(mut self, category: CommandCategory) -> Self {
    self.category = category;
    self
  }

  pub fn with_visibility(mut self, visibility: CommandVisibility) -> Self {
    self.visibility = visibility;
    self
  }

  pub fn with_argument(mut self, argument: Argument) -> Self {
    self.arguments.push(argument);
    self
  }

  pub fn with_option(mut self, option: Option) -> Self {
    self.options.push(option);
    self
  }

  pub fn with_flag(mut self, flag: Flag) -> Self {
    self.flags.push(flag);
    self
  }

  pub fn with_subcommand(mut self, name: String, subcommand: Command) -> Self {
    self.subcommands.insert(name, subcommand);
    self
  }

  pub fn deprecated(mut self, message: Option<String>) -> Self {
    self.deprecated = true;
    self.deprecation_message = message;
    self
  }

  pub fn clone(&self) -> Command {
    Command {
      name: self.name.clone(),
      description: self.description.clone(),
      usage: self.usage.clone(),
      long_description: self.long_description.clone(),
      examples: self.examples.clone(),
      aliases: self.aliases.clone(),
      subcommands: self.subcommands.clone(),
      arguments: self.arguments.clone(),
      options: self.options.clone(),
      flags: self.flags.clone(),
      handler: Box::new(|_| Ok(CommandResult::success("Command cloned"))),
      category: self.category,
      visibility: self.visibility,
      enabled: self.enabled,
      deprecated: self.deprecated,
      deprecation_message: self.deprecation_message.clone(),
      version: self.version.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
    }
  }
}

impl Argument {
  pub fn new(name: String, description: String) -> Self {
    Self {
      name,
      description,
      help_text: None,
      required: false,
      multiple: false,
      default_value: None,
      possible_values: Vec::new(),
      value_type: ValueType::String,
      validator: None,
    }
  }

  pub fn required(mut self) -> Self {
    self.required = true;
    self
  }

  pub fn multiple(mut self) -> Self {
    self.multiple = true;
    self
  }

  pub fn default(mut self, value: String) -> Self {
    self.default_value = Some(value);
    self
  }

  pub fn possible_values(mut self, values: Vec<String>) -> Self {
    self.possible_values = values;
    self
  }

  pub fn value_type(mut self, value_type: ValueType) -> Self {
    self.value_type = value_type;
    self
  }

  pub fn validator(mut self, validator: ArgumentValidator) -> Self {
    self.validator = Some(validator);
    self
  }

  pub fn clone(&self) -> Argument {
    Argument {
      name: self.name.clone(),
      description: self.description.clone(),
      help_text: self.help_text.clone(),
      required: self.required,
      multiple: self.multiple,
      default_value: self.default_value.clone(),
      possible_values: self.possible_values.clone(),
      value_type: self.value_type,
      validator: self.validator.clone(),
    }
  }
}

impl Option {
  pub fn new(name: String, description: String) -> Self {
    Self {
      name,
      short_name: None,
      long_name: None,
      description,
      help_text: None,
      required: false,
      multiple: false,
      default_value: None,
      possible_values: Vec::new(),
      value_type: ValueType::String,
      validator: None,
      global: false,
      deprecated: false,
      deprecation_message: None,
    }
  }

  pub fn short(mut self, name: char) -> Self {
    self.short_name = Some(name);
    self
  }

  pub fn long(mut self, name: String) -> Self {
    self.long_name = Some(name);
    self
  }

  pub fn required(mut self) -> Self {
    self.required = true;
    self
  }

  pub fn multiple(mut self) -> Self {
    self.multiple = true;
    self
  }

  pub fn default(mut self, value: String) -> Self {
    self.default_value = Some(value);
    self
  }

  pub fn possible_values(mut self, values: Vec<String>) -> Self {
    self.possible_values = values;
    self
  }

  pub fn value_type(mut self, value_type: ValueType) -> Self {
    self.value_type = value_type;
    self
  }

  pub fn global(mut self) -> Self {
    self.global = true;
    self
  }

  pub fn deprecated(mut self, message: Option<String>) -> Self {
    self.deprecated = true;
    self.deprecation_message = message;
    self
  }

  pub fn clone(&self) -> Option {
    Option {
      name: self.name.clone(),
      short_name: self.short_name,
      long_name: self.long_name.clone(),
      description: self.description.clone(),
      help_text: self.help_text.clone(),
      required: self.required,
      multiple: self.multiple,
      default_value: self.default_value.clone(),
      possible_values: self.possible_values.clone(),
      value_type: self.value_type,
      validator: self.validator.clone(),
      global: self.global,
      deprecated: self.deprecated,
      deprecation_message: self.deprecation_message.clone(),
    }
  }
}

impl Flag {
  pub fn new(name: String, description: String) -> Self {
    Self {
      name,
      short_name: None,
      long_name: None,
      description,
      help_text: None,
      default_value: false,
      global: false,
      deprecated: false,
      deprecation_message: None,
    }
  }

  pub fn short(mut self, name: char) -> Self {
    self.short_name = Some(name);
    self
  }

  pub fn long(mut self, name: String) -> Self {
    self.long_name = Some(name);
    self
  }

  pub fn default(mut self, value: bool) -> Self {
    self.default_value = value;
    self
  }

  pub fn global(mut self) -> Self {
    self.global = true;
    self
  }

  pub fn deprecated(mut self, message: Option<String>) -> Self {
    self.deprecated = true;
    self.deprecation_message = message;
    self
  }

  pub fn clone(&self) -> Flag {
    Flag {
      name: self.name.clone(),
      short_name: self.short_name,
      long_name: self.long_name.clone(),
      description: self.description.clone(),
      help_text: self.help_text.clone(),
      default_value: self.default_value,
      global: self.global,
      deprecated: self.deprecated,
      deprecation_message: self.deprecation_message.clone(),
    }
  }
}

impl CommandConfig {
  pub fn new() -> Self {
    Self {
      global_options: HashMap::new(),
      global_flags: HashMap::new(),
      verbosity: VerbosityLevel::Info,
      output_format: OutputFormat::Text,
      color_mode: ColorMode::Auto,
    }
  }

  pub fn clone(&self) -> CommandConfig {
    CommandConfig {
      global_options: self.global_options.clone(),
      global_flags: self.global_flags.clone(),
      verbosity: self.verbosity,
      output_format: self.output_format,
      color_mode: self.color_mode,
    }
  }
}

impl CommandSession {
  pub fn new() -> Self {
    Self {
      session_id: Uuid::new_v4().to_string(),
      user_id: std::env::var("USER").ok(),
      start_time: Utc::now(),
      variables: HashMap::new(),
      history: Vec::new(),
    }
  }

  pub fn clone(&self) -> CommandSession {
    CommandSession {
      session_id: self.session_id.clone(),
      user_id: self.user_id.clone(),
      start_time: self.start_time,
      variables: self.variables.clone(),
      history: self.history.clone(),
    }
  }
}

impl CommandResult {
  pub fn success(message: &str) -> Self {
    Self {
      success: true,
      message: message.to_string(),
      data: None,
      exit_code: 0,
      duration: std::time::Duration::from_secs(0),
      warnings: Vec::new(),
      errors: Vec::new(),
      metadata: HashMap::new(),
    }
  }

  pub fn error(message: &str) -> Self {
    Self {
      success: false,
      message: message.to_string(),
      data: None,
      exit_code: 1,
      duration: std::time::Duration::from_secs(0),
      warnings: Vec::new(),
      errors: vec![message.to_string()],
      metadata: HashMap::new(),
    }
  }

  pub fn with_data(message: &str, data: serde_json::Value) -> Self {
    Self {
      success: true,
      message: message.to_string(),
      data: Some(data),
      exit_code: 0,
      duration: std::time::Duration::from_secs(0),
      warnings: Vec::new(),
      errors: Vec::new(),
      metadata: HashMap::new(),
    }
  }

  pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
    self.warnings = warnings;
    self
  }

  pub fn with_errors(mut self, errors: Vec<String>) -> Self {
    self.errors = errors;
    self
  }

  pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
    self.metadata = metadata;
    self
  }

  pub fn clone(&self) -> CommandResult {
    CommandResult {
      success: self.success,
      message: self.message.clone(),
      data: self.data.clone(),
      exit_code: self.exit_code,
      duration: self.duration,
      warnings: self.warnings.clone(),
      errors: self.errors.clone(),
      metadata: self.metadata.clone(),
    }
  }
}

impl ArgumentValidator {
  pub fn new(validator_type: ValidatorType) -> Self {
    Self {
      validator_type,
      pattern: None,
      min_length: None,
      max_length: None,
      min_value: None,
      max_value: None,
      custom_validator: None,
    }
  }

  pub fn pattern(mut self, pattern: String) -> Self {
    self.pattern = Some(pattern);
    self
  }

  pub fn min_length(mut self, length: usize) -> Self {
    self.min_length = Some(length);
    self
  }

  pub fn max_length(mut self, length: usize) -> Self {
    self.max_length = Some(length);
    self
  }

  pub fn min_value(mut self, value: f64) -> Self {
    self.min_value = Some(value);
    self
  }

  pub fn max_value(mut self, value: f64) -> Self {
    self.max_value = Some(value);
    self
  }

  pub fn clone(&self) -> ArgumentValidator {
    ArgumentValidator {
      validator_type: self.validator_type,
      pattern: self.pattern.clone(),
      min_length: self.min_length,
      max_length: self.max_length,
      min_value: self.min_value,
      max_value: self.max_value,
      custom_validator: self.custom_validator.clone(),
    }
  }
}

impl ValidationResult {
  pub fn valid() -> Self {
    Self {
      valid: true,
      errors: Vec::new(),
      warnings: Vec::new(),
    }
  }

  pub fn invalid(errors: Vec<String>) -> Self {
    Self {
      valid: false,
      errors,
      warnings: Vec::new(),
    }
  }

  pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
    self.warnings = warnings;
    self
  }

  pub fn clone(&self) -> ValidationResult {
    ValidationResult {
      valid: self.valid,
      errors: self.errors.clone(),
      warnings: self.warnings.clone(),
    }
  }
}

pub fn create_command_registry() -> CommandRegistry {
  CommandRegistry::new()
}

pub fn create_command(name: String, description: String, handler: CommandHandler) -> Command {
  Command::new(name, description, handler)
}

pub fn create_argument(name: String, description: String) -> Argument {
  Argument::new(name, description)
}

pub fn create_option(name: String, description: String) -> Option {
  Option::new(name, description)
}

pub fn create_flag(name: String, description: String) -> Flag {
  Flag::new(name, description)
}

pub fn create_validator(validator_type: ValidatorType) -> ArgumentValidator {
  ArgumentValidator::new(validator_type)
}
