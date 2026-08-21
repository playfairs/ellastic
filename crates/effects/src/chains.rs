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

#[derive(Debug, Clone)]
pub struct EffectChain {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub steps: Vec<ChainStep>,
  pub parameters: HashMap<String, String>,
  pub tags: Vec<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub version: String,
  pub author: String,
  pub supported_media_types: Vec<MediaType>,
}

#[derive(Debug, Clone)]
pub struct ChainStep {
  pub id: Uuid,
  pub effect_type: String,
  pub parameters: HashMap<String, String>,
  pub enabled: bool,
  pub weight: f32,
  pub condition: Option<ChainCondition>,
  pub dependencies: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub enum ChainCondition {
  MediaCondition {
    media_type: MediaType,
  },
  ParameterCondition {
    parameter: String,
    operator: ConditionOperator,
    value: String,
  },
  SizeCondition {
    width: Option<u32>,
    height: Option<u32>,
  },
  CustomCondition {
    condition_function: Box<dyn Fn(&MediaProcessor) -> bool + Send + Sync>,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionOperator {
  Equals,
  NotEquals,
  GreaterThan,
  LessThan,
  GreaterThanOrEqual,
  LessThanOrEqual,
  Contains,
  NotContains,
}

#[derive(Debug, Clone)]
pub enum ChainExecutionMode {
  Sequential,
  Parallel,
  Conditional,
}

#[derive(Debug, Clone)]
pub struct ChainProcessor {
  chain: EffectChain,
  execution_mode: ChainExecutionMode,
  cache: Arc<RwLock<ChainCache>>,
  performance_stats: Arc<RwLock<ChainPerformanceStats>>,
}

impl EffectChain {
  pub fn new(name: String, description: String) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name,
      description,
      steps: Vec::new(),
      parameters: HashMap::new(),
      tags: Vec::new(),
      created_at: now,
      updated_at: now,
      version: "1.0.0".to_string(),
      author: "Ellastic Team".to_string(),
      supported_media_types: Vec::new(),
    }
  }

  pub fn with_steps(mut self, steps: Vec<ChainStep>) -> Self {
    self.steps = steps;
    self
  }

  pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
    self.parameters = parameters;
    self
  }

  pub fn with_tags(mut self, tags: Vec<String>) -> Self {
    self.tags = tags;
    self
  }

  pub fn with_supported_media_types(mut self, media_types: Vec<MediaType>) -> Self {
    self.supported_media_types = media_types;
    self
  }

  pub fn add_step(&mut self, step: ChainStep) {
    self.steps.push(step);
    self.update_timestamp();
  }

  pub fn remove_step(&mut self, id: Uuid) -> Option<ChainStep> {
    if let Some(index) = self.steps.iter().position(|step| step.id == id) {
      let step = self.steps.remove(index);
      self.update_timestamp();
      Some(step)
    } else {
      None
    }
  }

  pub fn get_step(&self, id: Uuid) -> Option<&ChainStep> {
    self.steps.iter().find(|step| step.id == id)
  }

  pub fn get_step_mut(&mut self, id: Uuid) -> Option<&mut ChainStep> {
    self.steps.iter_mut().find(|step| step.id == id)
  }

  pub fn reorder_steps(&mut self, new_order: &[Uuid]) -> Result<()> {
    if new_order.len() != self.steps.len() {
      return Err(EllasticError::InvalidParameter(
        "New order length doesn't match steps length".to_string(),
      ));
    }

    let new_steps: Vec<_> = new_order
      .iter()
      .map(|id| {
        self
          .steps
          .iter()
          .find(|step| step.id == *id)
          .cloned()
          .ok_or_else(|| EllasticError::InvalidParameter(format!("Step not found: {}", id)))
      })
      .collect::<Result<_>>()?;

    self.steps = new_steps;
    self.update_timestamp();
    Ok(())
  }

  pub fn validate(&self) -> Result<()> {
    for step in &self.steps {
      for dependency_id in &step.dependencies {
        if !self.steps.iter().any(|s| s.id == *dependency_id) {
          return Err(EllasticError::InvalidParameter(format!(
            "Dependency not found: {}",
            dependency_id
          )));
        }
      }
    }

    Ok(())
  }

  pub fn update_timestamp(&mut self) {
    self.updated_at = Utc::now();
  }

  pub fn clone(&self) -> EffectChain {
    EffectChain {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      steps: self.steps.clone(),
      parameters: self.parameters.clone(),
      tags: self.tags.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
      version: self.version.clone(),
      author: self.author.clone(),
      supported_media_types: self.supported_media_types.clone(),
    }
  }
}

impl ChainStep {
  pub fn new(effect_type: String, parameters: HashMap<String, String>) -> Self {
    Self {
      id: Uuid::new_v4(),
      effect_type,
      parameters,
      enabled: true,
      weight: 1.0,
      condition: None,
      dependencies: Vec::new(),
    }
  }

  pub fn with_weight(mut self, weight: f32) -> Self {
    self.weight = weight;
    self
  }

  pub fn with_condition(mut self, condition: ChainCondition) -> Self {
    self.condition = Some(condition);
    self
  }

  pub fn with_dependencies(mut self, dependencies: Vec<Uuid>) -> Self {
    self.dependencies = dependencies;
    self
  }

  pub fn enable(mut self) -> Self {
    self.enabled = true;
    self
  }

  pub fn disable(mut self) -> Self {
    self.enabled = false;
    self
  }

  pub fn clone(&self) -> ChainStep {
    ChainStep {
      id: self.id,
      effect_type: self.effect_type.clone(),
      parameters: self.parameters.clone(),
      enabled: self.enabled,
      weight: self.weight,
      condition: self.condition.clone(),
      dependencies: self.dependencies.clone(),
    }
  }
}

impl ChainProcessor {
  pub fn new(chain: EffectChain) -> Self {
    Self {
      chain,
      execution_mode: ChainExecutionMode::Sequential,
      cache: Arc::new(RwLock::new(ChainCache::new())),
      performance_stats: Arc::new(RwLock::new(ChainPerformanceStats::new())),
    }
  }

