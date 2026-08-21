use std::fmt;
use std::io;
use thiserror::Error;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, EllasticError>;

#[derive(Error, Debug)]
pub enum EllasticError {
  #[error("I/O error: {0}")]
  IoError(String),

  #[error("No processor available for media type: {0:?}")]
  NoProcessorForMediaType(String),

  #[error("Processing error: {0}")]
  ProcessingError(String),

  #[error("Invalid image data: expected {expected} bytes, got {actual}")]
  InvalidImageData { expected: usize, actual: usize },

  #[error("Coordinates out of bounds: x={x}, y={y}")]
  CoordinatesOutOfBounds { x: u32, y: u32 },

  #[error("Invalid pixel size: expected {expected}, got {actual}")]
  InvalidPixelSize { expected: usize, actual: usize },

  #[error("Invalid audio data: expected {expected} bytes, got {actual}")]
  InvalidAudioData { expected: usize, actual: usize },

  #[error("Invalid sample size: expected {expected}, got {actual}")]
  InvalidSampleSize { expected: usize, actual: usize },

  #[error("Invalid binary data: expected {expected} bytes, got {actual}")]
  InvalidBinaryData { expected: usize, actual: usize },

  #[error("Offset overflow: {0}")]
  OffsetOverflow(usize),

  #[error("Invalid pipeline step index: {0}")]
  InvalidPipelineStep(usize),

  #[error("Mismatched processing unit and data type")]
  MismatchedProcessingUnit,

  #[error("Task not found: {0}")]
  TaskNotFound(Uuid),

  #[error("Pipeline not found: {0}")]
  PipelineNotFound(Uuid),

  #[error("Media not found: {0}")]
  MediaNotFound(Uuid),

  #[error("Configuration error: {0}")]
  ConfigError(String),

  #[error("Serialization error: {0}")]
  SerializationError(String),

  #[error("Deserialization error: {0}")]
  DeserializationError(String),

  #[error("Lua error: {0}")]
  LuaError(String),

  #[error("Script execution failed: {0}")]
  ScriptExecutionError(String),

  #[error("Invalid script path: {0}")]
  InvalidScriptPath(String),

  #[error("Unsupported format: {0}")]
  UnsupportedFormat(String),

  #[error("Codec error: {0}")]
  CodecError(String),

  #[error("Export error: {0}")]
  ExportError(String),

  #[error("Invalid export parameters: {0}")]
  InvalidExportParameters(String),

  #[error("File not found: {0}")]
  FileNotFound(String),

  #[error("Permission denied: {0}")]
  PermissionDenied(String),

  #[error("Disk space insufficient")]
  InsufficientDiskSpace,

  #[error("Network error: {0}")]
  NetworkError(String),

  #[error("Timeout error: {0}")]
  TimeoutError(String),

  #[error("Memory allocation failed: {0}")]
  MemoryError(String),

  #[error("Thread pool error: {0}")]
  ThreadPoolError(String),

  #[error("Invalid parameter: {0}")]
  InvalidParameter(String),

  #[error("System error: {0}")]
  SystemError(String),

  #[error("Insufficient memory")]
  InsufficientMemory,

  #[error("Missing required parameter: {0}")]
  MissingParameter(String),

  #[error("Parameter validation failed: {0}")]
  ParameterValidationError(String),

  #[error("Invalid state: {0}")]
  InvalidState(String),

  #[error("Operation not supported: {0}")]
  UnsupportedOperation(String),

  #[error("Resource not available: {0}")]
  ResourceUnavailable(String),

  #[error("Concurrent modification detected")]
  ConcurrentModification,

  #[error("Lock acquisition failed: {0}")]
  LockError(String),

  #[error("Database error: {0}")]
  DatabaseError(String),

  #[error("Authentication failed: {0}")]
  AuthenticationError(String),

  #[error("Authorization failed: {0}")]
  AuthorizationError(String),

  #[error("Rate limit exceeded")]
  RateLimitExceeded,

  #[error("Quota exceeded: {0}")]
  QuotaExceeded(String),

  #[error("Service unavailable: {0}")]
  ServiceUnavailable(String),

  #[error("Internal server error: {0}")]
  InternalServerError(String),

  #[error("Unknown error: {0}")]
  Unknown(String),
}

impl From<io::Error> for EllasticError {
  fn from(err: io::Error) -> Self {
    EllasticError::IoError(err.to_string())
  }
}

impl From<serde_json::Error> for EllasticError {
  fn from(err: serde_json::Error) -> Self {
    EllasticError::SerializationError(err.to_string())
  }
}

impl From<toml::de::Error> for EllasticError {
  fn from(err: toml::de::Error) -> Self {
    EllasticError::DeserializationError(err.to_string())
  }
}

impl From<toml::ser::Error> for EllasticError {
  fn from(err: toml::ser::Error) -> Self {
    EllasticError::SerializationError(err.to_string())
  }
}

