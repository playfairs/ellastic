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
use libloading::{
  Library,
  Symbol,
};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::ffi::{
  CStr,
  CString,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EffectPlugin {
  pub id: Uuid,
  pub name: String,
  pub version: String,
  pub description: String,
  pub author: String,
  pub supported_media_types: Vec<MediaType>,
  pub parameters: HashMap<String, PluginParameter>,
  pub capabilities: PluginCapabilities,
  pub library_path: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PluginParameter {
  pub name: String,
  pub display_name: String,
  pub description: String,
  pub parameter_type: PluginParameterType,
  pub default_value: PluginParameterValue,
  pub min_value: Option<PluginParameterValue>,
  pub max_value: Option<PluginParameterValue>,
  pub step: Option<PluginParameterValue>,
  pub options: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginParameterType {
  Integer,
  Float,
  Boolean,
  String,
  Color,
  Vector2,
  Vector3,
  Vector4,
  Enum,
  File,
  Directory,
}

#[derive(Debug, Clone)]
pub enum PluginParameterValue {
  Integer(i64),
  Float(f64),
  Boolean(bool),
  String(String),
  Color([u8; 4]),
  Vector2([f32; 2]),
  Vector3([f32; 3]),
  Vector4([f32; 4]),
  Enum(String),
  File(String),
  Directory(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCapabilities {
  Process,
  ProcessAsync,
  Preview,
  Batch,
  Custom,
}

#[derive(Debug, Clone)]
pub struct PluginInstance {
  pub plugin: EffectPlugin,
  pub library: Option<Library>,
  pub process_fn: Option<PluginProcessFn>,
  pub process_async_fn: Option<PluginProcessAsyncFn>,
  pub preview_fn: Option<PluginPreviewFn>,
  pub cleanup_fn: Option<PluginCleanupFn>,
  pub user_data: *mut std::ffi::c_void,
}

pub type PluginProcessFn = unsafe extern "C" fn(
  input_data: *const u8,
  input_size: usize,
  output_data: *mut *mut u8,
  output_size: *mut usize,
  parameters: *const std::ffi::c_char,
  user_data: *mut std::ffi::c_void,
) -> i32;

pub type PluginProcessAsyncFn = unsafe extern "C" fn(
  input_data: *const u8,
  input_size: usize,
  callback: extern "C" fn(*mut u8, usize, *mut std::ffi::c_void),
  user_data: *mut std::ffi::c_void,
) -> i32;

pub type PluginPreviewFn = unsafe extern "C" fn(
  input_data: *const u8,
  input_size: usize,
  output_data: *mut *mut u8,
  output_size: *mut usize,
  preview_width: u32,
  preview_height: u32,
  parameters: *const std::ffi::c_char,
  user_data: *mut std::ffi::c_void,
) -> i32;

pub type PluginCleanupFn = unsafe extern "C" fn(user_data: *mut std::ffi::c_void);

#[derive(Debug, Clone)]
pub struct PluginManager {
  plugins: HashMap<Uuid, EffectPlugin>,
  plugins_by_name: HashMap<String, Uuid>,
  instances: HashMap<Uuid, PluginInstance>,
  plugin_paths: Vec<String>,
  auto_load: bool,
}

#[derive(Debug, Clone)]
pub struct PluginProcessor {
  instance: PluginInstance,
  parameters: HashMap<String, PluginParameterValue>,
  cache: Arc<RwLock<PluginCache>>,
  performance_stats: Arc<RwLock<PluginPerformanceStats>>,
}

#[derive(Debug, Clone)]
pub struct PluginCache {
  cache: HashMap<String, MediaProcessor>,
  max_size: usize,
}

#[derive(Debug, Clone)]
pub struct PluginPerformanceStats {
  execution_times: Vec<std::time::Duration>,
  total_executions: u64,
  total_time: std::time::Duration,
  min_time: std::time::Duration,
  max_time: std::time::Duration,
}

impl EffectPlugin {
  pub fn new(name: String, version: String, description: String) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name,
      version,
      description,
      author: "Unknown".to_string(),
      supported_media_types: Vec::new(),
      parameters: HashMap::new(),
      capabilities: PluginCapabilities::Process,
      library_path: None,
      created_at: now,
      updated_at: now,
    }
  }

  pub fn with_author(mut self, author: String) -> Self {
    self.author = author;
    self
  }

  pub fn with_supported_media_types(mut self, media_types: Vec<MediaType>) -> Self {
    self.supported_media_types = media_types;
    self
  }

  pub fn with_parameters(mut self, parameters: HashMap<String, PluginParameter>) -> Self {
    self.parameters = parameters;
    self
  }

  pub fn with_capabilities(mut self, capabilities: PluginCapabilities) -> Self {
    self.capabilities = capabilities;
    self
  }

  pub fn with_library_path(mut self, library_path: String) -> Self {
    self.library_path = Some(library_path);
    self
  }

  pub fn add_parameter(&mut self, parameter: PluginParameter) {
    self.parameters.insert(parameter.name.clone(), parameter);
    self.update_timestamp();
  }

  pub fn remove_parameter(&mut self, name: &str) -> Option<PluginParameter> {
    let parameter = self.parameters.remove(name);
    if parameter.is_some() {
      self.update_timestamp();
    }
    parameter
  }

  pub fn get_parameter(&self, name: &str) -> Option<&PluginParameter> {
    self.parameters.get(name)
  }

  pub fn supports_media_type(&self, media_type: MediaType) -> bool {
    self.supported_media_types.contains(&media_type)
  }

  pub fn has_capability(&self, capability: PluginCapabilities) -> bool {
    self.capabilities == capability
  }

  pub fn update_timestamp(&mut self) {
    self.updated_at = Utc::now();
  }

  pub fn clone(&self) -> EffectPlugin {
    EffectPlugin {
      id: self.id,
      name: self.name.clone(),
      version: self.version.clone(),
      description: self.description.clone(),
      author: self.author.clone(),
      supported_media_types: self.supported_media_types.clone(),
      parameters: self.parameters.clone(),
      capabilities: self.capabilities,
      library_path: self.library_path.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
    }
  }
}

impl PluginManager {
  pub fn new() -> Self {
    Self {
      plugins: HashMap::new(),
      plugins_by_name: HashMap::new(),
      instances: HashMap::new(),
      plugin_paths: Vec::new(),
      auto_load: false,
    }
  }

  pub fn with_plugin_paths(mut self, paths: Vec<String>) -> Self {
    self.plugin_paths = paths;
    self
  }

  pub fn with_auto_load(mut self, auto_load: bool) -> Self {
    self.auto_load = auto_load;
    self
  }

  pub fn register_plugin(&mut self, plugin: EffectPlugin) -> Result<()> {
    if self.plugins_by_name.contains_key(&plugin.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Plugin '{}' already registered",
        plugin.name
      )));
    }

    let id = plugin.id;
    self.plugins_by_name.insert(plugin.name.clone(), id);
    self.plugins.insert(id, plugin);

    Ok(())
  }

  pub fn unregister_plugin(&mut self, id: Uuid) -> Option<EffectPlugin> {
    if let Some(plugin) = self.plugins.remove(&id) {
      self.plugins_by_name.remove(&plugin.name);

      if let Some(mut instance) = self.instances.remove(&id) {
        instance.cleanup();
      }

      Some(plugin)
    } else {
      None
    }
  }

  pub fn get_plugin(&self, id: Uuid) -> Option<&EffectPlugin> {
    self.plugins.get(&id)
  }

  pub fn get_plugin_by_name(&self, name: &str) -> Option<&EffectPlugin> {
    self
      .plugins_by_name
      .get(name)
      .and_then(|id| self.plugins.get(id))
  }

  pub fn list_plugins(&self) -> Vec<&EffectPlugin> {
    self.plugins.values().collect()
  }

  pub fn list_plugins_by_media_type(&self, media_type: MediaType) -> Vec<&EffectPlugin> {
    self
      .plugins
      .values()
      .filter(|p| p.supports_media_type(media_type))
      .collect()
  }

  pub fn search_plugins(&self, query: &str) -> Vec<&EffectPlugin> {
    let query = query.to_lowercase();
    self
      .plugins
      .values()
      .filter(|plugin| {
        plugin.name.to_lowercase().contains(&query)
          || plugin.description.to_lowercase().contains(&query)
          || plugin.author.to_lowercase().contains(&query)
      })
      .collect()
  }

  pub fn load_plugin(&mut self, plugin_id: Uuid) -> Result<()> {
    let plugin = self
      .plugins
      .get(&plugin_id)
      .ok_or_else(|| EllasticError::InvalidParameter("Plugin not found".to_string()))?;

    if self.instances.contains_key(&plugin_id) {
      return Ok(());
    }

    let library_path = plugin
      .library_path
      .as_ref()
      .ok_or_else(|| EllasticError::InvalidParameter("Plugin has no library path".to_string()))?;

    let library = unsafe { Library::new(library_path) }
      .map_err(|e| EllasticError::IOError(format!("Failed to load plugin library: {}", e)))?;

    let instance = unsafe { self.create_instance(plugin, library)? };

    self.instances.insert(plugin_id, instance);
    Ok(())
  }

  pub fn unload_plugin(&mut self, plugin_id: Uuid) -> Result<()> {
    if let Some(mut instance) = self.instances.remove(&plugin_id) {
      instance.cleanup();
    }
    Ok(())
  }

  pub fn create_processor(&mut self, plugin_id: Uuid) -> Result<PluginProcessor> {
    let plugin = self
      .plugins
      .get(&plugin_id)
      .ok_or_else(|| EllasticError::InvalidParameter("Plugin not found".to_string()))?;

    if !self.instances.contains_key(&plugin_id) {
      self.load_plugin(plugin_id)?;
    }

    let instance = self
      .instances
      .get(&plugin_id)
      .ok_or_else(|| EllasticError::InvalidParameter("Plugin instance not found".to_string()))?
      .clone();

    let default_values = plugin
      .parameters
      .iter()
      .map(|(name, param)| (name.clone(), param.default_value.clone()))
      .collect();

    Ok(PluginProcessor {
      instance,
      parameters: default_values,
      cache: Arc::new(RwLock::new(PluginCache::new())),
      performance_stats: Arc::new(RwLock::new(PluginPerformanceStats::new())),
    })
  }

  pub fn load_all_plugins(&mut self) -> Result<()> {
    for plugin in self.plugins.values() {
      if let Some(library_path) = &plugin.library_path {
        let library = unsafe { Library::new(library_path) }
          .map_err(|e| EllasticError::IOError(format!("Failed to load plugin library: {}", e)))?;

        let instance = unsafe { self.create_instance(plugin, library)? };
        self.instances.insert(plugin.id, instance);
      }
    }
    Ok(())
  }

  pub fn unload_all_plugins(&mut self) {
    for (plugin_id, mut instance) in self.instances.drain() {
      instance.cleanup();
    }
  }

  pub fn scan_plugin_directory(&mut self, directory: &str) -> Result<()> {
    let plugin_files = self.find_plugin_files(directory)?;

    for plugin_file in plugin_files {
      if let Ok(plugin) = self.load_plugin_metadata(&plugin_file) {
        self.register_plugin(plugin)?;
      }
    }

    Ok(())
  }

  unsafe fn create_instance(
    &self,
    plugin: &EffectPlugin,
    library: Library,
  ) -> Result<PluginInstance> {
    let process_fn = if plugin.has_capability(PluginCapabilities::Process) {
      Some(
        library
          .get::<PluginProcessFn>(b"plugin_process")
          .map_err(|_| {
            EllasticError::InvalidParameter("Plugin missing process function".to_string())
          })?,
      )
    } else {
      None
    };

    let process_async_fn = if plugin.has_capability(PluginCapabilities::ProcessAsync) {
      Some(
        library
          .get::<PluginProcessAsyncFn>(b"plugin_process_async")
          .map_err(|_| {
            EllasticError::InvalidParameter("Plugin missing async process function".to_string())
          })?,
      )
    } else {
      None
    };

    let preview_fn = if plugin.has_capability(PluginCapabilities::Preview) {
      Some(
        library
          .get::<PluginPreviewFn>(b"plugin_preview")
          .map_err(|_| {
            EllasticError::InvalidParameter("Plugin missing preview function".to_string())
          })?,
      )
    } else {
      None
    };

    let cleanup_fn = library.get::<PluginCleanupFn>(b"plugin_cleanup").ok();

    Ok(PluginInstance {
      plugin: plugin.clone(),
      library: Some(library),
      process_fn,
      process_async_fn,
      preview_fn,
      cleanup_fn,
      user_data: std::ptr::null_mut(),
    })
  }

  fn find_plugin_files(&self, directory: &str) -> Result<Vec<String>> {
    let mut plugin_files = Vec::new();

    for entry in std::fs::read_dir(directory)
      .map_err(|e| EllasticError::IOError(format!("Failed to read plugin directory: {}", e)))?
    {
      let entry = entry
        .map_err(|e| EllasticError::IOError(format!("Failed to read directory entry: {}", e)))?;

      let path = entry.path();

      if path.is_file() {
        if let Some(extension) = path.extension() {
          if extension == "so" || extension == "dll" || extension == "dylib" {
            plugin_files.push(path.to_string_lossy().to_string());
          }
        }
      }
    }

    Ok(plugin_files)
  }

  fn load_plugin_metadata(&self, library_path: &str) -> Result<EffectPlugin> {
    let library = unsafe { Library::new(library_path) }
      .map_err(|e| EllasticError::IOError(format!("Failed to load plugin library: {}", e)))?;

    unsafe {
      let get_name = library
        .get::<extern "C" fn() -> *const std::ffi::c_char>(b"plugin_get_name")
        .map_err(|_| {
          EllasticError::InvalidParameter("Plugin missing get_name function".to_string())
        })?;

      let get_version = library
        .get::<extern "C" fn() -> *const std::ffi::c_char>(b"plugin_get_version")
        .map_err(|_| {
          EllasticError::InvalidParameter("Plugin missing get_version function".to_string())
        })?;

      let get_description = library
        .get::<extern "C" fn() -> *const std::ffi::c_char>(b"plugin_get_description")
        .map_err(|_| {
          EllasticError::InvalidParameter("Plugin missing get_description function".to_string())
        })?;

      let name = CStr::from_ptr(get_name()).to_string_lossy().to_string();
      let version = CStr::from_ptr(get_version()).to_string_lossy().to_string();
      let description = CStr::from_ptr(get_description())
        .to_string_lossy()
        .to_string();

      let mut plugin =
        EffectPlugin::new(name, version, description).with_library_path(library_path.to_string());

      if let Ok(get_author) =
        library.get::<extern "C" fn() -> *const std::ffi::c_char>(b"plugin_get_author")
      {
        let author = CStr::from_ptr(get_author()).to_string_lossy().to_string();
        plugin = plugin.with_author(author);
      }

      Ok(plugin)
    }
  }

  pub fn clear(&mut self) {
    self.unload_all_plugins();
    self.plugins.clear();
    self.plugins_by_name.clear();
  }

  pub fn len(&self) -> usize {
    self.plugins.len()
  }

  pub fn is_empty(&self) -> bool {
    self.plugins.is_empty()
  }

  pub fn clone(&self) -> PluginManager {
    PluginManager {
      plugins: self.plugins.clone(),
      plugins_by_name: self.plugins_by_name.clone(),
      instances: HashMap::new(),
      plugin_paths: self.plugin_paths.clone(),
      auto_load: self.auto_load,
    }
  }
}

