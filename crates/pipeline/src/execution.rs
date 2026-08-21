use crate::graph::{
  EdgeType,
  PipelineEdge,
  PipelineGraph,
  PipelineNode,
};
use crate::nodes::{
  NodeProcessor,
  NodeProcessorTrait,
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
  VecDeque,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PipelineExecutor {
  graph: PipelineGraph,
  execution_config: ExecutionConfig,
  cache_config: CacheConfig,
  node_processors: HashMap<Uuid, NodeProcessor>,
  execution_context: ExecutionContext,
  progress_callback: Option<Box<dyn PipelineProgressCallback + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
  pub parallel_execution: bool,
  pub max_concurrent_nodes: Option<usize>,
  pub enable_caching: bool,
  pub enable_profiling: bool,
  pub enable_debugging: bool,
  pub timeout_seconds: Option<u64>,
  pub retry_count: u32,
  pub memory_limit_mb: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
  pub enable_node_caching: bool,
  pub enable_graph_caching: bool,
  pub max_cache_size_mb: usize,
  pub cache_ttl_seconds: u64,
  pub enable_persistent_cache: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
  pub id: Uuid,
  pub start_time: DateTime<Utc>,
  pub end_time: Option<DateTime<Utc>>,
  pub status: ExecutionStatus,
  pub error: Option<String>,
  pub executed_nodes: Vec<Uuid>,
  pub node_results: HashMap<Uuid, MediaProcessor>,
  pub execution_times: HashMap<Uuid, std::time::Duration>,
  pub memory_usage: HashMap<Uuid, usize>,
  pub current_node: Option<Uuid>,
  pub progress: f32,
  pub total_nodes: u32,
  pub completed_nodes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
  Pending,
  Running,
  Completed,
  Failed,
  Cancelled,
  Timeout,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
  pub context_id: Uuid,
  pub output: MediaProcessor,
  pub execution_time: std::time::Duration,
  pub executed_nodes: Vec<Uuid>,
  pub node_results: HashMap<Uuid, MediaProcessor>,
  pub performance_metrics: ExecutionPerformanceMetrics,
  pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionPerformanceMetrics {
  pub total_time: std::time::Duration,
  pub average_node_time: std::time::Duration,
  pub slowest_node_time: std::time::Duration,
  pub fastest_node_time: std::time::Duration,
  pub memory_peak_mb: f64,
  pub cache_hit_rate: f64,
  pub parallel_efficiency: f64,
}

pub trait PipelineProgressCallback: Send + Sync {
  fn on_node_start(&self, node_id: Uuid, node_name: &str);
  fn on_node_complete(&self, node_id: Uuid, node_name: &str, duration: std::time::Duration);
  fn on_node_error(&self, node_id: Uuid, node_name: &str, error: &str);
  fn on_pipeline_progress(&self, completed_nodes: u32, total_nodes: u32);
  fn on_pipeline_complete(&self, total_duration: std::time::Duration);
  fn on_pipeline_error(&self, error: &str);
}

impl PipelineExecutor {
  pub fn new(
    graph: &PipelineGraph,
    execution_config: &ExecutionConfig,
    cache_config: &CacheConfig,
  ) -> Self {
    let mut executor = Self {
      graph: graph.clone(),
      execution_config: execution_config.clone(),
      cache_config: cache_config.clone(),
      node_processors: HashMap::new(),
      execution_context: ExecutionContext::new(),
      progress_callback: None,
    };

    executor.initialize_node_processors().unwrap_or_else(|e| {
      eprintln!("Warning: Failed to initialize node processors: {}", e);
    });

    executor
  }

  pub fn with_progress_callback(
    mut self,
    callback: Box<dyn PipelineProgressCallback + Send + Sync>,
  ) -> Self {
    self.progress_callback = Some(callback);
    self
  }

  pub fn graph(&self) -> &PipelineGraph {
    &self.graph
  }

  pub fn execution_config(&self) -> &ExecutionConfig {
    &self.execution_config
  }

  pub fn cache_config(&self) -> &CacheConfig {
    &self.cache_config
  }

  pub fn execution_context(&self) -> &ExecutionContext {
    &self.execution_context
  }

  pub fn progress_callback(&self) -> Option<&Box<dyn PipelineProgressCallback + Send + Sync>> {
    self.progress_callback.as_ref()
  }

  pub fn set_progress_callback(
    &mut self,
    callback: Box<dyn PipelineProgressCallback + Send + Sync>,
  ) {
    self.progress_callback = Some(callback);
  }

  pub fn execute(&mut self, input: MediaProcessor) -> Result<MediaProcessor> {
    let start_time = std::time::Instant::now();

    self.execution_context = ExecutionContext::new();
    self.execution_context.total_nodes = self.graph.nodes.len() as u32;
    self.execution_context.status = ExecutionStatus::Running;

    self.graph.validate()?;

    let result = if self.execution_config.parallel_execution {
      self.execute_parallel(input)
    } else {
      self.execute_sequential(input)
    };

    let execution_time = start_time.elapsed();
    self.execution_context.end_time = Some(Utc::now());
    self
      .execution_context
      .execution_times
      .insert(Uuid::new_v4(), execution_time);

    match result {
      Ok(output) => {
        self.execution_context.status = ExecutionStatus::Completed;
        if let Some(ref callback) = self.progress_callback {
          callback.on_pipeline_complete(execution_time);
        }
        Ok(output)
      }
      Err(e) => {
        self.execution_context.status = ExecutionStatus::Failed;
        self.execution_context.error = Some(e.to_string());
        if let Some(ref callback) = self.progress_callback {
          callback.on_pipeline_error(&e.to_string());
        }
        Err(e)
      }
    }
  }

  fn execute_sequential(&mut self, input: MediaProcessor) -> Result<MediaProcessor> {
    let execution_order = self.calculate_execution_order()?;
    let mut current_input = input;

    for node_id in execution_order {
      if let Some(timeout_seconds) = self.execution_config.timeout_seconds {
        let elapsed = Utc::now().signed_duration_since(self.execution_context.start_time);
        if elapsed.num_seconds() > timeout_seconds as i64 {
          self.execution_context.status = ExecutionStatus::Timeout;
          return Err(EllasticError::Timeout(
            "Pipeline execution timed out".to_string(),
          ));
        }
      }

      let node_result = self.execute_node(node_id, current_input)?;
      current_input = node_result;
    }

    Ok(current_input)
  }

  fn execute_parallel(&mut self, input: MediaProcessor) -> Result<MediaProcessor> {
    let execution_groups = self.calculate_parallel_groups()?;
    let mut current_input = input;

    for group in execution_groups {
      let mut group_results = Vec::new();

      for node_id in group {
        if let Some(timeout_seconds) = self.execution_config.timeout_seconds {
          let elapsed = Utc::now().signed_duration_since(self.execution_context.start_time);
          if elapsed.num_seconds() > timeout_seconds as i64 {
            self.execution_context.status = ExecutionStatus::Timeout;
            return Err(EllasticError::Timeout(
              "Pipeline execution timed out".to_string(),
            ));
          }
        }

        let node_result = self.execute_node(node_id, current_input.clone())?;
        group_results.push(node_result);
      }

      current_input = self.combine_parallel_results(group_results)?;
    }

    Ok(current_input)
  }

  fn execute_node(&mut self, node_id: Uuid, input: MediaProcessor) -> Result<MediaProcessor> {
    let node = self
      .graph
      .get_node(node_id)
      .ok_or_else(|| EllasticError::InvalidParameter(format!("Node {} not found", node_id)))?;

    if !node.enabled {
      return Ok(input);
    }

    let node_name = node.name.clone();
    let start_time = std::time::Instant::now();

    if let Some(ref callback) = self.progress_callback {
      callback.on_node_start(node_id, &node_name);
    }

    let result = self.execute_node_internal(node_id, input);

    let execution_time = start_time.elapsed();

    self.execution_context.executed_nodes.push(node_id);
    self.execution_context.completed_nodes += 1;
    self.execution_context.progress =
      self.execution_context.completed_nodes as f32 / self.execution_context.total_nodes as f32;
    self
      .execution_context
      .execution_times
      .insert(node_id, execution_time);
    self.execution_context.current_node = None;

    match &result {
      Ok(_) => {
        if let Some(ref callback) = self.progress_callback {
          callback.on_node_complete(node_id, &node_name, execution_time);
        }
        callback.on_pipeline_progress(
          self.execution_context.completed_nodes,
          self.execution_context.total_nodes,
        );
      }
      Err(e) => {
        if let Some(ref callback) = self.progress_callback {
          callback.on_node_error(node_id, &node_name, &e.to_string());
        }
      }
    }

    result
  }

  fn execute_node_internal(
    &mut self,
    node_id: Uuid,
    input: MediaProcessor,
  ) -> Result<MediaProcessor> {
    let node_processor = self.node_processors.get_mut(&node_id).ok_or_else(|| {
      EllasticError::InvalidParameter(format!("Node processor for {} not found", node_id))
    })?;

    let parameters = self.get_node_parameters(node_id);
    node_processor.process(input, parameters)
  }

  fn get_node_parameters(&self, node_id: Uuid) -> HashMap<String, String> {
    self
      .graph
      .get_node(node_id)
      .map(|node| node.parameters.clone())
      .unwrap_or_default()
  }

  fn combine_parallel_results(&self, results: Vec<MediaProcessor>) -> Result<MediaProcessor> {
    if results.is_empty() {
      return Err(EllasticError::InvalidParameter(
        "No results to combine".to_string(),
      ));
    }

    if results.len() == 1 {
      return Ok(results.into_iter().next().unwrap());
    }

    let mut result = results[0].clone();

    for processor in results.iter().skip(1) {
      result = result.blend(processor, ellastic_media::BlendMode::Overlay, 0.5)?;
    }

    Ok(result)
  }

  fn initialize_node_processors(&mut self) -> Result<()> {
    for node in self.graph.list_nodes() {
      let node_processor = crate::nodes::create_node_processor(node.clone())?;
      self.node_processors.insert(node.id, node_processor);
    }
    Ok(())
  }

  fn calculate_execution_order(&self) -> Result<Vec<Uuid>> {
    let mut order = Vec::new();
    let mut visited = HashSet::new();
    let node_map: HashMap<Uuid, &PipelineNode> = self
      .graph
      .nodes
      .iter()
      .map(|(id, node)| (id, node))
      .collect();

    for node in self.graph.list_nodes() {
      if !visited.contains(&node.id) {
        self.visit_node(node.id, &node_map, &mut visited, &mut order)?;
      }
    }

    Ok(order)
  }

  fn visit_node(
    &self,
    node_id: Uuid,
    node_map: &HashMap<Uuid, &PipelineNode>,
    visited: &mut HashSet<Uuid>,
    order: &mut Vec<Uuid>,
  ) -> Result<()> {
    if visited.contains(&node_id) {
      return Ok(());
    }

    if let Some(node) = node_map.get(&node_id) {
      let dependencies = self.get_node_dependencies(node);
      for dep_id in dependencies {
        if node_map.contains_key(&dep_id) {
          self.visit_node(dep_id, node_map, visited, order)?;
        }
      }

      visited.insert(node_id);
      order.push(node_id);
    }

    Ok(())
  }

  fn get_node_dependencies(&self, node: &PipelineNode) -> Vec<Uuid> {
    let mut dependencies = Vec::new();

    for edge in self.graph.list_edges() {
      if edge.to_node == node.id {
        dependencies.push(edge.from_node);
      }
    }

    dependencies
  }

  fn calculate_parallel_groups(&self) -> Result<Vec<Vec<Uuid>>> {
    let mut groups = Vec::new();
    let mut processed = HashSet::new();
    let node_map: HashMap<Uuid, &PipelineNode> = self
      .graph
      .nodes
      .iter()
      .map(|(id, node)| (id, node))
      .collect();

    while processed.len() < self.graph.nodes.len() {
      let mut current_group = Vec::new();

      for node in self.graph.list_nodes() {
        if !processed.contains(&node.id) {
          let dependencies = self.get_node_dependencies(node);
          let dependencies_met = dependencies.iter().all(|dep_id| processed.contains(dep_id));

          if dependencies_met {
            current_group.push(node.id);
          }
        }
      }

      if current_group.is_empty() {
        return Err(EllasticError::InvalidParameter(
          "Circular dependency detected in pipeline".to_string(),
        ));
      }

      for node_id in &current_group {
        processed.insert(*node_id);
      }

      groups.push(current_group);
    }

    Ok(groups)
  }

  pub fn get_execution_result(&self) -> ExecutionResult {
    let total_time = self
      .execution_context
      .end_time
      .map_or(std::time::Duration::ZERO, |end| {
        end
          .signed_duration_since(self.execution_context.start_time)
          .to_std()
          .unwrap_or(std::time::Duration::ZERO)
      });

    let performance_metrics = ExecutionPerformanceMetrics {
      total_time,
      average_node_time: if self.execution_context.execution_times.is_empty() {
        std::time::Duration::ZERO
      } else {
        let sum: std::time::Duration = self.execution_context.execution_times.values().sum();
        sum / self.execution_context.execution_times.len() as u32
      },
      slowest_node_time: self
        .execution_context
        .execution_times
        .values()
        .max()
        .unwrap_or(&std::time::Duration::ZERO),
      fastest_node_time: self
        .execution_context
        .execution_times
        .values()
        .min()
        .unwrap_or(&std::time::Duration::ZERO),
      memory_peak_mb: 0.0,
      cache_hit_rate: 0.0,
      parallel_efficiency: if self.execution_config.parallel_execution {
        0.8
      } else {
        1.0
      },
    };

    ExecutionResult {
      context_id: self.execution_context.id,
      output: self
        .execution_context
        .node_results
        .values()
        .last()
        .cloned()
        .unwrap_or_else(|| MediaProcessor::new(MediaData::empty())),
      execution_time: total_time,
      executed_nodes: self.execution_context.executed_nodes.clone(),
      node_results: self.execution_context.node_results.clone(),
      performance_metrics,
      errors: self.execution_context.error.clone().into_iter().collect(),
    }
  }

  pub fn cancel(&mut self) -> Result<()> {
    self.execution_context.status = ExecutionStatus::Cancelled;
    Ok(())
  }

  pub fn clone(&self) -> PipelineExecutor {
    PipelineExecutor {
      graph: self.graph.clone(),
      execution_config: self.execution_config.clone(),
      cache_config: self.cache_config.clone(),
      node_processors: self.node_processors.clone(),
      execution_context: self.execution_context.clone(),
      progress_callback: None,
    }
  }
}

impl ExecutionContext {
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      start_time: now,
      end_time: None,
      status: ExecutionStatus::Pending,
      error: None,
      executed_nodes: Vec::new(),
      node_results: HashMap::new(),
      execution_times: HashMap::new(),
      memory_usage: HashMap::new(),
      current_node: None,
      progress: 0.0,
      total_nodes: 0,
      completed_nodes: 0,
    }
  }

  pub fn clone(&self) -> ExecutionContext {
    ExecutionContext {
      id: self.id,
      start_time: self.start_time,
      end_time: self.end_time,
      status: self.status,
      error: self.error.clone(),
      executed_nodes: self.executed_nodes.clone(),
      node_results: self.node_results.clone(),
      execution_times: self.execution_times.clone(),
      memory_usage: self.memory_usage.clone(),
      current_node: self.current_node,
      progress: self.progress,
      total_nodes: self.total_nodes,
      completed_nodes: self.completed_nodes,
    }
  }
}

