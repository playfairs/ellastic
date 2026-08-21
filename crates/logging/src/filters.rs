use std::collections::HashSet;
use tracing_core::{
  Level,
  Metadata,
};
use tracing_subscriber::layer::{
  Context,
  Filter,
};
use tracing_subscriber::registry::LookupSpan;

pub struct ModuleFilter {
  allowed_modules: HashSet<String>,
  denied_modules: HashSet<String>,
  level: Level,
}

impl ModuleFilter {
  pub fn new(level: Level) -> Self {
    Self {
      allowed_modules: HashSet::new(),
      denied_modules: HashSet::new(),
      level,
    }
  }

  pub fn allow_module(mut self, module: impl Into<String>) -> Self {
    self.allowed_modules.insert(module.into());
    self
  }

  pub fn deny_module(mut self, module: impl Into<String>) -> Self {
    self.denied_modules.insert(module.into());
    self
  }

  pub fn allow_modules<I, S>(mut self, modules: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for module in modules {
      self.allowed_modules.insert(module.into());
    }
    self
  }

  pub fn deny_modules<I, S>(mut self, modules: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for module in modules {
      self.denied_modules.insert(module.into());
    }
    self
  }

  pub fn is_module_allowed(&self, module: &str) -> bool {
    if self
      .denied_modules
      .iter()
      .any(|denied| module.starts_with(denied))
    {
      return false;
    }

    if self.allowed_modules.is_empty() {
      return true;
    }

    self
      .allowed_modules
      .iter()
      .any(|allowed| module.starts_with(allowed))
  }
}

impl<S> Filter<S> for ModuleFilter
where
  S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
  fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
    if *metadata.level() > self.level {
      return false;
    }

    if let Some(module) = metadata.module_path() {
      self.is_module_allowed(module)
    } else {
      true
    }
  }
}

pub struct TargetFilter {
  allowed_targets: HashSet<String>,
  denied_targets: HashSet<String>,
  level: Level,
}

impl TargetFilter {
  pub fn new(level: Level) -> Self {
    Self {
      allowed_targets: HashSet::new(),
      denied_targets: HashSet::new(),
      level,
    }
  }

  pub fn allow_target(mut self, target: impl Into<String>) -> Self {
    self.allowed_targets.insert(target.into());
    self
  }

  pub fn deny_target(mut self, target: impl Into<String>) -> Self {
    self.denied_targets.insert(target.into());
    self
  }

  pub fn allow_targets<I, S>(mut self, targets: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for target in targets {
      self.allowed_targets.insert(target.into());
    }
    self
  }

  pub fn deny_targets<I, S>(mut self, targets: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for target in targets {
      self.denied_targets.insert(target.into());
    }
    self
  }

  pub fn is_target_allowed(&self, target: &str) -> bool {
    if self.denied_targets.contains(target) {
      return false;
    }

    if self.allowed_targets.is_empty() {
      return true;
    }

    self.allowed_targets.contains(target)
  }
}

impl<S> Filter<S> for TargetFilter
where
  S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
  fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
    if *metadata.level() > self.level {
      return false;
    }

    self.is_target_allowed(metadata.target())
  }
}

pub struct OperationFilter {
  allowed_operations: HashSet<String>,
  denied_operations: HashSet<String>,
  level: Level,
}

impl OperationFilter {
  pub fn new(level: Level) -> Self {
    Self {
      allowed_operations: HashSet::new(),
      denied_operations: HashSet::new(),
      level,
    }
  }

  pub fn allow_operation(mut self, operation: impl Into<String>) -> Self {
    self.allowed_operations.insert(operation.into());
    self
  }

  pub fn deny_operation(mut self, operation: impl Into<String>) -> Self {
    self.denied_operations.insert(operation.into());
    self
  }

  pub fn allow_operations<I, S>(mut self, operations: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for operation in operations {
      self.allowed_operations.insert(operation.into());
    }
    self
  }

  pub fn deny_operations<I, S>(mut self, operations: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for operation in operations {
      self.denied_operations.insert(operation.into());
    }
    self
  }

  pub fn is_operation_allowed(&self, operation: &str) -> bool {
    if self.denied_operations.contains(operation) {
      return false;
    }

    if self.allowed_operations.is_empty() {
      return true;
    }

    self.allowed_operations.contains(operation)
  }
}

impl<S> Filter<S> for OperationFilter
where
  S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
  fn enabled(&self, metadata: &Metadata<'_>, cx: Context<'_, S>) -> bool {
    if *metadata.level() > self.level {
      return false;
    }

    if let Some(span) = cx.lookup_current() {
      if let Some(operation) = span.extensions().get::<&'static str>() {
        return self.is_operation_allowed(operation);
      }
    }

    true
  }
}

