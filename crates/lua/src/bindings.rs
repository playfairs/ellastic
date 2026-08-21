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
  AnyUserData,
  Error as LuaError,
  Function,
  Lua,
  Table,
  UserData,
  Value,
};
use parking_lot::RwLock;
use rayon::prelude::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LuaBindings {
  lua: Lua,
  registered_types: HashMap<String, TypeRegistration>,
  registered_functions: HashMap<String, FunctionRegistration>,
  registered_modules: HashMap<String, ModuleRegistration>,
  type_registry: Arc<RwLock<TypeRegistry>>,
}

#[derive(Debug, Clone)]
pub struct TypeRegistration {
  pub name: String,
  pub type_id: Uuid,
  pub constructor: Function,
  pub methods: HashMap<String, Function>,
  pub metamethods: HashMap<String, Function>,
  pub fields: HashMap<String, FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
  pub field_type: FieldType,
  pub getter: Option<Function>,
  pub setter: Option<Function>,
  pub readonly: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
  String,
  Number,
  Boolean,
  Table,
  Function,
  UserData,
}

#[derive(Debug, Clone)]
pub struct FunctionRegistration {
  pub name: String,
  pub function: Function,
  pub signature: FunctionSignature,
  pub documentation: String,
  pub category: FunctionCategory,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
  pub parameters: Vec<ParameterInfo>,
  pub return_type: ReturnType,
  pub variadic: bool,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
  pub name: String,
  pub parameter_type: ParameterType,
  pub optional: bool,
  pub default_value: Option<String>,
  pub description: String,
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
  Debug,
  Custom,
}

#[derive(Debug, Clone)]
pub struct ModuleRegistration {
  pub name: String,
  pub module: Table,
  pub functions: HashMap<String, FunctionRegistration>,
  pub types: HashMap<String, TypeRegistration>,
  pub documentation: String,
}

