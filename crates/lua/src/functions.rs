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
  Table,
  Value,
};
use parking_lot::RwLock;
use rayon::prelude::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LuaFunctionRegistry {
  functions: HashMap<String, LuaFunction>,
  categories: HashMap<FunctionCategory, Vec<String>>,
  aliases: HashMap<String, String>,
  documentation: HashMap<String, FunctionDocumentation>,
}

#[derive(Debug, Clone)]
pub struct LuaFunction {
  pub name: String,
  pub function: Function,
  pub signature: FunctionSignature,
  pub category: FunctionCategory,
  pub description: String,
  pub examples: Vec<String>,
  pub parameters: Vec<ParameterInfo>,
  pub return_type: ReturnType,
  pub side_effects: bool,
  pub thread_safe: bool,
  pub deprecated: bool,
  pub version: String,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
  pub parameters: Vec<ParameterInfo>,
  pub return_type: ReturnType,
  pub variadic: bool,
  pub min_args: usize,
  pub max_args: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
  pub name: String,
  pub parameter_type: ParameterType,
  pub optional: bool,
  pub default_value: Option<String>,
  pub description: String,
  pub validation: Option<ValidationRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
  String,
  Number,
  Boolean,
  Table,
  Function,
  UserData,
  Any,
  MediaProcessor,
  EffectProcessor,
  PipelineProcessor,
}

#[derive(Debug, Clone)]
pub enum ReturnType {
  Single(ParameterType),
  Multiple(Vec<ParameterType>),
  Any,
  Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionCategory {
  Core,
  Media,
  Effects,
  Pipeline,
  Utility,
  Math,
  String,
  Table,
  Debug,
  Custom,
}

#[derive(Debug, Clone)]
pub struct FunctionDocumentation {
  pub summary: String,
  pub description: String,
  pub parameters: Vec<ParameterDocumentation>,
  pub returns: ReturnDocumentation,
  pub examples: Vec<String>,
  pub see_also: Vec<String>,
  pub notes: Vec<String>,
  pub version: String,
  pub deprecated: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParameterDocumentation {
  pub name: String,
  pub type_name: String,
  pub description: String,
  pub optional: bool,
  pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReturnDocumentation {
  pub type_name: String,
  pub description: String,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
  Min {
    value: f64,
  },
  Max {
    value: f64,
  },
  Range {
    min: f64,
    max: f64,
  },
  Pattern {
    pattern: String,
  },
  Enum {
    values: Vec<String>,
  },
  Custom {
    validator: Box<dyn Fn(&Value) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone)]
pub struct FunctionExecutor {
  lua: Lua,
  registry: Arc<RwLock<LuaFunctionRegistry>>,
  execution_context: Arc<RwLock<ExecutionContext>>,
  performance_tracker: Arc<RwLock<PerformanceTracker>>,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
  pub call_stack: Vec<FunctionCall>,
  pub variables: HashMap<String, Value>,
  pub current_function: Option<String>,
  pub execution_time: std::time::Duration,
  pub memory_usage: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
  pub function_name: String,
  pub parameters: Vec<Value>,
  pub start_time: DateTime<Utc>,
  pub end_time: Option<DateTime<Utc>>,
  pub duration: Option<std::time::Duration>,
  pub result: Option<Value>,
  pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceTracker {
  pub call_counts: HashMap<String, u64>,
  pub total_times: HashMap<String, std::time::Duration>,
  pub average_times: HashMap<String, std::time::Duration>,
  pub error_counts: HashMap<String, u64>,
  pub slow_functions: Vec<SlowFunction>,
}

#[derive(Debug, Clone)]
pub struct SlowFunction {
  pub function_name: String,
  pub average_time: std::time::Duration,
  pub call_count: u64,
  pub severity: PerformanceSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceSeverity {
  Low,
  Medium,
  High,
  Critical,
}

impl LuaFunctionRegistry {
  pub fn new() -> Self {
    Self {
      functions: HashMap::new(),
      categories: HashMap::new(),
      aliases: HashMap::new(),
      documentation: HashMap::new(),
    }
  }

  pub fn register_function(&mut self, function: LuaFunction) -> Result<()> {
    let name = function.name.clone();

    if self.functions.contains_key(&name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Function '{}' already registered",
        name
      )));
    }

    self
      .categories
      .entry(function.category)
      .or_insert_with(Vec::new)
      .push(name.clone());
    self.functions.insert(name.clone(), function);

    Ok(())
  }

  pub fn register_alias(&mut self, alias: String, function_name: String) -> Result<()> {
    if !self.functions.contains_key(&function_name) {
      return Err(EllasticError::InvalidParameter(format!(
        "Function '{}' not found",
        function_name
      )));
    }

    self.aliases.insert(alias, function_name);
    Ok(())
  }

  pub fn get_function(&self, name: &str) -> Option<&LuaFunction> {
    if let Some(function) = self.functions.get(name) {
      Some(function)
    } else if let Some(alias_target) = self.aliases.get(name) {
      self.functions.get(alias_target)
    } else {
      None
    }
  }

  pub fn list_functions(&self) -> Vec<&String> {
    self.functions.keys().collect()
  }

  pub fn list_functions_by_category(&self, category: FunctionCategory) -> Vec<&String> {
    self
      .categories
      .get(&category)
      .map_or(Vec::new(), |functions| functions.iter().collect())
  }

  pub fn search_functions(&self, query: &str) -> Vec<&LuaFunction> {
    let query = query.to_lowercase();
    self
      .functions
      .values()
      .filter(|function| {
        function.name.to_lowercase().contains(&query)
          || function.description.to_lowercase().contains(&query)
          || function
            .examples
            .iter()
            .any(|example| example.to_lowercase().contains(&query))
      })
      .collect()
  }

  pub fn get_documentation(&self, name: &str) -> Option<&FunctionDocumentation> {
    self.documentation.get(name)
  }

  pub fn clone(&self) -> LuaFunctionRegistry {
    LuaFunctionRegistry {
      functions: self.functions.clone(),
      categories: self.categories.clone(),
      aliases: self.aliases.clone(),
      documentation: self.documentation.clone(),
    }
  }
}

impl FunctionExecutor {
  pub fn new(lua: Lua) -> Self {
    Self {
      lua,
      registry: Arc::new(RwLock::new(LuaFunctionRegistry::new())),
      execution_context: Arc::new(RwLock::new(ExecutionContext::new())),
      performance_tracker: Arc::new(RwLock::new(PerformanceTracker::new())),
    }
  }

