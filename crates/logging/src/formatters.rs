use ellastic_errors::EllasticError;
use std::io;
use tracing::Level;
use tracing_subscriber::field::Visit;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

pub struct CompactFormatter;

impl tracing_subscriber::fmt::FormatEvent for CompactFormatter {
  fn format_event(
    &self,
    ctx: &tracing_subscriber::fmt::FmtContext<'_, Self>,
    writer: &mut dyn io::Write,
    event: &tracing::Event<'_>,
  ) -> io::Result<()> {
    let level = *event.metadata().level();
    write!(writer, "{}", level)?;

    if let Some(target) = event.metadata().target() {
      write!(writer, " [{}]", target)?;
    }

    let mut visitor = FieldVisitor::new(writer);
    event.record(&mut visitor);

    writeln!(writer)
  }
}

pub struct PrettyFormatter {
  show_target: bool,
  show_thread_id: bool,
  show_thread_name: bool,
}

impl PrettyFormatter {
  pub fn new() -> Self {
    Self {
      show_target: true,
      show_thread_id: true,
      show_thread_name: true,
    }
  }

  pub fn with_target(mut self, show: bool) -> Self {
    self.show_target = show;
    self
  }

  pub fn with_thread_id(mut self, show: bool) -> Self {
    self.show_thread_id = show;
    self
  }

  pub fn with_thread_name(mut self, show: bool) -> Self {
    self.show_thread_name = show;
    self
  }
}

impl Default for PrettyFormatter {
  fn default() -> Self {
    Self::new()
  }
}

impl tracing_subscriber::fmt::FormatEvent for PrettyFormatter {
  fn format_event(
    &self,
    ctx: &tracing_subscriber::fmt::FmtContext<'_, Self>,
    writer: &mut dyn io::Write,
    event: &tracing::Event<'_>,
  ) -> io::Result<()> {
    let level = *event.metadata().level();
    let level_color = level_color_code(level);

    write!(writer, "\x1b[{}m{}\x1b[0m", level_color, level)?;

    if self.show_target {
      if let Some(target) = event.metadata().target() {
        write!(writer, " \x1b[36m[{}]\x1b[0m", target)?;
      }
    }

    if self.show_thread_id {
      if let Some(thread_id) = std::thread::current().id().as_u64().get() {
        write!(writer, " \x1b[90m[Thread:{}]\x1b[0m", thread_id)?;
      }
    }

    if self.show_thread_name {
      if let Some(thread_name) = std::thread::current().name() {
        write!(writer, " \x1b[90m[{}]\x1b[0m", thread_name)?;
      }
    }

    write!(writer, ": ")?;

    let mut visitor = FieldVisitor::new(writer);
    event.record(&mut visitor);

    writeln!(writer)
  }
}

pub struct JsonFormatter {
  include_timestamp: bool,
  include_target: bool,
  include_thread: bool,
}

impl JsonFormatter {
  pub fn new() -> Self {
    Self {
      include_timestamp: true,
      include_target: true,
      include_thread: true,
    }
  }

  pub fn with_timestamp(mut self, include: bool) -> Self {
    self.include_timestamp = include;
    self
  }

  pub fn with_target(mut self, include: bool) -> Self {
    self.include_target = include;
    self
  }

  pub fn with_thread(mut self, include: bool) -> Self {
    self.include_thread = include;
    self
  }
}

impl Default for JsonFormatter {
  fn default() -> Self {
    Self::new()
  }
}

impl tracing_subscriber::fmt::FormatEvent for JsonFormatter {
  fn format_event(
    &self,
    ctx: &tracing_subscriber::fmt::FmtContext<'_, Self>,
    writer: &mut dyn io::Write,
    event: &tracing::Event<'_>,
  ) -> io::Result<()> {
    write!(writer, "{{")?;

    write!(writer, "\"level\":\"{}\"", event.metadata().level())?;

    if self.include_timestamp {
      let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
      write!(writer, ",\"timestamp\":{}", now)?;
    }

    if self.include_target {
      if let Some(target) = event.metadata().target() {
        write!(writer, ",\"target\":\"{}\"", target)?;
      }
    }

    if self.include_thread {
      if let Some(thread_id) = std::thread::current().id().as_u64().get() {
        write!(writer, ",\"thread_id\":{}", thread_id)?;
      }
      if let Some(thread_name) = std::thread::current().name() {
        write!(writer, ",\"thread_name\":\"{}\"", thread_name)?;
      }
    }

    write!(writer, ",\"fields\":{{")?;

    let mut first_field = true;
    let mut visitor = JsonFieldVisitor::new(writer, &mut first_field);
    event.record(&mut visitor);

    write!(writer, "}}")?;

    writeln!(writer, "}}")
  }
}