#[derive(Debug, Clone)]
pub struct TypeRegistry {
  pub types: HashMap<String, TypeRegistration>,
  pub type_mappings: HashMap<String, String>,
  pub inheritance_tree: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MediaProcessorBinding {
  pub processor: MediaProcessor,
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EffectProcessorBinding {
  pub processor: EffectProcessor,
  pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct PipelineProcessorBinding {
  pub processor: PipelineProcessor,
  pub metadata: HashMap<String, String>,
}

impl LuaBindings {
  pub fn new() -> Result<Self> {
    let lua = Lua::new();

    Ok(Self {
      lua,
      registered_types: HashMap::new(),
      registered_functions: HashMap::new(),
      registered_modules: HashMap::new(),
      type_registry: Arc::new(RwLock::new(TypeRegistry::new())),
    })
  }

  pub fn lua(&self) -> &Lua {
    &self.lua
  }

  pub fn register_type(&mut self, type_registration: TypeRegistration) -> Result<()> {
    let name = type_registration.name.clone();

    self
      .type_registry
      .write()
      .register_type(type_registration.clone())?;

    self.create_userdata_type(&type_registration)?;

    self.registered_types.insert(name, type_registration);

    Ok(())
  }

  pub fn register_function(&mut self, function_registration: FunctionRegistration) -> Result<()> {
    let name = function_registration.name.clone();
    let function = function_registration.function.clone();

    self.lua.globals().set(&name, function)?;

    self
      .registered_functions
      .insert(name, function_registration);

    Ok(())
  }

  pub fn register_module(&mut self, module_registration: ModuleRegistration) -> Result<()> {
    let name = module_registration.name.clone();
    let module = module_registration.module.clone();

    self.lua.globals().set(&name, module)?;

    self.registered_modules.insert(name, module_registration);

    Ok(())
  }

  pub fn get_type(&self, name: &str) -> Option<&TypeRegistration> {
    self.registered_types.get(name)
  }

  pub fn get_function(&self, name: &str) -> Option<&FunctionRegistration> {
    self.registered_functions.get(name)
  }

  pub fn get_module(&self, name: &str) -> Option<&ModuleRegistration> {
    self.registered_modules.get(name)
  }

  pub fn list_types(&self) -> Vec<&String> {
    self.registered_types.keys().collect()
  }

  pub fn list_functions(&self) -> Vec<&String> {
    self.registered_functions.keys().collect()
  }

  pub fn list_modules(&self) -> Vec<&String> {
    self.registered_modules.keys().collect()
  }

  fn create_userdata_type(&mut self, type_registration: &TypeRegistration) -> Result<()> {
    let lua = &self.lua;

    let metatable = lua.create_table()?;

    for (method_name, function) in &type_registration.metamethods {
      metatable.set(method_name, function.clone())?;
    }

    let methods_table = lua.create_table()?;
    for (method_name, function) in &type_registration.methods {
      methods_table.set(method_name, function.clone())?;
    }
    metatable.set("__index", methods_table)?;

    let type_name = type_registration.name.clone();
    lua
      .globals()
      .set(&type_name, type_registration.constructor.clone())?;

    Ok(())
  }

  pub fn register_media_processor_type(&mut self) -> Result<()> {
    let lua = &self.lua;

    let metatable = lua.create_table()?;

    metatable.set(
      "load",
      lua.create_function(|lua, path: String| {
        let processor = MediaProcessor::from_file(&path)
          .map_err(|e| EllasticError::IOError(format!("Failed to load media: {}", e)))?;
        Ok(MediaProcessorBinding {
          processor,
          metadata: HashMap::new(),
        })
      })?,
    )?;

    metatable.set(
      "save",
      lua.create_function(|lua, this: MediaProcessorBinding, path: String| {
        this
          .processor
          .save_to_file(&path)
          .map_err(|e| EllasticError::IOError(format!("Failed to save media: {}", e)))?;
        Ok(())
      })?,
    )?;

    metatable.set(
      "width",
      lua.create_function(|lua, this: MediaProcessorBinding| Ok(this.processor.width()))?,
    )?;

    metatable.set(
      "height",
      lua.create_function(|lua, this: MediaProcessorBinding| Ok(this.processor.height()))?,
    )?;

    metatable.set(
      "media_type",
      lua.create_function(|lua, this: MediaProcessorBinding| {
        format!("{:?}", this.processor.media_type())
      })?,
    )?;

    metatable.set(
      "apply_effect",
      lua.create_function(
        |lua, this: MediaProcessorBinding, effect_type: String, parameters: Table| {
          let effect_type = parse_effect_type(&effect_type)?;
          let mut effect_processor = EffectProcessor::new(effect_type);

          for pair in parameters.pairs::<String, Value>() {
            let (key, value) = pair.map_err(|e| {
              EllasticError::ScriptError(format!("Failed to iterate parameters: {}", e))
            })?;
            let param_value = lua_value_to_string(&value)?;
            effect_processor.set_parameter(key, param_value)?;
          }

          let result = effect_processor
            .apply_effect(&this.processor)
            .map_err(|e| {
              EllasticError::ProcessingError(format!("Effect application failed: {}", e))
            })?;

          Ok(MediaProcessorBinding {
            processor: result,
            metadata: HashMap::new(),
          })
        },
      )?,
    )?;

    metatable.set(
      "__tostring",
      lua.create_function(|lua, this: MediaProcessorBinding| {
        format!(
          "MediaProcessor({}x{}, {:?})",
          this.processor.width(),
          this.processor.height(),
          this.processor.media_type()
        )
      })?,
    )?;

    let media_processor_type = lua.create_table()?;
    media_processor_type.set("__metatable", metatable)?;

    lua.globals().set("MediaProcessor", media_processor_type)?;

    Ok(())
  }

  pub fn register_effect_processor_type(&mut self) -> Result<()> {
    let lua = &self.lua;

    let metatable = lua.create_table()?;

    metatable.set(
      "new",
      lua.create_function(|lua, effect_type: String| {
        let effect_type = parse_effect_type(&effect_type)?;
        let processor = EffectProcessor::new(effect_type);
        Ok(EffectProcessorBinding {
          processor,
          metadata: HashMap::new(),
        })
      })?,
    )?;

    metatable.set(
      "set_parameter",
      lua.create_function(
        |lua, this: EffectProcessorBinding, name: String, value: Value| {
          let value_str = lua_value_to_string(&value)?;
          this.processor.set_parameter(name, value_str)?;
          Ok(())
        },
      )?,
    )?;

    metatable.set(
      "get_parameter",
      lua.create_function(|lua, this: EffectProcessorBinding, name: String| {
        this
          .processor
          .get_parameter(&name)
          .map_err(|e| EllasticError::InvalidParameter(format!("Failed to get parameter: {}", e)))
      })?,
    )?;

    metatable.set(
      "apply",
      lua.create_function(
        |lua, this: EffectProcessorBinding, processor: MediaProcessorBinding| {
          let result = this
            .processor
            .apply_effect(&processor.processor)
            .map_err(|e| {
              EllasticError::ProcessingError(format!("Effect application failed: {}", e))
            })?;

          Ok(MediaProcessorBinding {
            processor: result,
            metadata: HashMap::new(),
          })
        },
      )?,
    )?;

    metatable.set(
      "__tostring",
      lua.create_function(|lua, this: EffectProcessorBinding| {
        format!("EffectProcessor({:?})", this.processor.effect_type())
      })?,
    )?;

    let effect_processor_type = lua.create_table()?;
    effect_processor_type.set("__metatable", metatable)?;

    lua
      .globals()
      .set("EffectProcessor", effect_processor_type)?;

    Ok(())
  }

  pub fn register_pipeline_processor_type(&mut self) -> Result<()> {
    let lua = &self.lua;

    let metatable = lua.create_table()?;

    metatable.set(
      "new",
      lua.create_function(|lua, name: String, description: String| {
        let processor = PipelineProcessor::new(name, description);
        Ok(PipelineProcessorBinding {
          processor,
          metadata: HashMap::new(),
        })
      })?,
    )?;

    metatable.set(
      "add_node",
      lua.create_function(
        |lua, this: PipelineProcessorBinding, node_type: String, node_name: String| {
          let node_id = Uuid::new_v4();
          Ok(node_id.to_string())
        },
      )?,
    )?;

    metatable.set(
      "execute",
      lua.create_function(
        |lua, this: PipelineProcessorBinding, input: MediaProcessorBinding| {
          let result = this.processor.execute(input.processor).map_err(|e| {
            EllasticError::ProcessingError(format!("Pipeline execution failed: {}", e))
          })?;

          Ok(MediaProcessorBinding {
            processor: result,
            metadata: HashMap::new(),
          })
        },
      )?,
    )?;

    metatable.set(
      "__tostring",
      lua.create_function(|lua, this: PipelineProcessorBinding| {
        format!("PipelineProcessor({})", this.processor.name())
      })?,
    )?;

    let pipeline_processor_type = lua.create_table()?;
    pipeline_processor_type.set("__metatable", metatable)?;

    lua
      .globals()
      .set("PipelineProcessor", pipeline_processor_type)?;

    Ok(())
  }

  pub fn register_core_functions(&mut self) -> Result<()> {
    let lua = &self.lua;

    lua.globals().set(
      "log",
      lua.create_function(|lua, level: String, message: String| {
        match level.as_str() {
          "trace" => tracing::trace!("{}", message),
          "debug" => tracing::debug!("{}", message),
          "info" => tracing::info!("{}", message),
          "warn" => tracing::warn!("{}", message),
          "error" => tracing::error!("{}", message),
          _ => tracing::info!("[{}] {}", level, message),
        }
        Ok(())
      })?,
    )?;

    lua.globals().set(
      "sleep",
      lua.create_function(|lua, seconds: f64| {
        std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
        Ok(())
      })?,
    )?;

    lua.globals().set(
      "random",
      lua.create_function(|lua, min: Option<f64>, max: Option<f64>| {
        let mut rng = create_random_generator();
        let value = rng.gen_range(min.unwrap_or(0.0), max.unwrap_or(1.0));
        Ok(value)
      })?,
    )?;

    lua.globals().set(
      "uuid",
      lua.create_function(|lua, ()| Uuid::new_v4().to_string())?,
    )?;

    lua.globals().set(
      "timestamp",
      lua.create_function(|lua, ()| Utc::now().timestamp())?,
    )?;

    lua.globals().set(
      "json_encode",
      lua.create_function(|lua, value: Value| {
        let json_value = lua_value_to_json(value)?;
        serde_json::to_string(&json_value)
          .map_err(|e| EllasticError::SerializationError(format!("JSON encoding failed: {}", e)))
      })?,
    )?;

    lua.globals().set(
      "json_decode",
      lua.create_function(|lua, json_str: String| {
        let json_value: JsonValue = serde_json::from_str(&json_str)
          .map_err(|e| EllasticError::SerializationError(format!("JSON decoding failed: {}", e)))?;
        json_value_to_lua(&lua, json_value)
      })?,
    )?;

    Ok(())
  }

  pub fn register_media_functions(&mut self) -> Result<()> {
    let lua = &self.lua;

    lua.globals().set(
      "load_image",
      lua.create_function(|lua, path: String| {
        let processor = MediaProcessor::from_file(&path)
          .map_err(|e| EllasticError::IOError(format!("Failed to load image: {}", e)))?;

        if processor.media_type() != MediaType::Image {
          return Err(EllasticError::InvalidParameter(
            "File is not an image".to_string(),
          ));
        }

        Ok(MediaProcessorBinding {
          processor,
          metadata: HashMap::new(),
        })
      })?,
    )?;

    lua.globals().set(
      "save_image",
      lua.create_function(|lua, binding: MediaProcessorBinding, path: String| {
        if binding.processor.media_type() != MediaType::Image {
          return Err(EllasticError::InvalidParameter(
            "Processor is not an image".to_string(),
          ));
        }

        binding
          .processor
          .save_to_file(&path)
          .map_err(|e| EllasticError::IOError(format!("Failed to save image: {}", e)))?;
        Ok(())
      })?,
    )?;

    lua.globals().set(
      "create_image",
      lua.create_function(|lua, width: u32, height: u32, format: String| {
        let media_data = MediaData::empty_image(width, height, &format)
          .map_err(|e| EllasticError::InvalidParameter(format!("Failed to create image: {}", e)))?;
        let processor = MediaProcessor::new(media_data);

        Ok(MediaProcessorBinding {
          processor,
          metadata: HashMap::new(),
        })
      })?,
    )?;

    lua.globals().set(
      "load_audio",
      lua.create_function(|lua, path: String| {
        let processor = MediaProcessor::from_file(&path)
          .map_err(|e| EllasticError::IOError(format!("Failed to load audio: {}", e)))?;

        if processor.media_type() != MediaType::Audio {
          return Err(EllasticError::InvalidParameter(
            "File is not audio".to_string(),
          ));
        }

        Ok(MediaProcessorBinding {
          processor,
          metadata: HashMap::new(),
        })
      })?,
    )?;

    lua.globals().set(
      "save_audio",
      lua.create_function(|lua, binding: MediaProcessorBinding, path: String| {
        if binding.processor.media_type() != MediaType::Audio {
          return Err(EllasticError::InvalidParameter(
            "Processor is not audio".to_string(),
          ));
        }

        binding
          .processor
          .save_to_file(&path)
          .map_err(|e| EllasticError::IOError(format!("Failed to save audio: {}", e)))?;
        Ok(())
      })?,
    )?;

    Ok(())
  }

  pub fn register_effect_functions(&mut self) -> Result<()> {
    let lua = &self.lua;

    lua.globals().set(
      "apply_effect",
      lua.create_function(
        |lua, effect_type: String, processor: MediaProcessorBinding, parameters: Table| {
          let effect_type = parse_effect_type(&effect_type)?;
          let mut effect_processor = EffectProcessor::new(effect_type);

          for pair in parameters.pairs::<String, Value>() {
            let (key, value) = pair.map_err(|e| {
              EllasticError::ScriptError(format!("Failed to iterate parameters: {}", e))
            })?;
            let param_value = lua_value_to_string(&value)?;
            effect_processor.set_parameter(key, param_value)?;
          }

          let result = effect_processor
            .apply_effect(&processor.processor)
            .map_err(|e| {
              EllasticError::ProcessingError(format!("Effect application failed: {}", e))
            })?;

          Ok(MediaProcessorBinding {
            processor: result,
            metadata: HashMap::new(),
          })
        },
      )?,
    )?;

    lua.globals().set(
      "create_effect",
      lua.create_function(|lua, effect_type: String| {
        let effect_type = parse_effect_type(&effect_type)?;
        let processor = EffectProcessor::new(effect_type);

        Ok(EffectProcessorBinding {
          processor,
          metadata: HashMap::new(),
        })
      })?,
    )?;

    Ok(())
  }

  pub fn register_pipeline_functions(&mut self) -> Result<()> {
    let lua = &self.lua;

    lua.globals().set(
      "create_pipeline",
      lua.create_function(|lua, name: String, description: String| {
        let processor = PipelineProcessor::new(name, description);

        Ok(PipelineProcessorBinding {
          processor,
          metadata: HashMap::new(),
        })
      })?,
    )?;

    lua.globals().set(
      "execute_pipeline",
      lua.create_function(
        |lua, binding: PipelineProcessorBinding, input: MediaProcessorBinding| {
          let result = binding.processor.execute(input.processor).map_err(|e| {
            EllasticError::ProcessingError(format!("Pipeline execution failed: {}", e))
          })?;

          Ok(MediaProcessorBinding {
            processor: result,
            metadata: HashMap::new(),
          })
        },
      )?,
    )?;

    Ok(())
  }

  pub fn register_debug_functions(&mut self) -> Result<()> {
    let lua = &self.lua;

    lua.globals().set(
      "debug_print",
      lua.create_function(|lua, message: String| {
        eprintln!("DEBUG: {}", message);
        Ok(())
      })?,
    )?;

    lua.globals().set(
      "debug_break",
      lua.create_function(|lua, ()| {
        eprintln!("DEBUG BREAKPOINT");
        Ok(())
      })?,
    )?;

    lua.globals().set(
      "debug_trace",
      lua.create_function(|lua, ()| {
        eprintln!("DEBUG STACK TRACE");
        Ok(())
      })?,
    )?;

    Ok(())
  }

  pub fn register_all_bindings(&mut self) -> Result<()> {
    self.register_core_functions()?;
    self.register_media_functions()?;
    self.register_effect_functions()?;
    self.register_pipeline_functions()?;
    self.register_debug_functions()?;
    self.register_media_processor_type()?;
    self.register_effect_processor_type()?;
    self.register_pipeline_processor_type()?;

    Ok(())
  }

  pub fn clone(&self) -> LuaBindings {
    Self::new().unwrap_or_else(|_| Self::new().unwrap())
  }
}

impl UserData for MediaProcessorBinding {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("width", |_lua, this, ()| Ok(this.processor.width()));
    methods.add_method("height", |_lua, this, ()| Ok(this.processor.height()));
    methods.add_method("media_type", |_lua, this, ()| {
      Ok(format!("{:?}", this.processor.media_type()))
    });
    methods.add_method("save", |_lua, this, path: String| {
      this
        .processor
        .save_to_file(&path)
        .map_err(|e| EllasticError::IOError(format!("Failed to save media: {}", e)))
    });
  }
}

impl UserData for EffectProcessorBinding {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method(
      "set_parameter",
      |_lua, this, name: String, value: String| this.processor.set_parameter(name, value),
    );
    methods.add_method("get_parameter", |_lua, this, name: String| {
      this.processor.get_parameter(&name)
    });
    methods.add_method("apply", |_lua, this, processor: MediaProcessorBinding| {
      this
        .processor
        .apply_effect(&processor.processor)
        .map_err(|e| EllasticError::ProcessingError(format!("Effect application failed: {}", e)))
    });
  }
}

impl UserData for PipelineProcessorBinding {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("execute", |_lua, this, input: MediaProcessorBinding| {
      this
        .processor
        .execute(input.processor)
        .map_err(|e| EllasticError::ProcessingError(format!("Pipeline execution failed: {}", e)))
    });
  }
}

