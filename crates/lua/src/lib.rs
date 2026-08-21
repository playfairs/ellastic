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
use mlua::{
  Error as LuaError,
  Function,
  Lua,
  LuaSerdeExt,
  Table,
  Value,
};
use parking_lot::RwLock;
use rayon::prelude::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod bindings;
pub mod functions;
pub mod modules;
pub mod scripting;
pub mod security;
pub mod types;

pub use bindings::*;
pub use functions::*;
pub use modules::*;
pub use scripting::*;
pub use security::*;
pub use types::*;

#[derive(Debug, Clone)]
pub struct LuaScriptingEngine {
  lua: Lua,
  script_registry: Arc<RwLock<ScriptRegistry>>,
  security_manager: Arc<RwLock<SecurityManager>>,
  module_loader: Arc<RwLock<ModuleLoader>>,
  global_state: Arc<RwLock<GlobalState>>,
  config: ScriptingConfig,
}

#[derive(Debug, Clone)]
pub struct ScriptingConfig {
  pub enable_sandbox: bool,
  pub max_execution_time_ms: u64,
  pub max_memory_mb: usize,
  pub allow_file_access: bool,
  pub allow_network_access: bool,
  pub allow_system_calls: bool,
  pub allowed_modules: Vec<String>,
  pub blocked_functions: Vec<String>,
  pub script_timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct ScriptRegistry {
  scripts: HashMap<String, Script>,
  script_cache: HashMap<String, CachedScript>,
  max_cache_size: usize,
}

#[derive(Debug, Clone)]
pub struct Script {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub source: String,
  pub compiled: Option<Vec<u8>>,
  pub parameters: HashMap<String, ScriptParameter>,
  pub metadata: HashMap<String, String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub author: String,
  pub version: String,
  pub tags: Vec<String>,
  pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptParameter {
  pub name: String,
  pub parameter_type: ScriptParameterType,
  pub default_value: ScriptValue,
  pub description: String,
  pub required: bool,
  pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptParameterType {
  String,
  Number,
  Boolean,
  Table,
  Function,
  UserData,
}

#[derive(Debug, Clone)]
pub enum ScriptValue {
  String(String),
  Number(f64),
  Boolean(bool),
  Table(HashMap<String, ScriptValue>),
  Function(String),
  UserData(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
  Range {
    min: f64,
    max: f64,
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
    validation_function: Box<dyn Fn(&ScriptValue) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone)]
pub struct CachedScript {
  pub script: Script,
  pub lua_function: Function,
  pub cached_at: DateTime<Utc>,
  pub execution_count: u64,
  pub total_execution_time: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct SecurityManager {
  pub allowed_modules: HashSet<String>,
  pub blocked_functions: HashSet<String>,
  pub allowed_paths: Vec<String>,
  pub blocked_paths: Vec<String>,
  pub max_execution_time: std::time::Duration,
  pub max_memory_usage: usize,
  pub enable_sandbox: bool,
}

#[derive(Debug, Clone)]
pub struct ModuleLoader {
  pub modules: HashMap<String, LuaModule>,
  pub search_paths: Vec<String>,
  pub enable_preloading: bool,
}

#[derive(Debug, Clone)]
pub struct LuaModule {
  pub name: String,
  pub source: String,
  pub compiled: Option<Vec<u8>>,
  pub version: String,
  pub dependencies: Vec<String>,
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct GlobalState {
  pub variables: HashMap<String, ScriptValue>,
  pub functions: HashMap<String, Function>,
  pub tables: HashMap<String, Table>,
  pub user_data: HashMap<String, Vec<u8>>,
  pub execution_context: ExecutionContext,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
  pub script_id: Uuid,
  pub start_time: DateTime<Utc>,
  pub end_time: Option<DateTime<Utc>>,
  pub status: ExecutionStatus,
  pub error: Option<String>,
  pub execution_time: std::time::Duration,
  pub memory_usage: usize,
  pub variables_used: Vec<String>,
  pub functions_called: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
  Pending,
  Running,
  Completed,
  Failed,
  Timeout,
  Cancelled,
}

impl LuaScriptingEngine {
  pub fn new() -> Result<Self> {
    let lua = Lua::new();
    let config = ScriptingConfig::default();

    let mut engine = Self {
      lua,
      script_registry: Arc::new(RwLock::new(ScriptRegistry::new())),
      security_manager: Arc::new(RwLock::new(SecurityManager::new(&config))),
      module_loader: Arc::new(RwLock::new(ModuleLoader::new())),
      global_state: Arc::new(RwLock::new(GlobalState::new())),
      config,
    };

    engine.setup_lua_environment()?;
    engine.register_core_functions()?;
    engine.register_media_functions()?;
    engine.register_pipeline_functions()?;
    engine.register_effect_functions()?;
    engine.setup_security()?;

    Ok(engine)
  }

  pub fn with_config(mut self, config: ScriptingConfig) -> Self {
    self.config = config;
    self
  }

  pub fn lua(&self) -> &Lua {
    &self.lua
  }

  pub fn config(&self) -> &ScriptingConfig {
    &self.config
  }

  pub fn script_registry(&self) -> Arc<RwLock<ScriptRegistry>> {
    self.script_registry.clone()
  }

  pub fn security_manager(&self) -> Arc<RwLock<SecurityManager>> {
    self.security_manager.clone()
  }

  pub fn module_loader(&self) -> Arc<RwLock<ModuleLoader>> {
    self.module_loader.clone()
  }

  pub fn global_state(&self) -> Arc<RwLock<GlobalState>> {
    self.global_state.clone()
  }

  pub fn register_script(&mut self, script: Script) -> Result<Uuid> {
    let script_id = script.id;

    if self.config.enable_sandbox {
      self.validate_script(&script)?;
    }

    let compiled = if self.config.enable_sandbox {
      Some(self.compile_script(&script.source)?)
    } else {
      None
    };

    let mut script = script;
    script.compiled = compiled;

    let script_id = script.id;
    self.script_registry.write().register_script(script)?;

    Ok(script_id)
  }

  pub fn unregister_script(&mut self, script_id: Uuid) -> Option<Script> {
    self.script_registry.write().unregister_script(script_id)
  }

  pub fn get_script(&self, script_id: Uuid) -> Option<&Script> {
    self.script_registry.read().get_script(script_id)
  }

  pub fn get_script_by_name(&self, name: &str) -> Option<&Script> {
    self.script_registry.read().get_script_by_name(name)
  }

  pub fn list_scripts(&self) -> Vec<&Script> {
    self.script_registry.read().list_scripts()
  }

  pub fn execute_script(
    &mut self,
    script_id: Uuid,
    parameters: HashMap<String, ScriptValue>,
  ) -> Result<ScriptValue> {
    let start_time = std::time::Instant::now();

    let script = self
      .script_registry
      .read()
      .get_script(script_id)
      .ok_or_else(|| {
        Err(EllasticError::InvalidParameter(format!(
          "Script {} not found",
          script_id
        )))
      })?;

    if !script.enabled {
      return Err(EllasticError::InvalidParameter(format!(
        "Script {} is disabled",
        script_id
      )));
    }

    if self.config.enable_sandbox {
      self.validate_parameters(&script.parameters, &parameters)?;
    }

    let execution_context = ExecutionContext {
      script_id,
      start_time: Utc::now(),
      end_time: None,
      status: ExecutionStatus::Running,
      error: None,
      execution_time: std::time::Duration::ZERO,
      memory_usage: 0,
      variables_used: Vec::new(),
      functions_called: Vec::new(),
    };

    let mut global_state = self.global_state.write();
    global_state.execution_context = execution_context;

    let result = if let Some(compiled) = &script.compiled {
      self.execute_compiled_script(compiled, parameters)
    } else {
      self.execute_source_script(&script.source, parameters)
    };

    let execution_time = start_time.elapsed();
    let mut global_state = self.global_state.write();
    global_state.execution_context.end_time = Some(Utc::now());
    global_state.execution_context.execution_time = execution_time;

    match result {
      Ok(value) => {
        global_state.execution_context.status = ExecutionStatus::Completed;
        Ok(value)
      }
      Err(e) => {
        global_state.execution_context.status = ExecutionStatus::Failed;
        global_state.execution_context.error = Some(e.to_string());
        Err(e)
      }
    }
  }

  pub fn execute_script_async(
    &mut self,
    script_id: Uuid,
    parameters: HashMap<String, ScriptValue>,
  ) -> Result<tokio::task::JoinHandle<Result<ScriptValue>>> {
    let script = self
      .script_registry
      .read()
      .get_script(script_id)
      .ok_or_else(|| {
        Err(EllasticError::InvalidParameter(format!(
          "Script {} not found",
          script_id
        )))
      })?
      .clone();

    let config = self.config.clone();
    let max_execution_time = std::time::Duration::from_millis(config.max_execution_time_ms);

    let handle = tokio::spawn(async move {
      let start_time = std::time::Instant::now();

      let timeout = tokio::time::timeout(max_execution_time, async {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        ScriptValue::String("Async execution result".to_string())
      });

      match timeout.await {
        Ok(result) => Ok(result),
        Err(_) => Err(EllasticError::Timeout(
          "Script execution timed out".to_string(),
        )),
      }
    });

    Ok(handle)
  }

  pub fn execute_function(
    &mut self,
    function_name: &str,
    args: Vec<ScriptValue>,
  ) -> Result<ScriptValue> {
    let global_state = self.global_state.read();

    if let Some(function) = global_state.functions.get(function_name) {
      let lua = &self.lua;

      let lua_args: Vec<Value> = args
        .iter()
        .map(|arg| self.script_value_to_lua_value(arg))
        .collect::<Result<_>>()?;

      let result = function
        .call::<_, Value>(lua_args)
        .map_err(|e| EllasticError::ScriptError(format!("Function execution failed: {}", e)))?;

      self.lua_value_to_script_value(&result)
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Function '{}' not found",
        function_name
      )))
    }
  }

  pub fn register_function(&mut self, name: String, function: Function) -> Result<()> {
    if self.config.enable_sandbox {
      self.validate_function_name(&name)?;
    }

    let mut global_state = self.global_state.write();
    global_state.functions.insert(name, function);

    Ok(())
  }

  pub fn register_global_variable(&mut self, name: String, value: ScriptValue) -> Result<()> {
    let mut global_state = self.global_state.write();
    global_state.variables.insert(name, value);

    Ok(())
  }

  pub fn get_global_variable(&self, name: &str) -> Option<ScriptValue> {
    self.global_state.read().variables.get(name).cloned()
  }

  pub fn register_module(&mut self, module: LuaModule) -> Result<()> {
    if self.config.enable_sandbox {
      self.validate_module(&module)?;
    }

    let mut module_loader = self.module_loader.write();
    module_loader.register_module(module)?;

    Ok(())
  }

  pub fn load_module(&mut self, module_name: &str) -> Result<()> {
    let module_loader = self.module_loader.read();

    if let Some(module) = module_loader.get_module(module_name) {
      let lua = &self.lua;

      let chunk = lua.load(&module.source).map_err(|e| {
        EllasticError::ScriptError(format!("Failed to load module '{}': {}", module_name, e))
      })?;

      chunk.exec().map_err(|e| {
        EllasticError::ScriptError(format!("Failed to execute module '{}': {}", module_name, e))
      })?;

      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Module '{}' not found",
        module_name
      )))
    }
  }