  pub fn with_execution_mode(mut self, execution_mode: ChainExecutionMode) -> Self {
    self.execution_mode = execution_mode;
    self
  }

  pub fn chain(&self) -> &EffectChain {
    &self.chain
  }

  pub fn chain_mut(&mut self) -> &mut EffectChain {
    &mut self.chain
  }

  pub fn execution_mode(&self) -> ChainExecutionMode {
    self.execution_mode
  }

  pub fn set_execution_mode(&mut self, execution_mode: ChainExecutionMode) {
    self.execution_mode = execution_mode;
  }

  pub fn cache(&self) -> Arc<RwLock<ChainCache>> {
    self.cache.clone()
  }

  pub fn performance_stats(&self) -> Arc<RwLock<ChainPerformanceStats>> {
    self.performance_stats.clone()
  }

  pub fn execute_chain(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
    let start_time = std::time::Instant::now();

    self.validate_media_type(media_processor)?;
    self.chain.validate()?;

    let cache_key = self.generate_cache_key(media_processor);

    if let Some(cached_result) = self.cache.read().get(&cache_key) {
      *media_processor = cached_result.clone();
      return Ok(());
    }

    let mut current_processor = media_processor.clone();

    match self.execution_mode {
      ChainExecutionMode::Sequential => {
        self.execute_sequential(&mut current_processor)?;
      }
      ChainExecutionMode::Parallel => {
        self.execute_parallel(&mut current_processor)?;
      }
      ChainExecutionMode::Conditional => {
        self.execute_conditional(&mut current_processor)?;
      }
    }

    *media_processor = current_processor;

    let processing_time = start_time.elapsed();
    self
      .performance_stats
      .write()
      .record_execution(processing_time);

    self.cache.write().put(cache_key, media_processor.clone());

    Ok(())
  }

  pub fn execute_chain_async(
    &mut self,
    media_processor: MediaProcessor,
  ) -> Result<tokio::task::JoinHandle<Result<MediaProcessor>>> {
    let chain = self.chain.clone();
    let execution_mode = self.execution_mode;
    let cache = self.cache.clone();
    let performance_stats = self.performance_stats.clone();

    let handle = tokio::spawn(async move {
      let start_time = std::time::Instant::now();

      let mut processor = media_processor;

      let cache_key = Self::generate_cache_key_static(&chain, &processor);

      if let Some(cached_result) = cache.read().get(&cache_key) {
        return Ok(cached_result);
      }

      match execution_mode {
        ChainExecutionMode::Sequential => {
          Self::execute_sequential_static(&mut processor, &chain)?;
        }
        ChainExecutionMode::Parallel => {
          Self::execute_parallel_static(&mut processor, &chain)?;
        }
        ChainExecutionMode::Conditional => {
          Self::execute_conditional_static(&mut processor, &chain)?;
        }
      }

      let processing_time = start_time.elapsed();
      performance_stats.write().record_execution(processing_time);

      cache.write().put(cache_key, processor.clone());

      Ok(processor)
    });

    Ok(handle)
  }

  pub fn preview_chain(
    &mut self,
    media_processor: &mut MediaProcessor,
    preview_size: (u32, u32),
  ) -> Result<()> {
    let original_size = (media_processor.width(), media_processor.height());

    if original_size != preview_size {
      media_processor.resize(preview_size.0, preview_size.1)?;
    }

    self.execute_chain(media_processor)?;

    if original_size != preview_size {
      media_processor.resize(original_size.0, original_size.1)?;
    }

    Ok(())
  }

  pub fn batch_execute_chain(
    &mut self,
    media_processors: &mut [MediaProcessor],
  ) -> Result<Vec<Result<()>>> {
    media_processors
      .par_iter_mut()
      .map(|processor| {
        let mut temp_processor = processor.clone();
        let mut temp_chain_processor = ChainProcessor::new(self.chain.clone());
        temp_chain_processor.execution_mode = self.execution_mode;

        match temp_chain_processor.execute_chain(&mut temp_processor) {
          Ok(()) => {
            *processor = temp_processor;
            Ok(())
          }
          Err(e) => Err(e),
        }
      })
      .collect()
  }

  pub fn batch_execute_chain_async(
    &mut self,
    media_processors: Vec<MediaProcessor>,
  ) -> Result<Vec<tokio::task::JoinHandle<Result<MediaProcessor>>>> {
    let chain = self.chain.clone();
    let execution_mode = self.execution_mode;
    let cache = self.cache.clone();
    let performance_stats = self.performance_stats.clone();

    let handles: Vec<_> = media_processors
      .into_iter()
      .map(|processor| {
        let chain = chain.clone();
        let execution_mode = execution_mode;
        let cache = cache.clone();
        let performance_stats = performance_stats.clone();

        tokio::spawn(async move {
          let start_time = std::time::Instant::now();

          let mut temp_processor = processor;

          let cache_key = Self::generate_cache_key_static(&chain, &temp_processor);

          if let Some(cached_result) = cache.read().get(&cache_key) {
            return Ok(cached_result);
          }

          match execution_mode {
            ChainExecutionMode::Sequential => {
              Self::execute_sequential_static(&mut temp_processor, &chain)?;
            }
            ChainExecutionMode::Parallel => {
              Self::execute_parallel_static(&mut temp_processor, &chain)?;
            }
            ChainExecutionMode::Conditional => {
              Self::execute_conditional_static(&mut temp_processor, &chain)?;
            }
          }

          let processing_time = start_time.elapsed();
          performance_stats.write().record_execution(processing_time);

          cache.write().put(cache_key, temp_processor.clone());

          Ok(temp_processor)
        })
      })
      .collect();

    Ok(handles)
  }

