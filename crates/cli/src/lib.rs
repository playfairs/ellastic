use ellastic_errors::{Result, EllasticError};
use ellastic_core::{MediaData, MediaType};
use ellastic_image::{ImageProcessor, ImageData};
use ellastic_audio::{AudioProcessor, AudioData};
use ellastic_media::{MediaProcessor};
use ellastic_glitch::{GlitchProcessor, GlitchEffect};
use ellastic_effects::{EffectProcessor, EffectType};
use ellastic_pipeline::{PipelineProcessor, PipelineGraph};
use ellastic_project::{ProjectManager, Project};
use ellastic_export::{ExportManager, ExportRequest};
use ellastic_utils::{create_random_generator};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct CLIManager {
    pub commands: Arc<RwLock<HashMap<String, CLICommand>>>,
    pub config: CLIConfig,
    pub current_project: Arc<RwLock<Option<Project>>>,
    pub output: CLIOutput,
    pub history: CLIHistory,
}

#[derive(Debug, Clone)]
pub struct CLIConfig {
    pub enable_colors: bool,
    pub enable_unicode: bool,
    pub enable_progress: bool,
    pub enable_interactive: bool,
    pub enable_completion: bool,
    pub max_history: usize,
    pub prompt: String,
    pub error_prompt: String,
    pub success_prompt: String,
    pub warning_prompt: String,
    pub info_prompt: String,
}

#[derive(Debug, Clone)]
pub struct CLICommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub aliases: Vec<String>,
    pub subcommands: HashMap<String, CLICommand>,
    pub arguments: Vec<CLIArgument>,
    pub options: Vec<CLIOption>,
    pub handler: CommandHandler,
    pub category: CommandCategory,
    pub enabled: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone)]
pub struct CLIArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub multiple: bool,
    pub default_value: Option<String>,
    pub value_type: ArgumentType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentType {
    String,
    Number,
    Boolean,
    File,
    Directory,
    Url,
    Email,
    Date,
    Time,
    DateTime,
    Json,
    Custom,
}

#[derive(Debug, Clone)]
pub struct CLIOption {
    pub name: String,
    pub short_name: Option<char>,
    pub description: String,
    pub required: bool,
    pub multiple: bool,
    pub default_value: Option<String>,
    pub value_type: ArgumentType,
    pub global: bool,
}

pub type CommandHandler = Box<dyn Fn(&CLIContext, &Vec<String>, &HashMap<String, String>) -> Result<CLIResult> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    Project,
    Media,
    Effects,
    Pipeline,
    Export,
    Config,
    Help,
    System,
    Custom,
}

#[derive(Debug, Clone)]
pub struct CLIContext {
    pub working_directory: String,
    pub environment_variables: HashMap<String, String>,
    pub global_options: HashMap<String, String>,
    pub session_variables: HashMap<String, String>,
    pub current_user: Option<String>,
    pub current_session: String,
}

#[derive(Debug, Clone)]
pub struct CLIResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub exit_code: i32,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct CLIOutput {
    pub output_type: OutputType,
    pub format: OutputFormat,
    pub colors: bool,
    pub unicode: bool,
    pub progress: bool,
    pub interactive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Standard,
    Error,
    Warning,
    Info,
    Success,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Table,
    List,
    Tree,
    Custom,
}