  pub fn with_registry(mut self, registry: LuaFunctionRegistry) -> Self {
    self.registry = Arc::new(RwLock::new(registry));
    self
  }

  pub fn register_function(&mut self, function: LuaFunction) -> Result<()> {
    self.registry.write().register_function(function)
  }

  pub fn execute_function(&mut self, name: &str, parameters: Vec<Value>) -> Result<Value> {
    let start_time = std::time::Instant::now();

    let function = self
      .registry
      .read()
      .get_function(name)
      .ok_or_else(|| {
        Err(EllasticError::InvalidParameter(format!(
          "Function '{}' not found",
          name
        )))
      })?
      .clone();

    self.validate_parameters(&function, &parameters)?;

    {
      let mut context = self.execution_context.write();
      context.current_function = Some(name.to_string());
      context.call_stack.push(FunctionCall {
        function_name: name.to_string(),
        parameters: parameters.clone(),
        start_time: Utc::now(),
        end_time: None,
        duration: None,
        result: None,
        error: None,
      });
    }

    let result = function.function.call::<_, Value>(parameters).map_err(|e| {
      EllasticError::ScriptError(format!("Function '{}' execution failed: {}", name, e))
    });

    let execution_time = start_time.elapsed();

    {
      let mut context = self.execution_context.write();
      context.execution_time += execution_time;

      if let Some(call) = context.call_stack.last_mut() {
        call.end_time = Some(Utc::now());
        call.duration = Some(execution_time);
        match &result {
          Ok(value) => call.result = Some(value.clone()),
          Err(e) => call.error = Some(e.to_string()),
        }
      }
    }

    {
      let mut tracker = self.performance_tracker.write();
      tracker.record_call(name, execution_time, result.is_err());
    }

    result
  }

  fn validate_parameters(&self, function: &LuaFunction, parameters: &[Value]) -> Result<()> {
    let signature = &function.signature;

    if parameters.len() < signature.min_args {
      return Err(EllasticError::InvalidParameter(format!(
        "Function '{}' requires at least {} arguments, got {}",
        function.name,
        signature.min_args,
        parameters.len()
      )));
    }

    if let Some(max_args) = signature.max_args {
      if parameters.len() > max_args {
        return Err(EllasticError::InvalidParameter(format!(
          "Function '{}' accepts at most {} arguments, got {}",
          function.name,
          max_args,
          parameters.len()
        )));
      }
    }

    for (i, param_info) in function.parameters.iter().enumerate() {
      if i < parameters.len() {
        let param_value = &parameters[i];
        self.validate_parameter(param_info, param_value)?;
      } else if !param_info.optional {
        return Err(EllasticError::InvalidParameter(format!(
          "Required parameter '{}' not provided for function '{}'",
          param_info.name, function.name
        )));
      }
    }

    Ok(())
  }