impl Drop for PluginInstance {
  fn drop(&mut self) {
    self.cleanup();
  }
}

impl PluginInstance {
  pub fn process(
    &mut self,
    input_data: &[u8],
    parameters: &HashMap<String, PluginParameterValue>,
  ) -> Result<Vec<u8>> {
    if let Some(process_fn) = self.process_fn {
      let param_string = self.serialize_parameters(parameters);
      let param_cstring = CString::new(param_string).map_err(|_| {
        EllasticError::InvalidParameter("Failed to serialize parameters".to_string())
      })?;

      let mut output_data: *mut u8 = std::ptr::null_mut();
      let mut output_size: usize = 0;

      let result = unsafe {
        process_fn(
          input_data.as_ptr(),
          input_data.len(),
          &mut output_data,
          &mut output_size,
          param_cstring.as_ptr(),
          self.user_data,
        )
      };

      if result == 0 {
        let output_slice = unsafe { std::slice::from_raw_parts(output_data, output_size) };
        let output_vec = output_slice.to_vec();

        unsafe {
          libc::free(output_data as *mut libc::c_void);
        }

        Ok(output_vec)
      } else {
        Err(EllasticError::ProcessingError(format!(
          "Plugin processing failed with code: {}",
          result
        )))
      }
    } else {
      Err(EllasticError::UnsupportedOperation(
        "Plugin doesn't support processing".to_string(),
      ))
    }
  }