impl Default for ExecutionConfig {
  fn default() -> Self {
    Self {
      parallel_execution: true,
      max_concurrent_nodes: Some(4),
      enable_caching: true,
      enable_profiling: false,
      enable_debugging: false,
      timeout_seconds: Some(300),
      retry_count: 3,
      memory_limit_mb: Some(1024),
    }
  }
}

impl Default for CacheConfig {
  fn default() -> Self {
    Self {
      enable_node_caching: true,
      enable_graph_caching: false,
      max_cache_size_mb: 512,
      cache_ttl_seconds: 3600,
      enable_persistent_cache: false,
    }
  }
}

#[derive(Debug, Clone)]
pub struct PipelineDebugger {
  enabled: bool,
  debug_info: Vec<DebugInfo>,
  breakpoints: HashSet<Uuid>,
  step_mode: bool,
  current_step: usize,
}

#[derive(Debug, Clone)]
pub struct DebugInfo {
  pub timestamp: DateTime<Uuid>,
  pub node_id: Uuid,
  pub node_name: String,
  pub debug_type: DebugType,
  pub message: String,
  pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugType {
  NodeStart,
  NodeComplete,
  NodeError,
  DataFlow,
  CacheHit,
  CacheMiss,
  Performance,
  Custom,
}

impl PipelineDebugger {
  pub fn new() -> Self {
    Self {
      enabled: false,
      debug_info: Vec::new(),
      breakpoints: HashSet::new(),
      step_mode: false,
      current_step: 0,
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

  pub fn add_breakpoint(&mut self, node_id: Uuid) {
    self.breakpoints.insert(node_id);
  }

  pub fn remove_breakpoint(&mut self, node_id: Uuid) {
    self.breakpoints.remove(&node_id);
  }

  pub fn log_debug_info(&mut self, debug_info: DebugInfo) {
    if self.enabled {
      self.debug_info.push(debug_info);
    }
  }

  pub fn should_break(&self, node_id: Uuid) -> bool {
    self.breakpoints.contains(&node_id) && self.step_mode
  }

  pub fn step(&mut self) -> bool {
    if self.step_mode {
      self.current_step += 1;
      true
    } else {
      false
    }
  }

  pub fn get_debug_info(&self) -> &[DebugInfo] {
    &self.debug_info
  }

  pub fn clear_debug_info(&mut self) {
    self.debug_info.clear();
  }

  pub fn clone(&self) -> PipelineDebugger {
    PipelineDebugger {
      enabled: self.enabled,
      debug_info: self.debug_info.clone(),
      breakpoints: self.breakpoints.clone(),
      step_mode: self.step_mode,
      current_step: self.current_step,
    }
  }
}

#[derive(Debug, Clone)]
pub struct PipelineProfiler {
  enabled: bool,
  profile_data: HashMap<Uuid, NodeProfile>,
  global_profile: GlobalProfile,
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
}

#[derive(Debug, Clone)]
pub struct GlobalProfile {
  pub total_executions: u32,
  pub total_time: std::time::Duration,
  pub average_time: std::time::Duration,
  pub memory_peak: usize,
  pub cache_hit_rate: f64,
}

impl PipelineProfiler {
  pub fn new() -> Self {
    Self {
      enabled: false,
      profile_data: HashMap::new(),
      global_profile: GlobalProfile {
        total_executions: 0,
        total_time: std::time::Duration::ZERO,
        average_time: std::time::Duration::ZERO,
        memory_peak: 0,
        cache_hit_rate: 0.0,
      },
    }
  }

  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  pub fn start_node_profiling(&mut self, node_id: Uuid, node_name: String) {
    if self.enabled {
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
          },
        );
      }
    }
  }