  fn validate_parameter(&self, param_info: &ParameterInfo, value: &Value) -> Result<()> {
    if !self.is_compatible_type(&param_info.parameter_type, value) {
      return Err(EllasticError::InvalidParameter(format!(
        "Parameter '{}' type mismatch: expected {:?}, got {:?}",
        param_info.name, param_info.parameter_type, value
      )));
    }

    if let Some(validation) = &param_info.validation {
      self.apply_validation(validation, value)?;
    }

    Ok(())
  }

  fn is_compatible_type(&self, param_type: &ParameterType, value: &Value) -> bool {
    match (param_type, value) {
      (ParameterType::String, Value::String(_)) => true,
      (ParameterType::Number, Value::Number(_)) => true,
      (ParameterType::Boolean, Value::Boolean(_)) => true,
      (ParameterType::Table, Value::Table(_)) => true,
      (ParameterType::Function, Value::Function(_)) => true,
      (ParameterType::UserData, Value::UserData(_)) => true,
      (ParameterType::Any, _) => true,
      (ParameterType::MediaProcessor, Value::UserData(_)) => true,
      (ParameterType::EffectProcessor, Value::UserData(_)) => true,
      (ParameterType::PipelineProcessor, Value::UserData(_)) => true,
      _ => false,
    }
  }