pub struct MediaFilter {
  level: Level,
  log_media_operations: bool,
  log_processing_operations: bool,
  log_lua_operations: bool,
  log_export_operations: bool,
}

impl MediaFilter {
  pub fn new(level: Level) -> Self {
    Self {
      level,
      log_media_operations: true,
      log_processing_operations: true,
      log_lua_operations: true,
      log_export_operations: true,
    }
  }

  pub fn with_media_operations(mut self, enabled: bool) -> Self {
    self.log_media_operations = enabled;
    self
  }

  pub fn with_processing_operations(mut self, enabled: bool) -> Self {
    self.log_processing_operations = enabled;
    self
  }

  pub fn with_lua_operations(mut self, enabled: bool) -> Self {
    self.log_lua_operations = enabled;
    self
  }

  pub fn with_export_operations(mut self, enabled: bool) -> Self {
    self.log_export_operations = enabled;
    self
  }

  pub fn only_media(mut self) -> Self {
    self.log_media_operations = true;
    self.log_processing_operations = false;
    self.log_lua_operations = false;
    self.log_export_operations = false;
    self
  }

  pub fn only_processing(mut self) -> Self {
    self.log_media_operations = false;
    self.log_processing_operations = true;
    self.log_lua_operations = false;
    self.log_export_operations = false;
    self
  }

  pub fn only_lua(mut self) -> Self {
    self.log_media_operations = false;
    self.log_processing_operations = false;
    self.log_lua_operations = true;
    self.log_export_operations = false;
    self
  }

  pub fn only_export(mut self) -> Self {
    self.log_media_operations = false;
    self.log_processing_operations = false;
    self.log_lua_operations = false;
    self.log_export_operations = true;
    self
  }

  fn should_log_span(&self, span_name: &str) -> bool {
    match span_name {
      "media_operation" => self.log_media_operations,
      "processing" => self.log_processing_operations,
      "lua_execution" => self.log_lua_operations,
      "export" => self.log_export_operations,
      _ => true,
    }
  }
}

impl<S> Filter<S> for MediaFilter
where
  S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
  fn enabled(&self, metadata: &Metadata<'_>, cx: Context<'_, S>) -> bool {
    if *metadata.level() > self.level {
      return false;
    }

    if let Some(span) = cx.lookup_current() {
      return self.should_log_span(span.name());
    }

    true
  }
}

pub struct PerformanceFilter {
  level: Level,
  log_slow_operations: bool,
  slow_threshold_ms: u64,
  log_fast_operations: bool,
  fast_threshold_ms: u64,
}

impl PerformanceFilter {
  pub fn new(level: Level) -> Self {
    Self {
      level,
      log_slow_operations: true,
      slow_threshold_ms: 1000,
      log_fast_operations: false,
      fast_threshold_ms: 100,
    }
  }

  pub fn with_slow_threshold(mut self, threshold_ms: u64) -> Self {
    self.slow_threshold_ms = threshold_ms;
    self
  }

  pub fn with_fast_threshold(mut self, threshold_ms: u64) -> Self {
    self.fast_threshold_ms = threshold_ms;
    self
  }

  pub fn log_slow(mut self, enabled: bool) -> Self {
    self.log_slow_operations = enabled;
    self
  }

  pub fn log_fast(mut self, enabled: bool) -> Self {
    self.log_fast_operations = enabled;
    self
  }

  pub fn only_slow(mut self) -> Self {
    self.log_slow_operations = true;
    self.log_fast_operations = false;
    self
  }

  pub fn only_fast(mut self) -> Self {
    self.log_slow_operations = false;
    self.log_fast_operations = true;
    self
  }

  pub fn both(mut self) -> Self {
    self.log_slow_operations = true;
    self.log_fast_operations = true;
    self
  }

  fn should_log(&self, duration_ms: Option<u64>) -> bool {
    if let Some(duration) = duration_ms {
      if self.log_slow_operations && duration >= self.slow_threshold_ms {
        return true;
      }
      if self.log_fast_operations && duration < self.fast_threshold_ms {
        return true;
      }
    }
    false
  }
}

impl<S> Filter<S> for PerformanceFilter
where
  S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
  fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
    if *metadata.level() > self.level {
      return false;
    }

    true
  }
}

pub struct ErrorFilter {
  level: Level,
  log_errors: bool,
  log_warnings: bool,
  log_panics: bool,
}

impl ErrorFilter {
  pub fn new(level: Level) -> Self {
    Self {
      level,
      log_errors: true,
      log_warnings: true,
      log_panics: true,
    }
  }

  pub fn with_errors(mut self, enabled: bool) -> Self {
    self.log_errors = enabled;
    self
  }

  pub fn with_warnings(mut self, enabled: bool) -> Self {
    self.log_warnings = enabled;
    self
  }

  pub fn with_panics(mut self, enabled: bool) -> Self {
    self.log_panics = enabled;
    self
  }