  pub fn process_async(
    &mut self,
    input_data: &[u8],
    callback: extern "C" fn(*mut u8, usize, *mut std::ffi::c_void),
    user_data: *mut std::ffi::c_void,
  ) -> Result<()> {
    if let Some(process_async_fn) = self.process_async_fn {
      let result =
        unsafe { process_async_fn(input_data.as_ptr(), input_data.len(), callback, user_data) };

      if result == 0 {
        Ok(())
      } else {
        Err(EllasticError::ProcessingError(format!(
          "Plugin async processing failed with code: {}",
          result
        )))
      }
    } else {
      Err(EllasticError::UnsupportedOperation(
        "Plugin doesn't support async processing".to_string(),
      ))
    }
  }

  pub fn preview(
    &mut self,
    input_data: &[u8],
    preview_width: u32,
    preview_height: u32,
    parameters: &HashMap<String, PluginParameterValue>,
  ) -> Result<Vec<u8>> {
    if let Some(preview_fn) = self.preview_fn {
      let param_string = self.serialize_parameters(parameters);
      let param_cstring = CString::new(param_string).map_err(|_| {
        EllasticError::InvalidParameter("Failed to serialize parameters".to_string())
      })?;

      let mut output_data: *mut u8 = std::ptr::null_mut();
      let mut output_size: usize = 0;

      let result = unsafe {
        preview_fn(
          input_data.as_ptr(),
          input_data.len(),
          &mut output_data,
          &mut output_size,
          preview_width,
          preview_height,
          param_cstring.as_ptr(),
          self.user_data,
        )
      };

      if result == 0 {
        let output_slice = unsafe { std::slice::from_raw_parts(output_data, output_size) };
        let output_vec = output_slice.to_vec();

        unsafe {
          libc::free(output_data as *mut libc::c_void);
        }

        Ok(output_vec)
      } else {
        Err(EllasticError::ProcessingError(format!(
          "Plugin preview failed with code: {}",
          result
        )))
      }
    } else {
      Err(EllasticError::UnsupportedOperation(
        "Plugin doesn't support preview".to_string(),
      ))
    }
  }