  fn apply_validation(&self, validation: &ValidationRule, value: &Value) -> Result<()> {
    match validation {
      ValidationRule::Min { value: min_val } => {
        if let Value::Number(n) = value {
          if *n < *min_val {
            return Err(EllasticError::InvalidParameter(format!(
              "Value {} is less than minimum {}",
              n, min_val
            )));
          }
        }
      }
      ValidationRule::Max { value: max_val } => {
        if let Value::Number(n) = value {
          if *n > *max_val {
            return Err(EllasticError::InvalidParameter(format!(
              "Value {} is greater than maximum {}",
              n, max_val
            )));
          }
        }
      }
      ValidationRule::Range { min, max } => {
        if let Value::Number(n) = value {
          if *n < *min || *n > *max {
            return Err(EllasticError::InvalidParameter(format!(
              "Value {} is not in range [{}, {}]",
              n, min, max
            )));
          }
        }
      }
      ValidationRule::Pattern { pattern } => {
        if let Value::String(s) = value {
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
      ValidationRule::Enum { values } => {
        if let Value::String(s) = value {
          if !values.contains(s) {
            return Err(EllasticError::InvalidParameter(format!(
              "Value '{}' is not in enum: {:?}",
              s, values
            )));
          }
        }
      }
      ValidationRule::Custom { validator } => {
        validator(value)?;
      }
    }

    Ok(())
  }

  pub fn get_registry(&self) -> Arc<RwLock<LuaFunctionRegistry>> {
    self.registry.clone()
  }

  pub fn get_execution_context(&self) -> Arc<RwLock<ExecutionContext>> {
    self.execution_context.clone()
  }

  pub fn get_performance_tracker(&self) -> Arc<RwLock<PerformanceTracker>> {
    self.performance_tracker.clone()
  }

  pub fn clone(&self) -> FunctionExecutor {
    FunctionExecutor {
      lua: self.lua.clone(),
      registry: self.registry.clone(),
      execution_context: self.execution_context.clone(),
      performance_tracker: self.performance_tracker.clone(),
    }
  }
}

impl ExecutionContext {
  pub fn new() -> Self {
    Self {
      call_stack: Vec::new(),
      variables: HashMap::new(),
      current_function: None,
      execution_time: std::time::Duration::ZERO,
      memory_usage: 0,
    }
  }

  pub fn clone(&self) -> ExecutionContext {
    ExecutionContext {
      call_stack: self.call_stack.clone(),
      variables: self.variables.clone(),
      current_function: self.current_function.clone(),
      execution_time: self.execution_time,
      memory_usage: self.memory_usage,
    }
  }
}

impl PerformanceTracker {
  pub fn new() -> Self {
    Self {
      call_counts: HashMap::new(),
      total_times: HashMap::new(),
      average_times: HashMap::new(),
      error_counts: HashMap::new(),
      slow_functions: Vec::new(),
    }
  }

  pub fn record_call(
    &mut self,
    function_name: &str,
    execution_time: std::time::Duration,
    had_error: bool,
  ) {
    *self
      .call_counts
      .entry(function_name.to_string())
      .or_insert(0) += 1;

    let total_time = self
      .total_times
      .entry(function_name.to_string())
      .or_insert_with(|| std::time::Duration::ZERO);
    *total_time += execution_time;

    let call_count = self.call_counts[function_name];
    let average_time = *total_time / call_count as u32;
    self
      .average_times
      .insert(function_name.to_string(), average_time);

    if had_error {
      *self
        .error_counts
        .entry(function_name.to_string())
        .or_insert(0) += 1;
    }

    self.update_slow_functions();
  }

  fn update_slow_functions(&mut self) {
    self.slow_functions.clear();

    for (function_name, &average_time) in &self.average_times {
      let call_count = self.call_counts[function_name];
      let severity = self.determine_performance_severity(average_time, call_count);

      if severity != PerformanceSeverity::Low {
        self.slow_functions.push(SlowFunction {
          function_name: function_name.clone(),
          average_time,
          call_count,
          severity,
        });
      }
    }

    self
      .slow_functions
      .sort_by(|a, b| match b.severity.cmp(&a.severity) {
        std::cmp::Ordering::Equal => b.average_time.cmp(&a.average_time),
        other => other,
      });
  }

  fn determine_performance_severity(
    &self,
    average_time: std::time::Duration,
    call_count: u64,
  ) -> PerformanceSeverity {
    if average_time > std::time::Duration::from_millis(100) {
      PerformanceSeverity::Critical
    } else if average_time > std::time::Duration::from_millis(50) {
      PerformanceSeverity::High
    } else if average_time > std::time::Duration::from_millis(20) {
      PerformanceSeverity::Medium
    } else {
      PerformanceSeverity::Low
    }
  }

  pub fn get_slow_functions(&self) -> &[SlowFunction] {
    &self.slow_functions
  }

  pub fn get_call_count(&self, function_name: &str) -> u64 {
    self.call_counts.get(function_name).copied().unwrap_or(0)
  }

  pub fn get_average_time(&self, function_name: &str) -> std::time::Duration {
    self
      .average_times
      .get(function_name)
      .copied()
      .unwrap_or(std::time::Duration::ZERO)
  }

  pub fn get_error_rate(&self, function_name: &str) -> f64 {
    let call_count = self.call_counts.get(function_name).copied().unwrap_or(0);
    let error_count = self.error_counts.get(function_name).copied().unwrap_or(0);

    if call_count > 0 {
      error_count as f64 / call_count as f64
    } else {
      0.0
    }
  }

  pub fn clone(&self) -> PerformanceTracker {
    PerformanceTracker {
      call_counts: self.call_counts.clone(),
      total_times: self.total_times.clone(),
      average_times: self.average_times.clone(),
      error_counts: self.error_counts.clone(),
      slow_functions: self.slow_functions.clone(),
    }
  }
}

pub fn create_core_functions(lua: &Lua) -> Result<Vec<LuaFunction>> {
  let mut functions = Vec::new();

  functions.push(LuaFunction {
    name: "print".to_string(),
    function: lua.create_function(|lua, args: Vec<Value>| {
      for arg in args {
        print!("{}", lua_value_to_string(&arg)?);
      }
      println!();
      Ok(())
    })?,
    signature: FunctionSignature {
      parameters: vec![ParameterInfo {
        name: "args".to_string(),
        parameter_type: ParameterType::Any,
        optional: false,
        default_value: None,
        description: "Values to print".to_string(),
        validation: None,
      }],
      return_type: ReturnType::Void,
      variadic: true,
      min_args: 0,
      max_args: None,
    },
    category: FunctionCategory::Core,
    description: "Print values to standard output".to_string(),
    examples: vec![
      "print(\"Hello, World!\")".to_string(),
      "print(1, 2, 3)".to_string(),
    ],
    parameters: vec![],
    return_type: ReturnType::Void,
    side_effects: true,
    thread_safe: false,
    deprecated: false,
    version: "1.0".to_string(),
  });

  functions.push(LuaFunction {
    name: "type".to_string(),
    function: lua.create_function(|lua, value: Value| {
      let type_name = match value {
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Boolean(_) => "boolean",
        Value::Table(_) => "table",
        Value::Function(_) => "function",
        Value::Error(_) => "error",
        Value::UserData(_) => "userdata",
        Value::Nil => "nil",
      };
      Ok(type_name.to_string())
    })?,
    signature: FunctionSignature {
      parameters: vec![ParameterInfo {
        name: "value".to_string(),
        parameter_type: ParameterType::Any,
        optional: false,
        default_value: None,
        description: "Value to check type of".to_string(),
        validation: None,
      }],
      return_type: ReturnType::Single(ParameterType::String),
      variadic: false,
      min_args: 1,
      max_args: Some(1),
    },
    category: FunctionCategory::Core,
    description: "Get the type of a value".to_string(),
    examples: vec!["type(\"hello\")".to_string(), "type(42)".to_string()],
    parameters: vec![],
    return_type: ReturnType::Single(ParameterType::String),
    side_effects: false,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  });

  functions.push(LuaFunction {
    name: "assert".to_string(),
    function: lua.create_function(|lua, condition: bool, message: Option<String>| {
      if !condition {
        let error_msg = message.unwrap_or_else(|| "assertion failed".to_string());
        return Err(LuaError::RuntimeError(error_msg));
      }
      Ok(())
    })?,
    signature: FunctionSignature {
      parameters: vec![
        ParameterInfo {
          name: "condition".to_string(),
          parameter_type: ParameterType::Boolean,
          optional: false,
          default_value: None,
          description: "Condition to assert".to_string(),
          validation: None,
        },
        ParameterInfo {
          name: "message".to_string(),
          parameter_type: ParameterType::String,
          optional: true,
          default_value: None,
          description: "Error message if assertion fails".to_string(),
          validation: None,
        },
      ],
      return_type: ReturnType::Void,
      variadic: false,
      min_args: 1,
      max_args: Some(2),
    },
    category: FunctionCategory::Core,
    description: "Assert that a condition is true".to_string(),
    examples: vec![
      "assert(x > 0, \"x must be positive\")".to_string(),
      "assert(condition)".to_string(),
    ],
    parameters: vec![],
    return_type: ReturnType::Void,
    side_effects: true,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  });

  Ok(functions)
}

pub fn create_math_functions(lua: &Lua) -> Result<Vec<LuaFunction>> {
  let mut functions = Vec::new();

  functions.push(LuaFunction {
    name: "abs".to_string(),
    function: lua.create_function(|lua, x: f64| Ok(x.abs()))?,
    signature: FunctionSignature {
      parameters: vec![ParameterInfo {
        name: "x".to_string(),
        parameter_type: ParameterType::Number,
        optional: false,
        default_value: None,
        description: "Number to get absolute value of".to_string(),
        validation: None,
      }],
      return_type: ReturnType::Single(ParameterType::Number),
      variadic: false,
      min_args: 1,
      max_args: Some(1),
    },
    category: FunctionCategory::Math,
    description: "Get the absolute value of a number".to_string(),
    examples: vec!["abs(-5)".to_string(), "abs(3.14)".to_string()],
    parameters: vec![],
    return_type: ReturnType::Single(ParameterType::Number),
    side_effects: false,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  });

  functions.push(LuaFunction {
    name: "min".to_string(),
    function: lua.create_function(|lua, values: Vec<f64>| {
      Ok(values.iter().fold(f64::INFINITY, |a, &b| a.min(*b)))
    })?,
    signature: FunctionSignature {
      parameters: vec![ParameterInfo {
        name: "values".to_string(),
        parameter_type: ParameterType::Number,
        optional: false,
        default_value: None,
        description: "Numbers to find minimum of".to_string(),
        validation: None,
      }],
      return_type: ReturnType::Single(ParameterType::Number),
      variadic: true,
      min_args: 1,
      max_args: None,
    },
    category: FunctionCategory::Math,
    description: "Find the minimum value among numbers".to_string(),
    examples: vec!["min(1, 2, 3)".to_string(), "min(-5, -2, -10)".to_string()],
    parameters: vec![],
    return_type: ReturnType::Single(ParameterType::Number),
    side_effects: false,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  });

  functions.push(LuaFunction {
    name: "max".to_string(),
    function: lua.create_function(|lua, values: Vec<f64>| {
      Ok(values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(*b)))
    })?,
    signature: FunctionSignature {
      parameters: vec![ParameterInfo {
        name: "values".to_string(),
        parameter_type: ParameterType::Number,
        optional: false,
        default_value: None,
        description: "Numbers to find maximum of".to_string(),
        validation: None,
      }],
      return_type: ReturnType::Single(ParameterType::Number),
      variadic: true,
      min_args: 1,
      max_args: None,
    },
    category: FunctionCategory::Math,
    description: "Find the maximum value among numbers".to_string(),
    examples: vec!["max(1, 2, 3)".to_string(), "max(5, 2, 10)".to_string()],
    parameters: vec![],
    return_type: ReturnType::Single(ParameterType::Number),
    side_effects: false,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  });

  Ok(functions)
}

pub fn create_string_functions(lua: &Lua) -> Result<Vec<LuaFunction>> {
  let mut functions = Vec::new();

  functions.push(LuaFunction {
    name: "len".to_string(),
    function: lua.create_function(|lua, s: String| Ok(s.len()))?,
    signature: FunctionSignature {
      parameters: vec![ParameterInfo {
        name: "s".to_string(),
        parameter_type: ParameterType::String,
        optional: false,
        default_value: None,
        description: "String to get length of".to_string(),
        validation: None,
      }],
      return_type: ReturnType::Single(ParameterType::Number),
      variadic: false,
      min_args: 1,
      max_args: Some(1),
    },
    category: FunctionCategory::String,
    description: "Get the length of a string".to_string(),
    examples: vec!["len(\"hello\")".to_string(), "len(\"\")".to_string()],
    parameters: vec![],
    return_type: ReturnType::Single(ParameterType::Number),
    side_effects: false,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  });

  functions.push(LuaFunction {
    name: "upper".to_string(),
    function: lua.create_function(|lua, s: String| Ok(s.to_uppercase()))?,
    signature: FunctionSignature {
      parameters: vec![ParameterInfo {
        name: "s".to_string(),
        parameter_type: ParameterType::String,
        optional: false,
        default_value: None,
        description: "String to convert to uppercase".to_string(),
        validation: None,
      }],
      return_type: ReturnType::Single(ParameterType::String),
      variadic: false,
      min_args: 1,
      max_args: Some(1),
    },
    category: FunctionCategory::String,
    description: "Convert a string to uppercase".to_string(),
    examples: vec![
      "upper(\"hello\")".to_string(),
      "upper(\"World\")".to_string(),
    ],
    parameters: vec![],
    return_type: ReturnType::Single(ParameterType::String),
    side_effects: false,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  });

  functions.push(LuaFunction {
    name: "lower".to_string(),
    function: lua.create_function(|lua, s: String| Ok(s.to_lowercase()))?,
    signature: FunctionSignature {
      parameters: vec![ParameterInfo {
        name: "s".to_string(),
        parameter_type: ParameterType::String,
        optional: false,
        default_value: None,
        description: "String to convert to lowercase".to_string(),
        validation: None,
      }],
      return_type: ReturnType::Single(ParameterType::String),
      variadic: false,
      min_args: 1,
      max_args: Some(1),
    },
    category: FunctionCategory::String,
    description: "Convert a string to lowercase".to_string(),
    examples: vec![
      "lower(\"HELLO\")".to_string(),
      "lower(\"World\")".to_string(),
    ],
    parameters: vec![],
    return_type: ReturnType::Single(ParameterType::String),
    side_effects: false,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  });

  Ok(functions)
}

fn lua_value_to_string(value: &Value) -> Result<String> {
  match value {
    Value::String(s) => Ok(s.clone()),
    Value::Number(n) => Ok(n.to_string()),
    Value::Boolean(b) => Ok(b.to_string()),
    Value::Table(_) => Ok("table".to_string()),
    Value::Function(_) => Ok("function".to_string()),
    Value::Error(_) => Ok("error".to_string()),
    Value::UserData(_) => Ok("userdata".to_string()),
    Value::Nil => Ok("nil".to_string()),
  }
}

pub fn create_lua_function_registry() -> LuaFunctionRegistry {
  LuaFunctionRegistry::new()
}

pub fn create_function_executor(lua: Lua) -> FunctionExecutor {
  FunctionExecutor::new(lua)
}

pub fn create_lua_function(
  name: String,
  function: Function,
  signature: FunctionSignature,
  category: FunctionCategory,
  description: String,
) -> LuaFunction {
  LuaFunction {
    name,
    function,
    signature,
    category,
    description,
    examples: Vec::new(),
    parameters: signature.parameters.clone(),
    return_type: signature.return_type.clone(),
    side_effects: false,
    thread_safe: true,
    deprecated: false,
    version: "1.0".to_string(),
  }
}