  fn validate_media_type(&self, media_processor: &MediaProcessor) -> Result<()> {
    if !self
      .chain
      .supported_media_types
      .contains(&media_processor.media_type())
    {
      return Err(EllasticError::InvalidParameter(format!(
        "Chain does not support media type: {:?}",
        media_processor.media_type()
      )));
    }

    Ok(())
  }

  fn generate_cache_key(&self, media_processor: &MediaProcessor) -> String {
    Self::generate_cache_key_static(&self.chain, media_processor)
  }

  fn generate_cache_key_static(chain: &EffectChain, media_processor: &MediaProcessor) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{
      Hash,
      Hasher,
    };

    let mut hasher = DefaultHasher::new();

    chain.id.hash(&mut hasher);

    for step in &chain.steps {
      if step.enabled {
        step.id.hash(&mut hasher);
        step.effect_type.hash(&mut hasher);
        step.weight.to_bits().hash(&mut hasher);

        let mut param_keys: Vec<_> = step.parameters.keys().collect();
        param_keys.sort();

        for key in param_keys {
          key.hash(&mut hasher);
          step.parameters[key].hash(&mut hasher);
        }

        for dependency_id in &step.dependencies {
          dependency_id.hash(&mut hasher);
        }
      }
    }

    media_processor.data().hash(&mut hasher);