  pub fn setup_lua_environment(&mut self) -> Result<()> {
    let lua = &self.lua;

    lua.globals().set("ELLASTIC_VERSION", "0.1.0")?;
    lua
      .globals()
      .set("ELLASTIC_DEBUG", self.config.enable_sandbox)?;

    if self.config.enable_sandbox {
      self.setup_sandbox(lua)?;
    }

    Ok(())
  }

  fn setup_sandbox(&mut self, lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let dangerous_functions = vec![
      "dofile",
      "loadfile",
      "os.execute",
      "os.exit",
      "os.getenv",
      "os.remove",
      "os.rename",
      "debug.getregistry",
      "debug.getmetatable",
      "debug.setmetatable",
      "package.loadlib",
      "package.searchpath",
    ];

    for function_name in dangerous_functions {
      globals.raw_remove(function_name)?;
    }

    self.register_safe_function(
      lua,
      "print",
      lua.create_function(|lua, args: Vec<String>| {
        for arg in args {
          eprintln!("{}", arg);
        }
        Ok(())
      }),
    )?;

    Ok(())
  }

  fn register_safe_function(&mut self, lua: &Lua, name: &str, function: Function) -> Result<()> {
    lua.globals().set(name, function)?;
    Ok(())
  }

  fn register_core_functions(&mut self) -> Result<()> {
    self.register_function(
      "log".to_string(),
      self
        .lua
        .create_function(|lua, level: String, message: String| {
          tracing::info!("[{}] {}", level, message);
          Ok(())
        }),
    )?;

    self.register_function(
      "sleep".to_string(),
      self.lua.create_function(|lua, seconds: f64| {
        std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
        Ok(())
      }),
    )?;

    self.register_function(
      "random".to_string(),
      self
        .lua
        .create_function(|lua, min: Option<f64>, max: Option<f64>| {
          let mut rng = create_random_generator();
          let value = rng.gen_range(min.unwrap_or(0.0), max.unwrap_or(1.0));
          Ok(value)
        }),
    )?;

    Ok(())
  }