impl TypeRegistry {
  pub fn new() -> Self {
    Self {
      types: HashMap::new(),
      type_mappings: HashMap::new(),
      inheritance_tree: HashMap::new(),
    }
  }

  pub fn register_type(&mut self, type_registration: TypeRegistration) -> Result<()> {
    if self.types.contains_key(&type_registration.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Type '{}' already registered",
        type_registration.name
      )));
    }

    self
      .types
      .insert(type_registration.name.clone(), type_registration);
    Ok(())
  }

  pub fn get_type(&self, name: &str) -> Option<&TypeRegistration> {
    self.types.get(name)
  }

  pub fn list_types(&self) -> Vec<&String> {
    self.types.keys().collect()
  }

  pub fn clone(&self) -> TypeRegistry {
    TypeRegistry {
      types: self.types.clone(),
      type_mappings: self.type_mappings.clone(),
      inheritance_tree: self.inheritance_tree.clone(),
    }
  }
}

fn parse_effect_type(effect_type_str: &str) -> Result<EffectType> {
  match effect_type_str {
    "brightness" => Ok(EffectType::Image {
      effect_type: "brightness".to_string(),
    }),
    "contrast" => Ok(EffectType::Image {
      effect_type: "contrast".to_string(),
    }),
    "saturation" => Ok(EffectType::Image {
      effect_type: "saturation".to_string(),
    }),
    "blur" => Ok(EffectType::Image {
      effect_type: "blur".to_string(),
    }),
    "sharpen" => Ok(EffectType::Image {
      effect_type: "sharpen".to_string(),
    }),
    "reverb" => Ok(EffectType::Audio {
      effect_type: "reverb".to_string(),
    }),
    "echo" => Ok(EffectType::Audio {
      effect_type: "echo".to_string(),
    }),
    "distortion" => Ok(EffectType::Audio {
      effect_type: "distortion".to_string(),
    }),
    "pixel_sort" => Ok(EffectType::Image {
      effect_type: "pixel_sort".to_string(),
    }),
    "data_mosh" => Ok(EffectType::Image {
      effect_type: "data_mosh".to_string(),
    }),
    _ => Err(EllasticError::InvalidParameter(format!(
      "Unknown effect type: {}",
      effect_type_str
    ))),
  }
}