struct FieldVisitor<'a> {
  writer: &'a mut dyn io::Write,
  first_field: bool,
}

impl<'a> FieldVisitor<'a> {
  fn new(writer: &'a mut dyn io::Write) -> Self {
    Self {
      writer,
      first_field: true,
    }
  }
}

impl<'a> Visit for FieldVisitor<'a> {
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
    if self.first_field {
      self.first_field = false;
    } else {
      let _ = write!(self.writer, " ");
    }
    let _ = write!(self.writer, "{}={:?}", field.name(), value);
  }
}

struct JsonFieldVisitor<'a> {
  writer: &'a mut dyn io::Write,
  first_field: &'a mut bool,
}

impl<'a> JsonFieldVisitor<'a> {
  fn new(writer: &'a mut dyn io::Write, first_field: &'a mut bool) -> Self {
    Self {
      writer,
      first_field,
    }
  }
}

impl<'a> Visit for JsonFieldVisitor<'a> {
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
    if !*self.first_field {
      let _ = write!(self.writer, ",");
    } else {
      *self.first_field = false;
    }
    let _ = write!(self.writer, "\"{}\":\"{:?}\"", field.name(), value);
  }

  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
    if !*self.first_field {
      let _ = write!(self.writer, ",");
    } else {
      *self.first_field = false;
    }
    let _ = write!(self.writer, "\"{}\":\"{}\"", field.name(), value);
  }

  fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
    if !*self.first_field {
      let _ = write!(self.writer, ",");
    } else {
      *self.first_field = false;
    }
    let _ = write!(self.writer, "\"{}\":{}", field.name(), value);
  }

  fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
    if !*self.first_field {
      let _ = write!(self.writer, ",");
    } else {
      *self.first_field = false;
    }
    let _ = write!(self.writer, "\"{}\":{}", field.name(), value);
  }

  fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
    if !*self.first_field {
      let _ = write!(self.writer, ",");
    } else {
      *self.first_field = false;
    }
    let _ = write!(self.writer, "\"{}\":{}", field.name(), value);
  }
}

pub struct CustomTimeFormat;

impl FormatTime for CustomTimeFormat {
  fn format_time(&self, writer: &mut Writer<'_>) -> io::Result<()> {
    let now = std::time::SystemTime::now();
    let datetime = now
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default();
    let secs = datetime.as_secs();

    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    write!(writer, "{:02}:{:02}:{:02}", hours, minutes, seconds)
  }
}

pub struct Iso8601TimeFormat;

impl FormatTime for Iso8601TimeFormat {
  fn format_time(&self, writer: &mut Writer<'_>) -> io::Result<()> {
    let now = std::time::SystemTime::now();
    let datetime = chrono::DateTime::<chrono::Utc>::from(now);
    write!(writer, "{}", datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
  }
}

pub struct UnixTimeFormat;

impl FormatTime for UnixTimeFormat {
  fn format_time(&self, writer: &mut Writer<'_>) -> io::Result<()> {
    let now = std::time::SystemTime::now();
    let timestamp = now
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default();
    write!(writer, "{}", timestamp.as_secs())
  }
}

fn level_color_code(level: Level) -> u8 {
  match level {
    Level::TRACE => 90,
    Level::DEBUG => 36,
    Level::INFO => 32,
    Level::WARN => 33,
    Level::ERROR => 31,
  }
}

pub struct MediaOperationFormatter;

impl MediaOperationFormatter {
  pub fn format_operation_start(&self, media_id: &str, operation: &str, format: &str) -> String {
    format!(
      "\x1b[36m[MEDIA]\x1b[0m \x1b[32mSTART\x1b[0m {} {} (format: {})",
      media_id, operation, format
    )
  }

  pub fn format_operation_success(
    &self,
    media_id: &str,
    operation: &str,
    duration_ms: u64,
  ) -> String {
    format!(
      "\x1b[36m[MEDIA]\x1b[0m \x1b[32mSUCCESS\x1b[0m {} {} ({}ms)",
      media_id, operation, duration_ms
    )
  }