  pub fn only_errors(mut self) -> Self {
    self.log_errors = true;
    self.log_warnings = false;
    self
  }

  pub fn only_warnings(mut self) -> Self {
    self.log_errors = false;
    self.log_warnings = true;
    self
  }

  pub fn only_critical(mut self) -> Self {
    self.log_errors = true;
    self.log_warnings = false;
    self.level = Level::ERROR;
    self
  }
}

impl<S> Filter<S> for ErrorFilter
where
  S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
  fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
    if *metadata.level() > self.level {
      return false;
    }

    match *metadata.level() {
      Level::ERROR => self.log_errors,
      Level::WARN => self.log_warnings,
      _ => true,
    }
  }
}

pub struct CompositeFilter {
  filters: Vec<Box<dyn tracing_subscriber::layer::Filter<S> + Send + Sync>>,
}

impl CompositeFilter {
  pub fn new() -> Self {
    Self {
      filters: Vec::new(),
    }
  }

  pub fn add_filter<F>(mut self, filter: F) -> Self
  where
    F: tracing_subscriber::layer::Filter<S> + Send + Sync + 'static,
  {
    self.filters.push(Box::new(filter));
    self
  }

  pub fn with_filters<F>(mut self, filters: Vec<F>) -> Self
  where
    F: tracing_subscriber::layer::Filter<S> + Send + Sync + 'static,
  {
    for filter in filters {
      self.filters.push(Box::new(filter));
    }
    self
  }
}

impl<S> Filter<S> for CompositeFilter
where
  S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
  fn enabled(&self, metadata: &Metadata<'_>, cx: Context<'_, S>) -> bool {
    self
      .filters
      .iter()
      .all(|filter| filter.enabled(metadata, cx.clone()))
  }
}

impl Default for CompositeFilter {
  fn default() -> Self {
    Self::new()
  }
}

pub fn create_module_filter(level: Level) -> ModuleFilter {
  ModuleFilter::new(level)
}

pub fn create_target_filter(level: Level) -> TargetFilter {
  TargetFilter::new(level)
}

pub fn create_operation_filter(level: Level) -> OperationFilter {
  OperationFilter::new(level)
}

pub fn create_media_filter(level: Level) -> MediaFilter {
  MediaFilter::new(level)
}

pub fn create_performance_filter(level: Level) -> PerformanceFilter {
  PerformanceFilter::new(level)
}

pub fn create_error_filter(level: Level) -> ErrorFilter {
  ErrorFilter::new(level)
}

pub fn create_composite_filter() -> CompositeFilter {
  CompositeFilter::new()
}

pub mod presets {
  use super::*;
  use tracing_core::Level;

  pub fn development_filter() -> ModuleFilter {
    ModuleFilter::new(Level::DEBUG).allow_modules([
      "ellastic_core",
      "ellastic_errors",
      "ellastic_config",
      "ellastic_logging",
      "ellastic_utils",
      "ellastic_bytes",
      "ellastic_image",
      "ellastic_audio",
      "ellastic_media",
      "ellastic_glitch",
      "ellastic_effects",
      "ellastic_pipeline",
      "ellastic_lua",
      "ellastic_export",
      "ellastic_ui",
      "ellastic_cli",
      "ellastic_app",
    ])
  }

  pub fn production_filter() -> CompositeFilter {
    CompositeFilter::new()
      .add_filter(ModuleFilter::new(Level::INFO).allow_modules([
        "ellastic_core",
        "ellastic_errors",
        "ellastic_config",
        "ellastic_logging",
      ]))
      .add_filter(ErrorFilter::new(Level::ERROR).only_errors())
  }

  pub fn testing_filter() -> TargetFilter {
    TargetFilter::new(Level::WARN).deny_targets([
      "tracing::span",
      "tracing::dispatcher",
      "tokio",
      "hyper",
      "reqwest",
    ])
  }

  pub fn media_only_filter() -> MediaFilter {
    MediaFilter::new(Level::INFO).only_media()
  }

  pub fn processing_only_filter() -> MediaFilter {
    MediaFilter::new(Level::INFO).only_processing()
  }

  pub fn lua_only_filter() -> MediaFilter {
    MediaFilter::new(Level::INFO).only_lua()
  }

  pub fn export_only_filter() -> MediaFilter {
    MediaFilter::new(Level::INFO).only_export()
  }

  pub fn performance_filter() -> PerformanceFilter {
    PerformanceFilter::new(Level::INFO)
      .only_slow()
      .with_slow_threshold(500)
  }

  pub fn quiet_filter() -> ErrorFilter {
    ErrorFilter::new(Level::ERROR).only_errors()
  }

  pub fn verbose_filter() -> ModuleFilter {
    ModuleFilter::new(Level::TRACE)
  }
}