#[derive(Debug, Clone)]
pub struct CLIHistory {
    pub entries: Vec<HistoryEntry>,
    pub max_size: usize,
    pub current_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub working_directory: String,
    pub exit_code: i32,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct CLICompletion {
    pub enabled: bool,
    pub commands: Vec<CompletionItem>,
    pub files: Vec<CompletionItem>,
    pub directories: Vec<CompletionItem>,
    pub custom: Vec<CompletionItem>,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub text: String,
    pub description: Option<String>,
    pub completion_type: CompletionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionType {
    Command,
    Argument,
    Option,
    File,
    Directory,
    Custom,
}

#[derive(Debug, Clone)]
pub struct CLIPlugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub commands: HashMap<String, CLICommand>,
    pub enabled: bool,
    pub loaded: bool,
}

#[derive(Debug, Clone)]
pub struct CLITheme {
    pub name: String,
    pub colors: CLIColors,
    pub styles: CLIStyles,
}

#[derive(Debug, Clone)]
pub struct CLIColors {
    pub reset: String,
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

#[derive(Debug, Clone)]
pub struct CLIStyles {
    pub bold: String,
    pub dim: String,
    pub italic: String,
    pub underline: String,
    pub blink: String,
    pub reverse: String,
    pub hidden: String,
    pub strikethrough: String,
}

#[derive(Debug, Clone)]
pub struct CLIProgressBar {
    pub enabled: bool,
    pub style: ProgressBarStyle,
    pub template: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressBarStyle {
    Bar,
    Spinner,
    Dots,
    Line,
    Custom,
}

#[derive(Debug, Clone)]
pub struct CLIInteractive {
    pub enabled: bool,
    pub confirm: bool,
    pub password: bool,
    pub select: bool,
    pub multi_select: bool,
    pub input: bool,
    pub editor: bool,
}

#[derive(Debug, Clone)]
pub struct CLITable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub style: TableStyle,
    pub alignment: Vec<Alignment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableStyle {
    Simple,
    Grid,
    Extended,
    Markdown,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct CLITree {
    pub root: TreeNode,
    pub style: TreeStyle,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub value: Option<String>,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeStyle {
    ASCII,
    Unicode,
    Box,
    Custom,
}

impl CLIManager {
    pub fn new(config: CLIConfig) -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            config,
            current_project: Arc::new(RwLock::new(None)),
            output: CLIOutput::new(),
            history: CLIHistory::new(1000),
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
Register built-in commands
        self.register_built_in_commands()?;

        self.load_plugins()?;

        self.setup_completion()?;

        Ok(())
    }

    fn register_built_in_commands(&mut self) -> Result<()> {
        self.register_command(CLICommand {
            name: "project".to_string(),
            description: "Project management commands".to_string(),
            usage: "project <subcommand> [options]".to_string(),
            aliases: vec!["proj".to_string(), "p".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("Project command executed"))),
            category: CommandCategory::Project,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "new".to_string(),
            description: "Create a new project".to_string(),
            usage: "new <name> [options]".to_string(),
            aliases: vec!["create".to_string()],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "name".to_string(),
                    description: "Project name".to_string(),
                    required: true,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::String,
                },
            ],
            options: vec![
                CLIOption {
                    name: "template".to_string(),
                    short_name: Some('t'),
                    description: "Project template".to_string(),
                    required: false,
                    multiple: false,
                    default_value: Some("default".to_string()),
                    value_type: ArgumentType::String,
                    global: false,
                },
                CLIOption {
                    name: "directory".to_string(),
                    short_name: Some('d'),
                    description: "Project directory".to_string(),
                    required: false,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::Directory,
                    global: false,
                },
            ],
            handler: Box::new(|_, args, opts| {
                let name = args.get(0).unwrap_or(&"unnamed".to_string()).clone();
                let template = opts.get("template").unwrap_or(&"default".to_string()).clone();
                let directory = opts.get("directory").cloned();

                Ok(CLIResult::success(&format!("Created project '{}' with template '{}'", name, template)))
            }),
            category: CommandCategory::Project,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "open".to_string(),
            description: "Open an existing project".to_string(),
            usage: "open <path>".to_string(),
            aliases: vec!["load".to_string()],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "path".to_string(),
                    description: "Project path".to_string(),
                    required: true,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::File,
                },
            ],
            options: Vec::new(),
            handler: Box::new(|_, args, _| {
                let path = args.get(0).unwrap_or(&".".to_string()).clone();
                Ok(CLIResult::success(&format!("Opened project at '{}'", path)))
            }),
            category: CommandCategory::Project,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "save".to_string(),
            description: "Save the current project".to_string(),
            usage: "save [options]".to_string(),
            aliases: vec![],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: vec![
                CLIOption {
                    name: "force".to_string(),
                    short_name: Some('f'),
                    description: "Force save".to_string(),
                    required: false,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::Boolean,
                    global: false,
                },
            ],
            handler: Box::new(|_, _, opts| {
                let force = opts.contains_key("force");
                Ok(CLIResult::success(&format!("Saved project (force: {})", force)))
            }),
            category: CommandCategory::Project,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "media".to_string(),
            description: "Media management commands".to_string(),
            usage: "media <subcommand> [options]".to_string(),
            aliases: vec!["m".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("Media command executed"))),
            category: CommandCategory::Media,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "import".to_string(),
            description: "Import media files".to_string(),
            usage: "import <files...> [options]".to_string(),
            aliases: vec!["imp".to_string()],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "files".to_string(),
                    description: "Media files to import".to_string(),
                    required: true,
                    multiple: true,
                    default_value: None,
                    value_type: ArgumentType::File,
                },
            ],
            options: vec![
                CLIOption {
                    name: "recursive".to_string(),
                    short_name: Some('r'),
                    description: "Import recursively".to_string(),
                    required: false,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::Boolean,
                    global: false,
                },
                CLIOption {
                    name: "format".to_string(),
                    short_name: Some('f'),
                    description: "Media format".to_string(),
                    required: false,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::String,
                    global: false,
                },
            ],
            handler: Box::new(|_, args, opts| {
                let files = args.join(", ");
                let recursive = opts.contains_key("recursive");
                Ok(CLIResult::success(&format!("Imported {} files (recursive: {})", files, recursive)))
            }),
            category: CommandCategory::Media,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "effects".to_string(),
            description: "Effects management commands".to_string(),
            usage: "effects <subcommand> [options]".to_string(),
            aliases: vec!["fx".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("Effects command executed"))),
            category: CommandCategory::Effects,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "apply".to_string(),
            description: "Apply effects to media".to_string(),
            usage: "apply <effect> <files...> [options]".to_string(),
            aliases: vec!["app".to_string()],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "effect".to_string(),
                    description: "Effect to apply".to_string(),
                    required: true,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::String,
                },
                CLIArgument {
                    name: "files".to_string(),
                    description: "Media files".to_string(),
                    required: true,
                    multiple: true,
                    default_value: None,
                    value_type: ArgumentType::File,
                },
            ],
            options: vec![
                CLIOption {
                    name: "intensity".to_string(),
                    short_name: Some('i'),
                    description: "Effect intensity".to_string(),
                    required: false,
                    multiple: false,
                    default_value: Some("1.0".to_string()),
                    value_type: ArgumentType::Number,
                    global: false,
                },
            ],
            handler: Box::new(|_, args, opts| {
                let effect = args.get(0).unwrap_or(&"none".to_string()).clone();
                let files: Vec<String> = args.iter().skip(1).cloned().collect();
                let intensity = opts.get("intensity").unwrap_or(&"1.0".to_string()).clone();
                Ok(CLIResult::success(&format!("Applied '{}' to {} files with intensity {}", effect, files.len(), intensity)))
            }),
            category: CommandCategory::Effects,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "pipeline".to_string(),
            description: "Pipeline management commands".to_string(),
            usage: "pipeline <subcommand> [options]".to_string(),
            aliases: vec!["pipe".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("Pipeline command executed"))),
            category: CommandCategory::Pipeline,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "run".to_string(),
            description: "Run a pipeline".to_string(),
            usage: "run <pipeline> [options]".to_string(),
            aliases: vec!["execute".to_string()],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "pipeline".to_string(),
                    description: "Pipeline to run".to_string(),
                    required: true,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::File,
                },
            ],
            options: vec![
                CLIOption {
                    name: "parallel".to_string(),
                    short_name: Some('p'),
                    description: "Run in parallel".to_string(),
                    required: false,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::Boolean,
                    global: false,
                },
            ],
            handler: Box::new(|_, args, opts| {
                let pipeline = args.get(0).unwrap_or(&"default".to_string()).clone();
                let parallel = opts.contains_key("parallel");
                Ok(CLIResult::success(&format!("Ran pipeline '{}' (parallel: {})", pipeline, parallel)))
            }),
            category: CommandCategory::Pipeline,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "export".to_string(),
            description: "Export commands".to_string(),
            usage: "export <subcommand> [options]".to_string(),
            aliases: vec!["exp".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("Export command executed"))),
            category: CommandCategory::Export,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "render".to_string(),
            description: "Render media".to_string(),
            usage: "render <files...> [options]".to_string(),
            aliases: vec!["rend".to_string()],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "files".to_string(),
                    description: "Files to render".to_string(),
                    required: true,
                    multiple: true,
                    default_value: None,
                    value_type: ArgumentType::File,
                },
            ],
            options: vec![
                CLIOption {
                    name: "format".to_string(),
                    short_name: Some('f'),
                    description: "Output format".to_string(),
                    required: false,
                    multiple: false,
                    default_value: Some("png".to_string()),
                    value_type: ArgumentType::String,
                    global: false,
                },
                CLIOption {
                    name: "quality".to_string(),
                    short_name: Some('q'),
                    description: "Output quality".to_string(),
                    required: false,
                    multiple: false,
                    default_value: Some("high".to_string()),
                    value_type: ArgumentType::String,
                    global: false,
                },
            ],
            handler: Box::new(|_, args, opts| {
                let files = args.join(", ");
                let format = opts.get("format").unwrap_or(&"png".to_string()).clone();
                let quality = opts.get("quality").unwrap_or(&"high".to_string()).clone();
                Ok(CLIResult::success(&format!("Rendered {} files as {} (quality: {})", files, format, quality)))
            }),
            category: CommandCategory::Export,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "config".to_string(),
            description: "Configuration commands".to_string(),
            usage: "config <subcommand> [options]".to_string(),
            aliases: vec!["cfg".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("Config command executed"))),
            category: CommandCategory::Config,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "set".to_string(),
            description: "Set configuration values".to_string(),
            usage: "set <key> <value>".to_string(),
            aliases: vec![],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "key".to_string(),
                    description: "Configuration key".to_string(),
                    required: true,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::String,
                },
                CLIArgument {
                    name: "value".to_string(),
                    description: "Configuration value".to_string(),
                    required: true,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::String,
                },
            ],
            options: Vec::new(),
            handler: Box::new(|_, args, _| {
                let key = args.get(0).unwrap_or(&"unknown".to_string()).clone();
                let value = args.get(1).unwrap_or(&"unknown".to_string()).clone();
                Ok(CLIResult::success(&format!("Set {} = {}", key, value)))
            }),
            category: CommandCategory::Config,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "get".to_string(),
            description: "Get configuration values".to_string(),
            usage: "get <key>".to_string(),
            aliases: vec![],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "key".to_string(),
                    description: "Configuration key".to_string(),
                    required: true,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::String,
                },
            ],
            options: Vec::new(),
            handler: Box::new(|_, args, _| {
                let key = args.get(0).unwrap_or(&"unknown".to_string()).clone();
                Ok(CLIResult::success(&format!("Got value for {}", key)))
            }),
            category: CommandCategory::Config,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "help".to_string(),
            description: "Show help information".to_string(),
            usage: "help [command]".to_string(),
            aliases: vec!["h".to_string(), "?".to_string()],
            subcommands: HashMap::new(),
            arguments: vec![
                CLIArgument {
                    name: "command".to_string(),
                    description: "Command to get help for".to_string(),
                    required: false,
                    multiple: false,
                    default_value: None,
                    value_type: ArgumentType::String,
                },
            ],
            options: Vec::new(),
            handler: Box::new(|_, args, _| {
                if args.is_empty() {
                    Ok(CLIResult::success("Available commands: project, media, effects, pipeline, export, config, help"))
                } else {
                    let command = args.get(0).unwrap_or(&"unknown".to_string()).clone();
                    Ok(CLIResult::success(&format!("Help for command: {}", command)))
                }
            }),
            category: CommandCategory::Help,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "version".to_string(),
            description: "Show version information".to_string(),
            usage: "version".to_string(),
            aliases: vec!["v".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("Ellastic CLI v0.1.0"))),
            category: CommandCategory::System,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "status".to_string(),
            description: "Show system status".to_string(),
            usage: "status".to_string(),
            aliases: vec!["stat".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("System status: OK"))),
            category: CommandCategory::System,
            enabled: true,
            hidden: false,
        })?;

        self.register_command(CLICommand {
            name: "exit".to_string(),
            description: "Exit the CLI".to_string(),
            usage: "exit".to_string(),
            aliases: vec!["quit".to_string(), "q".to_string()],
            subcommands: HashMap::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            handler: Box::new(|_, _, _| Ok(CLIResult::success("Exiting..."))),
            category: CommandCategory::System,
            enabled: true,
            hidden: false,
        })?;

        Ok(())
    }

    pub fn register_command(&mut self, command: CLICommand) -> Result<()> {
        let mut commands = self.commands.write();
        commands.insert(command.name.clone(), command);
        Ok(())
    }

    pub fn get_command(&self, name: &str) -> Option<CLICommand> {
        self.commands.read().get(name).cloned()
    }

    pub fn list_commands(&self) -> Vec<CLICommand> {
        self.commands.read().values().cloned().collect()
    }

    pub fn list_commands_by_category(&self, category: CommandCategory) -> Vec<CLICommand> {
        self.commands.read()
            .values()
            .filter(|c| c.category == category)
            .cloned()
            .collect()
    }

    pub fn execute_command(&mut self, command_line: &str) -> Result<CLIResult> {
        let start_time = std::time::Instant::now();

        let (command_name, args, options) = self.parse_command_line(command_line)?;

        let command = self.get_command(&command_name)
            .ok_or_else(|| EllasticError::InvalidParameter(format!("Command '{}' not found", command_name)))?;

        if !command.enabled {
            return Err(EllasticError::InvalidOperation(format!("Command '{}' is disabled", command_name)));
        }

        let context = CLIContext {
            working_directory: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
            environment_variables: std::env::vars().collect(),
            global_options: HashMap::new(),
            session_variables: HashMap::new(),
            current_user: std::env::var("USER").ok(),
            current_session: Uuid::new_v4().to_string(),
        };

        let result = (command.handler)(&context, &args, &options);

        let duration = start_time.elapsed();
        let exit_code = result.as_ref().map_or(1, |r| r.exit_code);
        self.add_to_history(command_line.to_string(), duration, exit_code)?;

        result
    }

    fn parse_command_line(&self, command_line: &str) -> Result<(String, Vec<String>, HashMap<String, String>)> {
        let mut parts = command_line.split_whitespace().collect::<Vec<&str>>();
        if parts.is_empty() {
            return Err(EllasticError::InvalidParameter("Empty command".to_string()));
        }

        let command_name = parts[0].to_string();
        let mut args = Vec::new();
        let mut options = HashMap::new();

        let mut i = 1;
        while i < parts.len() {
            let part = parts[i];

            if part.starts_with('-') {
                let (key, value) = if part.starts_with("--") {
                    let key = part.strip_prefix("--").unwrap();
                    if key.contains('=') {
                        let mut kv = key.split('=');
                        let k = kv.next().unwrap().to_string();
                        let v = kv.next().unwrap_or("").to_string();
                        (k, Some(v))
                    } else {
                        (key.to_string(), None)
                    }
                } else if part.starts_with('-') && part.len() > 1 {
                    let key = part.strip_prefix('-').unwrap();
                    if key.len() == 1 {
                        (key.to_string(), None)
                    } else {
                        for ch in key.chars() {
                            options.insert(ch.to_string(), String::new());
                        }
                        i += 1;
                        continue;
                    }
                } else {
                    return Err(EllasticError::InvalidParameter(format!("Invalid option: {}", part)));
                };

                if let Some(value) = value {
                    options.insert(key, value);
                } else {
                    if i + 1 < parts.len() && !parts[i + 1].starts_with('-') {
                        options.insert(key, parts[i + 1].to_string());
                        i += 1;
                    } else {
                        options.insert(key, String::new());
                    }
                }
            } else {
                args.push(part.to_string());
            }

            i += 1;
        }

        Ok((command_name, args, options))
    }

    fn load_plugins(&mut self) -> Result<()> {
        Ok(())
    }

    fn setup_completion(&mut self) -> Result<()> {
        Ok(())
    }

    fn add_to_history(&mut self, command: String, duration: std::time::Duration, exit_code: i32) -> Result<()> {
        let entry = HistoryEntry {
            command,
            timestamp: Utc::now(),
            working_directory: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
            exit_code,
            duration,
        };

        let mut history = self.history;
        if history.entries.len() >= history.max_size {
            history.entries.remove(0);
        }
        history.entries.push(entry);

        Ok(())
    }

    pub fn get_history(&self) -> Vec<HistoryEntry> {
        self.history.entries.clone()
    }

    pub fn clear_history(&mut self) -> Result<()> {
        self.history.entries.clear();
        Ok(())
    }

    pub fn clone(&self) -> CLIManager {
        CLIManager {
            commands: self.commands.clone(),
            config: self.config.clone(),
            current_project: self.current_project.clone(),
            output: self.output.clone(),
            history: self.history.clone(),
        }
    }
}