impl From<mlua::Error> for EllasticError {
  fn from(err: mlua::Error) -> Self {
    EllasticError::LuaError(err.to_string())
  }
}

#[derive(Error, Debug)]
#[error("Validation error in field '{field}': {message}")]
pub struct ValidationError {
  pub field: String,
  pub message: String,
}

impl ValidationError {
  pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
    Self {
      field: field.into(),
      message: message.into(),
    }
  }
}

#[derive(Error, Debug)]
pub enum ErrorSeverity {
  #[error("Warning: {0}")]
  Warning(String),

  #[error("Error: {0}")]
  Error(String),

  #[error("Critical: {0}")]
  Critical(String),
}

#[derive(Error, Debug)]
pub struct ErrorContext {
  pub error: EllasticError,
  pub context: String,
  pub timestamp: std::time::SystemTime,
  pub severity: ErrorSeverity,
}

impl ErrorContext {
  pub fn new(error: EllasticError, context: impl Into<String>, severity: ErrorSeverity) -> Self {
    Self {
      error,
      context: context.into(),
      timestamp: std::time::SystemTime::now(),
      severity,
    }
  }

  pub fn warning(error: EllasticError, context: impl Into<String>) -> Self {
    let message = format!("{}", error);
    Self::new(error, context, ErrorSeverity::Warning(message))
  }

  pub fn error(error: EllasticError, context: impl Into<String>) -> Self {
    let message = format!("{}", error);
    Self::new(error, context, ErrorSeverity::Error(message))
  }

  pub fn critical(error: EllasticError, context: impl Into<String>) -> Self {
    let message = format!("{}", error);
    Self::new(
      error,
      context,
      ErrorSeverity::Critical(message),
    )
  }
}

pub trait ErrorExt<T> {
  fn with_context(self, context: impl Into<String>) -> std::result::Result<T, ErrorContext>;
  fn with_warning_context(self, context: impl Into<String>)
  -> std::result::Result<T, ErrorContext>;
  fn with_error_context(self, context: impl Into<String>) -> std::result::Result<T, ErrorContext>;
  fn with_critical_context(
    self,
    context: impl Into<String>,
  ) -> std::result::Result<T, ErrorContext>;
}

impl<T> ErrorExt<T> for Result<T> {
  fn with_context(self, context: impl Into<String>) -> std::result::Result<T, ErrorContext> {
    self.map_err(|e| ErrorContext::error(e, context))
  }

  fn with_warning_context(
    self,
    context: impl Into<String>,
  ) -> std::result::Result<T, ErrorContext> {
    self.map_err(|e| ErrorContext::warning(e, context))
  }

  fn with_error_context(self, context: impl Into<String>) -> std::result::Result<T, ErrorContext> {
    self.map_err(|e| ErrorContext::error(e, context))
  }

  fn with_critical_context(
    self,
    context: impl Into<String>,
  ) -> std::result::Result<T, ErrorContext> {
    self.map_err(|e| ErrorContext::critical(e, context))
  }
}

#[derive(Debug)]
pub struct ErrorReporter {
  errors: Vec<ErrorContext>,
}

impl ErrorReporter {
  pub fn new() -> Self {
    Self { errors: Vec::new() }
  }

  pub fn report(&mut self, error_context: ErrorContext) {
    self.errors.push(error_context);
  }

  pub fn report_error(&mut self, error: EllasticError, context: impl Into<String>) {
    self.report(ErrorContext::error(error, context));
  }

  pub fn report_warning(&mut self, error: EllasticError, context: impl Into<String>) {
    self.report(ErrorContext::warning(error, context));
  }

  pub fn report_critical(&mut self, error: EllasticError, context: impl Into<String>) {
    self.report(ErrorContext::critical(error, context));
  }

  pub fn clear(&mut self) {
    self.errors.clear();
  }

  pub fn len(&self) -> usize {
    self.errors.len()
  }

  pub fn is_empty(&self) -> bool {
    self.errors.is_empty()
  }

  pub fn get_errors(&self) -> &[ErrorContext] {
    &self.errors
  }

  pub fn get_errors_by_severity(&self, severity: &ErrorSeverity) -> Vec<&ErrorContext> {
    self
      .errors
      .iter()
      .filter(|e| std::mem::discriminant(&e.severity) == std::mem::discriminant(severity))
      .collect()
  }

  pub fn has_critical_errors(&self) -> bool {
    self
      .errors
      .iter()
      .any(|e| matches!(e.severity, ErrorSeverity::Critical(_)))
  }

  pub fn has_errors(&self) -> bool {
    self
      .errors
      .iter()
      .any(|e| matches!(e.severity, ErrorSeverity::Error(_)))
  }

  pub fn has_warnings(&self) -> bool {
    self
      .errors
      .iter()
      .any(|e| matches!(e.severity, ErrorSeverity::Warning(_)))
  }
}

impl Default for ErrorReporter {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for ErrorContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "[{}] {} - {}",
      self
        .timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs(),
      self.context,
      self.severity
    )
  }
}