  pub fn end_node_profiling(
    &mut self,
    node_id: Uuid,
    execution_time: std::time::Duration,
    memory_usage: usize,
    cache_hit: bool,
  ) {
    if self.enabled {
      if let Some(profile) = self.profile_data.get_mut(&node_id) {
        profile.execution_count += 1;
        profile.total_time += execution_time;
        profile.min_time = profile.min_time.min(execution_time);
        profile.max_time = profile.max_time.max(execution_time);
        profile.average_time = profile.total_time / profile.execution_count as u32;
        profile.memory_usage.push(memory_usage);

        if cache_hit {
          profile.cache_hits += 1;
        } else {
          profile.cache_misses += 1;
        }

        self.global_profile.total_executions += 1;
        self.global_profile.total_time += execution_time;
        self.global_profile.average_time =
          self.global_profile.total_time / self.global_profile.total_executions as u32;
        self.global_profile.memory_peak = self.global_profile.memory_peak.max(memory_usage);

        let total_cache_accesses = profile.cache_hits + profile.cache_misses;
        if total_cache_accesses > 0 {
          self.global_profile.cache_hit_rate =
            profile.cache_hits as f64 / total_cache_accesses as f64;
        }
      }
    }
  }

  pub fn get_node_profile(&self, node_id: Uuid) -> Option<&NodeProfile> {
    self.profile_data.get(&node_id)
  }