fn lua_value_to_string(value: &Value) -> Result<String> {
  match value {
    Value::String(s) => Ok(s.clone()),
    Value::Number(n) => Ok(n.to_string()),
    Value::Boolean(b) => Ok(b.to_string()),
    Value::Table(_) => Err(EllasticError::InvalidParameter(
      "Table cannot be converted to string".to_string(),
    )),
    Value::Function(_) => Err(EllasticError::InvalidParameter(
      "Function cannot be converted to string".to_string(),
    )),
    Value::Error(_) => Err(EllasticError::ScriptError(
      "Lua error cannot be converted to string".to_string(),
    )),
    Value::UserData(_) => Err(EllasticError::InvalidParameter(
      "UserData cannot be converted to string".to_string(),
    )),
  }
}

fn lua_value_to_json(value: &Value) -> Result<JsonValue> {
  match value {
    Value::String(s) => Ok(JsonValue::String(s.clone())),
    Value::Number(n) => Ok(JsonValue::Number(n.to_string().parse().unwrap_or(0.0))),
    Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
    Value::Table(table) => {
      let mut json_object = serde_json::Map::new();

      for pair in table.pairs::<String, Value>() {
        let (key, val) = pair
          .map_err(|e| EllasticError::ScriptError(format!("Failed to iterate table: {}", e)))?;
        let json_val = lua_value_to_json(&val)?;
        json_object.insert(key, json_val);
      }

      Ok(JsonValue::Object(json_object))
    }
    Value::Function(_) => Err(EllasticError::InvalidParameter(
      "Function cannot be converted to JSON".to_string(),
    )),
    Value::Error(_) => Err(EllasticError::ScriptError(
      "Lua error cannot be converted to JSON".to_string(),
    )),
    Value::UserData(_) => Err(EllasticError::InvalidParameter(
      "UserData cannot be converted to JSON".to_string(),
    )),
  }
}