  pub fn format_operation_error(&self, media_id: &str, operation: &str, error: &str) -> String {
    format!(
      "\x1b[36m[MEDIA]\x1b[0m \x1b[31mERROR\x1b[0m {} {} - {}",
      media_id, operation, error
    )
  }
}

pub struct ProcessingFormatter;

impl ProcessingFormatter {
  pub fn format_pipeline_start(&self, pipeline_id: &str, task_count: usize) -> String {
    format!(
      "\x1b[35m[PIPELINE]\x1b[0m \x1b[32mSTART\x1b[0m {} ({} tasks)",
      pipeline_id, task_count
    )
  }

  pub fn format_task_start(&self, pipeline_id: &str, task_id: &str, task_name: &str) -> String {
    format!(
      "\x1b[35m[TASK]\x1b[0m \x1b[32mSTART\x1b[0m {} {} {}",
      pipeline_id, task_id, task_name
    )
  }

  pub fn format_task_complete(&self, pipeline_id: &str, task_id: &str, duration_ms: u64) -> String {
    format!(
      "\x1b[35m[TASK]\x1b[0m \x1b[32mCOMPLETE\x1b[0m {} {} ({}ms)",
      pipeline_id, task_id, duration_ms
    )
  }

  pub fn format_task_error(&self, pipeline_id: &str, task_id: &str, error: &str) -> String {
    format!(
      "\x1b[35m[TASK]\x1b[0m \x1b[31mERROR\x1b[0m {} {} - {}",
      pipeline_id, task_id, error
    )
  }

  pub fn format_pipeline_complete(
    &self,
    pipeline_id: &str,
    duration_ms: u64,
    success_count: usize,
    error_count: usize,
  ) -> String {
    format!(
      "\x1b[35m[PIPELINE]\x1b[0m \x1b[32mCOMPLETE\x1b[0m {} ({}ms, {} success, {} errors)",
      pipeline_id, duration_ms, success_count, error_count
    )
  }
}

pub struct LuaFormatter;

impl LuaFormatter {
  pub fn format_script_start(&self, script_name: &str) -> String {
    format!("\x1b[33m[LUA]\x1b[0m \x1b[32mSTART\x1b[0m {}", script_name)
  }

  pub fn format_script_complete(&self, script_name: &str, duration_ms: u64) -> String {
    format!(
      "\x1b[33m[LUA]\x1b[0m \x1b[32mCOMPLETE\x1b[0m {} ({}ms)",
      script_name, duration_ms
    )
  }

  pub fn format_script_error(&self, script_name: &str, error: &str) -> String {
    format!(
      "\x1b[33m[LUA]\x1b[0m \x1b[31mERROR\x1b[0m {} - {}",
      script_name, error
    )
  }

  pub fn format_script_timeout(&self, script_name: &str, timeout_ms: u64) -> String {
    format!(
      "\x1b[33m[LUA]\x1b[0m \x1b[31mTIMEOUT\x1b[0m {} (exceeded {}ms)",
      script_name, timeout_ms
    )
  }
}

pub struct ExportFormatter;

impl ExportFormatter {
  pub fn format_export_start(&self, format: &str, output_path: &str) -> String {
    format!(
      "\x1b[34m[EXPORT]\x1b[0m \x1b[32mSTART\x1b[0m {} -> {}",
      format, output_path
    )
  }

  pub fn format_export_complete(
    &self,
    format: &str,
    output_path: &str,
    file_size: u64,
    duration_ms: u64,
  ) -> String {
    format!(
      "\x1b[34m[EXPORT]\x1b[0m \x1b[32mCOMPLETE\x1b[0m {} -> {} ({} bytes, {}ms)",
      format, output_path, file_size, duration_ms
    )
  }

  pub fn format_export_error(&self, format: &str, output_path: &str, error: &str) -> String {
    format!(
      "\x1b[34m[EXPORT]\x1b[0m \x1b[31mERROR\x1b[0m {} -> {} - {}",
      format, output_path, error
    )
  }
}

pub fn create_compact_formatter() -> CompactFormatter {
  CompactFormatter
}

pub fn create_pretty_formatter() -> PrettyFormatter {
  PrettyFormatter::new()
}

pub fn create_json_formatter() -> JsonFormatter {
  JsonFormatter::new()
}

pub fn create_media_formatter() -> MediaOperationFormatter {
  MediaOperationFormatter
}

pub fn create_processing_formatter() -> ProcessingFormatter {
  ProcessingFormatter
}

pub fn create_lua_formatter() -> LuaFormatter {
  LuaFormatter
}

pub fn create_export_formatter() -> ExportFormatter {
  ExportFormatter
}
