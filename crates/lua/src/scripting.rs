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
pub struct ScriptExecutor {
  lua: Lua,
  execution_context: Arc<RwLock<ExecutionContext>>,
  security_context: Arc<RwLock<SecurityContext>>,
  performance_monitor: Arc<RwLock<PerformanceMonitor>>,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
  pub script_id: Uuid,
  pub execution_id: Uuid,
  pub start_time: DateTime<Utc>,
  pub end_time: Option<DateTime<Utc>>,
  pub status: ExecutionStatus,
  pub error: Option<String>,
  pub execution_time: std::time::Duration,
  pub memory_usage: usize,
  pub variables_used: HashMap<String, ScriptValue>,
  pub functions_called: Vec<String>,
  pub modules_loaded: Vec<String>,
  pub sandbox_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
  Pending,
  Running,
  Completed,
  Failed,
  Timeout,
  Cancelled,
  SecurityViolation,
}

#[derive(Debug, Clone)]
pub struct SecurityContext {
  pub sandbox_enabled: bool,
  pub allowed_modules: HashSet<String>,
  pub blocked_functions: HashSet<String>,
  pub allowed_paths: Vec<String>,
  pub blocked_paths: Vec<String>,
  pub max_execution_time: std::time::Duration,
  pub max_memory_usage: usize,
  pub current_memory_usage: usize,
  pub security_violations: Vec<SecurityViolation>,
}