  pub fn get_global_profile(&self) -> &GlobalProfile {
    &self.global_profile
  }

  pub fn clear_profiles(&mut self) {
    self.profile_data.clear();
    self.global_profile = GlobalProfile {
      total_executions: 0,
      total_time: std::time::Duration::ZERO,
      average_time: std::time::Duration::ZERO,
      memory_peak: 0,
      cache_hit_rate: 0.0,
    };
  }

  pub fn clone(&self) -> PipelineProfiler {
    PipelineProfiler {
      enabled: self.enabled,
      profile_data: self.profile_data.clone(),
      global_profile: self.global_profile.clone(),
    }
  }
}

pub fn create_pipeline_executor(
  graph: &PipelineGraph,
  execution_config: &ExecutionConfig,
  cache_config: &CacheConfig,
) -> PipelineExecutor {
  PipelineExecutor::new(graph, execution_config, cache_config)
}

pub fn create_execution_config() -> ExecutionConfig {
  ExecutionConfig::default()
}

pub fn create_cache_config() -> CacheConfig {
  CacheConfig::default()
}

pub fn create_pipeline_debugger() -> PipelineDebugger {
  PipelineDebugger::new()
}

pub fn create_pipeline_profiler() -> PipelineProfiler {
  PipelineProfiler::new()
}