fn json_value_to_lua(lua: &Lua, json_value: JsonValue) -> Result<Value> {
  match json_value {
    JsonValue::String(s) => Ok(Value::String(s)),
    JsonValue::Number(n) => Ok(Value::Number(n.as_f64().unwrap_or(0.0))),
    JsonValue::Bool(b) => Ok(Value::Boolean(b)),
    JsonValue::Array(arr) => {
      let lua_table = lua.create_table()?;
      for (i, val) in arr.iter().enumerate() {
        let lua_val = json_value_to_lua(lua, val.clone())?;
        lua_table.set(i + 1, lua_val)?;
      }
      Ok(Value::Table(lua_table))
    }
    JsonValue::Object(obj) => {
      let lua_table = lua.create_table()?;
      for (key, val) in obj {
        let lua_val = json_value_to_lua(lua, val)?;
        lua_table.set(key, lua_val)?;
      }
      Ok(Value::Table(lua_table))
    }
    JsonValue::Null => Ok(Value::Nil),
  }
}

pub fn create_lua_bindings() -> Result<LuaBindings> {
  LuaBindings::new()
}

pub fn create_type_registration(name: String, constructor: Function) -> TypeRegistration {
  TypeRegistration {
    name,
    type_id: Uuid::new_v4(),
    constructor,
    methods: HashMap::new(),
    metamethods: HashMap::new(),
    fields: HashMap::new(),
  }
}

pub fn create_function_registration(
  name: String,
  function: Function,
  signature: FunctionSignature,
  documentation: String,
  category: FunctionCategory,
) -> FunctionRegistration {
  FunctionRegistration {
    name,
    function,
    signature,
    documentation,
    category,
  }
}

pub fn create_module_registration(
  name: String,
  module: Table,
  documentation: String,
) -> ModuleRegistration {
  ModuleRegistration {
    name,
    module,
    functions: HashMap::new(),
    types: HashMap::new(),
    documentation,
  }
}