  fn serialize_parameters(&self, parameters: &HashMap<String, PluginParameterValue>) -> String {
    let mut param_string = String::new();

    for (name, value) in parameters {
      if !param_string.is_empty() {
        param_string.push(';');
      }

      param_string.push_str(&name);
      param_string.push('=');
      param_string.push_str(&self.value_to_string(value));
    }

    param_string
  }

  fn value_to_string(&self, value: &PluginParameterValue) -> String {
    match value {
      PluginParameterValue::Integer(v) => v.to_string(),
      PluginParameterValue::Float(v) => v.to_string(),
      PluginParameterValue::Boolean(v) => v.to_string(),
      PluginParameterValue::String(v) => v.clone(),
      PluginParameterValue::Color(v) => {
        format!("#{:02X}{:02X}{:02X}{:02X}", v[0], v[1], v[2], v[3])
      }
      PluginParameterValue::Vector2(v) => format!("{},{}", v[0], v[1]),
      PluginParameterValue::Vector3(v) => format!("{},{},{}", v[0], v[1], v[2]),
      PluginParameterValue::Vector4(v) => format!("{},{},{},{}", v[0], v[1], v[2], v[3]),
      PluginParameterValue::Enum(v) => v.clone(),
      PluginParameterValue::File(v) => v.clone(),
      PluginParameterValue::Directory(v) => v.clone(),
    }
  }