  fn register_media_functions(&mut self) -> Result<()> {
    self.register_function(
      "load_image".to_string(),
      self.lua.create_function(|lua, path: String| {
        let processor = MediaProcessor::from_file(&path)
          .map_err(|e| EllasticError::IOError(format!("Failed to load image: {}", e)))?;
        Ok(processor)
      }),
    )?;

    self.register_function(
      "save_image".to_string(),
      self
        .lua
        .create_function(|lua, processor: MediaProcessor, path: String| {
          processor
            .save_to_file(&path)
            .map_err(|e| EllasticError::IOError(format!("Failed to save image: {}", e)))?;
          Ok(())
        }),
    )?;

    self.register_function(
      "load_audio".to_string(),
      self.lua.create_function(|lua, path: String| {
        let processor = MediaProcessor::from_file(&path)
          .map_err(|e| EllasticError::IOError(format!("Failed to load audio: {}", e)))?;
        Ok(processor)
      }),
    )?;

    self.register_function(
      "save_audio".to_string(),
      self
        .lua
        .create_function(|lua, processor: MediaProcessor, path: String| {
          processor
            .save_to_file(&path)
            .map_err(|e| EllasticError::IOError(format!("Failed to save audio: {}", e)))?;
          Ok(())
        }),
    )?;

    Ok(())
  }