#[derive(Debug, Clone)]
pub struct SecurityViolation {
  pub violation_type: SecurityViolationType,
  pub description: String,
  pub timestamp: DateTime<Utc>,
  pub stack_trace: Vec<String>,
  pub severity: SecuritySeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityViolationType {
  BlockedFunction,
  UnauthorizedModule,
  PathAccessViolation,
  MemoryLimitExceeded,
  TimeLimitExceeded,
  DangerousOperation,
  SystemCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecuritySeverity {
  Low,
  Medium,
  High,
  Critical,
}

#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
  pub execution_times: Vec<std::time::Duration>,
  pub memory_usage_samples: Vec<usize>,
  pub function_call_counts: HashMap<String, u64>,
  pub module_load_times: HashMap<String, std::time::Duration>,
  pub total_executions: u64,
  pub average_execution_time: std::time::Duration,
  pub peak_memory_usage: usize,
  pub bottlenecks: Vec<PerformanceBottleneck>,
}

#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
  pub function_name: String,
  pub execution_time: std::time::Duration,
  pub call_count: u64,
  pub average_time: std::time::Duration,
  pub severity: BottleneckSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottleneckSeverity {
  Low,
  Medium,
  High,
  Critical,
}

#[derive(Debug, Clone)]
pub struct ScriptDebugger {
  pub enabled: bool,
  pub breakpoints: HashMap<String, Breakpoint>,
  pub step_mode: bool,
  pub current_line: Option<u32>,
  pub call_stack: Vec<CallStackFrame>,
  pub variables: HashMap<String, ScriptValue>,
  pub debug_output: Vec<DebugMessage>,
}

#[derive(Debug, Clone)]
pub struct Breakpoint {
  pub id: Uuid,
  pub file: String,
  pub line: u32,
  pub condition: Option<String>,
  pub hit_count: u32,
  pub enabled: bool,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CallStackFrame {
  pub function_name: String,
  pub file: String,
  pub line: u32,
  pub locals: HashMap<String, ScriptValue>,
}

#[derive(Debug, Clone)]
pub struct DebugMessage {
  pub timestamp: DateTime<Utc>,
  pub level: DebugLevel,
  pub message: String,
  pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLevel {
  Trace,
  Debug,
  Info,
  Warning,
  Error,
}

#[derive(Debug, Clone)]
pub struct ScriptProfiler {
  pub enabled: bool,
  pub sampling_rate: f64,
  pub profile_data: HashMap<String, FunctionProfile>,
  pub global_profile: GlobalProfile,
  pub start_time: DateTime<Utc>,
  pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct FunctionProfile {
  pub function_name: String,
  pub call_count: u64,
  pub total_time: std::time::Duration,
  pub average_time: std::time::Duration,
  pub min_time: std::time::Duration,
  pub max_time: std::time::Duration,
  pub memory_usage: Vec<usize>,
  pub call_stack_depth: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct GlobalProfile {
  pub total_functions: u64,
  pub total_execution_time: std::time::Duration,
  pub average_function_time: std::time::Duration,
  pub memory_efficiency: f64,
  pub execution_efficiency: f64,
}

#[derive(Debug, Clone)]
pub struct ScriptValidator {
  pub validation_rules: Vec<ValidationRule>,
  pub syntax_checker: SyntaxChecker,
  pub security_checker: SecurityChecker,
  pub performance_checker: PerformanceChecker,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
  pub rule_name: String,
  pub rule_type: ValidationRuleType,
  pub description: String,
  pub severity: ValidationSeverity,
  pub check_function: Box<dyn Fn(&str) -> Result<ValidationResult> + Send + Sync>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationRuleType {
  Syntax,
  Security,
  Performance,
  Style,
  Logic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
  Info,
  Warning,
  Error,
  Critical,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
  pub passed: bool,
  pub message: String,
  pub suggestions: Vec<String>,
  pub line_number: Option<u32>,
  pub column_number: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SyntaxChecker {
  pub lua_parser: LuaParser,
  pub error_recovery: bool,
  pub strict_mode: bool,
}

#[derive(Debug, Clone)]
pub struct LuaParser {
  pub tokens: Vec<Token>,
  pub ast: Option<AST>,
  pub parse_errors: Vec<ParseError>,
}

#[derive(Debug, Clone)]
pub struct Token {
  pub token_type: TokenType,
  pub value: String,
  pub line: u32,
  pub column: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
  Identifier,
  Keyword,
  String,
  Number,
  Operator,
  Punctuation,
  Comment,
  Whitespace,
  EOF,
}

#[derive(Debug, Clone)]
pub struct AST {
  pub node_type: ASTNodeType,
  pub children: Vec<AST>,
  pub value: Option<String>,
  pub line: u32,
  pub column: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASTNodeType {
  Program,
  Block,
  Function,
  Call,
  Assignment,
  Return,
  If,
  While,
  For,
  Repeat,
  Local,
  Global,
  Table,
  BinaryOp,
  UnaryOp,
  Literal,
  Variable,
}

#[derive(Debug, Clone)]
pub struct ParseError {
  pub error_type: ParseErrorType,
  pub message: String,
  pub line: u32,
  pub column: u32,
  pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorType {
  UnexpectedToken,
  ExpectedToken,
  InvalidSyntax,
  UnmatchedParentheses,
  InvalidExpression,
  InvalidStatement,
  InvalidBlock,
  InvalidFunction,
  InvalidAssignment,
}

#[derive(Debug, Clone)]
pub struct SecurityChecker {
  pub dangerous_patterns: Vec<String>,
  pub blocked_modules: HashSet<String>,
  pub allowed_functions: HashSet<String>,
  pub max_complexity: u32,
  pub max_nesting_depth: u32,
}

#[derive(Debug, Clone)]
pub struct PerformanceChecker {
  pub max_function_length: u32,
  pub max_loop_iterations: u32,
  pub max_recursion_depth: u32,
  pub max_variable_count: u32,
  pub max_table_size: u32,
}

impl ScriptExecutor {
  pub fn new() -> Result<Self> {
    let lua = Lua::new();

    Ok(Self {
      lua,
      execution_context: Arc::new(RwLock::new(ExecutionContext::new())),
      security_context: Arc::new(RwLock::new(SecurityContext::new())),
      performance_monitor: Arc::new(RwLock::new(PerformanceMonitor::new())),
    })
  }

  pub fn with_security_context(mut self, security_context: SecurityContext) -> Self {
    self.security_context = Arc::new(RwLock::new(security_context));
    self
  }

  pub fn with_performance_monitor(mut self, performance_monitor: PerformanceMonitor) -> Self {
    self.performance_monitor = Arc::new(RwLock::new(performance_monitor));
    self
  }

  pub fn execute(
    &mut self,
    script_source: &str,
    parameters: HashMap<String, ScriptValue>,
  ) -> Result<ScriptValue> {
    let execution_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();

    {
      let mut context = self.execution_context.write();
      context.execution_id = execution_id;
      context.start_time = Utc::now();
      context.end_time = None;
      context.status = ExecutionStatus::Running;
      context.error = None;
      context.execution_time = std::time::Duration::ZERO;
      context.memory_usage = 0;
      context.variables_used.clear();
      context.functions_called.clear();
      context.modules_loaded.clear();
    }

    if self.security_context.read().sandbox_enabled {
      self.setup_sandbox(&execution_id)?;
    }

    self.setup_parameters(parameters)?;

    let result = self.execute_script(script_source, &execution_id);

    let execution_time = start_time.elapsed();
    {
      let mut context = self.execution_context.write();
      context.end_time = Some(Utc::now());
      context.execution_time = execution_time;

      match &result {
        Ok(_) => {
          context.status = ExecutionStatus::Completed;
        }
        Err(e) => {
          context.status = ExecutionStatus::Failed;
          context.error = Some(e.to_string());
        }
      }
    }

    self
      .performance_monitor
      .write()
      .record_execution(execution_time);

    result
  }

  pub fn execute_async(
    &mut self,
    script_source: String,
    parameters: HashMap<String, ScriptValue>,
  ) -> Result<tokio::task::JoinHandle<Result<ScriptValue>>> {
    let execution_id = Uuid::new_v4();
    let max_execution_time = self.security_context.read().max_execution_time;

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

  fn setup_sandbox(&mut self, execution_id: &Uuid) -> Result<()> {
    let lua = &self.lua;
    let globals = lua.globals();

    let sandbox = lua.create_table()?;

    let safe_globals = vec![
      "string", "table", "math", "utf8", "pairs", "ipairs", "type", "tostring", "tonumber",
    ];

    for global_name in safe_globals {
      if let Ok(value) = globals.get::<_, Value>(global_name) {
        sandbox.set(global_name, value)?;
      }
    }

    globals.set("_G", sandbox.clone())?;
    globals.set("_ENV", sandbox)?;

    {
      let mut context = self.execution_context.write();
      context.sandbox_id = Some(*execution_id);
    }

    Ok(())
  }

  fn setup_parameters(&mut self, parameters: HashMap<String, ScriptValue>) -> Result<()> {
    let globals = self.lua.globals();

    for (name, value) in parameters {
      let lua_value = self.script_value_to_lua_value(&value)?;
      globals.set(name, lua_value)?;

      let mut context = self.execution_context.write();
      context.variables_used.insert(name, value);
    }

    Ok(())
  }

  fn execute_script(&mut self, script_source: &str, execution_id: &Uuid) -> Result<ScriptValue> {
    let lua = &self.lua;

    self.check_security(script_source)?;

    let chunk = lua
      .load(script_source)
      .map_err(|e| EllasticError::ScriptError(format!("Failed to load script: {}", e)))?;

    let result = chunk
      .exec()
      .map_err(|e| EllasticError::ScriptError(format!("Script execution failed: {}", e)))?;

    self.lua_value_to_script_value(&result)
  }

  fn check_security(&mut self, script_source: &str) -> Result<()> {
    let security_context = self.security_context.read();

    for blocked_function in &security_context.blocked_functions {
      if script_source.contains(blocked_function) {
        let violation = SecurityViolation {
          violation_type: SecurityViolationType::BlockedFunction,
          description: format!("Script contains blocked function: {}", blocked_function),
          timestamp: Utc::now(),
          stack_trace: Vec::new(),
          severity: SecuritySeverity::High,
        };

        let mut context = self.execution_context.write();
        context.status = ExecutionStatus::SecurityViolation;
        context.error = Some(violation.description.clone());

        return Err(EllasticError::SecurityError(violation.description));
      }
    }

    let dangerous_patterns = vec![
      "dofile",
      "loadfile",
      "os.execute",
      "os.exit",
      "debug.getregistry",
      "package.loadlib",
      "package.searchpath",
      "collectgarbage",
    ];

    for pattern in &dangerous_patterns {
      if script_source.contains(pattern) {
        let violation = SecurityViolation {
          violation_type: SecurityViolationType::DangerousOperation,
          description: format!("Script contains dangerous pattern: {}", pattern),
          timestamp: Utc::now(),
          stack_trace: Vec::new(),
          severity: SecuritySeverity::Critical,
        };

        let mut context = self.execution_context.write();
        context.status = ExecutionStatus::SecurityViolation;
        context.error = Some(violation.description.clone());

        return Err(EllasticError::SecurityError(violation.description));
      }
    }

    if script_source.len() > security_context.max_memory_usage {
      let violation = SecurityViolation {
        violation_type: SecurityViolationType::MemoryLimitExceeded,
        description: format!(
          "Script size exceeds memory limit: {} > {}",
          script_source.len(),
          security_context.max_memory_usage
        ),
        timestamp: Utc::now(),
        stack_trace: Vec::new(),
        severity: SecuritySeverity::Medium,
      };

      let mut context = self.execution_context.write();
      context.status = ExecutionStatus::SecurityViolation;
      context.error = Some(violation.description.clone());

      return Err(EllasticError::SecurityError(violation.description));
    }

    Ok(())
  }

  fn script_value_to_lua_value(&self, value: &ScriptValue) -> Result<Value> {
    match value {
      ScriptValue::String(s) => Ok(Value::String(s.clone())),
      ScriptValue::Number(n) => Ok(Value::Number(*n)),
      ScriptValue::Boolean(b) => Ok(Value::Boolean(*b)),
      ScriptValue::Table(table) => {
        let lua_table = self.lua.create_table()?;
        for (key, value) in table {
          let lua_value = self.script_value_to_lua_value(value)?;
          lua_table.set(key, lua_value)?;
        }
        Ok(Value::Table(lua_table))
      }
      ScriptValue::Function(_) => Err(EllasticError::UnsupportedOperation(
        "Functions cannot be converted to Lua values".to_string(),
      )),
      ScriptValue::UserData(data) => Ok(Value::UserData(self.lua.create_userdata(data.clone())?)),
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

  pub fn get_execution_context(&self) -> Arc<RwLock<ExecutionContext>> {
    self.execution_context.clone()
  }

  pub fn get_security_context(&self) -> Arc<RwLock<SecurityContext>> {
    self.security_context.clone()
  }

  pub fn get_performance_monitor(&self) -> Arc<RwLock<PerformanceMonitor>> {
    self.performance_monitor.clone()
  }

  pub fn clone(&self) -> ScriptExecutor {
    Self::new().unwrap_or_else(|_| Self::new().unwrap())
  }
}

impl ExecutionContext {
  pub fn new() -> Self {
    Self {
      script_id: Uuid::new_v4(),
      execution_id: Uuid::new_v4(),
      start_time: Utc::now(),
      end_time: None,
      status: ExecutionStatus::Pending,
      error: None,
      execution_time: std::time::Duration::ZERO,
      memory_usage: 0,
      variables_used: HashMap::new(),
      functions_called: Vec::new(),
      modules_loaded: Vec::new(),
      sandbox_id: None,
    }
  }

  pub fn clone(&self) -> ExecutionContext {
    ExecutionContext {
      script_id: self.script_id,
      execution_id: self.execution_id,
      start_time: self.start_time,
      end_time: self.end_time,
      status: self.status,
      error: self.error.clone(),
      execution_time: self.execution_time,
      memory_usage: self.memory_usage,
      variables_used: self.variables_used.clone(),
      functions_called: self.functions_called.clone(),
      modules_loaded: self.modules_loaded.clone(),
      sandbox_id: self.sandbox_id,
    }
  }
}

impl SecurityContext {
  pub fn new() -> Self {
    Self {
      sandbox_enabled: true,
      allowed_modules: HashSet::new(),
      blocked_functions: HashSet::new(),
      allowed_paths: Vec::new(),
      blocked_paths: Vec::new(),
      max_execution_time: std::time::Duration::from_secs(30),
      max_memory_usage: 100 * 1024 * 1024,
      current_memory_usage: 0,
      security_violations: Vec::new(),
    }
  }

  pub fn with_sandbox_enabled(mut self, enabled: bool) -> Self {
    self.sandbox_enabled = enabled;
    self
  }

  pub fn with_allowed_modules(mut self, modules: Vec<String>) -> Self {
    self.allowed_modules.extend(modules);
    self
  }

  pub fn with_blocked_functions(mut self, functions: Vec<String>) -> Self {
    self.blocked_functions.extend(functions);
    self
  }

  pub fn with_max_execution_time(mut self, duration: std::time::Duration) -> Self {
    self.max_execution_time = duration;
    self
  }

  pub fn with_max_memory_usage(mut self, bytes: usize) -> Self {
    self.max_memory_usage = bytes;
    self
  }

  pub fn record_violation(&mut self, violation: SecurityViolation) {
    self.security_violations.push(violation);
  }

  pub fn clone(&self) -> SecurityContext {
    SecurityContext {
      sandbox_enabled: self.sandbox_enabled,
      allowed_modules: self.allowed_modules.clone(),
      blocked_functions: self.blocked_functions.clone(),
      allowed_paths: self.allowed_paths.clone(),
      blocked_paths: self.blocked_paths.clone(),
      max_execution_time: self.max_execution_time,
      max_memory_usage: self.max_memory_usage,
      current_memory_usage: self.current_memory_usage,
      security_violations: self.security_violations.clone(),
    }
  }
}

impl PerformanceMonitor {
  pub fn new() -> Self {
    Self {
      execution_times: Vec::new(),
      memory_usage_samples: Vec::new(),
      function_call_counts: HashMap::new(),
      module_load_times: HashMap::new(),
      total_executions: 0,
      average_execution_time: std::time::Duration::ZERO,
      peak_memory_usage: 0,
      bottlenecks: Vec::new(),
    }
  }

  pub fn record_execution(&mut self, execution_time: std::time::Duration) {
    self.execution_times.push(execution_time);
    self.total_executions += 1;

    if self.execution_times.len() > 100 {
      self.execution_times.drain(0..50);
    }

    self.update_average_execution_time();
    self.identify_bottlenecks();
  }

  pub fn record_function_call(&mut self, function_name: &str) {
    *self
      .function_call_counts
      .entry(function_name.to_string())
      .or_insert(0) += 1;
  }

  pub fn record_memory_usage(&mut self, memory_usage: usize) {
    self.memory_usage_samples.push(memory_usage);
    self.peak_memory_usage = self.peak_memory_usage.max(memory_usage);

    if self.memory_usage_samples.len() > 100 {
      self.memory_usage_samples.drain(0..50);
    }
  }

  pub fn record_module_load_time(&mut self, module_name: &str, load_time: std::time::Duration) {
    self
      .module_load_times
      .insert(module_name.to_string(), load_time);
  }

  fn update_average_execution_time(&mut self) {
    if self.execution_times.is_empty() {
      self.average_execution_time = std::time::Duration::ZERO;
    } else {
      let total: std::time::Duration = self.execution_times.iter().sum();
      self.average_execution_time = total / self.execution_times.len() as u32;
    }
  }

  fn identify_bottlenecks(&mut self) {
    self.bottlenecks.clear();

    for (function_name, &call_count) in &self.function_call_counts {
      if call_count > 100 {
        let severity = if call_count > 1000 {
          BottleneckSeverity::Critical
        } else if call_count > 500 {
          BottleneckSeverity::High
        } else {
          BottleneckSeverity::Medium
        };

        self.bottlenecks.push(PerformanceBottleneck {
          function_name: function_name.clone(),
          execution_time: std::time::Duration::from_millis(10),
          call_count,
          average_time: std::time::Duration::from_millis(10),
          severity,
        });
      }
    }

    self
      .bottlenecks
      .sort_by(|a, b| match b.severity.cmp(&a.severity) {
        std::cmp::Ordering::Equal => b.call_count.cmp(&a.call_count),
        other => other,
      });
  }

  pub fn clone(&self) -> PerformanceMonitor {
    PerformanceMonitor {
      execution_times: self.execution_times.clone(),
      memory_usage_samples: self.memory_usage_samples.clone(),
      function_call_counts: self.function_call_counts.clone(),
      module_load_times: self.module_load_times.clone(),
      total_executions: self.total_executions,
      average_execution_time: self.average_execution_time,
      peak_memory_usage: self.peak_memory_usage,
      bottlenecks: self.bottlenecks.clone(),
    }
  }
}

impl ScriptDebugger {
  pub fn new() -> Self {
    Self {
      enabled: false,
      breakpoints: HashMap::new(),
      step_mode: false,
      current_line: None,
      call_stack: Vec::new(),
      variables: HashMap::new(),
      debug_output: Vec::new(),
    }
  }

  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  pub fn with_step_mode(mut self, step_mode: bool) -> Self {
    self.step_mode = step_mode;
    self
  }

  pub fn add_breakpoint(&mut self, file: String, line: u32) -> Result<Uuid> {
    let breakpoint = Breakpoint {
      id: Uuid::new_v4(),
      file,
      line,
      condition: None,
      hit_count: 0,
      enabled: true,
      created_at: Utc::now(),
    };

    let id = breakpoint.id;
    self.breakpoints.insert(id.to_string(), breakpoint);
    Ok(id)
  }

  pub fn remove_breakpoint(&mut self, id: &str) -> bool {
    self.breakpoints.remove(id).is_some()
  }

  pub fn should_break(&self, file: &str, line: u32) -> bool {
    if !self.enabled {
      return false;
    }

    for breakpoint in self.breakpoints.values() {
      if breakpoint.enabled && breakpoint.file == file && breakpoint.line == line {
        return true;
      }
    }

    false
  }

  pub fn log_debug(
    &mut self,
    level: DebugLevel,
    message: String,
    context: HashMap<String, String>,
  ) {
    self.debug_output.push(DebugMessage {
      timestamp: Utc::now(),
      level,
      message,
      context,
    });
  }

  pub fn get_debug_output(&self) -> &[DebugMessage] {
    &self.debug_output
  }

  pub fn clear_debug_output(&mut self) {
    self.debug_output.clear();
  }

  pub fn clone(&self) -> ScriptDebugger {
    ScriptDebugger {
      enabled: self.enabled,
      breakpoints: self.breakpoints.clone(),
      step_mode: self.step_mode,
      current_line: self.current_line,
      call_stack: self.call_stack.clone(),
      variables: self.variables.clone(),
      debug_output: self.debug_output.clone(),
    }
  }
}

impl ScriptProfiler {
  pub fn new() -> Self {
    Self {
      enabled: false,
      sampling_rate: 1.0,
      profile_data: HashMap::new(),
      global_profile: GlobalProfile::new(),
      start_time: Utc::now(),
      end_time: None,
    }
  }

  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  pub fn with_sampling_rate(mut self, rate: f64) -> Self {
    self.sampling_rate = rate.clamp(0.0, 1.0);
    self
  }

  pub fn start_profiling(&mut self) {
    self.start_time = Utc::now();
    self.end_time = None;
    self.profile_data.clear();
  }

  pub fn stop_profiling(&mut self) {
    self.end_time = Some(Utc::now());
    self.calculate_global_profile();
  }

  pub fn record_function_call(
    &mut self,
    function_name: &str,
    execution_time: std::time::Duration,
    memory_usage: usize,
    call_depth: u32,
  ) {
    if !self.enabled {
      return;
    }

    let profile = self
      .profile_data
      .entry(function_name.to_string())
      .or_insert_with(|| FunctionProfile {
        function_name: function_name.to_string(),
        call_count: 0,
        total_time: std::time::Duration::ZERO,
        average_time: std::time::Duration::ZERO,
        min_time: std::time::Duration::MAX,
        max_time: std::time::Duration::ZERO,
        memory_usage: Vec::new(),
        call_stack_depth: Vec::new(),
      });

    profile.call_count += 1;
    profile.total_time += execution_time;
    profile.average_time = profile.total_time / profile.call_count as u32;
    profile.min_time = profile.min_time.min(execution_time);
    profile.max_time = profile.max_time.max(execution_time);
    profile.memory_usage.push(memory_usage);
    profile.call_stack_depth.push(call_depth);
  }

  fn calculate_global_profile(&mut self) {
    let total_functions = self.profile_data.len() as u64;
    let total_execution_time: std::time::Duration =
      self.profile_data.values().map(|p| p.total_time).sum();

    self.global_profile.total_functions = total_functions;
    self.global_profile.total_execution_time = total_execution_time;
    self.global_profile.average_function_time = if total_functions > 0 {
      total_execution_time / total_functions as u32
    } else {
      std::time::Duration::ZERO
    };

    let total_memory: usize = self
      .profile_data
      .values()
      .map(|p| p.memory_usage.iter().sum::<usize>())
      .sum();
    let average_memory = if total_functions > 0 {
      total_memory / total_functions as usize
    } else {
      0
    };
    self.global_profile.memory_efficiency = if average_memory > 0 {
      1.0 / average_memory as f64
    } else {
      1.0
    };

    let max_time = self
      .profile_data
      .values()
      .map(|p| p.max_time)
      .max()
      .unwrap_or(std::time::Duration::ZERO);
    self.global_profile.execution_efficiency = if max_time > std::time::Duration::ZERO {
      self.global_profile.average_function_time.as_secs_f64() / max_time.as_secs_f64()
    } else {
      1.0
    };
  }

  pub fn get_profile_data(&self) -> &HashMap<String, FunctionProfile> {
    &self.profile_data
  }

  pub fn get_global_profile(&self) -> &GlobalProfile {
    &self.global_profile
  }

  pub fn clone(&self) -> ScriptProfiler {
    ScriptProfiler {
      enabled: self.enabled,
      sampling_rate: self.sampling_rate,
      profile_data: self.profile_data.clone(),
      global_profile: self.global_profile.clone(),
      start_time: self.start_time,
      end_time: self.end_time,
    }
  }
}

impl GlobalProfile {
  pub fn new() -> Self {
    Self {
      total_functions: 0,
      total_execution_time: std::time::Duration::ZERO,
      average_function_time: std::time::Duration::ZERO,
      memory_efficiency: 0.0,
      execution_efficiency: 0.0,
    }
  }

  pub fn clone(&self) -> GlobalProfile {
    GlobalProfile {
      total_functions: self.total_functions,
      total_execution_time: self.total_execution_time,
      average_function_time: self.average_function_time,
      memory_efficiency: self.memory_efficiency,
      execution_efficiency: self.execution_efficiency,
    }
  }
}

impl FunctionProfile {
  pub fn clone(&self) -> FunctionProfile {
    FunctionProfile {
      function_name: self.function_name.clone(),
      call_count: self.call_count,
      total_time: self.total_time,
      average_time: self.average_time,
      min_time: self.min_time,
      max_time: self.max_time,
      memory_usage: self.memory_usage.clone(),
      call_stack_depth: self.call_stack_depth.clone(),
    }
  }
}

pub fn create_script_executor() -> Result<ScriptExecutor> {
  ScriptExecutor::new()
}

pub fn create_security_context() -> SecurityContext {
  SecurityContext::new()
}

pub fn create_performance_monitor() -> PerformanceMonitor {
  PerformanceMonitor::new()
}

pub fn create_script_debugger() -> ScriptDebugger {
  ScriptDebugger::new()
}

pub fn create_script_profiler() -> ScriptProfiler {
  ScriptProfiler::new()
}