  fn cleanup(&mut self) {
    if let Some(cleanup_fn) = self.cleanup_fn {
      unsafe {
        cleanup_fn(self.user_data);
      }
    }
  }

  pub fn clone(&self) -> PluginInstance {
    PluginInstance {
      plugin: self.plugin.clone(),
      library: None,
      process_fn: None,
      process_async_fn: None,
      preview_fn: None,
      cleanup_fn: None,
      user_data: std::ptr::null_mut(),
    }
  }
}

impl PluginProcessor {
  pub fn instance(&self) -> &PluginInstance {
    &self.instance
  }

  pub fn parameters(&self) -> &HashMap<String, PluginParameterValue> {
    &self.parameters
  }

  pub fn parameters_mut(&mut self) -> &mut HashMap<String, PluginParameterValue> {
    &mut self.parameters
  }

  pub fn cache(&self) -> Arc<RwLock<PluginCache>> {
    self.cache.clone()
  }

  pub fn performance_stats(&self) -> Arc<RwLock<PluginPerformanceStats>> {
    self.performance_stats.clone()
  }

  pub fn set_parameter(&mut self, name: String, value: PluginParameterValue) -> Result<()> {
    if let Some(parameter) = self.instance.plugin.get_parameter(&name) {
      self.validate_parameter_value(parameter, &value)?;
      self.parameters.insert(name, value);
      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Unknown parameter: {}",
        name
      )))
    }
  }

  pub fn get_parameter(&self, name: &str) -> Option<&PluginParameterValue> {
    self.parameters.get(name)
  }

  pub fn reset_parameter(&mut self, name: &str) -> Result<()> {
    if let Some(parameter) = self.instance.plugin.get_parameter(name) {
      self
        .parameters
        .insert(name.to_string(), parameter.default_value.clone());
      Ok(())
    } else {
      Err(EllasticError::InvalidParameter(format!(
        "Unknown parameter: {}",
        name
      )))
    }
  }

  pub fn reset_all_parameters(&mut self) {
    self.parameters = self
      .instance
      .plugin
      .parameters
      .iter()
      .map(|(name, param)| (name.clone(), param.default_value.clone()))
      .collect();
  }

  pub fn process(&mut self, media_processor: &mut MediaProcessor) -> Result<()> {
    let start_time = std::time::Instant::now();

    let input_data = media_processor.data().to_bytes();
    let output_data = self.instance.process(&input_data, &self.parameters)?;

    let new_media_data = MediaData::from_bytes(&output_data)?;
    *media_processor = MediaProcessor::new(new_media_data);

    let processing_time = start_time.elapsed();
    self
      .performance_stats
      .write()
      .record_execution(processing_time);

    Ok(())
  }

  pub fn process_async(
    &mut self,
    media_processor: MediaProcessor,
  ) -> Result<tokio::task::JoinHandle<Result<MediaProcessor>>> {
    let input_data = media_processor.data().to_bytes();
    let instance = self.instance.clone();
    let parameters = self.parameters.clone();
    let performance_stats = self.performance_stats.clone();

    let handle = tokio::spawn(async move {
      let start_time = std::time::Instant::now();

      let output_data = {
        let mut instance = instance;
        instance.process(&input_data, &parameters)?
      };

      let new_media_data = MediaData::from_bytes(&output_data)?;
      let result = MediaProcessor::new(new_media_data);

      let processing_time = start_time.elapsed();
      performance_stats.write().record_execution(processing_time);

      Ok(result)
    });

    Ok(handle)
  }

  pub fn preview(
    &mut self,
    media_processor: &mut MediaProcessor,
    preview_size: (u32, u32),
  ) -> Result<()> {
    let original_size = (media_processor.width(), media_processor.height());

    if original_size != preview_size {
      media_processor.resize(preview_size.0, preview_size.1)?;
    }

    self.process(media_processor)?;

    if original_size != preview_size {
      media_processor.resize(original_size.0, original_size.1)?;
    }

    Ok(())
  }

  pub fn batch_process(
    &mut self,
    media_processors: &mut [MediaProcessor],
  ) -> Result<Vec<Result<()>>> {
    media_processors
      .par_iter_mut()
      .map(|processor| {
        let mut temp_processor = processor.clone();
        let mut temp_plugin_processor = PluginProcessor {
          instance: self.instance.clone(),
          parameters: self.parameters.clone(),
          cache: self.cache.clone(),
          performance_stats: self.performance_stats.clone(),
        };

        match temp_plugin_processor.process(&mut temp_processor) {
          Ok(()) => {
            *processor = temp_processor;
            Ok(())
          }
          Err(e) => Err(e),
        }
      })
      .collect()
  }

  fn validate_parameter_value(
    &self,
    parameter: &PluginParameter,
    value: &PluginParameterValue,
  ) -> Result<()> {
    if !self.is_compatible_type(&parameter.parameter_type, value) {
      return Err(EllasticError::InvalidParameter(format!(
        "Parameter '{}' type mismatch: expected {:?}, got {:?}",
        parameter.name, parameter.parameter_type, value
      )));
    }

    if let (Some(min), Some(max)) = (&parameter.min_value, &parameter.max_value) {
      if !self.is_in_range(value, min, max) {
        return Err(EllasticError::InvalidParameter(format!(
          "Parameter '{}' value out of range: {:?} not in [{:?}, {:?}]",
          parameter.name, value, min, max
        )));
      }
    }

    Ok(())
  }

  fn is_compatible_type(
    &self,
    parameter_type: &PluginParameterType,
    value: &PluginParameterValue,
  ) -> bool {
    match (parameter_type, value) {
      (PluginParameterType::Integer, PluginParameterValue::Integer(_)) => true,
      (PluginParameterType::Float, PluginParameterValue::Float(_)) => true,
      (PluginParameterType::Boolean, PluginParameterValue::Boolean(_)) => true,
      (PluginParameterType::String, PluginParameterValue::String(_)) => true,
      (PluginParameterType::Color, PluginParameterValue::Color(_)) => true,
      (PluginParameterType::Vector2, PluginParameterValue::Vector2(_)) => true,
      (PluginParameterType::Vector3, PluginParameterValue::Vector3(_)) => true,
      (PluginParameterType::Vector4, PluginParameterValue::Vector4(_)) => true,
      (PluginParameterType::Enum, PluginParameterValue::Enum(_)) => true,
      (PluginParameterType::File, PluginParameterValue::File(_)) => true,
      (PluginParameterType::Directory, PluginParameterValue::Directory(_)) => true,
      _ => false,
    }
  }

  fn is_in_range(
    &self,
    value: &PluginParameterValue,
    min: &PluginParameterValue,
    max: &PluginParameterValue,
  ) -> bool {
    match (value, min, max) {
      (
        PluginParameterValue::Integer(v),
        PluginParameterValue::Integer(min_v),
        PluginParameterValue::Integer(max_v),
      ) => v >= *min_v && v <= *max_v,
      (
        PluginParameterValue::Float(v),
        PluginParameterValue::Float(min_v),
        PluginParameterValue::Float(max_v),
      ) => v >= *min_v && v <= *max_v,
      (
        PluginParameterValue::String(v),
        PluginParameterValue::String(min_v),
        PluginParameterValue::String(max_v),
      ) => v >= min_v && v <= max_v,
      _ => true,
    }
  }

  pub fn clone(&self) -> PluginProcessor {
    PluginProcessor {
      instance: self.instance.clone(),
      parameters: self.parameters.clone(),
      cache: self.cache.clone(),
      performance_stats: self.performance_stats.clone(),
    }
  }
}