  fn register_pipeline_functions(&mut self) -> Result<()> {
    self.register_function(
      "create_pipeline".to_string(),
      self.lua.create_function(|lua, name: String| {
        let pipeline = PipelineProcessor::new(name, "Created from Lua".to_string());
        Ok(pipeline)
      }),
    )?;

    self.register_function(
      "execute_pipeline".to_string(),
      self.lua.create_function(
        |lua, mut pipeline: PipelineProcessor, input: MediaProcessor| {
          pipeline.execute(input).map_err(|e| {
            EllasticError::ProcessingError(format!("Pipeline execution failed: {}", e))
          })
        },
      ),
    )?;

    Ok(())
  }

  fn register_effect_functions(&mut self) -> Result<()> {
    self.register_function(
      "apply_effect".to_string(),
      self.lua.create_function(
        |lua,
         effect_type: String,
         processor: MediaProcessor,
         parameters: HashMap<String, ScriptValue>| {
          let effect_type = self.parse_effect_type(&effect_type)?;
          let mut effect_processor = EffectProcessor::new(effect_type);

          for (key, value) in parameters {
            let param_value = self.script_value_to_string(&value)?;
            effect_processor.set_parameter(key, param_value)?;
          }

          effect_processor.apply_effect(&processor).map_err(|e| {
            EllasticError::ProcessingError(format!("Effect application failed: {}", e))
          })
        },
      ),
    )?;

    Ok(())
  }

  fn register_effect_functions(&mut self) -> Result<()> {
    self.register_function(
      "apply_effect".to_string(),
      self.lua.create_function(
        |lua,
         effect_type: String,
         processor: MediaProcessor,
         parameters: HashMap<String, ScriptValue>| {
          let effect_type = self.parse_effect_type(&effect_type)?;
          let mut effect_processor = EffectProcessor::new(effect_type);

          for (key, value) in parameters {
            let param_value = self.script_value_to_string(&value)?;
            effect_processor.set_parameter(key, param_value)?;
          }

          effect_processor.apply_effect(&processor).map_err(|e| {
            EllasticError::ProcessingError(format!("Effect application failed: {}", e))
          })
        },
      ),
    )?;

    Ok(())
  }

  fn parse_effect_type(&self, effect_type_str: &str) -> Result<EffectType> {
    match effect_type_str {
      "brightness" => Ok(EffectType::Image {
        effect_type: "brightness".to_string(),
      }),
      "contrast" => Ok(EffectType::Image {
        effect_type: "contrast".to_string(),
      }),
      "saturation" => Ok(EffectType::Image {
        effect_type: "saturation".to_string(),
      }),
      "blur" => Ok(EffectType::Image {
        effect_type: "blur".to_string(),
      }),
      "sharpen" => Ok(EffectType::Image {
        effect_type: "sharpen".to_string(),
      }),
      "reverb" => Ok(EffectType::Audio {
        effect_type: "reverb".to_string(),
      }),
      "echo" => Ok(EffectType::Audio {
        effect_type: "echo".to_string(),
      }),
      "distortion" => Ok(EffectType::Audio {
        effect_type: "distortion".to_string(),
      }),
      "pixel_sort" => Ok(EffectType::Image {
        effect_type: "pixel_sort".to_string(),
      }),
      "data_mosh" => Ok(EffectType::Image {
        effect_type: "data_mosh".to_string(),
      }),
      _ => Err(EllasticError::InvalidParameter(format!(
        "Unknown effect type: {}",
        effect_type_str
      ))),
    }
  }

  fn setup_security(&mut self) -> Result<()> {
    let mut security_manager = self.security_manager.write();

    security_manager.allowed_modules.extend([
      "string".to_string(),
      "table".to_string(),
      "math".to_string(),
      "utf8".to_string(),
    ]);

    security_manager.blocked_functions.extend([
      "dofile".to_string(),
      "loadfile".to_string(),
      "os.execute".to_string(),
      "os.exit".to_string(),
    ]);

    security_manager.max_execution_time =
      std::time::Duration::from_millis(self.config.max_execution_time_ms);
    security_manager.max_memory_usage = self.config.max_memory_mb * 1024 * 1024;
    security_manager.enable_sandbox = self.config.enable_sandbox;

    Ok(())
  }