    format!(
      "{}:{}:{}",
      chain.id,
      hasher.finish(),
      media_processor.data().len()
    )
  }

  fn execute_sequential(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
    let execution_order = self.calculate_execution_order()?;

    for step_id in execution_order {
      if let Some(step) = self.chain.steps.iter().find(|s| s.id == step_id) {
        if step.enabled && self.evaluate_condition(&step.condition, media_processor)? {
          self.execute_step(media_processor, step)?;
        }
      }
    }

    Ok(())
  }

  fn execute_parallel(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
    let execution_groups = self.calculate_parallel_groups()?;

    for group in execution_groups {
      let mut group_processors = Vec::new();

      for step_id in group {
        if let Some(step) = self.chain.steps.iter().find(|s| s.id == step_id) {
          if step.enabled && self.evaluate_condition(&step.condition, media_processor)? {
            let mut temp_processor = media_processor.clone();
            self.execute_step(&mut temp_processor, step)?;
            group_processors.push(temp_processor);
          }
        }
      }

      if !group_processors.is_empty() {
        let combined_processor = self.combine_parallel_results(group_processors)?;
        *media_processor = combined_processor;
      }
    }

    Ok(())
  }

  fn execute_conditional(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
    let execution_order = self.calculate_execution_order()?;

    for step_id in execution_order {
      if let Some(step) = self.chain.steps.iter().find(|s| s.id == step_id) {
        if step.enabled {
          if self.evaluate_condition(&step.condition, media_processor)? {
            self.execute_step(media_processor, step)?;
          }
        }
      }
    }

    Ok(())
  }

  fn execute_sequential_static(
    media_processor: &mut MediaProcessor,
    chain: &EffectChain,
  ) -> Result<()> {
    let execution_order = Self::calculate_execution_order_static(chain)?;

    for step_id in execution_order {
      if let Some(step) = chain.steps.iter().find(|s| s.id == step_id) {
        if step.enabled && Self::evaluate_condition_static(&step.condition, media_processor)? {
          Self::execute_step_static(media_processor, step)?;
        }
      }
    }

    Ok(())
  }

  fn execute_parallel_static(
    media_processor: &mut MediaProcessor,
    chain: &EffectChain,
  ) -> Result<()> {
    let execution_groups = Self::calculate_parallel_groups_static(chain)?;

    for group in execution_groups {
      let mut group_processors = Vec::new();

      for step_id in group {
        if let Some(step) = chain.steps.iter().find(|s| s.id == step_id) {
          if step.enabled && Self::evaluate_condition_static(&step.condition, media_processor)? {
            let mut temp_processor = media_processor.clone();
            Self::execute_step_static(&mut temp_processor, step)?;
            group_processors.push(temp_processor);
          }
        }
      }

      if !group_processors.is_empty() {
        let combined_processor = Self::combine_parallel_results_static(group_processors)?;
        *media_processor = combined_processor;
      }
    }

    Ok(())
  }

  fn execute_conditional_static(
    media_processor: &mut MediaProcessor,
    chain: &EffectChain,
  ) -> Result<()> {
    let execution_order = Self::calculate_execution_order_static(chain)?;

    for step_id in execution_order {
      if let Some(step) = chain.steps.iter().find(|s| s.id == step_id) {
        if step.enabled {
          if Self::evaluate_condition_static(&step.condition, media_processor)? {
            Self::execute_step_static(media_processor, step)?;
          }
        }
      }
    }

    Ok(())
  }

  fn calculate_execution_order(&self) -> Result<Vec<Uuid>> {
    Self::calculate_execution_order_static(&self.chain)
  }

  fn calculate_execution_order_static(chain: &EffectChain) -> Result<Vec<Uuid>> {
    let mut order = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let step_map: HashMap<Uuid, &ChainStep> =
      chain.steps.iter().map(|step| (step.id, step)).collect();

    for step in &chain.steps {
      if !visited.contains(&step.id) {
        Self::visit_step(step.id, &step_map, &mut visited, &mut order)?;
      }
    }

    Ok(order)
  }

  fn visit_step(
    step_id: Uuid,
    step_map: &HashMap<Uuid, &ChainStep>,
    visited: &mut std::collections::HashSet<Uuid>,
    order: &mut Vec<Uuid>,
  ) -> Result<()> {
    if visited.contains(&step_id) {
      return Ok(());
    }

    if let Some(step) = step_map.get(&step_id) {
      for dependency_id in &step.dependencies {
        Self::visit_step(*dependency_id, step_map, visited, order)?;
      }

      visited.insert(step_id);
      order.push(step_id);
    }

    Ok(())
  }

  fn calculate_parallel_groups(&self) -> Result<Vec<Vec<Uuid>>> {
    Self::calculate_parallel_groups_static(&self.chain)
  }

  fn calculate_parallel_groups_static(chain: &EffectChain) -> Result<Vec<Vec<Uuid>>> {
    let mut groups = Vec::new();
    let mut processed = std::collections::HashSet::new();
    let step_map: HashMap<Uuid, &ChainStep> =
      chain.steps.iter().map(|step| (step.id, step)).collect();

    while processed.len() < chain.steps.len() {
      let mut current_group = Vec::new();

      for step in &chain.steps {
        if !processed.contains(&step.id) {
          let dependencies_met = step
            .dependencies
            .iter()
            .all(|dep_id| processed.contains(dep_id));

          if dependencies_met {
            current_group.push(step.id);
          }
        }
      }

      if current_group.is_empty() {
        return Err(EllasticError::InvalidParameter(
          "Circular dependency detected in chain".to_string(),
        ));
      }

      for step_id in &current_group {
        processed.insert(*step_id);
      }

      groups.push(current_group);
    }

    Ok(groups)
  }

  fn evaluate_condition(
    &self,
    condition: &Option<ChainCondition>,
    media_processor: &MediaProcessor,
  ) -> Result<bool> {
    Self::evaluate_condition_static(condition, media_processor)
  }

  fn evaluate_condition_static(
    condition: &Option<ChainCondition>,
    media_processor: &MediaProcessor,
  ) -> Result<bool> {
    match condition {
      Some(ChainCondition::MediaCondition { media_type }) => {
        Ok(&media_processor.media_type() == media_type)
      }
      Some(ChainCondition::ParameterCondition {
        parameter,
        operator,
        value,
      }) => {
        let param_value = media_processor
          .get_metadata_parameter(parameter)
          .ok_or_else(|| {
            EllasticError::InvalidParameter(format!("Parameter not found: {}", parameter))
          })?;

        let result = match operator {
          ConditionOperator::Equals => param_value == value,
          ConditionOperator::NotEquals => param_value != value,
          ConditionOperator::GreaterThan => param_value > value,
          ConditionOperator::LessThan => param_value < value,
          ConditionOperator::GreaterThanOrEqual => param_value >= value,
          ConditionOperator::LessThanOrEqual => param_value <= value,
          ConditionOperator::Contains => param_value.contains(value),
          ConditionOperator::NotContains => !param_value.contains(value),
        };

        Ok(result)
      }
      Some(ChainCondition::SizeCondition { width, height }) => {
        let (actual_width, actual_height) = (media_processor.width(), media_processor.height());

        let width_ok = width.map_or(true, |w| actual_width == w);
        let height_ok = height.map_or(true, |h| actual_height == h);

        Ok(width_ok && height_ok)
      }
      Some(ChainCondition::CustomCondition { condition_function }) => {
        Ok(condition_function(media_processor))
      }
      None => Ok(true),
    }
  }

  fn execute_step(&mut self, media_processor: &mut MediaProcessor, step: &ChainStep) -> Result<()> {
    Self::execute_step_static(media_processor, step)
  }

  fn execute_step_static(media_processor: &mut MediaProcessor, step: &ChainStep) -> Result<()> {
    match step.effect_type.as_str() {
      "brightness" => {
        if let Some(amount) = step.parameters.get("amount") {
          let amount = amount.parse::<f32>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid brightness amount".to_string())
          })?;
          media_processor.adjust_brightness(amount * step.weight)?;
        }
      }
      "contrast" => {
        if let Some(amount) = step.parameters.get("amount") {
          let amount = amount
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid contrast amount".to_string()))?;
          media_processor.adjust_contrast(amount * step.weight)?;
        }
      }
      "saturation" => {
        if let Some(amount) = step.parameters.get("amount") {
          let amount = amount.parse::<f32>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid saturation amount".to_string())
          })?;
          media_processor.adjust_saturation(amount * step.weight)?;
        }
      }
      "gamma" => {
        if let Some(gamma) = step.parameters.get("gamma") {
          let gamma = gamma
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid gamma value".to_string()))?;
          media_processor.adjust_gamma(gamma)?;
        }
      }
      "blur" => {
        if let Some(radius) = step.parameters.get("radius") {
          let radius = radius
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid blur radius".to_string()))?;
          media_processor.blur(radius * step.weight)?;
        }
      }
      "sharpen" => {
        if let Some(amount) = step.parameters.get("amount") {
          let amount = amount
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid sharpen amount".to_string()))?;
          media_processor.sharpen(amount * step.weight)?;
        }
      }
      "edge_detection" => {
        media_processor.edge_detection()?;
      }
      "emboss" => {
        media_processor.emboss()?;
      }
      "grayscale" => {
        media_processor.grayscale()?;
      }
      "sepia" => {
        media_processor.sepia()?;
      }
      "invert" => {
        media_processor.invert()?;
      }
      "hue_rotate" => {
        if let Some(angle) = step.parameters.get("angle") {
          let angle = angle
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid hue rotate angle".to_string()))?;
          media_processor.hue_rotate(angle * step.weight)?;
        }
      }
      "reverb" => {
        if let Some(room_size) = step.parameters.get("room_size") {
          let room_size = room_size
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid reverb room_size".to_string()))?;
          media_processor.reverb(room_size * step.weight)?;
        }
      }
      "echo" => {
        if let Some(delay) = step.parameters.get("delay") {
          let delay = delay
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid echo delay".to_string()))?;
          media_processor.echo(delay * step.weight)?;
        }
      }
      "distortion" => {
        if let Some(amount) = step.parameters.get("amount") {
          let amount = amount.parse::<f32>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid distortion amount".to_string())
          })?;
          media_processor.distortion(amount * step.weight)?;
        }
      }
      "compressor" => {
        if let Some(ratio) = step.parameters.get("ratio") {
          let ratio = ratio
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid compressor ratio".to_string()))?;
          media_processor.compressor(ratio)?;
        }
      }
      "low_pass" => {
        if let Some(cutoff) = step.parameters.get("cutoff") {
          let cutoff = cutoff
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid low_pass cutoff".to_string()))?;
          media_processor.low_pass_filter(cutoff)?;
        }
      }
      "high_pass" => {
        if let Some(cutoff) = step.parameters.get("cutoff") {
          let cutoff = cutoff
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid high_pass cutoff".to_string()))?;
          media_processor.high_pass_filter(cutoff)?;
        }
      }
      "rotate" => {
        if let Some(angle) = step.parameters.get("angle") {
          let angle = angle
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid rotate angle".to_string()))?;
          media_processor.rotate(angle * step.weight)?;
        }
      }
      "scale" => {
        if let (Some(scale_x), Some(scale_y)) = (
          step.parameters.get("scale_x"),
          step.parameters.get("scale_y"),
        ) {
          let scale_x = scale_x
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid scale_x".to_string()))?;
          let scale_y = scale_y
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid scale_y".to_string()))?;
          media_processor.scale(scale_x * step.weight, scale_y * step.weight)?;
        }
      }
      "flip" => {
        if let Some(direction) = step.parameters.get("direction") {
          match direction.as_str() {
            "horizontal" => media_processor.flip_horizontal()?,
            "vertical" => media_processor.flip_vertical()?,
            "both" => {
              media_processor.flip_horizontal()?;
              media_processor.flip_vertical()?;
            }
            _ => {
              return Err(EllasticError::InvalidParameter(
                "Invalid flip direction".to_string(),
              ));
            }
          }
        }
      }
      "crop" => {
        if let (Some(x), Some(y), Some(width), Some(height)) = (
          step.parameters.get("x"),
          step.parameters.get("y"),
          step.parameters.get("width"),
          step.parameters.get("height"),
        ) {
          let x = x
            .parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid crop x".to_string()))?;
          let y = y
            .parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid crop y".to_string()))?;
          let width = width
            .parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid crop width".to_string()))?;
          let height = height
            .parse::<u32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid crop height".to_string()))?;
          media_processor.crop(x, y, width, height)?;
        }
      }
      "pixel_sort" => {
        if let Some(threshold) = step.parameters.get("threshold") {
          let threshold = threshold.parse::<f32>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid pixel_sort threshold".to_string())
          })?;
          let mode = step
            .parameters
            .get("mode")
            .map(|s| match s.as_str() {
              "brightness" => crate::effects::PixelSortMode::Brightness,
              "hue" => crate::effects::PixelSortMode::Hue,
              "saturation" => crate::effects::PixelSortMode::Saturation,
              "random" => crate::effects::PixelSortMode::Random,
              _ => crate::effects::PixelSortMode::Brightness,
            })
            .unwrap_or(crate::effects::PixelSortMode::Brightness);

          Self::apply_pixel_sort_static(media_processor, threshold * step.weight, mode)?;
        }
      }
      "data_mosh" => {
        if let Some(intensity) = step.parameters.get("intensity") {
          let intensity = intensity.parse::<f32>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid data_mosh intensity".to_string())
          })?;
          let preserve_size = step
            .parameters
            .get("preserve_size")
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);

          Self::apply_data_mosh_static(media_processor, intensity * step.weight, preserve_size)?;
        }
      }
      "bit_crush" => {
        if let Some(bit_depth) = step.parameters.get("bit_depth") {
          let bit_depth = bit_depth.parse::<u8>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid bit_crush bit_depth".to_string())
          })?;
          let sample_rate_reduction = step
            .parameters
            .get("sample_rate_reduction")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);

          media_processor.bit_crush(bit_depth, sample_rate_reduction)?;
        }
      }
      "frame_duplication" => {
        if let Some(count) = step.parameters.get("count") {
          let count = count.parse::<u32>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid frame_duplication count".to_string())
          })?;
          media_processor.duplicate_frames(count)?;
        }
      }
      "frame_dropping" => {
        if let Some(count) = step.parameters.get("count") {
          let count = count.parse::<u32>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid frame_dropping count".to_string())
          })?;
          media_processor.drop_frames(count)?;
        }
      }
      "time_stretch" => {
        if let Some(ratio) = step.parameters.get("ratio") {
          let ratio = ratio.parse::<f32>().map_err(|_| {
            EllasticError::InvalidParameter("Invalid time_stretch ratio".to_string())
          })?;
          media_processor.time_stretch(ratio)?;
        }
      }
      "reverse_playback" => {
        media_processor.reverse_playback()?;
      }
      "blend" => {
        if let (Some(overlay_path), Some(mode), Some(mix_ratio)) = (
          step.parameters.get("overlay"),
          step.parameters.get("mode"),
          step.parameters.get("mix_ratio"),
        ) {
          let overlay_processor = MediaProcessor::from_file(overlay_path)?;
          let blend_mode = match mode.as_str() {
            "add" => ellastic_media::BlendMode::Add,
            "multiply" => ellastic_media::BlendMode::Multiply,
            "screen" => ellastic_media::BlendMode::Screen,
            "overlay" => ellastic_media::BlendMode::Overlay,
            "difference" => ellastic_media::BlendMode::Difference,
            _ => {
              return Err(EllasticError::InvalidParameter(
                "Invalid blend mode".to_string(),
              ));
            }
          };

          let mix_ratio = mix_ratio
            .parse::<f32>()
            .map_err(|_| EllasticError::InvalidParameter("Invalid blend mix_ratio".to_string()))?;

          media_processor.blend(&overlay_processor, blend_mode, mix_ratio * step.weight)?;
        }
      }
      "composite" => {
        if let Some(composite_path) = step.parameters.get("composite") {
          let composite_processor = MediaProcessor::from_file(composite_path)?;
          media_processor.composite(&composite_processor)?;
        }
      }
      "mask" => {
        if let Some(mask_path) = step.parameters.get("mask") {
          let mask_processor = MediaProcessor::from_file(mask_path)?;
          media_processor.apply_mask(&mask_processor)?;
        }
      }
      _ => {
        return Err(EllasticError::UnsupportedOperation(format!(
          "Unknown chain step effect: {}",
          step.effect_type
        )));
      }
    }

    Ok(())
  }

  fn combine_parallel_results(&self, processors: Vec<MediaProcessor>) -> Result<MediaProcessor> {
    Self::combine_parallel_results_static(processors)
  }

  fn combine_parallel_results_static(processors: Vec<MediaProcessor>) -> Result<MediaProcessor> {
    if processors.is_empty() {
      return Err(EllasticError::InvalidParameter(
        "No processors to combine".to_string(),
      ));
    }

    if processors.len() == 1 {
      return Ok(processors.into_iter().next().unwrap());
    }

    let mut result = processors[0].clone();

    for processor in processors.iter().skip(1) {
      result.blend(processor, ellastic_media::BlendMode::Overlay, 0.5)?;
    }

    Ok(result)
  }

  fn apply_pixel_sort_static(
    media_processor: &mut MediaProcessor,
    threshold: f32,
    mode: crate::effects::PixelSortMode,
  ) -> Result<()> {
    if let Some(image_processor) = media_processor.image_processor_mut() {
      let image_data = image_processor.data();
      let mut new_image_data = image_data.clone();
      let data = &mut new_image_data.data;
      let width = image_data.width;
      let height = image_data.height;
      let channels = image_data.channels as usize;

      let mut pixels = Vec::new();

      for y in 0..height {
        for x in 0..width {
          let pixel_index = (y * width + x) * channels as u32;
          let pixel_start = pixel_index as usize;

          if pixel_start + channels <= data.len() {
            let pixel_data = data[pixel_start..pixel_start + channels].to_vec();
            let brightness = pixel_data.iter().take(3).sum::<u8>() as f32 / 3.0;

            if brightness >= threshold {
              pixels.push((x, y, pixel_data));
            }
          }
        }
      }

      match mode {
        crate::effects::PixelSortMode::Brightness => {
          pixels.sort_by(|_, a, b| {
            let brightness_a = a.2.iter().take(3).sum::<u8>() as f32 / 3.0;
            let brightness_b = b.2.iter().take(3).sum::<u8>() as f32 / 3.0;
            brightness_a
              .partial_cmp(&brightness_b)
              .unwrap_or(std::cmp::Ordering::Equal)
          });
        }
        crate::effects::PixelSortMode::Hue => {
          pixels.sort_by(|_, a, b| {
            let hue_a = Self::calculate_hue_static(&a.2);
            let hue_b = Self::calculate_hue_static(&b.2);
            hue_a
              .partial_cmp(&hue_b)
              .unwrap_or(std::cmp::Ordering::Equal)
          });
        }
        crate::effects::PixelSortMode::Saturation => {
          pixels.sort_by(|_, a, b| {
            let sat_a = Self::calculate_saturation_static(&a.2);
            let sat_b = Self::calculate_saturation_static(&b.2);
            sat_a
              .partial_cmp(&sat_b)
              .unwrap_or(std::cmp::Ordering::Equal)
          });
        }
        crate::effects::PixelSortMode::Random => {
          let mut rng = create_random_generator();
          rng.shuffle(&mut pixels);
        }
      }

      for (x, y, pixel_data) in pixels {
        let pixel_index = (y * width + x) * channels as u32;
        let pixel_start = pixel_index as usize;

        if pixel_start + channels <= data.len() {
          data[pixel_start..pixel_start + channels].copy_from_slice(&pixel_data);
        }
      }

      *image_processor = ImageProcessor::from_image_data(new_image_data);
    }

    Ok(())
  }

  fn apply_data_mosh_static(
    media_processor: &mut MediaProcessor,
    intensity: f32,
    preserve_size: bool,
  ) -> Result<()> {
    if let Some(image_processor) = media_processor.image_processor_mut() {
      let image_data = image_processor.data();
      let mut new_image_data = image_data.clone();
      let data = &mut new_image_data.data;

      let corruption_count = (data.len() as f32 * intensity) as usize;
      let mut rng = create_random_generator();

      for _ in 0..corruption_count {
        let pos = rng.gen_range(0, data.len() as u64) as usize;
        if pos < data.len() {
          data[pos] = rng.gen_range(0, 256) as u8;
        }
      }

      *image_processor = ImageProcessor::from_image_data(new_image_data);
    }

    Ok(())
  }

  fn calculate_hue_static(pixel: &[u8]) -> f32 {
    if pixel.len() < 3 {
      return 0.0;
    }

    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta == 0.0 {
      return 0.0;
    }

    let hue = if max == r {
      ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
      ((b - r) / delta + 2.0) / 6.0
    } else {
      ((r - g) / delta + 4.0) / 6.0
    };

    hue
  }

  fn calculate_saturation_static(pixel: &[u8]) -> f32 {
    if pixel.len() < 3 {
      return 0.0;
    }

    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta == 0.0 {
      return 0.0;
    }

    let lightness = (max + min) / 2.0;

    if lightness == 0.0 {
      return 0.0;
    }

    let saturation = if lightness <= 0.5 {
      delta / (max + min)
    } else {
      delta / (2.0 - max - min)
    };

    saturation
  }

  pub fn clone(&self) -> ChainProcessor {
    ChainProcessor {
      chain: self.chain.clone(),
      execution_mode: self.execution_mode,
      cache: self.cache.clone(),
      performance_stats: self.performance_stats.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ChainCache {
  cache: HashMap<String, MediaProcessor>,
  max_size: usize,
}

impl ChainCache {
  pub fn new() -> Self {
    Self {
      cache: HashMap::new(),
      max_size: 200,
    }
  }

  pub fn with_max_size(max_size: usize) -> Self {
    Self {
      cache: HashMap::new(),
      max_size,
    }
  }

  pub fn get(&self, key: &str) -> Option<&MediaProcessor> {
    self.cache.get(key)
  }

  pub fn put(&mut self, key: String, processor: MediaProcessor) {
    if self.cache.len() >= self.max_size {
      if let Some(oldest_key) = self.cache.keys().next().cloned() {
        self.cache.remove(&oldest_key);
      }
    }
    self.cache.insert(key, processor);
  }

  pub fn remove(&mut self, key: &str) -> Option<MediaProcessor> {
    self.cache.remove(key)
  }

  pub fn clear(&mut self) {
    self.cache.clear();
  }

  pub fn len(&self) -> usize {
    self.cache.len()
  }

  pub fn is_empty(&self) -> bool {
    self.cache.is_empty()
  }

  pub fn clone(&self) -> ChainCache {
    ChainCache {
      cache: self.cache.clone(),
      max_size: self.max_size,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ChainPerformanceStats {
  execution_times: Vec<std::time::Duration>,
  total_executions: u64,
  total_time: std::time::Duration,
  min_time: std::time::Duration,
  max_time: std::time::Duration,
}

impl ChainPerformanceStats {
  pub fn new() -> Self {
    Self {
      execution_times: Vec::new(),
      total_executions: 0,
      total_time: std::time::Duration::ZERO,
      min_time: std::time::Duration::MAX,
      max_time: std::time::Duration::ZERO,
    }
  }

  pub fn record_execution(&mut self, execution_time: std::time::Duration) {
    self.execution_times.push(execution_time);
    self.total_executions += 1;
    self.total_time += execution_time;

    if execution_time < self.min_time {
      self.min_time = execution_time;
    }

    if execution_time > self.max_time {
      self.max_time = execution_time;
    }

    if self.execution_times.len() > 200 {
      self.execution_times.drain(0..100);
    }
  }

  pub fn average_time(&self) -> std::time::Duration {
    if self.total_executions == 0 {
      std::time::Duration::ZERO
    } else {
      self.total_time / self.total_executions as u32
    }
  }

  pub fn min_time(&self) -> std::time::Duration {
    self.min_time
  }

  pub fn max_time(&self) -> std::time::Duration {
    self.max_time
  }

  pub fn total_executions(&self) -> u64 {
    self.total_executions
  }

  pub fn total_time(&self) -> std::time::Duration {
    self.total_time
  }

  pub fn recent_average_time(&self, count: usize) -> std::time::Duration {
    let recent_times: Vec<_> = self.execution_times.iter().rev().take(count).collect();
    if recent_times.is_empty() {
      std::time::Duration::ZERO
    } else {
      let sum: std::time::Duration = recent_times.iter().sum();
      sum / recent_times.len() as u32
    }
  }

  pub fn reset(&mut self) {
    self.execution_times.clear();
    self.total_executions = 0;
    self.total_time = std::time::Duration::ZERO;
    self.min_time = std::time::Duration::MAX;
    self.max_time = std::time::Duration::ZERO;
  }

  pub fn clone(&self) -> ChainPerformanceStats {
    ChainPerformanceStats {
      execution_times: self.execution_times.clone(),
      total_executions: self.total_executions,
      total_time: self.total_time,
      min_time: self.min_time,
      max_time: self.max_time,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ChainLibrary {
  chains: HashMap<Uuid, EffectChain>,
  chains_by_name: HashMap<String, Uuid>,
  chains_by_category: HashMap<String, Vec<Uuid>>,
}

impl ChainLibrary {
  pub fn new() -> Self {
    Self {
      chains: HashMap::new(),
      chains_by_name: HashMap::new(),
      chains_by_category: HashMap::new(),
    }
  }

  pub fn add_chain(&mut self, chain: EffectChain) -> Result<()> {
    if self.chains_by_name.contains_key(&chain.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Chain '{}' already exists",
        chain.name
      )));
    }

    let id = chain.id;
    self.chains_by_name.insert(chain.name.clone(), id);

    for tag in &chain.tags {
      self
        .chains_by_category
        .entry(tag.clone())
        .or_insert_with(Vec::new)
        .push(id);
    }

    self.chains.insert(id, chain);

    Ok(())
  }

  pub fn remove_chain(&mut self, id: Uuid) -> Option<EffectChain> {
    if let Some(chain) = self.chains.remove(&id) {
      self.chains_by_name.remove(&chain.name);

      for tag in &chain.tags {
        if let Some(chains) = self.chains_by_category.get_mut(tag) {
          chains.retain(|&chain_id| chain_id != id);
        }
      }

      Some(chain)
    } else {
      None
    }
  }

  pub fn get_chain(&self, id: Uuid) -> Option<&EffectChain> {
    self.chains.get(&id)
  }

  pub fn get_chain_by_name(&self, name: &str) -> Option<&EffectChain> {
    self
      .chains_by_name
      .get(name)
      .and_then(|id| self.chains.get(id))
  }

  pub fn list_chains(&self) -> Vec<&EffectChain> {
    self.chains.values().collect()
  }

  pub fn list_chains_by_category(&self, category: &str) -> Vec<&EffectChain> {
    self
      .chains_by_category
      .get(category)
      .map(|ids| ids.iter().filter_map(|id| self.chains.get(id)).collect())
      .unwrap_or_default()
  }

  pub fn search_chains(&self, query: &str) -> Vec<&EffectChain> {
    let query = query.to_lowercase();
    self
      .chains
      .values()
      .filter(|chain| {
        chain.name.to_lowercase().contains(&query)
          || chain.description.to_lowercase().contains(&query)
          || chain
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query))
      })
      .collect()
  }

  pub fn clear(&mut self) {
    self.chains.clear();
    self.chains_by_name.clear();
    self.chains_by_category.clear();
  }

  pub fn len(&self) -> usize {
    self.chains.len()
  }

  pub fn is_empty(&self) -> bool {
    self.chains.is_empty()
  }

  pub fn clone(&self) -> ChainLibrary {
    ChainLibrary {
      chains: self.chains.clone(),
      chains_by_name: self.chains_by_name.clone(),
      chains_by_category: self.chains_by_category.clone(),
    }
  }

  pub fn load_default_chains(&mut self) -> Result<()> {
    self.load_basic_chains()?;
    self.load_glitch_chains()?;
    self.load_filter_chains()?;
    self.load_transform_chains()?;

    Ok(())
  }

  fn load_basic_chains(&mut self) -> Result<()> {
    let chains = vec![
      EffectChain::new(
        "photo_enhancement".to_string(),
        "Basic photo enhancement chain".to_string(),
      )
      .with_steps(vec![
        ChainStep::new(
          "brightness".to_string(),
          [("amount".to_string(), "0.1".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.8),
        ChainStep::new(
          "contrast".to_string(),
          [("amount".to_string(), "0.2".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.7),
        ChainStep::new(
          "saturation".to_string(),
          [("amount".to_string(), "0.1".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.6),
      ])
      .with_tags(vec![
        "photo".to_string(),
        "enhancement".to_string(),
        "basic".to_string(),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
      EffectChain::new(
        "audio_cleanup".to_string(),
        "Basic audio cleanup chain".to_string(),
      )
      .with_steps(vec![
        ChainStep::new(
          "low_pass".to_string(),
          [("cutoff".to_string(), "20000.0".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.8),
        ChainStep::new(
          "compressor".to_string(),
          [("ratio".to_string(), "3.0".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.7),
        ChainStep::new(
          "reverb".to_string(),
          [("room_size".to_string(), "0.2".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.3),
      ])
      .with_tags(vec![
        "audio".to_string(),
        "cleanup".to_string(),
        "basic".to_string(),
      ])
      .with_supported_media_types(vec![MediaType::Audio]),
    ];

    for chain in chains {
      self.add_chain(chain)?;
    }

    Ok(())
  }

  fn load_glitch_chains(&mut self) -> Result<()> {
    let chains = vec![
      EffectChain::new(
        "digital_art".to_string(),
        "Digital art glitch chain".to_string(),
      )
      .with_steps(vec![
        ChainStep::new(
          "pixel_sort".to_string(),
          [
            ("threshold".to_string(), "0.6".to_string()),
            ("mode".to_string(), "brightness".to_string()),
          ]
          .into_iter()
          .collect(),
        )
        .with_weight(1.0),
        ChainStep::new(
          "data_mosh".to_string(),
          [
            ("intensity".to_string(), "0.4".to_string()),
            ("preserve_size".to_string(), "true".to_string()),
          ]
          .into_iter()
          .collect(),
        )
        .with_dependencies(vec![]),
        ChainStep::new(
          "hue_rotate".to_string(),
          [("angle".to_string(), "20.0".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.5),
      ])
      .with_tags(vec![
        "glitch".to_string(),
        "digital_art".to_string(),
        "artistic".to_string(),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
    ];

    for chain in chains {
      self.add_chain(chain)?;
    }

    Ok(())
  }

  fn load_filter_chains(&mut self) -> Result<()> {
    let chains = vec![
      EffectChain::new(
        "portrait_enhancement".to_string(),
        "Portrait enhancement filter chain".to_string(),
      )
      .with_steps(vec![
        ChainStep::new(
          "blur".to_string(),
          [("radius".to_string(), "2.0".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.6),
        ChainStep::new(
          "sharpen".to_string(),
          [("amount".to_string(), "0.8".to_string())]
            .into_iter()
            .collect(),
        )
        .with_dependencies(vec![]),
        ChainStep::new(
          "brightness".to_string(),
          [("amount".to_string(), "0.1".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.4),
      ])
      .with_tags(vec![
        "filter".to_string(),
        "portrait".to_string(),
        "enhancement".to_string(),
      ])
      .with_supported_media_types(vec![MediaType::Image]),
    ];

    for chain in chains {
      self.add_chain(chain)?;
    }

    Ok(())
  }

  fn load_transform_chains(&mut self) -> Result<()> {
    let chains = vec![
      EffectChain::new(
        "video_effects".to_string(),
        "Video transformation chain".to_string(),
      )
      .with_steps(vec![
        ChainStep::new(
          "scale".to_string(),
          [
            ("scale_x".to_string(), "1.2".to_string()),
            ("scale_y".to_string(), "1.2".to_string()),
          ]
          .into_iter()
          .collect(),
        )
        .with_weight(1.0),
        ChainStep::new(
          "rotate".to_string(),
          [("angle".to_string(), "5.0".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.5),
        ChainStep::new(
          "time_stretch".to_string(),
          [("ratio".to_string(), "0.9".to_string())]
            .into_iter()
            .collect(),
        )
        .with_weight(0.8),
      ])
      .with_tags(vec![
        "transform".to_string(),
        "video".to_string(),
        "effects".to_string(),
      ])
      .with_supported_media_types(vec![MediaType::Video]),
    ];

    for chain in chains {
      self.add_chain(chain)?;
    }

    Ok(())
  }
}

pub fn create_chain_processor(chain: EffectChain) -> ChainProcessor {
  ChainProcessor::new(chain)
}

pub fn create_chain_processor_with_mode(
  chain: EffectChain,
  execution_mode: ChainExecutionMode,
) -> ChainProcessor {
  ChainProcessor::new(chain).with_execution_mode(execution_mode)
}

pub fn create_effect_chain(name: String, description: String) -> EffectChain {
  EffectChain::new(name, description)
}

pub fn create_chain_step(effect_type: String, parameters: HashMap<String, String>) -> ChainStep {
  ChainStep::new(effect_type, parameters)
}

pub fn create_chain_cache() -> ChainCache {
  ChainCache::new()
}

pub fn create_chain_cache_with_max_size(max_size: usize) -> ChainCache {
  ChainCache::with_max_size(max_size)
}

pub fn create_chain_performance_stats() -> ChainPerformanceStats {
  ChainPerformanceStats::new()
}

pub fn create_chain_library() -> ChainLibrary {
  ChainLibrary::new()
}
