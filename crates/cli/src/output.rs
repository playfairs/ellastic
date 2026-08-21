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
pub struct OutputManager {
    pub config: OutputConfig,
    pub formatters: Arc<RwLock<HashMap<OutputFormat, OutputFormatter>>>,
    pub themes: Arc<RwLock<HashMap<String, OutputTheme>>>,
    pub renderers: Arc<RwLock<HashMap<OutputType, OutputRenderer>>>,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub default_format: OutputFormat,
    pub default_theme: String,
    pub enable_colors: bool,
    pub enable_unicode: bool,
    pub enable_progress: bool,
    pub enable_interactive: bool,
    pub enable_pager: bool,
    pub pager_command: Option<String>,
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,
    tab_size: usize,
    line_wrap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Toml,
    Xml,
    Csv,
    Table,
    List,
    Tree,
    Markdown,
    Html,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Standard,
    Error,
    Warning,
    Info,
    Success,
    Debug,
    Trace,
    Custom,
}

#[derive(Debug, Clone)]
pub struct OutputFormatter {
    pub format: OutputFormat,
    pub name: String,
    pub description: String,
    pub mime_type: String,
    pub file_extension: String,
    pub supports_colors: bool,
    pub supports_unicode: bool,
    pub supports_interactive: bool,
    pub formatter_fn: FormatterFn,
}