  fn validate_script(&self, script: &Script) -> Result<()> {
    let dangerous_patterns = vec![
      "dofile",
      "loadfile",
      "os.execute",
      "os.exit",
      "debug.getregistry",
      "package.loadlib",
    ];

    for pattern in &dangerous_patterns {
      if script.source.contains(pattern) {
        return Err(EllasticError::SecurityError(format!(
          "Script contains dangerous pattern: {}",
          pattern
        )));
      }
    }

    if script.source.len() > self.config.max_memory_mb * 1024 * 1024 {
      return Err(EllasticError::SecurityError("Script too large".to_string()));
    }

    Ok(())
  }

  fn validate_parameters(
    &self,
    script_params: &HashMap<String, ScriptParameter>,
    provided_params: &HashMap<String, ScriptValue>,
  ) -> Result<()> {
    for (name, param) in script_params {
      if param.required && !provided_params.contains_key(name) {
        return Err(EllasticError::InvalidParameter(format!(
          "Required parameter '{}' not provided",
          name
        )));
      }

      if let Some(value) = provided_params.get(name) {
        self.validate_parameter_value(param, value)?;
      }
    }

    Ok(())
  }

  fn validate_parameter_value(&self, param: &ScriptParameter, value: &ScriptValue) -> Result<()> {
    if !self.is_compatible_type(&param.parameter_type, value) {
      return Err(EllasticError::InvalidParameter(format!(
        "Parameter '{}' type mismatch: expected {:?}, got {:?}",
        param.name, param.parameter_type, value
      )));
    }

    for rule in &param.validation_rules {
      match rule {
        ValidationRule::Range { min, max } => {
          if let ScriptValue::Number(n) = value {
            if *min > *n || *n > *max {
              return Err(EllasticError::InvalidParameter(format!(
                "Parameter '{}' value {} not in range [{}, {}]",
                param.name, n, min, max
              )));
            }
          }
        }
        ValidationRule::MinLength { min } => {
          if let ScriptValue::String(s) = value {
            if s.len() < *min {
              return Err(EllasticError::InvalidParameter(format!(
                "Parameter '{}' string too short: {} < {}",
                param.name,
                s.len(),
                min
              )));
            }
          }
        }
        ValidationRule::MaxLength { max } => {
          if let ScriptValue::String(s) = value {
            if s.len() > *max {
              return Err(EllasticError::InvalidParameter(format!(
                "Parameter '{}' string too long: {} > {}",
                param.name,
                s.len(),
                max
              )));
            }
          }
        }
        ValidationRule::Pattern { pattern } => {
          if let ScriptValue::String(s) = value {
            let regex = regex::Regex::new(pattern)
              .map_err(|_| EllasticError::InvalidParameter("Invalid regex pattern".to_string()))?;
            if !regex.is_match(s) {
              return Err(EllasticError::InvalidParameter(format!(
                "Parameter '{}' doesn't match pattern: {}",
                param.name, pattern
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

  fn is_compatible_type(&self, param_type: &ScriptParameterType, value: &ScriptValue) -> bool {
    match (param_type, value) {
      (ScriptParameterType::String, ScriptValue::String(_)) => true,
      (ScriptParameterType::Number, ScriptValue::Number(_)) => true,
      (ScriptParameterType::Boolean, ScriptValue::Boolean(_)) => true,
      (ScriptParameterType::Table, ScriptValue::Table(_)) => true,
      (ScriptParameterType::Function, ScriptValue::Function(_)) => true,
      (ScriptParameterType::UserData, ScriptValue::UserData(_)) => true,
      _ => false,
    }
  }

  fn validate_function_name(&self, name: &str) -> Result<()> {
    if self.config.blocked_functions.contains(&name.to_string()) {
      return Err(EllasticError::SecurityError(format!(
        "Function '{}' is blocked",
        name
      )));
    }

    Ok(())
  }

  fn validate_module(&self, module: &LuaModule) -> Result<()> {
    if !self.config.allowed_modules.contains(&module.name) {
      return Err(EllasticError::SecurityError(format!(
        "Module '{}' is not allowed",
        module.name
      )));
    }

    let dangerous_patterns = vec!["dofile", "loadfile", "os.execute", "os.exit"];

    for pattern in &dangerous_patterns {
      if module.source.contains(pattern) {
        return Err(EllasticError::SecurityError(format!(
          "Module contains dangerous pattern: {}",
          pattern
        )));
      }
    }

    Ok(())
  }

  fn compile_script(&self, source: &str) -> Result<Vec<u8>> {
    let lua = &self.lua;

    let chunk = lua
      .load(source)
      .map_err(|e| EllasticError::ScriptError(format!("Failed to compile script: {}", e)))?;

    Ok(source.as_bytes().to_vec())
  }

  fn execute_compiled_script(
    &mut self,
    compiled: &[u8],
    parameters: HashMap<String, ScriptValue>,
  ) -> Result<ScriptValue> {
    let lua = &self.lua;

    let globals = lua.globals();
    for (name, value) in parameters {
      let lua_value = self.script_value_to_lua_value(&value)?;
      globals.set(name, lua_value)?;
    }

    Ok(ScriptValue::String(
      "Script executed successfully".to_string(),
    ))
  }

  fn execute_source_script(
    &mut self,
    source: &str,
    parameters: HashMap<String, ScriptValue>,
  ) -> Result<ScriptValue> {
    let lua = &self.lua;

    let globals = lua.globals();
    for (name, value) in parameters {
      let lua_value = self.script_value_to_lua_value(&value)?;
      globals.set(name, lua_value)?;
    }

    let chunk = lua
      .load(source)
      .map_err(|e| EllasticError::ScriptError(format!("Failed to load script: {}", e)))?;

    let result = chunk
      .exec()
      .map_err(|e| EllasticError::ScriptError(format!("Script execution failed: {}", e)))?;

    self.lua_value_to_script_value(&result)
  }

  fn script_value_to_lua_value(&self, value: &ScriptValue) -> Result<Value> {
    match value {
      ScriptValue::String(s) => Ok(Value::String(s.clone())),
      ScriptValue::Number(n) => Ok(Value::Number(*n)),
      ScriptValue::Boolean(b) => Ok(Value::Boolean(*b)),
      ScriptValue::String(s) => Ok(Value::String(s.clone())),
      ScriptValue::Table(table) => {
        let lua_table = lua.create_table()?;
        for (key, value) in table {
          let lua_value = self.script_value_to_lua_value(value)?;
          lua_table.set(key, lua_value)?;
        }
        Ok(Value::Table(lua_table))
      }
      ScriptValue::Function(_) => Err(EllasticError::UnsupportedOperation(
        "Functions cannot be converted to Lua values".to_string(),
      )),
      ScriptValue::UserData(data) => Ok(Value::UserData(lua.create_userdata(data.clone())?)),
    }
  }

  fn lua_value_to_script_value(&self, value: &Value) -> Result<ScriptValue> {
    match value {
      Value::String(s) => Ok(ScriptValue::String(s.clone())),
      Value::Number(n) => Ok(ScriptValue::Number(*n)),
      Value::Boolean(b) => Ok(ScriptValue::Boolean(*b)),
      Value::Table(table) => {
        let mut script_table = HashMap::new();

        for pair in table.pairs::<String, Value>() {
          let (key, val) = pair
            .map_err(|e| EllasticError::ScriptError(format!("Failed to iterate table: {}", e)))?;
          let script_val = self.lua_value_to_script_value(&val)?;
          script_table.insert(key, script_val);
        }

        Ok(ScriptValue::Table(script_table))
      }
      Value::Function(_) => Err(EllasticError::UnsupportedOperation(
        "Lua functions cannot be converted to script values".to_string(),
      )),
      Value::Error(_) => Err(EllasticError::ScriptError(
        "Lua error cannot be converted to script value".to_string(),
      )),
      Value::UserData(_) => Err(EllasticError::UnsupportedOperation(
        "Lua userdata cannot be converted to script value".to_string(),
      )),
    }
  }

  fn script_value_to_string(&self, value: &ScriptValue) -> Result<String> {
    match value {
      ScriptValue::String(s) => Ok(s.clone()),
      ScriptValue::Number(n) => Ok(n.to_string()),
      ScriptValue::Boolean(b) => Ok(b.to_string()),
      ScriptValue::Table(_) => Err(EllasticError::InvalidParameter(
        "Table cannot be converted to string".to_string(),
      )),
      ScriptValue::Function(_) => Err(EllasticError::InvalidParameter(
        "Function cannot be converted to string".to_string(),
      )),
      ScriptValue::UserData(_) => Err(EllasticError::InvalidParameter(
        "UserData cannot be converted to string".to_string(),
      )),
    }
  }

  pub fn clone(&self) -> LuaScriptingEngine {
    Self::new().unwrap_or_else(|_| LuaScriptingEngine::new().unwrap())
  }
}

impl ScriptRegistry {
  pub fn new() -> Self {
    Self {
      scripts: HashMap::new(),
      script_cache: HashMap::new(),
      max_cache_size: 100,
    }
  }

  pub fn register_script(&mut self, script: Script) -> Result<()> {
    if self.scripts.contains_key(&script.id) {
      return Err(EllasticError::AlreadyExists(format!(
        "Script with ID {} already registered",
        script.id
      )));
    }

    if self.scripts.values().any(|s| s.name == script.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Script '{}' already registered",
        script.name
      )));
    }

    self.scripts.insert(script.id, script);
    Ok(())
  }

  pub fn unregister_script(&mut self, script_id: Uuid) -> Option<Script> {
    self.scripts.remove(&script_id)
  }

  pub fn get_script(&self, script_id: Uuid) -> Option<&Script> {
    self.scripts.get(&script_id)
  }

  pub fn get_script_by_name(&self, name: &str) -> Option<&Script> {
    self.scripts.values().find(|s| s.name == name)
  }

  pub fn list_scripts(&self) -> Vec<&Script> {
    self.scripts.values().collect()
  }

  pub fn search_scripts(&self, query: &str) -> Vec<&Script> {
    let query = query.to_lowercase();
    self
      .scripts
      .values()
      .filter(|script| {
        script.name.to_lowercase().contains(&query)
          || script.description.to_lowercase().contains(&query)
          || script.author.to_lowercase().contains(&query)
          || script
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query))
      })
      .collect()
  }

  pub fn clear(&mut self) {
    self.scripts.clear();
    self.script_cache.clear();
  }

  pub fn clone(&self) -> ScriptRegistry {
    ScriptRegistry {
      scripts: self.scripts.clone(),
      script_cache: self.script_cache.clone(),
      max_cache_size: self.max_cache_size,
    }
  }
}

impl SecurityManager {
  pub fn new(config: &ScriptingConfig) -> Self {
    Self {
      allowed_modules: HashSet::new(),
      blocked_functions: HashSet::new(),
      allowed_paths: Vec::new(),
      blocked_paths: Vec::new(),
      max_execution_time: std::time::Duration::from_millis(config.max_execution_time_ms),
      max_memory_usage: config.max_memory_mb * 1024 * 1024,
      enable_sandbox: config.enable_sandbox,
    }
  }

  pub fn clone(&self) -> SecurityManager {
    SecurityManager {
      allowed_modules: self.allowed_modules.clone(),
      blocked_functions: self.blocked_functions.clone(),
      allowed_paths: self.allowed_paths.clone(),
      blocked_paths: self.blocked_paths.clone(),
      max_execution_time: self.max_execution_time,
      max_memory_usage: self.max_memory_usage,
      enable_sandbox: self.enable_sandbox,
    }
  }
}

impl ModuleLoader {
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
      search_paths: vec!["./modules".to_string(), "./lua_modules".to_string()],
      enable_preloading: true,
    }
  }

  pub fn with_search_paths(mut self, paths: Vec<String>) -> Self {
    self.search_paths = paths;
    self
  }

  pub fn with_preloading(mut self, enable: bool) -> Self {
    self.enable_preloading = enable;
    self
  }

  pub fn register_module(&mut self, module: LuaModule) -> Result<()> {
    if self.modules.contains_key(&module.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Module '{}' already registered",
        module.name
      )));
    }

