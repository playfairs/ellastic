use crate::execution::{
  ExecutionContext,
  ExecutionStatus,
  PipelineExecutor,
};
use crate::graph::{
  EdgeType,
  PipelineEdge,
  PipelineGraph,
  PipelineNode,
  PipelineNodeType,
};
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
use ellastic_utils::create_random_generator;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::{
  HashMap,
  HashSet,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PipelineDebugger {
  enabled: bool,
  debug_info: Vec<DebugInfo>,
  breakpoints: HashSet<Uuid>,
  step_mode: bool,
  current_step: usize,
  debug_level: DebugLevel,
  output_format: DebugOutputFormat,
}

#[derive(Debug, Clone)]
pub struct DebugInfo {
  pub timestamp: DateTime<Utc>,
  pub node_id: Uuid,
  pub node_name: String,
  pub debug_type: DebugType,
  pub message: String,
  pub data: HashMap<String, String>,
  pub severity: DebugSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugType {
  NodeStart,
  NodeComplete,
  NodeError,
  NodeWarning,
  DataFlow,
  CacheHit,
  CacheMiss,
  Performance,
  Memory,
  Validation,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSeverity {
  Trace,
  Debug,
  Info,
  Warning,
  Error,
  Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLevel {
  None,
  Basic,
  Standard,
  Detailed,
  Verbose,
  Trace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugOutputFormat {
  Console,
  Log,
  File,
  Structured,
}

#[derive(Debug, Clone)]
pub struct DebugSession {
  pub id: Uuid,
  pub start_time: DateTime<Utc>,
  pub end_time: Option<DateTime<Utc>>,
  pub status: DebugSessionStatus,
  pub breakpoints: HashSet<Uuid>,
  pub step_count: usize,
  pub debug_info: Vec<DebugInfo>,
  pub session_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSessionStatus {
  Active,
  Paused,
  Completed,
  Aborted,
}

#[derive(Debug, Clone)]
pub struct DebugBreakpoint {
  pub id: Uuid,
  pub node_id: Uuid,
  pub condition: Option<String>,
  pub action: BreakpointAction,
  pub hit_count: u32,
  pub enabled: bool,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointAction {
  Pause,
  Log,
  Inspect,
  Modify,
}

#[derive(Debug, Clone)]
pub struct DebugTrace {
  pub id: Uuid,
  pub trace_type: TraceType,
  pub timestamp: DateTime<Utc>,
  pub data: HashMap<String, String>,
  pub stack_trace: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceType {
  Execution,
  Data,
  Memory,
  Performance,
  Custom,
}

#[derive(Debug, Clone)]
pub struct DebugProfiler {
  pub id: Uuid,
  pub name: String,
  pub enabled: bool,
  pub profile_data: HashMap<Uuid, NodeProfile>,
  pub global_profile: GlobalProfile,
  pub sampling_rate: f64,
  pub max_samples: usize,
}

#[derive(Debug, Clone)]
pub struct NodeProfile {
  pub node_id: Uuid,
  pub node_name: String,
  pub execution_count: u32,
  pub total_time: std::time::Duration,
  pub min_time: std::time::Duration,
  pub max_time: std::time::Duration,
  pub average_time: std::time::Duration,
  pub memory_usage: Vec<usize>,
  pub cache_hits: u32,
  pub cache_misses: u32,
  pub error_count: u32,
  pub last_execution: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GlobalProfile {
  pub total_executions: u32,
  pub total_time: std::time::Duration,
  pub average_time: std::time::Duration,
  pub memory_peak: usize,
  pub cache_hit_rate: f64,
  pub error_rate: f64,
  pub throughput: f64,
}

#[derive(Debug, Clone)]
pub struct DebugInspector {
  pub id: Uuid,
  pub inspection_points: HashMap<Uuid, InspectionPoint>,
  pub inspection_history: Vec<InspectionRecord>,
}

#[derive(Debug, Clone)]
pub struct InspectionPoint {
  pub id: Uuid,
  pub node_id: Uuid,
  pub name: String,
  pub inspection_type: InspectionType,
  pub enabled: bool,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectionType {
  Data,
  State,
  Parameters,
  Performance,
  Memory,
  Custom,
}

#[derive(Debug, Clone)]
pub struct InspectionRecord {
  pub id: Uuid,
  pub inspection_point_id: Uuid,
  pub timestamp: DateTime<Utc>,
  pub data: HashMap<String, String>,
  pub snapshot: Option<String>,
}

impl PipelineDebugger {
  pub fn new() -> Self {
    Self {
      enabled: false,
      debug_info: Vec::new(),
      breakpoints: HashSet::new(),
      step_mode: false,
      current_step: 0,
      debug_level: DebugLevel::Standard,
      output_format: DebugOutputFormat::Console,
    }
  }

  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  pub fn with_debug_level(mut self, level: DebugLevel) -> Self {
    self.debug_level = level;
    self
  }

  pub fn with_output_format(mut self, format: DebugOutputFormat) -> Self {
    self.output_format = format;
    self
  }

  pub fn with_step_mode(mut self, step_mode: bool) -> Self {
    self.step_mode = step_mode;
    self
  }

  pub fn enable(&mut self) {
    self.enabled = true;
  }

  pub fn disable(&mut self) {
    self.enabled = false;
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn add_breakpoint(&mut self, node_id: Uuid) -> Result<Uuid> {
    let breakpoint_id = Uuid::new_v4();
    self.breakpoints.insert(node_id);

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: "Debugger".to_string(),
      debug_type: DebugType::Custom,
      message: format!("Added breakpoint for node {}", node_id),
      data: HashMap::new(),
      severity: DebugSeverity::Debug,
    });

    Ok(breakpoint_id)
  }

  pub fn remove_breakpoint(&mut self, node_id: Uuid) -> bool {
    let removed = self.breakpoints.remove(&node_id);

    if removed {
      self.log_debug(DebugInfo {
        timestamp: Utc::now(),
        node_id,
        node_name: "Debugger".to_string(),
        debug_type: DebugType::Custom,
        message: format!("Removed breakpoint for node {}", node_id),
        data: HashMap::new(),
        severity: DebugSeverity::Debug,
      });
    }

    removed
  }

  pub fn has_breakpoint(&self, node_id: Uuid) -> bool {
    self.breakpoints.contains(&node_id)
  }

  pub fn list_breakpoints(&self) -> Vec<Uuid> {
    self.breakpoints.iter().copied().collect()
  }

  pub fn clear_breakpoints(&mut self) {
    self.breakpoints.clear();

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id: Uuid::new_v4(),
      node_name: "Debugger".to_string(),
      debug_type: DebugType::Custom,
      message: "Cleared all breakpoints".to_string(),
      data: HashMap::new(),
      severity: DebugSeverity::Debug,
    });
  }

  pub fn step(&mut self) -> bool {
    if self.step_mode {
      self.current_step += 1;

      self.log_debug(DebugInfo {
        timestamp: Utc::now(),
        node_id: Uuid::new_v4(),
        node_name: "Debugger".to_string(),
        debug_type: DebugType::Custom,
        message: format!("Step {} taken", self.current_step),
        data: HashMap::new(),
        severity: DebugSeverity::Trace,
      });

      true
    } else {
      false
    }
  }

  pub fn should_break(&self, node_id: Uuid) -> bool {
    self.breakpoints.contains(&node_id) && self.step_mode
  }

  pub fn log_debug(&mut self, debug_info: DebugInfo) {
    if self.enabled && self.should_log(&debug_info.debug_type, &debug_info.severity) {
      self.debug_info.push(debug_info);
      self.output_debug_info(&self.debug_info.last().unwrap());
    }
  }

  fn should_log(&self, debug_type: &DebugType, severity: &DebugSeverity) -> bool {
    match self.debug_level {
      DebugLevel::None => false,
      DebugLevel::Basic => matches!(
        severity,
        DebugSeverity::Warning | DebugSeverity::Error | DebugSeverity::Critical
      ),
      DebugLevel::Standard => matches!(
        severity,
        DebugSeverity::Info
          | DebugSeverity::Warning
          | DebugSeverity::Error
          | DebugSeverity::Critical
      ),
      DebugLevel::Detailed => matches!(
        severity,
        DebugSeverity::Debug
          | DebugSeverity::Info
          | DebugSeverity::Warning
          | DebugSeverity::Error
          | DebugSeverity::Critical
      ),
      DebugLevel::Verbose => matches!(
        severity,
        DebugSeverity::Trace
          | DebugSeverity::Debug
          | DebugSeverity::Info
          | DebugSeverity::Warning
          | DebugSeverity::Error
          | DebugSeverity::Critical
      ),
      DebugLevel::Trace => true,
    }
  }

  fn output_debug_info(&self, debug_info: &DebugInfo) {
    match self.output_format {
      DebugOutputFormat::Console => {
        self.output_to_console(debug_info);
      }
      DebugOutputFormat::Log => {
        self.output_to_log(debug_info);
      }
      DebugOutputFormat::File => {
        self.output_to_file(debug_info);
      }
      DebugOutputFormat::Structured => {
        self.output_structured(debug_info);
      }
    }
  }

  fn output_to_console(&self, debug_info: &DebugInfo) {
    let timestamp = debug_info.timestamp.format("%H:%M:%S%.3f");
    let severity = format!("[{:?}]", debug_info.severity);
    let node_info = format!("{}({})", debug_info.node_name, debug_info.node_id);
    let message = &debug_info.message;

    eprintln!("{} {} {}: {}", timestamp, severity, node_info, message);
  }

  fn output_to_log(&self, debug_info: &DebugInfo) {
    match debug_info.severity {
      DebugSeverity::Trace => tracing::trace!("{}: {}", debug_info.node_name, debug_info.message),
      DebugSeverity::Debug => tracing::debug!("{}: {}", debug_info.node_name, debug_info.message),
      DebugSeverity::Info => tracing::info!("{}: {}", debug_info.node_name, debug_info.message),
      DebugSeverity::Warning => tracing::warn!("{}: {}", debug_info.node_name, debug_info.message),
      DebugSeverity::Error => tracing::error!("{}: {}", debug_info.node_name, debug_info.message),
      DebugSeverity::Critical => {
        tracing::error!("CRITICAL {}: {}", debug_info.node_name, debug_info.message)
      }
    }
  }

  fn output_to_file(&self, debug_info: &DebugInfo) {
    if let Ok(mut file) = std::fs::OpenOptions::new()
      .create(true)
      .append(true)
      .open("ellastic_debug.log")
    {
      let line = format!(
        "{} [{:?}] {}({}): {}\n",
        debug_info.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
        debug_info.severity,
        debug_info.node_name,
        debug_info.node_id,
        debug_info.message
      );

      let _ = file.write_all(line.as_bytes());
    }
  }

  fn output_structured(&self, debug_info: &DebugInfo) {
    if let Ok(json) = serde_json::to_string(debug_info) {
      tracing::info!("DEBUG: {}", json);
    }
  }

  pub fn on_node_start(&mut self, node_id: Uuid, node_name: &str) {
    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::NodeStart,
      message: format!("Node '{}' started execution", node_name),
      data: HashMap::new(),
      severity: DebugSeverity::Info,
    });
  }

  pub fn on_node_complete(
    &mut self,
    node_id: Uuid,
    node_name: &str,
    duration: std::time::Duration,
  ) {
    let mut data = HashMap::new();
    data.insert("duration_ms".to_string(), duration.as_millis().to_string());

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::NodeComplete,
      message: format!("Node '{}' completed in {:?}", node_name, duration),
      data,
      severity: DebugSeverity::Info,
    });
  }

  pub fn on_node_error(&mut self, node_id: Uuid, node_name: &str, error: &str) {
    let mut data = HashMap::new();
    data.insert("error".to_string(), error.to_string());

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::NodeError,
      message: format!("Node '{}' failed: {}", node_name, error),
      data,
      severity: DebugSeverity::Error,
    });
  }

  pub fn on_node_warning(&mut self, node_id: Uuid, node_name: &str, warning: &str) {
    let mut data = HashMap::new();
    data.insert("warning".to_string(), warning.to_string());

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::NodeWarning,
      message: format!("Node '{}' warning: {}", node_name, warning),
      data,
      severity: DebugSeverity::Warning,
    });
  }

  pub fn on_data_flow(&mut self, from_node: Uuid, to_node: Uuid, data_size: usize) {
    let mut data = HashMap::new();
    data.insert("from_node".to_string(), from_node.to_string());
    data.insert("to_node".to_string(), to_node.to_string());
    data.insert("data_size".to_string(), data_size.to_string());

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id: from_node,
      node_name: "DataFlow".to_string(),
      debug_type: DebugType::DataFlow,
      message: format!(
        "Data flow: {} -> {} ({} bytes)",
        from_node, to_node, data_size
      ),
      data,
      severity: DebugSeverity::Trace,
    });
  }

  pub fn on_cache_hit(&mut self, node_id: Uuid, node_name: &str, cache_key: &str) {
    let mut data = HashMap::new();
    data.insert("cache_key".to_string(), cache_key.to_string());

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::CacheHit,
      message: format!(
        "Cache hit for node '{}' with key '{}'",
        node_name, cache_key
      ),
      data,
      severity: DebugSeverity::Debug,
    });
  }

  pub fn on_cache_miss(&mut self, node_id: Uuid, node_name: &str, cache_key: &str) {
    let mut data = HashMap::new();
    data.insert("cache_key".to_string(), cache_key.to_string());

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::CacheMiss,
      message: format!(
        "Cache miss for node '{}' with key '{}'",
        node_name, cache_key
      ),
      data,
      severity: DebugSeverity::Debug,
    });
  }

  pub fn on_performance(&mut self, node_id: Uuid, node_name: &str, metrics: &HashMap<String, f64>) {
    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::Performance,
      message: format!("Performance metrics for node '{}'", node_name),
      data: metrics
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect(),
      severity: DebugSeverity::Debug,
    });
  }

  pub fn on_memory(&mut self, node_id: Uuid, node_name: &str, memory_usage: usize) {
    let mut data = HashMap::new();
    data.insert("memory_bytes".to_string(), memory_usage.to_string());

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::Memory,
      message: format!(
        "Memory usage for node '{}': {} bytes",
        node_name, memory_usage
      ),
      data,
      severity: DebugSeverity::Debug,
    });
  }

  pub fn on_validation(
    &mut self,
    node_id: Uuid,
    node_name: &str,
    validation_result: bool,
    message: &str,
  ) {
    let mut data = HashMap::new();
    data.insert(
      "validation_result".to_string(),
      validation_result.to_string(),
    );

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id,
      node_name: node_name.to_string(),
      debug_type: DebugType::Validation,
      message: format!(
        "Validation for node '{}': {} - {}",
        node_name, validation_result, message
      ),
      data,
      severity: if validation_result {
        DebugSeverity::Debug
      } else {
        DebugSeverity::Warning
      },
    });
  }

  pub fn get_debug_info(&self) -> &[DebugInfo] {
    &self.debug_info
  }

  pub fn get_debug_info_by_node(&self, node_id: Uuid) -> Vec<&DebugInfo> {
    self
      .debug_info
      .iter()
      .filter(|info| info.node_id == node_id)
      .collect()
  }

  pub fn get_debug_info_by_type(&self, debug_type: DebugType) -> Vec<&DebugInfo> {
    self
      .debug_info
      .iter()
      .filter(|info| info.debug_type == debug_type)
      .collect()
  }

  pub fn get_debug_info_by_severity(&self, severity: DebugSeverity) -> Vec<&DebugInfo> {
    self
      .debug_info
      .iter()
      .filter(|info| info.severity == severity)
      .collect()
  }

  pub fn clear_debug_info(&mut self) {
    self.debug_info.clear();

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id: Uuid::new_v4(),
      node_name: "Debugger".to_string(),
      debug_type: DebugType::Custom,
      message: "Cleared debug info".to_string(),
      data: HashMap::new(),
      severity: DebugSeverity::Debug,
    });
  }

  pub fn export_debug_info(&self, format: DebugExportFormat) -> Result<String> {
    match format {
      DebugExportFormat::JSON => serde_json::to_string_pretty(&self.debug_info).map_err(|e| {
        EllasticError::SerializationError(format!("Failed to export debug info as JSON: {}", e))
      }),
      DebugExportFormat::CSV => self.export_debug_info_csv(),
      DebugExportFormat::Text => self.export_debug_info_text(),
    }
  }

  fn export_debug_info_csv(&self) -> Result<String> {
    let mut csv = String::new();
    csv.push_str("timestamp,node_id,node_name,debug_type,severity,message\n");

    for info in &self.debug_info {
      csv.push_str(&format!(
        "{},{},{},{:?},{:?},{}\n",
        info.timestamp.to_rfc3339(),
        info.node_id,
        info.node_name,
        info.debug_type,
        info.severity,
        info.message
      ));
    }

    Ok(csv)
  }

  fn export_debug_info_text(&self) -> Result<String> {
    let mut text = String::new();
    text.push_str("Pipeline Debug Information\n");
    text.push_str("========================\n\n");

    for info in &self.debug_info {
      text.push_str(&format!("Timestamp: {}\n", info.timestamp.to_rfc3339()));
      text.push_str(&format!("Node: {} ({})\n", info.node_name, info.node_id));
      text.push_str(&format!("Type: {:?}\n", info.debug_type));
      text.push_str(&format!("Severity: {:?}\n", info.severity));
      text.push_str(&format!("Message: {}\n", info.message));

      if !info.data.is_empty() {
        text.push_str("Data:\n");
        for (key, value) in &info.data {
          text.push_str(&format!("  {}: {}\n", key, value));
        }
      }

      text.push_str("\n---\n\n");
    }

    Ok(text)
  }

  pub fn start_debug_session(&mut self) -> DebugSession {
    let session = DebugSession {
      id: Uuid::new_v4(),
      start_time: Utc::now(),
      end_time: None,
      status: DebugSessionStatus::Active,
      breakpoints: self.breakpoints.clone(),
      step_count: self.current_step,
      debug_info: self.debug_info.clone(),
      session_data: HashMap::new(),
    };

    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id: session.id,
      node_name: "DebugSession".to_string(),
      debug_type: DebugType::Custom,
      message: format!("Started debug session {}", session.id),
      data: HashMap::new(),
      severity: DebugSeverity::Info,
    });

    session
  }

  pub fn end_debug_session(&mut self, session_id: Uuid) {
    self.log_debug(DebugInfo {
      timestamp: Utc::now(),
      node_id: session_id,
      node_name: "DebugSession".to_string(),
      debug_type: DebugType::Custom,
      message: format!("Ended debug session {}", session_id),
      data: HashMap::new(),
      severity: DebugSeverity::Info,
    });
  }

  pub fn clone(&self) -> PipelineDebugger {
    PipelineDebugger {
      enabled: self.enabled,
      debug_info: self.debug_info.clone(),
      breakpoints: self.breakpoints.clone(),
      step_mode: self.step_mode,
      current_step: self.current_step,
      debug_level: self.debug_level,
      output_format: self.output_format,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugExportFormat {
  JSON,
  CSV,
  Text,
}

impl DebugSession {
  pub fn pause(&mut self) {
    self.status = DebugSessionStatus::Paused;
  }

  pub fn resume(&mut self) {
    self.status = DebugSessionStatus::Active;
  }

  pub fn complete(&mut self) {
    self.status = DebugSessionStatus::Completed;
    self.end_time = Some(Utc::now());
  }

  pub fn abort(&mut self) {
    self.status = DebugSessionStatus::Aborted;
    self.end_time = Some(Utc::now());
  }

  pub fn duration(&self) -> Option<chrono::Duration> {
    self
      .end_time
      .map(|end| end.signed_duration_since(self.start_time))
  }

  pub fn clone(&self) -> DebugSession {
    DebugSession {
      id: self.id,
      start_time: self.start_time,
      end_time: self.end_time,
      status: self.status,
      breakpoints: self.breakpoints.clone(),
      step_count: self.step_count,
      debug_info: self.debug_info.clone(),
      session_data: self.session_data.clone(),
    }
  }
}

impl DebugProfiler {
  pub fn new(name: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      name,
      enabled: false,
      profile_data: HashMap::new(),
      global_profile: GlobalProfile::new(),
      sampling_rate: 1.0,
      max_samples: 1000,
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

  pub fn with_max_samples(mut self, max_samples: usize) -> Self {
    self.max_samples = max_samples;
    self
  }

  pub fn enable(&mut self) {
    self.enabled = true;
  }

  pub fn disable(&mut self) {
    self.enabled = false;
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn start_node_profiling(&mut self, node_id: Uuid, node_name: String) {
    if !self.enabled {
      return;
    }

    if !self.profile_data.contains_key(&node_id) {
      self.profile_data.insert(
        node_id,
        NodeProfile {
          node_id,
          node_name,
          execution_count: 0,
          total_time: std::time::Duration::ZERO,
          min_time: std::time::Duration::MAX,
          max_time: std::time::Duration::ZERO,
          average_time: std::time::Duration::ZERO,
          memory_usage: Vec::new(),
          cache_hits: 0,
          cache_misses: 0,
          error_count: 0,
          last_execution: Utc::now(),
        },
      );
    }
  }

  pub fn end_node_profiling(
    &mut self,
    node_id: Uuid,
    execution_time: std::time::Duration,
    memory_usage: usize,
    cache_hit: bool,
    error: bool,
  ) {
    if !self.enabled {
      return;
    }

    if let Some(profile) = self.profile_data.get_mut(&node_id) {
      profile.execution_count += 1;
      profile.total_time += execution_time;
      profile.min_time = profile.min_time.min(execution_time);
      profile.max_time = profile.max_time.max(execution_time);
      profile.average_time = profile.total_time / profile.execution_count as u32;
      profile.memory_usage.push(memory_usage);

      if profile.memory_usage.len() > self.max_samples {
        profile
          .memory_usage
          .drain(0..profile.memory_usage.len() / 2);
      }

      if cache_hit {
        profile.cache_hits += 1;
      } else {
        profile.cache_misses += 1;
      }

      if error {
        profile.error_count += 1;
      }

      profile.last_execution = Utc::now();

      self.update_global_profile();
    }
  }

  fn update_global_profile(&mut self) {
    let total_executions: u32 = self.profile_data.values().map(|p| p.execution_count).sum();
    let total_time: std::time::Duration = self.profile_data.values().map(|p| p.total_time).sum();
    let total_errors: u32 = self.profile_data.values().map(|p| p.error_count).sum();
    let total_cache_hits: u32 = self.profile_data.values().map(|p| p.cache_hits).sum();
    let total_cache_accesses: u32 = self
      .profile_data
      .values()
      .map(|p| p.cache_hits + p.cache_misses)
      .sum();

    self.global_profile.total_executions = total_executions;
    self.global_profile.total_time = total_time;
    self.global_profile.average_time = if total_executions > 0 {
      total_time / total_executions as u32
    } else {
      std::time::Duration::ZERO
    };
    self.global_profile.memory_peak = self
      .profile_data
      .values()
      .map(|p| p.memory_usage.iter().max().unwrap_or(&0))
      .max()
      .unwrap_or(&0)
      .clone();
    self.global_profile.cache_hit_rate = if total_cache_accesses > 0 {
      total_cache_hits as f64 / total_cache_accesses as f64
    } else {
      0.0
    };
    self.global_profile.error_rate = if total_executions > 0 {
      total_errors as f64 / total_executions as f64
    } else {
      0.0
    };
    self.global_profile.throughput = if total_time > std::time::Duration::ZERO {
      total_executions as f64 / total_time.as_secs_f64()
    } else {
      0.0
    };
  }

  pub fn get_node_profile(&self, node_id: Uuid) -> Option<&NodeProfile> {
    self.profile_data.get(&node_id)
  }

  pub fn get_global_profile(&self) -> &GlobalProfile {
    &self.global_profile
  }

  pub fn clear_profiles(&mut self) {
    self.profile_data.clear();
    self.global_profile = GlobalProfile::new();
  }

  pub fn export_profiles(&self) -> Result<String> {
    serde_json::to_string_pretty(&self.profile_data)
      .map_err(|e| EllasticError::SerializationError(format!("Failed to export profiles: {}", e)))
  }

  pub fn clone(&self) -> DebugProfiler {
    DebugProfiler {
      id: self.id,
      name: self.name.clone(),
      enabled: self.enabled,
      profile_data: self.profile_data.clone(),
      global_profile: self.global_profile.clone(),
      sampling_rate: self.sampling_rate,
      max_samples: self.max_samples,
    }
  }
}

impl GlobalProfile {
  pub fn new() -> Self {
    Self {
      total_executions: 0,
      total_time: std::time::Duration::ZERO,
      average_time: std::time::Duration::ZERO,
      memory_peak: 0,
      cache_hit_rate: 0.0,
      error_rate: 0.0,
      throughput: 0.0,
    }
  }

  pub fn clone(&self) -> GlobalProfile {
    GlobalProfile {
      total_executions: self.total_executions,
      total_time: self.total_time,
      average_time: self.average_time,
      memory_peak: self.memory_peak,
      cache_hit_rate: self.cache_hit_rate,
      error_rate: self.error_rate,
      throughput: self.throughput,
    }
  }
}

impl DebugInspector {
  pub fn new() -> Self {
    Self {
      id: Uuid::new_v4(),
      inspection_points: HashMap::new(),
      inspection_history: Vec::new(),
    }
  }

  pub fn add_inspection_point(
    &mut self,
    node_id: Uuid,
    name: String,
    inspection_type: InspectionType,
  ) -> Result<Uuid> {
    let inspection_point = InspectionPoint {
      id: Uuid::new_v4(),
      node_id,
      name,
      inspection_type,
      enabled: true,
      created_at: Utc::now(),
    };

    let id = inspection_point.id;
    self.inspection_points.insert(id, inspection_point);
    Ok(id)
  }

  pub fn remove_inspection_point(&mut self, id: Uuid) -> Option<InspectionPoint> {
    self.inspection_points.remove(&id)
  }

  pub fn enable_inspection_point(&mut self, id: Uuid) -> bool {
    if let Some(point) = self.inspection_points.get_mut(&id) {
      point.enabled = true;
      true
    } else {
      false
    }
  }

  pub fn disable_inspection_point(&mut self, id: Uuid) -> bool {
    if let Some(point) = self.inspection_points.get_mut(&id) {
      point.enabled = false;
      true
    } else {
      false
    }
  }

  pub fn inspect(
    &mut self,
    inspection_point_id: Uuid,
    data: HashMap<String, String>,
    snapshot: Option<String>,
  ) -> Result<Uuid> {
    let record = InspectionRecord {
      id: Uuid::new_v4(),
      inspection_point_id,
      timestamp: Utc::now(),
      data,
      snapshot,
    };

    let record_id = record.id;
    self.inspection_history.push(record);
    Ok(record_id)
  }

  pub fn get_inspection_point(&self, id: Uuid) -> Option<&InspectionPoint> {
    self.inspection_points.get(&id)
  }

  pub fn list_inspection_points(&self) -> Vec<&InspectionPoint> {
    self.inspection_points.values().collect()
  }

  pub fn get_inspection_history(&self) -> &[InspectionRecord] {
    &self.inspection_history
  }

  pub fn get_inspection_history_for_point(
    &self,
    inspection_point_id: Uuid,
  ) -> Vec<&InspectionRecord> {
    self
      .inspection_history
      .iter()
      .filter(|record| record.inspection_point_id == inspection_point_id)
      .collect()
  }

  pub fn clear_inspection_history(&mut self) {
    self.inspection_history.clear();
  }

  pub fn clone(&self) -> DebugInspector {
    DebugInspector {
      id: self.id,
      inspection_points: self.inspection_points.clone(),
      inspection_history: self.inspection_history.clone(),
    }
  }
}

pub fn create_pipeline_debugger() -> PipelineDebugger {
  PipelineDebugger::new()
}

pub fn create_debug_profiler(name: String) -> DebugProfiler {
  DebugProfiler::new(name)
}

pub fn create_debug_inspector() -> DebugInspector {
  DebugInspector::new()
}