pub type FormatterFn = Box<dyn Fn(&OutputData, &OutputTheme) -> Result<String> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct OutputData {
    pub content: OutputContent,
    pub metadata: OutputMetadata,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub enum OutputContent {
    Text(String),
    Structured(serde_json::Value),
    Table(TableData),
    List(ListData),
    Tree(TreeData),
    Binary(Vec<u8>),
    Mixed(Vec<OutputContent>),
}

#[derive(Debug, Clone)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub style: TableStyle,
    pub alignment: Vec<Alignment>,
    pub borders: bool,
    pub padding: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableStyle {
    Simple,
    Grid,
    Extended,
    Markdown,
    Ascii,
    Unicode,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct ListData {
    pub items: Vec<ListItem>,
    pub style: ListStyle,
    pub numbering: bool,
    pub indentation: usize,
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: String,
    pub level: usize,
    pub children: Vec<ListItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStyle {
    Bullets,
    Dashes,
    Asterisks,
    Numbers,
    Letters,
    Roman,
    Custom,
}

#[derive(Debug, Clone)]
pub struct TreeData {
    pub root: TreeNode,
    pub style: TreeStyle,
    pub show_lines: bool,
    pub indentation: usize,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub value: Option<String>,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeStyle {
    Ascii,
    Unicode,
    Box,
    Double,
    Round,
    Custom,
}

#[derive(Debug, Clone)]
pub struct OutputMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct OutputTheme {
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
    pub styles: ThemeStyles,
    pub icons: ThemeIcons,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
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
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
}

#[derive(Debug, Clone)]
pub struct ThemeStyles {
    pub bold: String,
    pub dim: String,
    pub italic: String,
    pub underline: String,
    pub blink: String,
    pub reverse: String,
    pub hidden: String,
    pub strikethrough: String,
    pub reset_all: String,
}

#[derive(Debug, Clone)]
pub struct ThemeIcons {
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub bullet: String,
    pub arrow_right: String,
    pub arrow_down: String,
    pub folder: String,
    pub file: String,
    pub loading: String,
    pub check: String,
    pub cross: String,
}

#[derive(Debug, Clone)]
pub struct OutputRenderer {
    pub output_type: OutputType,
    pub name: String,
    pub description: String,
    pub stream: OutputStream,
    pub renderer_fn: RendererFn,
}

pub type RendererFn = Box<dyn Fn(&String, &OutputConfig) -> Result<()> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
    File,
    Pipe,
    Socket,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub enabled: bool,
    pub style: ProgressBarStyle,
    pub template: String,
    pub width: Option<usize>,
    pub show_percentage: bool,
    pub show_elapsed: bool,
    pub show_eta: bool,
    pub show_rate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressBarStyle {
    Bar,
    Spinner,
    Dots,
    Line,
    Blocks,
    Custom,
}

#[derive(Debug, Clone)]
pub struct InteractivePrompt {
    pub enabled: bool,
    pub style: PromptStyle,
    pub default_input: Option<String>,
    pub placeholder: Option<String>,
    pub validator: Option<PromptValidator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptStyle {
    Line,
    Password,
    Select,
    MultiSelect,
    Confirm,
    Editor,
    Custom,
}

#[derive(Debug, Clone)]
pub struct PromptValidator {
    pub validator_type: ValidatorType,
    pub pattern: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub custom_validator: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorType {
    Required,
    Pattern,
    Length,
    Range,
    Email,
    Url,
    Custom,
}

#[derive(Debug, Clone)]
pub struct OutputBuffer {
    pub content: Vec<String>,
    pub max_size: usize,
    pub auto_flush: bool,
}

#[derive(Debug, Clone)]
pub struct OutputPager {
    pub enabled: bool,
    pub command: String,
    pub args: Vec<String>,
    pub auto_pager: bool,
    pub min_lines: usize,
}

impl OutputManager {
    pub fn new(config: OutputConfig) -> Self {
        let mut manager = Self {
            config,
            formatters: Arc::new(RwLock::new(HashMap::new())),
            themes: Arc::new(RwLock::new(HashMap::new())),
            renderers: Arc::new(RwLock::new(HashMap::new())),
        };

        manager.register_default_formatters().unwrap();

        manager.register_default_themes().unwrap();

        manager.register_default_renderers().unwrap();

        manager
    }

    fn register_default_formatters(&mut self) -> Result<()> {
        let mut formatters = self.formatters.write();

        formatters.insert(OutputFormat::Text, OutputFormatter {
            format: OutputFormat::Text,
            name: "text".to_string(),
            description: "Plain text formatter".to_string(),
            mime_type: "text/plain".to_string(),
            file_extension: "txt".to_string(),
            supports_colors: true,
            supports_unicode: true,
            supports_interactive: true,
            formatter_fn: Box::new(|data, theme| {
                match &data.content {
                    OutputContent::Text(text) => Ok(text.clone()),
                    OutputContent::Structured(value) => Ok(serde_json::to_string_pretty(value)?),
                    OutputContent::Table(table) => Self::format_table(table, theme),
                    OutputContent::List(list) => Self::format_list(list, theme),
                    OutputContent::Tree(tree) => Self::format_tree(tree, theme),
                    OutputContent::Binary(_) => Ok("[Binary data]".to_string()),
                    OutputContent::Mixed(contents) => {
                        let mut result = String::new();
                        for content in contents {
                            result.push_str(&Self::format_content(content, theme)?);
                        }
                        Ok(result)
                    }
                }
            }),
        });

        formatters.insert(OutputFormat::Json, OutputFormatter {
            format: OutputFormat::Json,
            name: "json".to_string(),
            description: "JSON formatter".to_string(),
            mime_type: "application/json".to_string(),
            file_extension: "json".to_string(),
            supports_colors: false,
            supports_unicode: false,
            supports_interactive: false,
            formatter_fn: Box::new(|data, _| {
                match &data.content {
                    OutputContent::Structured(value) => Ok(serde_json::to_string_pretty(value)?),
                    _ => Err(EllasticError::InvalidOperation("JSON formatter requires structured data".to_string())),
                }
            }),
        });

        formatters.insert(OutputFormat::Yaml, OutputFormatter {
            format: OutputFormat::Yaml,
            name: "yaml".to_string(),
            description: "YAML formatter".to_string(),
            mime_type: "application/x-yaml".to_string(),
            file_extension: "yaml".to_string(),
            supports_colors: false,
            supports_unicode: false,
            supports_interactive: false,
            formatter_fn: Box::new(|data, _| {
                match &data.content {
                    OutputContent::Structured(value) => {
                        Ok(serde_json::to_string(value)?)
                    }
                    _ => Err(EllasticError::InvalidOperation("YAML formatter requires structured data".to_string())),
                }
            }),
        });

        formatters.insert(OutputFormat::Table, OutputFormatter {
            format: OutputFormat::Table,
            name: "table".to_string(),
            description: "Table formatter".to_string(),
            mime_type: "text/plain".to_string(),
            file_extension: "txt".to_string(),
            supports_colors: true,
            supports_unicode: true,
            supports_interactive: false,
            formatter_fn: Box::new(|data, theme| {
                match &data.content {
                    OutputContent::Table(table) => Self::format_table(table, theme),
                    _ => Err(EllasticError::InvalidOperation("Table formatter requires table data".to_string())),
                }
            }),
        });

        formatters.insert(OutputFormat::Markdown, OutputFormatter {
            format: OutputFormat::Markdown,
            name: "markdown".to_string(),
            description: "Markdown formatter".to_string(),
            mime_type: "text/markdown".to_string(),
            file_extension: "md".to_string(),
            supports_colors: false,
            supports_unicode: true,
            supports_interactive: false,
            formatter_fn: Box::new(|data, theme| {
                match &data.content {
                    OutputContent::Text(text) => Ok(text.clone()),
                    OutputContent::Table(table) => Self::format_markdown_table(table),
                    OutputContent::List(list) => Self::format_markdown_list(list),
                    _ => Err(EllasticError::InvalidOperation("Markdown formatter not supported for this content type".to_string())),
                }
            }),
        });

        Ok(())
    }

    fn register_default_themes(&mut self) -> Result<()> {
        let mut themes = self.themes.write();

        themes.insert("default".to_string(), OutputTheme {
            name: "default".to_string(),
            description: "Default output theme".to_string(),
            colors: ThemeColors::default(),
            styles: ThemeStyles::default(),
            icons: ThemeIcons::default(),
        });

        themes.insert("dark".to_string(), OutputTheme {
            name: "dark".to_string(),
            description: "Dark output theme".to_string(),
            colors: ThemeColors::dark(),
            styles: ThemeStyles::default(),
            icons: ThemeIcons::default(),
        });

        themes.insert("light".to_string(), OutputTheme {
            name: "light".to_string(),
            description: "Light output theme".to_string(),
            colors: ThemeColors::light(),
            styles: ThemeStyles::default(),
            icons: ThemeIcons::default(),
        });

        themes.insert("nocolor".to_string(), OutputTheme {
            name: "nocolor".to_string(),
            description: "No color theme".to_string(),
            colors: ThemeColors::nocolor(),
            styles: ThemeStyles::nocolor(),
            icons: ThemeIcons::unicode(),
        });

        Ok(())
    }

    fn register_default_renderers(&mut self) -> Result<()> {
        let mut renderers = self.renderers.write();

        renderers.insert(OutputType::Standard, OutputRenderer {
            output_type: OutputType::Standard,
            name: "stdout".to_string(),
            description: "Standard output renderer".to_string(),
            stream: OutputStream::Stdout,
            renderer_fn: Box::new(|content, _| {
                println!("{}", content);
                Ok(())
            }),
        });

        renderers.insert(OutputType::Error, OutputRenderer {
            output_type: OutputType::Error,
            name: "stderr".to_string(),
            description: "Error output renderer".to_string(),
            stream: OutputStream::Stderr,
            renderer_fn: Box::new(|content, _| {
                eprintln!("{}", content);
                Ok(())
            }),
        });

        Ok(())
    }

    pub fn output(&mut self, data: OutputData, output_type: OutputType) -> Result<()> {
        let formatter = {
            let formatters = self.formatters.read();
            formatters.get(&self.config.default_format).cloned()
                .ok_or_else(|| EllasticError::InvalidParameter(format!("Formatter not found for format: {:?}", self.config.default_format)))?
        };

        let theme = {
            let themes = self.themes.read();
            themes.get(&self.config.default_theme).cloned()
                .unwrap_or_else(|| OutputTheme::default())
        };

        let formatted = (formatter.formatter_fn)(&data, &theme)?;

        let renderer = {
            let renderers = self.renderers.read();
            renderers.get(&output_type).cloned()
                .ok_or_else(|| EllasticError::InvalidParameter(format!("Renderer not found for output type: {:?}", output_type)))?
        };

        (renderer.renderer_fn)(&formatted, &self.config)?;

        Ok(())
    }

    pub fn output_text(&mut self, text: String, output_type: OutputType) -> Result<()> {
        let data = OutputData {
            content: OutputContent::Text(text),
            metadata: OutputMetadata::new(),
            timestamp: Utc::now(),
            source: "cli".to_string(),
        };

        self.output(data, output_type)
    }

    pub fn output_structured(&mut self, value: serde_json::Value, output_type: OutputType) -> Result<()> {
        let data = OutputData {
            content: OutputContent::Structured(value),
            metadata: OutputMetadata::new(),
            timestamp: Utc::now(),
            source: "cli".to_string(),
        };

        self.output(data, output_type)
    }

    pub fn output_table(&mut self, table: TableData, output_type: OutputType) -> Result<()> {
        let data = OutputData {
            content: OutputContent::Table(table),
            metadata: OutputMetadata::new(),
            timestamp: Utc::now(),
            source: "cli".to_string(),
        };

        self.output(data, output_type)
    }

    pub fn output_list(&mut self, list: ListData, output_type: OutputType) -> Result<()> {
        let data = OutputData {
            content: OutputContent::List(list),
            metadata: OutputMetadata::new(),
            timestamp: Utc::now(),
            source: "cli".to_string(),
        };

        self.output(data, output_type)
    }

    pub fn output_tree(&mut self, tree: TreeData, output_type: OutputType) -> Result<()> {
        let data = OutputData {
            content: OutputContent::Tree(tree),
            metadata: OutputMetadata::new(),
            timestamp: Utc::now(),
            source: "cli".to_string(),
        };

        self.output(data, output_type)
    }

    fn format_table(table: &TableData, theme: &OutputTheme) -> Result<String> {
        let mut result = String::new();

        if table.headers.is_empty() {
            return Ok(result);
        }

        let mut widths = vec![0; table.headers.len()];

        for (i, header) in table.headers.iter().enumerate() {
            widths[i] = widths[i].max(header.len());
        }

        for row in &table.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        match table.style {
            TableStyle::Simple => Self::format_simple_table(table, &widths, theme),
            TableStyle::Grid => Self::format_grid_table(table, &widths, theme),
            TableStyle::Extended => Self::format_extended_table(table, &widths, theme),
            TableStyle::Markdown => Self::format_markdown_table(table),
            TableStyle::Ascii => Self::format_ascii_table(table, &widths, theme),
            TableStyle::Unicode => Self::format_unicode_table(table, &widths, theme),
            TableStyle::Custom => Self::format_simple_table(table, &widths, theme),
        }
    }

    fn format_simple_table(table: &TableData, widths: &[usize], theme: &OutputTheme) -> Result<String> {
        let mut result = String::new();

        for (i, header) in table.headers.iter().enumerate() {
            if i > 0 {
                result.push_str("  ");
            }
            result.push_str(&format!("{:<width$}", header, width = widths[i]));
        }
        result.push('\n');

        for row in &table.rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    result.push_str("  ");
                }
                result.push_str(&format!("{:<width$}", cell, width = widths[i]));
            }
            result.push('\n');
        }

        Ok(result)
    }

    fn format_grid_table(table: &TableData, widths: &[usize], theme: &OutputTheme) -> Result<String> {
        let mut result = String::new();

        result.push('+');
        for &width in widths {
            result.push_str(&format!("-{}-", "-".repeat(width)));
            result.push('+');
        }
        result.push('\n');

        result.push('|');
        for (i, header) in table.headers.iter().enumerate() {
            result.push_str(&format!(" {:<width$} ", header, width = widths[i]));
            result.push('|');
        }
        result.push('\n');

        result.push('+');
        for &width in widths {
            result.push_str(&format!("-{}-", "-".repeat(width)));
            result.push('+');
        }
        result.push('\n');

        for row in &table.rows {
            result.push('|');
            for (i, cell) in row.iter().enumerate() {
                result.push_str(&format!(" {:<width$} ", cell, width = widths[i]));
                result.push('|');
            }
            result.push('\n');
        }

        result.push('+');
        for &width in widths {
            result.push_str(&format!("-{}-", "-".repeat(width)));
            result.push('+');
        }
        result.push('\n');

        Ok(result)
    }

    fn format_extended_table(table: &TableData, widths: &[usize], theme: &OutputTheme) -> Result<String> {
        Self::format_grid_table(table, widths, theme)
    }

    fn format_markdown_table(table: &TableData) -> Result<String> {
        let mut result = String::new();

        result.push('|');
        for header in &table.headers {
            result.push_str(&format!(" {} |", header));
        }
        result.push('\n');

        result.push('|');
        for _ in &table.headers {
            result.push_str(" --- |");
        }
        result.push('\n');

        for row in &table.rows {
            result.push('|');
            for cell in row {
                result.push_str(&format!(" {} |", cell));
            }
            result.push('\n');
        }

        Ok(result)
    }

    fn format_ascii_table(table: &TableData, widths: &[usize], theme: &OutputTheme) -> Result<String> {
        Self::format_simple_table(table, widths, theme)
    }

    fn format_unicode_table(table: &TableData, widths: &[usize], theme: &OutputTheme) -> Result<String> {
        let mut result = String::new();

        result.push('┌');
        for (i, &width) in widths.iter().enumerate() {
            result.push_str(&"─".repeat(width + 2));
            if i < widths.len() - 1 {
                result.push('┬');
            }
        }
        result.push('┐');
        result.push('\n');

        result.push('│');
        for (i, header) in table.headers.iter().enumerate() {
            result.push_str(&format!(" {:<width$} ", header, width = widths[i]));
            result.push('│');
        }
        result.push('\n');

        result.push('├');
        for (i, &width) in widths.iter().enumerate() {
            result.push_str(&"─".repeat(width + 2));
            if i < widths.len() - 1 {
                result.push('┼');
            }
        }
        result.push('┤');
        result.push('\n');

        for row in &table.rows {
            result.push('│');
            for (i, cell) in row.iter().enumerate() {
                result.push_str(&format!(" {:<width$} ", cell, width = widths[i]));
                result.push('│');
            }
            result.push('\n');
        }

        result.push('└');
        for (i, &width) in widths.iter().enumerate() {
            result.push_str(&"─".repeat(width + 2));
            if i < widths.len() - 1 {
                result.push('┴');
            }
        }
        result.push('┘');
        result.push('\n');

        Ok(result)
    }

    fn format_list(list: &ListData, theme: &OutputTheme) -> Result<String> {
        let mut result = String::new();
        Self::format_list_items(&list.items, &mut result, list.style, list.numbering, list.indentation, 0);
        Ok(result)
    }

    fn format_list_items(items: &[ListItem], result: &mut String, style: ListStyle, numbering: bool, indentation: usize, level: usize) {
        for (i, item) in items.iter().enumerate() {
            for _ in 0..level * indentation {
                result.push(' ');
            }

            if numbering {
                let marker = match style {
                    ListStyle::Numbers => format!("{}.", i + 1),
                    ListStyle::Letters => Self::number_to_letter(i),
                    ListStyle::Roman => Self::number_to_roman(i + 1),
                    _ => format!("{}.", i + 1),
                };
                result.push_str(&format!("{} ", marker));
            } else {
                let bullet = match style {
                    ListStyle::Bullets => "•",
                    ListStyle::Dashes => "–",
                    ListStyle::Asterisks => "*",
                    _ => "•",
                };
                result.push_str(&format!("{} ", bullet));
            }

            result.push_str(&item.content);
            result.push('\n');

            if !item.children.is_empty() {
                Self::format_list_items(&item.children, result, style, numbering, indentation, level + 1);
            }
        }
    }

    fn number_to_letter(n: usize) -> String {
        if n < 26 {
            format!("{}", char::from(b'a' + n as u8))
        } else {
            format!("{}.{}", n / 26 + 1, char::from(b'a' + (n % 26) as u8))
        }
    }

    fn number_to_roman(n: usize) -> String {
        let roman_numerals = vec![
            ("M", 1000), ("CM", 900), ("D", 500), ("CD", 400),
            ("C", 100), ("XC", 90), ("L", 50), ("XL", 40),
            ("X", 10), ("IX", 9), ("V", 5), ("IV", 4), ("I", 1),
        ];

        let mut result = String::new();
        let mut num = n;

        for (symbol, value) in roman_numerals {
            while num >= value {
                result.push_str(symbol);
                num -= value;
            }
        }

        result
    }

    fn format_tree(tree: &TreeData, theme: &OutputTheme) -> Result<String> {
        let mut result = String::new();
        Self::format_tree_node(&tree.root, &mut result, tree.style, tree.show_lines, tree.indentation, 0, true);
        Ok(result)
    }

    fn format_tree_node(node: &TreeNode, result: &mut String, style: TreeStyle, show_lines: bool, indentation: usize, level: usize, is_last: bool) {
        for _ in 0..level {
            result.push_str("  ");
        }

        if level > 0 {
            let connector = match style {
                TreeStyle::Ascii => if is_last { "└─" } else { "├─" },
                TreeStyle::Unicode => if is_last { "└─" } else { "├─" },
                TreeStyle::Box => if is_last { "└─" } else { "├─" },
                TreeStyle::Double => if is_last { "╰─" } else { "├─" },
                TreeStyle::Round => if is_last { "╰─" } else { "├─" },
                TreeStyle::Custom => if is_last { "└─" } else { "├─" },
            };
            result.push_str(connector);
        }

        if let Some(icon) = &node.icon {
            result.push_str(&format!("{} ", icon));
        }

        result.push_str(&node.name);

        if let Some(value) = &node.value {
            result.push_str(&format!(" ({})", value));
        }

        result.push('\n');

        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            Self::format_tree_node(child, result, style, show_lines, indentation, level + 1, is_last_child);
        }
    }

    fn format_content(content: &OutputContent, theme: &OutputTheme) -> Result<String> {
        match content {
            OutputContent::Text(text) => Ok(text.clone()),
            OutputContent::Structured(value) => Ok(serde_json::to_string_pretty(value)?),
            OutputContent::Table(table) => Self::format_table(table, theme),
            OutputContent::List(list) => Self::format_list(list, theme),
            OutputContent::Tree(tree) => Self::format_tree(tree, theme),
            OutputContent::Binary(_) => Ok("[Binary data]".to_string()),
            OutputContent::Mixed(contents) => {
                let mut result = String::new();
                for content in contents {
                    result.push_str(&Self::format_content(content, theme)?);
                }
                Ok(result)
            }
        }
    }

    pub fn format_markdown_table(table: &TableData) -> Result<String> {
        Self::format_markdown_table(table)
    }

    fn format_markdown_list(list: &ListData) -> Result<String> {
        let mut result = String::new();
        Self::format_markdown_list_items(&list.items, &mut result, list.numbering, 0);
        Ok(result)
    }

    fn format_markdown_list_items(items: &[ListItem], result: &mut String, numbering: bool, level: usize) {
        for (i, item) in items.iter().enumerate() {
            for _ in 0..level {
                result.push_str("  ");
            }

            if numbering {
                result.push_str(&format!("{}. ", i + 1));
            } else {
                result.push_str("- ");
            }

            result.push_str(&item.content);
            result.push('\n');

            if !item.children.is_empty() {
                Self::format_markdown_list_items(&item.children, result, numbering, level + 1);
            }
        }
    }

    pub fn clone(&self) -> OutputManager {
        OutputManager {
            config: self.config.clone(),
            formatters: self.formatters.clone(),
            themes: self.themes.clone(),
            renderers: self.renderers.clone(),
        }
    }
}

impl OutputMetadata {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            tags: Vec::new(),
            categories: Vec::new(),
            author: None,
            version: None,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn clone(&self) -> OutputMetadata {
        OutputMetadata {
            title: self.title.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            categories: self.categories.clone(),
            author: self.author.clone(),
            version: self.version.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl ThemeColors {
    pub fn default() -> Self {
        Self {
            reset: "\x1b[0m".to_string(),
            black: "\x1b[30m".to_string(),
            red: "\x1b[31m".to_string(),
            green: "\x1b[32m".to_string(),
            yellow: "\x1b[33m".to_string(),
            blue: "\x1b[34m".to_string(),
            magenta: "\x1b[35m".to_string(),
            cyan: "\x1b[36m".to_string(),
            white: "\x1b[37m".to_string(),
            bright_black: "\x1b[90m".to_string(),
            bright_red: "\x1b[91m".to_string(),
            bright_green: "\x1b[92m".to_string(),
            bright_yellow: "\x1b[93m".to_string(),
            bright_blue: "\x1b[94m".to_string(),
            bright_magenta: "\x1b[95m".to_string(),
            bright_cyan: "\x1b[96m".to_string(),
            bright_white: "\x1b[97m".to_string(),
            background: "\x1b[48m".to_string(),
            foreground: "\x1b[38m".to_string(),
            accent: "\x1b[96m".to_string(),
            success: "\x1b[92m".to_string(),
            warning: "\x1b[93m".to_string(),
            error: "\x1b[91m".to_string(),
            info: "\x1b[94m".to_string(),
        }
    }

    pub fn dark() -> Self {
        Self::default()
    }

    pub fn light() -> Self {
        Self {
            ..Self::default()
        }
    }

    pub fn nocolor() -> Self {
        Self {
            reset: String::new(),
            black: String::new(),
            red: String::new(),
            green: String::new(),
            yellow: String::new(),
            blue: String::new(),
            magenta: String::new(),
            cyan: String::new(),
            white: String::new(),
            bright_black: String::new(),
            bright_red: String::new(),
            bright_green: String::new(),
            bright_yellow: String::new(),
            bright_blue: String::new(),
            bright_magenta: String::new(),
            bright_cyan: String::new(),
            bright_white: String::new(),
            background: String::new(),
            foreground: String::new(),
            accent: String::new(),
            success: String::new(),
            warning: String::new(),
            error: String::new(),
            info: String::new(),
        }
    }

    pub fn clone(&self) -> ThemeColors {
        ThemeColors {
            reset: self.reset.clone(),
            black: self.black.clone(),
            red: self.red.clone(),
            green: self.green.clone(),
            yellow: self.yellow.clone(),
            blue: self.blue.clone(),
            magenta: self.magenta.clone(),
            cyan: self.cyan.clone(),
            white: self.white.clone(),
            bright_black: self.bright_black.clone(),
            bright_red: self.bright_red.clone(),
            bright_green: self.bright_green.clone(),
            bright_yellow: self.bright_yellow.clone(),
            bright_blue: self.bright_blue.clone(),
            bright_magenta: self.bright_magenta.clone(),
            bright_cyan: self.bright_cyan.clone(),
            bright_white: self.bright_white.clone(),
            background: self.background.clone(),
            foreground: self.foreground.clone(),
            accent: self.accent.clone(),
            success: self.success.clone(),
            warning: self.warning.clone(),
            error: self.error.clone(),
            info: self.info.clone(),
        }
    }
}

impl ThemeStyles {
    pub fn default() -> Self {
        Self {
            bold: "\x1b[1m".to_string(),
            dim: "\x1b[2m".to_string(),
            italic: "\x1b[3m".to_string(),
            underline: "\x1b[4m".to_string(),
            blink: "\x1b[5m".to_string(),
            reverse: "\x1b[7m".to_string(),
            hidden: "\x1b[8m".to_string(),
            strikethrough: "\x1b[9m".to_string(),
            reset_all: "\x1b[0m".to_string(),
        }
    }

    pub fn nocolor() -> Self {
        Self {
            bold: String::new(),
            dim: String::new(),
            italic: String::new(),
            underline: String::new(),
            blink: String::new(),
            reverse: String::new(),
            hidden: String::new(),
            strikethrough: String::new(),
            reset_all: String::new(),
        }
    }

    pub fn clone(&self) -> ThemeStyles {
        ThemeStyles {
            bold: self.bold.clone(),
            dim: self.dim.clone(),
            italic: self.italic.clone(),
            underline: self.underline.clone(),
            blink: self.blink.clone(),
            reverse: self.reverse.clone(),
            hidden: self.hidden.clone(),
            strikethrough: self.strikethrough.clone(),
            reset_all: self.reset_all.clone(),
        }
    }
}

impl ThemeIcons {
    pub fn default() -> Self {
        Self {
            success: "✓".to_string(),
            warning: "⚠".to_string(),
            error: "✗".to_string(),
            info: "ℹ".to_string(),
            bullet: "•".to_string(),
            arrow_right: "→".to_string(),
            arrow_down: "↓".to_string(),
            folder: "📁".to_string(),
            file: "📄".to_string(),
            loading: "⏳".to_string(),
            check: "✓".to_string(),
            cross: "✗".to_string(),
        }
    }

    pub fn unicode() -> Self {
        Self {
            success: "✓".to_string(),
            warning: "⚠".to_string(),
            error: "✗".to_string(),
            info: "ℹ".to_string(),
            bullet: "•".to_string(),
            arrow_right: "→".to_string(),
            arrow_down: "↓".to_string(),
            folder: "📁".to_string(),
            file: "📄".to_string(),
            loading: "⏳".to_string(),
            check: "✓".to_string(),
            cross: "✗".to_string(),
        }
    }

    pub fn ascii() -> Self {
        Self {
            success: "+".to_string(),
            warning: "!".to_string(),
            error: "x".to_string(),
            info: "i".to_string(),
            bullet: "*".to_string(),
            arrow_right: "->".to_string(),
            arrow_down: "v".to_string(),
            folder: "[".to_string(),
            file: "-".to_string(),
            loading: "...".to_string(),
            check: "+".to_string(),
            cross: "x".to_string(),
        }
    }

    pub fn clone(&self) -> ThemeIcons {
        ThemeIcons {
            success: self.success.clone(),
            warning: self.warning.clone(),
            error: self.error.clone(),
            info: self.info.clone(),
            bullet: self.bullet.clone(),
            arrow_right: self.arrow_right.clone(),
            arrow_down: self.arrow_down.clone(),
            folder: self.folder.clone(),
            file: self.file.clone(),
            loading: self.loading.clone(),
            check: self.check.clone(),
            cross: self.cross.clone(),
        }
    }
}

impl OutputTheme {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            colors: ThemeColors::default(),
            styles: ThemeStyles::default(),
            icons: ThemeIcons::default(),
        }
    }

    pub fn clone(&self) -> OutputTheme {
        OutputTheme {
            name: self.name.clone(),
            description: self.description.clone(),
            colors: self.colors.clone(),
            styles: self.styles.clone(),
            icons: self.icons.clone(),
        }
    }
}

impl Default fn default() -> Self {
        Self {
            default_format: OutputFormat::Text,
            default_theme: "default".to_string(),
            enable_colors: true,
            enable_unicode: true,
            enable_progress: true,
            enable_interactive: true,
            enable_pager: false,
            pager_command: None,
            max_width: None,
            max_height: None,
            tab_size: 4,
            line_wrap: true,
        }
}

impl Default fn default() -> Self {
        Self {
            content: OutputContent::Text(String::new()),
            metadata: OutputMetadata::new(),
            timestamp: Utc::now(),
            source: String::new(),
        }
}

impl Default fn default() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            style: TableStyle::Simple,
            alignment: vec![Alignment::Left; 0],
            borders: false,
            padding: 1,
        }
}

impl Default fn default() -> Self {
        Self {
            items: Vec::new(),
            style: ListStyle::Bullets,
            numbering: false,
            indentation: 2,
        }
}

impl Default fn default() -> Self {
        Self {
            root: TreeNode {
                name: String::new(),
                value: None,
                children: Vec::new(),
                expanded: false,
                icon: None,
            },
            style: TreeStyle::Unicode,
            show_lines: true,
            indentation: 2,
        }
}

impl Default fn default() -> Self {
        Self {
            name: String::new(),
            value: None,
            children: Vec::new(),
            expanded: false,
            icon: None,
        }
}

impl Default fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: "Default theme".to_string(),
            colors: ThemeColors::default(),
            styles: ThemeStyles::default(),
            icons: ThemeIcons::default(),
        }
}

pub fn create_output_manager(config: OutputConfig) -> OutputManager {
    OutputManager::new(config)
}

pub fn create_output_config() -> OutputConfig {
    OutputConfig::default()
}

pub fn create_output_data(content: OutputContent) -> OutputData {
    OutputData {
        content,
        metadata: OutputMetadata::new(),
        timestamp: Utc::now(),
        source: "cli".to_string(),
    }
}

pub fn create_table_data(headers: Vec<String>, rows: Vec<Vec<String>>) -> TableData {
    TableData {
        headers,
        rows,
        style: TableStyle::Simple,
        alignment: vec![Alignment::Left; headers.len()],
        borders: false,
        padding: 1,
    }
}

pub fn create_list_data(items: Vec<ListItem>) -> ListData {
    ListData {
        items,
        style: ListStyle::Bullets,
        numbering: false,
        indentation: 2,
    }
}

pub fn create_tree_data(root: TreeNode) -> TreeData {
    TreeData {
        root,
        style: TreeStyle::Unicode,
        show_lines: true,
        indentation: 2,
    }
}