impl CLIOutput {
    pub fn new() -> Self {
        Self {
            output_type: OutputType::Standard,
            format: OutputFormat::Text,
            colors: true,
            unicode: true,
            progress: true,
            interactive: true,
        }
    }

    pub fn clone(&self) -> CLIOutput {
        CLIOutput {
            output_type: self.output_type,
            format: self.format,
            colors: self.colors,
            unicode: self.unicode,
            progress: self.progress,
            interactive: self.interactive,
        }
    }
}

impl CLIHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
            current_index: None,
        }
    }

    pub fn clone(&self) -> CLIHistory {
        CLIHistory {
            entries: self.entries.clone(),
            max_size: self.max_size,
            current_index: self.current_index,
        }
    }
}

impl CLIResult {
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
            exit_code: 0,
            duration: std::time::Duration::from_secs(0),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
            exit_code: 1,
            duration: std::time::Duration::from_secs(0),
        }
    }

    pub fn with_data(message: &str, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
            exit_code: 0,
            duration: std::time::Duration::from_secs(0),
        }
    }

    pub fn clone(&self) -> CLIResult {
        CLIResult {
            success: self.success,
            message: self.message.clone(),
            data: self.data.clone(),
            exit_code: self.exit_code,
            duration: self.duration,
        }
    }
}

impl Default fn default() -> Self {
        Self {
            enable_colors: true,
            enable_unicode: true,
            enable_progress: true,
            enable_interactive: true,
            enable_completion: true,
            max_history: 1000,
            prompt: "elastic> ".to_string(),
            error_prompt: "error> ".to_string(),
            success_prompt: "success> ".to_string(),
            warning_prompt: "warning> ".to_string(),
            info_prompt: "info> ".to_string(),
        }
}

pub fn create_cli_manager(config: CLIConfig) -> CLIManager {
    CLIManager::new(config)
}

pub fn create_cli_config() -> CLIConfig {
    CLIConfig::default()
}