impl PluginCache {
  pub fn new() -> Self {
    Self {
      cache: HashMap::new(),
      max_size: 100,
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

  pub fn clone(&self) -> PluginCache {
    PluginCache {
      cache: self.cache.clone(),
      max_size: self.max_size,
    }
  }
}

impl PluginPerformanceStats {
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

    if self.execution_times.len() > 100 {
      self.execution_times.drain(0..50);
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

  pub fn clone(&self) -> PluginPerformanceStats {
    PluginPerformanceStats {
      execution_times: self.execution_times.clone(),
      total_executions: self.total_executions,
      total_time: self.total_time,
      min_time: self.min_time,
      max_time: self.max_time,
    }
  }
}

pub fn create_effect_plugin(name: String, version: String, description: String) -> EffectPlugin {
  EffectPlugin::new(name, version, description)
}

pub fn create_plugin_manager() -> PluginManager {
  PluginManager::new()
}

pub fn create_plugin_processor(instance: PluginInstance) -> PluginProcessor {
  PluginProcessor {
    instance,
    parameters: HashMap::new(),
    cache: Arc::new(RwLock::new(PluginCache::new())),
    performance_stats: Arc::new(RwLock::new(PluginPerformanceStats::new())),
  }
}

pub fn create_plugin_cache() -> PluginCache {
  PluginCache::new()
}

pub fn create_plugin_cache_with_max_size(max_size: usize) -> PluginCache {
  PluginCache::with_max_size(max_size)
}

pub fn create_plugin_performance_stats() -> PluginPerformanceStats {
  PluginPerformanceStats::new()
}
