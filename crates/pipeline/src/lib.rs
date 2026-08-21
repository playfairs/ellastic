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
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod analysis;
pub mod caching;
pub mod debugging;
pub mod execution;
pub mod graph;
pub mod nodes;
pub mod optimization;
pub mod serialization;

pub use analysis::*;
pub use caching::*;
pub use debugging::*;
pub use execution::*;
pub use graph::*;
pub use nodes::*;
pub use optimization::*;
pub use serialization::*;

#[derive(Debug, Clone)]
pub struct PipelineProcessor {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub graph: PipelineGraph,
  pub execution_config: ExecutionConfig,
  pub optimization_config: OptimizationConfig,
  pub cache_config: CacheConfig,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
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
pub struct OptimizationConfig {
  pub enable_optimization: bool,
  pub optimization_level: OptimizationLevel,
  pub enable_dead_code_elimination: bool,
  pub enable_node_fusion: bool,
  pub enable_pipeline_parallelization: bool,
  pub enable_memory_optimization: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
  None,
  Basic,
  Standard,
  Aggressive,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
  pub enable_node_caching: bool,
  pub enable_graph_caching: bool,
  pub max_cache_size_mb: usize,
  pub cache_ttl_seconds: u64,
  pub enable_persistent_cache: bool,
}

impl PipelineProcessor {
  pub fn new(name: String, description: String) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name,
      description,
      graph: PipelineGraph::new(),
      execution_config: ExecutionConfig::default(),
      optimization_config: OptimizationConfig::default(),
      cache_config: CacheConfig::default(),
      created_at: now,
      updated_at: now,
    }
  }

  pub fn with_graph(mut self, graph: PipelineGraph) -> Self {
    self.graph = graph;
    self
  }

  pub fn with_execution_config(mut self, config: ExecutionConfig) -> Self {
    self.execution_config = config;
    self
  }

  pub fn with_optimization_config(mut self, config: OptimizationConfig) -> Self {
    self.optimization_config = config;
    self
  }

  pub fn with_cache_config(mut self, config: CacheConfig) -> Self {
    self.cache_config = config;
    self
  }

  pub fn graph(&self) -> &PipelineGraph {
    &self.graph
  }

  pub fn graph_mut(&mut self) -> &mut PipelineGraph {
    &mut self.graph
  }

  pub fn execution_config(&self) -> &ExecutionConfig {
    &self.execution_config
  }

  pub fn execution_config_mut(&mut self) -> &mut ExecutionConfig {
    &mut self.execution_config
  }

  pub fn optimization_config(&self) -> &OptimizationConfig {
    &self.optimization_config
  }

  pub fn optimization_config_mut(&mut self) -> &mut OptimizationConfig {
    &mut self.optimization_config
  }

  pub fn cache_config(&self) -> &CacheConfig {
    &self.cache_config
  }

  pub fn cache_config_mut(&mut self) -> &mut CacheConfig {
    &mut self.cache_config
  }

  pub fn add_node(&mut self, node: PipelineNode) -> Result<Uuid> {
    let node_id = self.graph.add_node(node)?;
    self.update_timestamp();
    Ok(node_id)
  }

  pub fn remove_node(&mut self, node_id: Uuid) -> Result<PipelineNode> {
    let node = self.graph.remove_node(node_id)?;
    self.update_timestamp();
    Ok(node)
  }

  pub fn add_edge(&mut self, from_node: Uuid, to_node: Uuid, edge_type: EdgeType) -> Result<Uuid> {
    let edge_id = self.graph.add_edge(from_node, to_node, edge_type)?;
    self.update_timestamp();
    Ok(edge_id)
  }

  pub fn remove_edge(&mut self, edge_id: Uuid) -> Result<PipelineEdge> {
    let edge = self.graph.remove_edge(edge_id)?;
    self.update_timestamp();
    Ok(edge)
  }

  pub fn execute(&mut self, input: MediaProcessor) -> Result<MediaProcessor> {
    let mut executor =
      PipelineExecutor::new(&self.graph, &self.execution_config, &self.cache_config);

    if self.optimization_config.enable_optimization {
      let optimizer = PipelineOptimizer::new(&self.optimization_config);
      executor = optimizer.optimize_executor(executor)?;
    }

    executor.execute(input)
  }

  pub fn execute_async(
    &mut self,
    input: MediaProcessor,
  ) -> Result<tokio::task::JoinHandle<Result<MediaProcessor>>> {
    let graph = self.graph.clone();
    let execution_config = self.execution_config.clone();
    let cache_config = self.cache_config.clone();
    let optimization_config = self.optimization_config.clone();

    let handle = tokio::spawn(async move {
      let mut executor = PipelineExecutor::new(&graph, &execution_config, &cache_config);

      if optimization_config.enable_optimization {
        let optimizer = PipelineOptimizer::new(&optimization_config);
        executor = optimizer.optimize_executor(executor)?;
      }

      executor.execute(input)
    });

    Ok(handle)
  }

  pub fn execute_with_progress(
    &mut self,
    input: MediaProcessor,
    progress_callback: Box<dyn PipelineProgressCallback + Send + Sync>,
  ) -> Result<MediaProcessor> {
    let mut executor =
      PipelineExecutor::new(&self.graph, &self.execution_config, &self.cache_config);
    executor.set_progress_callback(progress_callback);

    if self.optimization_config.enable_optimization {
      let optimizer = PipelineOptimizer::new(&self.optimization_config);
      executor = optimizer.optimize_executor(executor)?;
    }

    executor.execute(input)
  }

  pub fn validate(&self) -> Result<()> {
    self.graph.validate()
  }

  pub fn analyze(&self) -> Result<PipelineAnalysis> {
    let analyzer = PipelineAnalyzer::new(&self.graph);
    analyzer.analyze()
  }

  pub fn optimize(&mut self) -> Result<()> {
    if self.optimization_config.enable_optimization {
      let optimizer = PipelineOptimizer::new(&self.optimization_config);
      self.graph = optimizer.optimize_graph(&self.graph)?;
      self.update_timestamp();
    }
    Ok(())
  }

  pub fn serialize(&self) -> Result<Vec<u8>> {
    let serializer = PipelineSerializer::new();
    serializer.serialize(&self.graph)
  }

  pub fn deserialize(&mut self, data: &[u8]) -> Result<()> {
    let serializer = PipelineSerializer::new();
    self.graph = serializer.deserialize(data)?;
    self.update_timestamp();
    Ok(())
  }

  pub fn clone(&self) -> PipelineProcessor {
    PipelineProcessor {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      graph: self.graph.clone(),
      execution_config: self.execution_config.clone(),
      optimization_config: self.optimization_config.clone(),
      cache_config: self.cache_config.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
    }
  }

  fn update_timestamp(&mut self) {
    self.updated_at = Utc::now();
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

impl Default for OptimizationConfig {
  fn default() -> Self {
    Self {
      enable_optimization: true,
      optimization_level: OptimizationLevel::Standard,
      enable_dead_code_elimination: true,
      enable_node_fusion: true,
      enable_pipeline_parallelization: true,
      enable_memory_optimization: true,
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

pub trait PipelineProgressCallback: Send + Sync {
  fn on_node_start(&self, node_id: Uuid, node_name: &str);
  fn on_node_complete(&self, node_id: Uuid, node_name: &str, duration: std::time::Duration);
  fn on_node_error(&self, node_id: Uuid, node_name: &str, error: &str);
  fn on_pipeline_progress(&self, completed_nodes: u32, total_nodes: u32);
  fn on_pipeline_complete(&self, total_duration: std::time::Duration);
  fn on_pipeline_error(&self, error: &str);
}

#[derive(Debug, Clone)]
pub struct PipelineManager {
  processors: HashMap<Uuid, PipelineProcessor>,
  processors_by_name: HashMap<String, Uuid>,
  templates: HashMap<String, PipelineTemplate>,
  default_processor: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct PipelineTemplate {
  pub name: String,
  pub description: String,
  pub graph_template: PipelineGraph,
  pub parameters: HashMap<String, TemplateParameter>,
  pub supported_media_types: Vec<MediaType>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TemplateParameter {
  pub name: String,
  pub parameter_type: TemplateParameterType,
  pub default_value: TemplateParameterValue,
  pub description: String,
  pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateParameterType {
  String,
  Integer,
  Float,
  Boolean,
  Enum(Vec<String>),
  MediaType,
  EffectType,
}

#[derive(Debug, Clone)]
pub enum TemplateParameterValue {
  String(String),
  Integer(i64),
  Float(f64),
  Boolean(bool),
  Enum(String),
  MediaType(MediaType),
  EffectType(EffectType),
}

impl PipelineManager {
  pub fn new() -> Self {
    Self {
      processors: HashMap::new(),
      processors_by_name: HashMap::new(),
      templates: HashMap::new(),
      default_processor: None,
    }
  }

  pub fn register_processor(&mut self, processor: PipelineProcessor) -> Result<()> {
    if self.processors_by_name.contains_key(&processor.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Pipeline processor '{}' already registered",
        processor.name
      )));
    }

    let id = processor.id;
    self.processors_by_name.insert(processor.name.clone(), id);
    self.processors.insert(id, processor);

    if self.default_processor.is_none() {
      self.default_processor = Some(id);
    }

    Ok(())
  }

  pub fn unregister_processor(&mut self, id: Uuid) -> Option<PipelineProcessor> {
    if let Some(processor) = self.processors.remove(&id) {
      self.processors_by_name.remove(&processor.name);

      if self.default_processor == Some(id) {
        self.default_processor = self.processors.keys().next();
      }

      Some(processor)
    } else {
      None
    }
  }

  pub fn get_processor(&self, id: Uuid) -> Option<&PipelineProcessor> {
    self.processors.get(&id)
  }

  pub fn get_processor_by_name(&self, name: &str) -> Option<&PipelineProcessor> {
    self
      .processors_by_name
      .get(name)
      .and_then(|id| self.processors.get(id))
  }

  pub fn get_default_processor(&self) -> Option<&PipelineProcessor> {
    self
      .default_processor
      .and_then(|id| self.processors.get(&id))
  }

  pub fn list_processors(&self) -> Vec<&PipelineProcessor> {
    self.processors.values().collect()
  }

  pub fn search_processors(&self, query: &str) -> Vec<&PipelineProcessor> {
    let query = query.to_lowercase();
    self
      .processors
      .values()
      .filter(|processor| {
        processor.name.to_lowercase().contains(&query)
          || processor.description.to_lowercase().contains(&query)
      })
      .collect()
  }

  pub fn add_template(&mut self, template: PipelineTemplate) -> Result<()> {
    if self.templates.contains_key(&template.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Pipeline template '{}' already exists",
        template.name
      )));
    }

    self.templates.insert(template.name.clone(), template);
    Ok(())
  }

  pub fn remove_template(&mut self, name: &str) -> Option<PipelineTemplate> {
    self.templates.remove(name)
  }

  pub fn get_template(&self, name: &str) -> Option<&PipelineTemplate> {
    self.templates.get(name)
  }

  pub fn list_templates(&self) -> Vec<&PipelineTemplate> {
    self.templates.values().collect()
  }

  pub fn create_processor_from_template(
    &self,
    template_name: &str,
    name: String,
    description: String,
    parameters: HashMap<String, TemplateParameterValue>,
  ) -> Result<PipelineProcessor> {
    let template = self.get_template(template_name).ok_or_else(|| {
      EllasticError::InvalidParameter(format!("Template '{}' not found", template_name))
    })?;

    let mut graph = template.graph_template.clone();

    self.apply_template_parameters(&mut graph, &template.parameters, &parameters)?;

    let processor = PipelineProcessor::new(name, description).with_graph(graph);

    Ok(processor)
  }

  fn apply_template_parameters(
    &self,
    graph: &mut PipelineGraph,
    template_params: &HashMap<String, TemplateParameter>,
    provided_params: &HashMap<String, TemplateParameterValue>,
  ) -> Result<()> {
    for (name, param) in template_params {
      if param.required && !provided_params.contains_key(name) {
        return Err(EllasticError::InvalidParameter(format!(
          "Required template parameter '{}' not provided",
          name
        )));
      }
    }

    for node in graph.nodes_mut() {
      if let PipelineNodeType::TemplateNode {
        template_node_name, ..
      } = &node.node_type
      {
        if let Some(param_value) = provided_params.get(template_node_name) {
          self.apply_parameter_to_node(node, param_value)?;
        }
      }
    }

    Ok(())
  }

  fn apply_parameter_to_node(
    &self,
    node: &mut PipelineNode,
    param_value: &TemplateParameterValue,
  ) -> Result<()> {
    match param_value {
      TemplateParameterValue::String(value) => {
        if let Some(parameters) = node.parameters_mut() {
          parameters.insert("template_value".to_string(), value.clone());
        }
      }
      TemplateParameterValue::Integer(value) => {
        if let Some(parameters) = node.parameters_mut() {
          parameters.insert("template_value".to_string(), value.to_string());
        }
      }
      TemplateParameterValue::Float(value) => {
        if let Some(parameters) = node.parameters_mut() {
          parameters.insert("template_value".to_string(), value.to_string());
        }
      }
      TemplateParameterValue::Boolean(value) => {
        if let Some(parameters) = node.parameters_mut() {
          parameters.insert("template_value".to_string(), value.to_string());
        }
      }
      TemplateParameterValue::Enum(value) => {
        if let Some(parameters) = node.parameters_mut() {
          parameters.insert("template_value".to_string(), value.clone());
        }
      }
      TemplateParameterValue::MediaType(media_type) => {
        if let Some(parameters) = node.parameters_mut() {
          parameters.insert("media_type".to_string(), format!("{:?}", media_type));
        }
      }
      TemplateParameterValue::EffectType(effect_type) => {
        if let Some(parameters) = node.parameters_mut() {
          parameters.insert("effect_type".to_string(), format!("{:?}", effect_type));
        }
      }
    }

    Ok(())
  }

  pub fn clear(&mut self) {
    self.processors.clear();
    self.processors_by_name.clear();
    self.templates.clear();
    self.default_processor = None;
  }

  pub fn len(&self) -> usize {
    self.processors.len()
  }

  pub fn is_empty(&self) -> bool {
    self.processors.is_empty()
  }

  pub fn clone(&self) -> PipelineManager {
    PipelineManager {
      processors: self.processors.clone(),
      processors_by_name: self.processors_by_name.clone(),
      templates: self.templates.clone(),
      default_processor: self.default_processor,
    }
  }

  pub fn load_default_templates(&mut self) -> Result<()> {
    self.load_basic_templates()?;
    self.load_image_processing_templates()?;
    self.load_audio_processing_templates()?;
    self.load_glitch_templates()?;
    self.load_batch_processing_templates()?;

    Ok(())
  }

  fn load_basic_templates(&mut self) -> Result<()> {
    let templates = vec![
      PipelineTemplate {
        name: "simple_chain".to_string(),
        description: "Simple linear processing chain".to_string(),
        graph_template: PipelineGraph::new(),
        parameters: HashMap::new(),
        supported_media_types: vec![MediaType::Image, MediaType::Audio, MediaType::Video],
        created_at: Utc::now(),
      },
      PipelineTemplate {
        name: "parallel_processing".to_string(),
        description: "Parallel processing with multiple branches".to_string(),
        graph_template: PipelineGraph::new(),
        parameters: HashMap::new(),
        supported_media_types: vec![MediaType::Image, MediaType::Audio, MediaType::Video],
        created_at: Utc::now(),
      },
    ];

    for template in templates {
      self.add_template(template)?;
    }

    Ok(())
  }

  fn load_image_processing_templates(&mut self) -> Result<()> {
    let templates = vec![
      PipelineTemplate {
        name: "image_enhancement".to_string(),
        description: "Complete image enhancement pipeline".to_string(),
        graph_template: PipelineGraph::new(),
        parameters: [
          (
            "brightness".to_string(),
            TemplateParameter {
              name: "brightness".to_string(),
              parameter_type: TemplateParameterType::Float,
              default_value: TemplateParameterValue::Float(0.0),
              description: "Brightness adjustment".to_string(),
              required: false,
            },
          ),
          (
            "contrast".to_string(),
            TemplateParameter {
              name: "contrast".to_string(),
              parameter_type: TemplateParameterType::Float,
              default_value: TemplateParameterValue::Float(0.0),
              description: "Contrast adjustment".to_string(),
              required: false,
            },
          ),
          (
            "saturation".to_string(),
            TemplateParameter {
              name: "saturation".to_string(),
              parameter_type: TemplateParameterType::Float,
              default_value: TemplateParameterValue::Float(0.0),
              description: "Saturation adjustment".to_string(),
              required: false,
            },
          ),
        ]
        .into_iter()
        .collect(),
        supported_media_types: vec![MediaType::Image],
        created_at: Utc::now(),
      },
      PipelineTemplate {
        name: "glitch_art".to_string(),
        description: "Artistic glitch effects pipeline".to_string(),
        graph_template: PipelineGraph::new(),
        parameters: [
          (
            "glitch_intensity".to_string(),
            TemplateParameter {
              name: "glitch_intensity".to_string(),
              parameter_type: TemplateParameterType::Float,
              default_value: TemplateParameterValue::Float(0.5),
              description: "Glitch intensity level".to_string(),
              required: false,
            },
          ),
          (
            "glitch_type".to_string(),
            TemplateParameter {
              name: "glitch_type".to_string(),
              parameter_type: TemplateParameterType::Enum(vec![
                "pixel_sort".to_string(),
                "data_mosh".to_string(),
                "bit_crush".to_string(),
                "noise".to_string(),
              ]),
              default_value: TemplateParameterValue::Enum("pixel_sort".to_string()),
              description: "Type of glitch effect".to_string(),
              required: false,
            },
          ),
        ]
        .into_iter()
        .collect(),
        supported_media_types: vec![MediaType::Image],
        created_at: Utc::now(),
      },
    ];

    for template in templates {
      self.add_template(template)?;
    }

    Ok(())
  }

  fn load_audio_processing_templates(&mut self) -> Result<()> {
    let templates = vec![
      PipelineTemplate {
        name: "audio_cleanup".to_string(),
        description: "Audio cleanup and enhancement pipeline".to_string(),
        graph_template: PipelineGraph::new(),
        parameters: [
          (
            "noise_reduction".to_string(),
            TemplateParameter {
              name: "noise_reduction".to_string(),
              parameter_type: TemplateParameterType::Float,
              default_value: TemplateParameterValue::Float(0.5),
              description: "Noise reduction level".to_string(),
              required: false,
            },
          ),
          (
            "compression".to_string(),
            TemplateParameter {
              name: "compression".to_string(),
              parameter_type: TemplateParameterType::Float,
              default_value: TemplateParameterValue::Float(3.0),
              description: "Compression ratio".to_string(),
              required: false,
            },
          ),
        ]
        .into_iter()
        .collect(),
        supported_media_types: vec![MediaType::Audio],
        created_at: Utc::now(),
      },
      PipelineTemplate {
        name: "audio_glitch".to_string(),
        description: "Audio glitch effects pipeline".to_string(),
        graph_template: PipelineGraph::new(),
        parameters: [
          (
            "bit_crush_depth".to_string(),
            TemplateParameter {
              name: "bit_crush_depth".to_string(),
              parameter_type: TemplateParameterType::Integer,
              default_value: TemplateParameterValue::Integer(8),
              description: "Bit crush depth".to_string(),
              required: false,
            },
          ),
          (
            "distortion_amount".to_string(),
            TemplateParameter {
              name: "distortion_amount".to_string(),
              parameter_type: TemplateParameterType::Float,
              default_value: TemplateParameterValue::Float(0.5),
              description: "Distortion amount".to_string(),
              required: false,
            },
          ),
        ]
        .into_iter()
        .collect(),
        supported_media_types: vec![MediaType::Audio],
        created_at: Utc::now(),
      },
    ];

    for template in templates {
      self.add_template(template)?;
    }

    Ok(())
  }

  fn load_glitch_templates(&mut self) -> Result<()> {
    let templates = vec![PipelineTemplate {
      name: "extreme_glitch".to_string(),
      description: "Extreme glitch effects pipeline".to_string(),
      graph_template: PipelineGraph::new(),
      parameters: [
        (
          "corruption_level".to_string(),
          TemplateParameter {
            name: "corruption_level".to_string(),
            parameter_type: TemplateParameterType::Float,
            default_value: TemplateParameterValue::Float(0.8),
            description: "Data corruption level".to_string(),
            required: false,
          },
        ),
        (
          "randomness".to_string(),
          TemplateParameter {
            name: "randomness".to_string(),
            parameter_type: TemplateParameterType::Float,
            default_value: TemplateParameterValue::Float(1.0),
            description: "Randomness factor".to_string(),
            required: false,
          },
        ),
      ]
      .into_iter()
      .collect(),
      supported_media_types: vec![MediaType::Image, MediaType::Audio, MediaType::Video],
      created_at: Utc::now(),
    }];

    for template in templates {
      self.add_template(template)?;
    }

    Ok(())
  }

  fn load_batch_processing_templates(&mut self) -> Result<()> {
    let templates = vec![PipelineTemplate {
      name: "batch_enhancement".to_string(),
      description: "Batch enhancement pipeline".to_string(),
      graph_template: PipelineGraph::new(),
      parameters: [
        (
          "batch_size".to_string(),
          TemplateParameter {
            name: "batch_size".to_string(),
            parameter_type: TemplateParameterType::Integer,
            default_value: TemplateParameterValue::Integer(10),
            description: "Batch processing size".to_string(),
            required: false,
          },
        ),
        (
          "parallel_processing".to_string(),
          TemplateParameter {
            name: "parallel_processing".to_string(),
            parameter_type: TemplateParameterType::Boolean,
            default_value: TemplateParameterValue::Boolean(true),
            description: "Enable parallel processing".to_string(),
            required: false,
          },
        ),
      ]
      .into_iter()
      .collect(),
      supported_media_types: vec![MediaType::Image, MediaType::Audio, MediaType::Video],
      created_at: Utc::now(),
    }];

    for template in templates {
      self.add_template(template)?;
    }

    Ok(())
  }
}

pub fn create_pipeline_processor(name: String, description: String) -> PipelineProcessor {
  PipelineProcessor::new(name, description)
}

pub fn create_execution_config() -> ExecutionConfig {
  ExecutionConfig::default()
}

pub fn create_optimization_config() -> OptimizationConfig {
  OptimizationConfig::default()
}

pub fn create_cache_config() -> CacheConfig {
  CacheConfig::default()
}

pub fn create_pipeline_manager() -> PipelineManager {
  PipelineManager::new()
}

pub fn create_pipeline_template(
  name: String,
  description: String,
  graph_template: PipelineGraph,
  parameters: HashMap<String, TemplateParameter>,
  supported_media_types: Vec<MediaType>,
) -> PipelineTemplate {
  PipelineTemplate {
    name,
    description,
    graph_template,
    parameters,
    supported_media_types,
    created_at: Utc::now(),
  }
}