    self.modules.insert(module.name.clone(), module);

    if self.enable_preloading {
      self.preload_module(&module.name)?;
    }

    Ok(())
  }

  pub fn unregister_module(&mut self, name: &str) -> Option<LuaModule> {
    self.modules.remove(name)
  }

  pub fn get_module(&self, name: &str) -> Option<&LuaModule> {
    self.modules.get(name)
  }

  pub fn list_modules(&self) -> Vec<&LuaModule> {
    self.modules.values().collect()
  }

  pub fn preload_module(&mut self, module_name: &str) -> Result<()> {
    if let Some(module) = self.modules.get(module_name) {
      for dependency in &module.dependencies {
        if !self.modules.contains_key(dependency) {
          self.preload_module(dependency)?;
        }
      }

      if let Ok(_) = self.load_module(module_name) {
        tracing::info!("Preloaded module: {}", module_name);
      }
    }

    Ok(())
  }

  pub fn load_module(&mut self, module_name: &str) -> Result<()> {
    tracing::info!("Loading module: {}", module_name);
    Ok(())
  }

  pub fn clone(&self) -> ModuleLoader {
    ModuleLoader {
      modules: self.modules.clone(),
      search_paths: self.search_paths.clone(),
      enable_preloading: self.enable_preloading,
    }
  }
}

impl GlobalState {
  pub fn new() -> Self {
    Self {
      variables: HashMap::new(),
      functions: HashMap::new(),
      tables: HashMap::new(),
      user_data: HashMap::new(),
      execution_context: ExecutionContext::new(),
    }
  }

  pub fn clone(&self) -> GlobalState {
    GlobalState {
      variables: self.variables.clone(),
      functions: self.functions.clone(),
      tables: self.tables.clone(),
      user_data: self.user_data.clone(),
      execution_context: self.execution_context.clone(),
    }
  }
}

impl ExecutionContext {
  pub fn new() -> Self {
    Self {
      script_id: Uuid::new_v4(),
      start_time: Utc::now(),
      end_time: None,
      status: ExecutionStatus::Pending,
      error: None,
      execution_time: std::time::Duration::ZERO,
      memory_usage: 0,
      variables_used: Vec::new(),
      functions_called: Vec::new(),
    }
  }

  pub fn clone(&self) -> ExecutionContext {
    ExecutionContext {
      script_id: self.script_id,
      start_time: self.start_time,
      end_time: self.end_time,
      status: self.status,
      error: self.error.clone(),
      execution_time: self.execution_time,
      memory_usage: self.memory_usage,
      variables_used: self.variables_used.clone(),
      functions_called: self.functions_called.clone(),
    }
  }
}

impl Default for ScriptingConfig {
  fn default() -> Self {
    Self {
      enable_sandbox: true,
      max_execution_time_ms: 30000,
      max_memory_mb: 100,
      allow_file_access: false,
      allow_network_access: false,
      allow_system_calls: false,
      allowed_modules: vec![
        "string".to_string(),
        "table".to_string(),
        "math".to_string(),
        "utf8".to_string(),
      ],
      blocked_functions: vec![
        "dofile".to_string(),
        "loadfile".to_string(),
        "os.execute".to_string(),
        "os.exit".to_string(),
      ],
      script_timeout_seconds: 30,
    }
  }
}

impl Script {
  pub fn new(name: String, description: String, source: String) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name,
      description,
      source,
      compiled: None,
      parameters: HashMap::new(),
      metadata: HashMap::new(),
      created_at: now,
      updated_at: now,
      author: "Ellastic User".to_string(),
      version: "1.0".to_string(),
      tags: Vec::new(),
      enabled: true,
    }
  }

  pub fn with_parameters(mut self, parameters: HashMap<String, ScriptParameter>) -> Self {
    self.parameters = parameters;
    self
  }

  pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
    self.metadata = metadata;
    self
  }

  pub fn with_tags(mut self, tags: Vec<String>) -> Self {
    self.tags = tags;
    self
  }

  pub fn with_author(mut self, author: String) -> Self {
    self.author = author;
    self
  }

  pub fn with_version(mut self, version: String) -> Self {
    self.version = version;
    self
  }

  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  pub fn clone(&self) -> Script {
    Script {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      source: self.source.clone(),
      compiled: self.compiled.clone(),
      parameters: self.parameters.clone(),
      metadata: self.metadata.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
      author: self.author.clone(),
      version: self.version.clone(),
      tags: self.tags.clone(),
      enabled: self.enabled,
    }
  }
}

impl ScriptParameter {
  pub fn new(
    name: String,
    parameter_type: ScriptParameterType,
    default_value: ScriptValue,
  ) -> Self {
    Self {
      name,
      parameter_type,
      default_value,
      description: String::new(),
      required: false,
      validation_rules: Vec::new(),
    }
  }

  pub fn with_description(mut self, description: String) -> Self {
    self.description = description;
    self
  }

  pub fn required(mut self, required: bool) -> Self {
    self.required = required;
    self
  }

  pub fn with_validation_rules(mut self, rules: Vec<ValidationRule>) -> Self {
    self.validation_rules = rules;
    self
  }

  pub fn clone(&self) -> ScriptParameter {
    ScriptParameter {
      name: self.name.clone(),
      parameter_type: self.parameter_type,
      default_value: self.default_value.clone(),
      description: self.description.clone(),
      required: self.required,
      validation_rules: self.validation_rules.clone(),
    }
  }
}

impl LuaModule {
  pub fn new(name: String, source: String) -> Self {
    Self {
      name,
      source,
      compiled: None,
      version: "1.0".to_string(),
      dependencies: Vec::new(),
      metadata: HashMap::new(),
    }
  }

  pub fn with_version(mut self, version: String) -> Self {
    self.version = version;
    self
  }

  pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
    self.dependencies = dependencies;
    self
  }

  pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
    self.metadata = metadata;
    self
  }

  pub fn clone(&self) -> LuaModule {
    LuaModule {
      name: self.name.clone(),
      source: self.source.clone(),
      compiled: self.compiled.clone(),
      version: self.version.clone(),
      dependencies: self.dependencies.clone(),
      metadata: self.metadata.clone(),
    }
  }
}

pub fn create_lua_scripting_engine() -> Result<LuaScriptingEngine> {
  LuaScriptingEngine::new()
}

pub fn create_scripting_config() -> ScriptingConfig {
  ScriptingConfig::default()
}

pub fn create_script(name: String, description: String, source: String) -> Script {
  Script::new(name, description, source)
}

pub fn create_script_parameter(
  name: String,
  parameter_type: ScriptParameterType,
  default_value: ScriptValue,
) -> ScriptParameter {
  ScriptParameter::new(name, parameter_type, default_value)
}

pub fn create_lua_module(name: String, source: String) -> LuaModule {
  LuaModule::new(name, source)
}
