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
pub struct InteractiveManager {
    pub config: InteractiveConfig,
    pub prompts: Arc<RwLock<HashMap<String, InteractivePrompt>>>,
    pub dialogs: Arc<RwLock<HashMap<String, InteractiveDialog>>>,
    pub menus: Arc<RwLock<HashMap<String, InteractiveMenu>>>,
    pub wizards: Arc<RwLock<HashMap<String, InteractiveWizard>>>,
}

#[derive(Debug, Clone)]
pub struct InteractiveConfig {
    pub enable_interactive: bool,
    pub default_theme: String,
    pub timeout: Option<u64>,
    pub auto_confirm: bool,
    pub show_help: bool,
    pub show_defaults: bool,
    pub max_attempts: u32,
    pub clear_screen: bool,
}

#[derive(Debug, Clone)]
pub struct InteractivePrompt {
    pub id: String,
    pub name: String,
    pub description: String,
    pub message: String,
    pub prompt_type: PromptType,
    pub default_value: Option<String>,
    pub placeholder: Option<String>,
    pub validator: Option<PromptValidator>,
    pub formatter: Option<PromptFormatter>,
    pub required: bool,
    pub sensitive: bool,
    pub multiline: bool,
    pub auto_complete: Option<AutoComplete>,
    pub history: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptType {
    Text,
    Password,
    Number,
    Integer,
    Float,
    Boolean,
    Select,
    MultiSelect,
    Confirm,
    File,
    Directory,
    Date,
    Time,
    DateTime,
    Email,
    Url,
    Color,
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
    pub allowed_values: Vec<String>,
    pub custom_validator: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorType {
    Required,
    Pattern,
    Length,
    Range,
    Enum,
    Email,
    Url,
    File,
    Directory,
    Custom,
}

#[derive(Debug, Clone)]
pub struct PromptFormatter {
    pub formatter_type: FormatterType,
    pub format: String,
    pub case_sensitive: bool,
    pub trim: bool,
    pub normalize: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatterType {
    Lowercase,
    Uppercase,
    TitleCase,
    CamelCase,
    SnakeCase,
    KebabCase,
    Custom,
}

#[derive(Debug, Clone)]
pub struct AutoComplete {
    pub completions: Vec<String>,
    pub completion_type: CompletionType,
    pub case_sensitive: bool,
    pub fuzzy: bool,
    pub max_suggestions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionType {
    Static,
    Dynamic,
    File,
    Directory,
    Command,
    Custom,
}

#[derive(Debug, Clone)]
pub struct InteractiveDialog {
    pub id: String,
    pub name: String,
    pub description: String,
    pub title: String,
    pub message: String,
    pub dialog_type: DialogType,
    pub buttons: Vec<DialogButton>,
    pub default_button: Option<String>,
    pub cancel_button: Option<String>,
    pub modal: bool,
    pub resizable: bool,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    Info,
    Warning,
    Error,
    Question,
    Input,
    Progress,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogButton {
    pub id: String,
    pub label: String,
    pub button_type: ButtonType,
    pub action: Option<DialogAction>,
    pub shortcut: Option<char>,
    pub default: bool,
    pub cancel: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonType {
    OK,
    Cancel,
    Yes,
    No,
    Retry,
    Abort,
    Ignore,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogAction {
    pub action_type: ActionType,
    pub target: Option<String>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Close,
    Submit,
    Reset,
    Navigate,
    Execute,
    Custom,
}

#[derive(Debug, Clone)]
pub struct InteractiveMenu {
    pub id: String,
    pub name: String,
    pub description: String,
    pub title: String,
    pub menu_type: MenuType,
    pub items: Vec<MenuItem>,
    pub default_item: Option<String>,
    pub multi_select: bool,
    pub show_numbers: bool,
    pub show_shortcuts: bool,
    pub wrap_around: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuType {
    Simple,
    Cascading,
    Context,
    Toolbar,
    Sidebar,
    Custom,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub shortcut: Option<char>,
    pub icon: Option<String>,
    pub enabled: bool,
    pub checked: bool,
    pub separator: bool,
    pub submenu: Option<Vec<MenuItem>>,
    pub action: Option<MenuAction>,
}

#[derive(Debug, Clone)]
pub struct MenuAction {
    pub action_type: MenuActionType,
    pub command: Option<String>,
    pub parameters: HashMap<String, String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuActionType {
    Command,
    Navigate,
    Toggle,
    Open,
    Close,
    Execute,
    Custom,
}

#[derive(Debug, Clone)]
pub struct InteractiveWizard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub title: String,
    pub steps: Vec<WizardStep>,
    pub current_step: usize,
    pub allow_skip: bool,
    pub allow_back: bool,
    pub show_progress: bool,
    pub cancelable: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WizardStep {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub step_type: StepType,
    pub content: StepContent,
    pub validation: Option<StepValidation>,
    pub conditional: Option<StepConditional>,
    pub required: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepType {
    Introduction,
    Information,
    Input,
    Confirmation,
    Summary,
    Completion,
    Custom,
}

#[derive(Debug, Clone)]
pub enum StepContent {
    Text(String),
    Prompt(InteractivePrompt),
    Form(Vec<FormField>),
    Table(TableData),
    Progress(ProgressData),
    Custom(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub id: String,
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default_value: Option<String>,
    pub options: Vec<FieldOption>,
    pub validation: Option<FieldValidation>,
    pub help_text: Option<String>,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Password,
    Number,
    Email,
    Url,
    Date,
    Time,
    DateTime,
    Select,
    MultiSelect,
    Radio,
    Checkbox,
    Textarea,
    File,
    Directory,
    Color,
    Custom,
}

#[derive(Debug, Clone)]
pub struct FieldOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct FieldValidation {
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub custom_validator: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StepValidation {
    pub validator_type: StepValidatorType,
    pub conditions: Vec<ValidationCondition>,
    pub error_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepValidatorType {
    Required,
    Conditional,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ValidationCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    IsEmpty,
    IsNotEmpty,
}

#[derive(Debug, Clone)]
pub struct StepConditional {
    pub conditions: Vec<ValidationCondition>,
    pub logic: ConditionalLogic,
    pub action: ConditionalAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalLogic {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalAction {
    Show,
    Hide,
    Enable,
    Disable,
    Require,
    Optional,
    Skip,
}

#[derive(Debug, Clone)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub selectable: bool,
    pub multi_select: bool,
    pub sortable: bool,
}

#[derive(Debug, Clone)]
pub struct ProgressData {
    pub current: u64,
    pub total: u64,
    pub message: Option<String>,
    pub show_percentage: bool,
    pub show_eta: bool,
    pub show_rate: bool,
}

impl InteractiveManager {
    pub fn new(config: InteractiveConfig) -> Self {
        Self {
            config,
            prompts: Arc::new(RwLock::new(HashMap::new())),
            dialogs: Arc::new(RwLock::new(HashMap::new())),
            menus: Arc::new(RwLock::new(HashMap::new())),
            wizards: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_prompt(&mut self, prompt: InteractivePrompt) -> Result<()> {
        let mut prompts = self.prompts.write();
        prompts.insert(prompt.id.clone(), prompt);
        Ok(())
    }

    pub fn register_dialog(&mut self, dialog: InteractiveDialog) -> Result<()> {
        let mut dialogs = self.dialogs.write();
        dialogs.insert(dialog.id.clone(), dialog);
        Ok(())
    }

    pub fn register_menu(&mut self, menu: InteractiveMenu) -> Result<()> {
        let mut menus = self.menus.write();
        menus.insert(menu.id.clone(), menu);
        Ok(())
    }

    pub fn register_wizard(&mut self, wizard: InteractiveWizard) -> Result<()> {
        let mut wizards = self.wizards.write();
        wizards.insert(wizard.id.clone(), wizard);
        Ok(())
    }

    pub fn prompt(&mut self, prompt_id: &str) -> Result<PromptResult> {
        if !self.config.enable_interactive {
            return Err(EllasticError::InvalidOperation("Interactive mode is disabled".to_string()));
        }

        let prompt = {
            let prompts = self.prompts.read();
            prompts.get(prompt_id).cloned()
                .ok_or_else(|| EllasticError::InvalidParameter(format!("Prompt '{}' not found", prompt_id)))?
        };

        self.execute_prompt(&prompt)
    }

    fn execute_prompt(&self, prompt: &InteractivePrompt) -> Result<PromptResult> {

        let input = if let Some(default) = &prompt.default_value {
            default.clone()
        } else {
            match prompt.prompt_type {
                PromptType::Text => "default text".to_string(),
                PromptType::Password => "********".to_string(),
                PromptType::Number => "42".to_string(),
                PromptType::Integer => "42".to_string(),
                PromptType::Float => "42.0".to_string(),
                PromptType::Boolean => "true".to_string(),
                PromptType::Select => "option1".to_string(),
                PromptType::MultiSelect => "option1,option2".to_string(),
                PromptType::Confirm => "y".to_string(),
                PromptType::File => "/path/to/file".to_string(),
                PromptType::Directory => "/path/to/directory".to_string(),
                PromptType::Date => "2023-01-01".to_string(),
                PromptType::Time => "12:00:00".to_string(),
                PromptType::DateTime => "2023-01-01T12:00:00".to_string(),
                PromptType::Email => "user@example.com".to_string(),
                PromptType::Url => "https://example.com".to_string(),
                PromptType::Color => "#FF0000".to_string(),
                PromptType::Custom => "custom".to_string(),
            }
        };

        if let Some(validator) = &prompt.validator {
            self.validate_input(&input, validator)?;
        }

        let formatted_input = if let Some(formatter) = &prompt.formatter {
            self.format_input(&input, formatter)?
        } else {
            input
        };

        Ok(PromptResult {
            success: true,
            value: formatted_input,
            cancelled: false,
            timeout: false,
            attempts: 1,
        })
    }

    fn validate_input(&self, input: &str, validator: &PromptValidator) -> Result<()> {
        match validator.validator_type {
            ValidatorType::Required => {
                if input.is_empty() {
                    return Err(EllasticError::InvalidParameter(validator.error_message.as_ref()
                        .unwrap_or(&"Input is required".to_string()).clone()));
                }
            }
            ValidatorType::Pattern => {
                if let Some(pattern) = &validator.pattern {
                }
            }
            ValidatorType::Length => {
                let length = input.len();
                if let Some(min_length) = validator.min_length {
                    if length < min_length {
                        return Err(EllasticError::InvalidParameter(format!("Input must be at least {} characters", min_length)));
                    }
                }
                if let Some(max_length) = validator.max_length {
                    if length > max_length {
                        return Err(EllasticError::InvalidParameter(format!("Input must be at most {} characters", max_length)));
                    }
                }
            }
            ValidatorType::Range => {
                if let Ok(num_value) = input.parse::<f64>() {
                    if let Some(min_value) = validator.min_value {
                        if num_value < min_value {
                            return Err(EllasticError::InvalidParameter(format!("Value must be at least {}", min_value)));
                        }
                    }
                    if let Some(max_value) = validator.max_value {
                        if num_value > max_value {
                            return Err(EllasticError::InvalidParameter(format!("Value must be at most {}", max_value)));
                        }
                    }
                } else {
                    return Err(EllasticError::InvalidParameter("Input must be a number".to_string()));
                }
            }
            ValidatorType::Enum => {
                if !validator.allowed_values.contains(&input.to_string()) {
                    return Err(EllasticError::InvalidParameter(format!("Input must be one of: {:?}", validator.allowed_values)));
                }
            }
            ValidatorType::Email => {
                if !input.contains('@') {
                    return Err(EllasticError::InvalidParameter("Invalid email address".to_string()));
                }
            }
            ValidatorType::Url => {
                if !input.starts_with("http://") && !input.starts_with("https://") {
                    return Err(EllasticError::InvalidParameter("Invalid URL".to_string()));
                }
            }
            ValidatorType::File => {
            }
            ValidatorType::Directory => {
            }
            ValidatorType::Custom => {
            }
        }

        Ok(())
    }

    fn format_input(&self, input: &str, formatter: &PromptFormatter) -> Result<String> {
        let mut result = input.to_string();

        if formatter.trim {
            result = result.trim().to_string();
        }

        match formatter.formatter_type {
            FormatterType::Lowercase => {
                if !formatter.case_sensitive {
                    result = result.to_lowercase();
                }
            }
            FormatterType::Uppercase => {
                if !formatter.case_sensitive {
                    result = result.to_uppercase();
                }
            }
            FormatterType::TitleCase => {
                result = result.chars().enumerate()
                    .map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_string() })
                    .collect();
            }
            FormatterType::CamelCase => {
            }
            FormatterType::SnakeCase => {
            }
            FormatterType::KebabCase => {
            }
            FormatterType::Custom => {
            }
        }

        if formatter.normalize {
        }

        Ok(result)
    }

    pub fn dialog(&mut self, dialog_id: &str) -> Result<DialogResult> {
        if !self.config.enable_interactive {
            return Err(EllasticError::InvalidOperation("Interactive mode is disabled".to_string()));
        }

        let dialog = {
            let dialogs = self.dialogs.read();
            dialogs.get(dialog_id).cloned()
                .ok_or_else(|| EllasticError::InvalidParameter(format!("Dialog '{}' not found", dialog_id)))?
        };

        self.execute_dialog(&dialog)
    }

    fn execute_dialog(&self, dialog: &InteractiveDialog) -> Result<DialogResult> {

        let button_id = dialog.default_button.as_ref()
            .or_else(|| dialog.buttons.first().map(|b| &b.id))
            .cloned()
            .unwrap_or_else(|| "ok".to_string());

        Ok(DialogResult {
            success: true,
            button_id,
            cancelled: false,
            timeout: false,
        })
    }

    pub fn menu(&mut self, menu_id: &str) -> Result<MenuResult> {
        if !self.config.enable_interactive {
            return Err(EllasticError::InvalidOperation("Interactive mode is disabled".to_string()));
        }

        let menu = {
            let menus = self.menus.read();
            menus.get(menu_id).cloned()
                .ok_or_else(|| EllasticError::InvalidParameter(format!("Menu '{}' not found", menu_id)))?
        };

        self.execute_menu(&menu)
    }

    fn execute_menu(&self, menu: &InteractiveMenu) -> Result<MenuResult> {

        let item_id = menu.default_item.as_ref()
            .or_else(|| menu.items.first().map(|i| &i.id))
            .cloned()
            .unwrap_or_else(|| "item1".to_string());

        Ok(MenuResult {
            success: true,
            item_id,
            cancelled: false,
            timeout: false,
        })
    }

    pub fn wizard(&mut self, wizard_id: &str) -> Result<WizardResult> {
        if !self.config.enable_interactive {
            return Err(EllasticError::InvalidOperation("Interactive mode is disabled".to_string()));
        }

        let wizard = {
            let wizards = self.wizards.read();
            wizards.get(wizard_id).cloned()
                .ok_or_else(|| EllasticError::InvalidParameter(format!("Wizard '{}' not found", wizard_id)))?
        };

        self.execute_wizard(&wizard)
    }

    fn execute_wizard(&self, wizard: &InteractiveWizard) -> Result<WizardResult> {

        let mut step_results = HashMap::new();

        for step in &wizard.steps {
            let result = match &step.content {
                StepContent::Prompt(prompt) => {
                    let prompt_result = self.execute_prompt(prompt)?;
                    Some(serde_json::to_value(prompt_result).unwrap())
                }
                StepContent::Form(fields) => {
                    let mut form_data = HashMap::new();
                    for field in fields {
                        form_data.insert(field.id.clone(), serde_json::Value::String(field.default_value.clone().unwrap_or_default()));
                    }
                    Some(serde_json::to_value(form_data).unwrap())
                }
                _ => None,
            };

            if let Some(result) = result {
                step_results.insert(step.id.clone(), result);
            }
        }

        Ok(WizardResult {
            success: true,
            step_results,
            cancelled: false,
            completed: true,
        })
    }

    pub fn get_prompt(&self, prompt_id: &str) -> Option<InteractivePrompt> {
        self.prompts.read().get(prompt_id).cloned()
    }

    pub fn get_dialog(&self, dialog_id: &str) -> Option<InteractiveDialog> {
        self.dialogs.read().get(dialog_id).cloned()
    }

    pub fn get_menu(&self, menu_id: &str) -> Option<InteractiveMenu> {
        self.menus.read().get(menu_id).cloned()
    }

    pub fn get_wizard(&self, wizard_id: &str) -> Option<InteractiveWizard> {
        self.wizards.read().get(wizard_id).cloned()
    }

    pub fn list_prompts(&self) -> Vec<InteractivePrompt> {
        self.prompts.read().values().cloned().collect()
    }

    pub fn list_dialogs(&self) -> Vec<InteractiveDialog> {
        self.dialogs.read().values().cloned().collect()
    }

    pub fn list_menus(&self) -> Vec<InteractiveMenu> {
        self.menus.read().values().cloned().collect()
    }

    pub fn list_wizards(&self) -> Vec<InteractiveWizard> {
        self.wizards.read().values().cloned().collect()
    }

    pub fn clone(&self) -> InteractiveManager {
        InteractiveManager {
            config: self.config.clone(),
            prompts: self.prompts.clone(),
            dialogs: self.dialogs.clone(),
            menus: self.menus.clone(),
            wizards: self.wizards.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromptResult {
    pub success: bool,
    pub value: String,
    pub cancelled: bool,
    pub timeout: bool,
    pub attempts: u32,
}

#[derive(Debug, Clone)]
pub struct DialogResult {
    pub success: bool,
    pub button_id: String,
    pub cancelled: bool,
    pub timeout: bool,
}

#[derive(Debug, Clone)]
pub struct MenuResult {
    pub success: bool,
    pub item_id: String,
    pub cancelled: bool,
    pub timeout: bool,
}

#[derive(Debug, Clone)]
pub struct WizardResult {
    pub success: bool,
    pub step_results: HashMap<String, serde_json::Value>,
    pub cancelled: bool,
    pub completed: bool,
}

impl InteractivePrompt {
    pub fn new(id: String, name: String, message: String, prompt_type: PromptType) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            message,
            prompt_type,
            default_value: None,
            placeholder: None,
            validator: None,
            formatter: None,
            required: false,
            sensitive: false,
            multiline: false,
            auto_complete: None,
            history: false,
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_default(mut self, value: String) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn with_validator(mut self, validator: PromptValidator) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn with_formatter(mut self, formatter: PromptFormatter) -> Self {
        self.formatter = Some(formatter);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }

    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn with_auto_complete(mut self, auto_complete: AutoComplete) -> Self {
        self.auto_complete = Some(auto_complete);
        self
    }

    pub fn with_history(mut self) -> Self {
        self.history = true;
        self
    }

    pub fn clone(&self) -> InteractivePrompt {
        InteractivePrompt {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            message: self.message.clone(),
            prompt_type: self.prompt_type,
            default_value: self.default_value.clone(),
            placeholder: self.placeholder.clone(),
            validator: self.validator.clone(),
            formatter: self.formatter.clone(),
            required: self.required,
            sensitive: self.sensitive,
            multiline: self.multiline,
            auto_complete: self.auto_complete.clone(),
            history: self.history,
            created_at: self.created_at,
        }
    }
}

impl InteractiveDialog {
    pub fn new(id: String, name: String, title: String, message: String, dialog_type: DialogType) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            title,
            message,
            dialog_type,
            buttons: Vec::new(),
            default_button: None,
            cancel_button: None,
            modal: true,
            resizable: false,
            width: None,
            height: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_button(mut self, button: DialogButton) -> Self {
        self.buttons.push(button);
        self
    }

    pub fn with_default_button(mut self, button_id: String) -> Self {
        self.default_button = Some(button_id);
        self
    }

    pub fn with_cancel_button(mut self, button_id: String) -> Self {
        self.cancel_button = Some(button_id);
        self
    }

    pub fn modal(mut self) -> Self {
        self.modal = true;
        self
    }

    pub fn resizable(mut self) -> Self {
        self.resizable = true;
        self
    }

    pub fn with_size(mut self, width: usize, height: usize) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn clone(&self) -> InteractiveDialog {
        InteractiveDialog {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            title: self.title.clone(),
            message: self.message.clone(),
            dialog_type: self.dialog_type,
            buttons: self.buttons.clone(),
            default_button: self.default_button.clone(),
            cancel_button: self.cancel_button.clone(),
            modal: self.modal,
            resizable: self.resizable,
            width: self.width,
            height: self.height,
            created_at: self.created_at,
        }
    }
}

impl InteractiveMenu {
    pub fn new(id: String, name: String, title: String, menu_type: MenuType) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            title,
            menu_type,
            items: Vec::new(),
            default_item: None,
            multi_select: false,
            show_numbers: true,
            show_shortcuts: true,
            wrap_around: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn with_default_item(mut self, item_id: String) -> Self {
        self.default_item = Some(item_id);
        self
    }

    pub fn multi_select(mut self) -> Self {
        self.multi_select = true;
        self
    }

    pub fn show_numbers(mut self) -> Self {
        self.show_numbers = true;
        self
    }

    pub fn show_shortcuts(mut self) -> Self {
        self.show_shortcuts = true;
        self
    }

    pub fn wrap_around(mut self) -> Self {
        self.wrap_around = true;
        self
    }

    pub fn clone(&self) -> InteractiveMenu {
        InteractiveMenu {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            title: self.title.clone(),
            menu_type: self.menu_type,
            items: self.items.clone(),
            default_item: self.default_item.clone(),
            multi_select: self.multi_select,
            show_numbers: self.show_numbers,
            show_shortcuts: self.show_shortcuts,
            wrap_around: self.wrap_around,
            created_at: self.created_at,
        }
    }
}

impl InteractiveWizard {
    pub fn new(id: String, name: String, title: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            title,
            steps: Vec::new(),
            current_step: 0,
            allow_skip: false,
            allow_back: true,
            show_progress: true,
            cancelable: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_step(mut self, step: WizardStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn allow_skip(mut self) -> Self {
        self.allow_skip = true;
        self
    }

    pub fn allow_back(mut self) -> Self {
        self.allow_back = true;
        self
    }

    pub fn show_progress(mut self) -> Self {
        self.show_progress = true;
        self
    }

    pub fn cancelable(mut self) -> Self {
        self.cancelable = true;
        self
    }

    pub fn clone(&self) -> InteractiveWizard {
        InteractiveWizard {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            title: self.title.clone(),
            steps: self.steps.clone(),
            current_step: self.current_step,
            allow_skip: self.allow_skip,
            allow_back: self.allow_back,
            show_progress: self.show_progress,
            cancelable: self.cancelable,
            created_at: self.created_at,
        }
    }
}

impl WizardStep {
    pub fn new(id: String, name: String, title: String, step_type: StepType, content: StepContent) -> Self {
        Self {
            id,
            name,
            title,
            description: String::new(),
            step_type,
            content,
            validation: None,
            conditional: None,
            required: false,
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_validation(mut self, validation: StepValidation) -> Self {
        self.validation = Some(validation);
        self
    }

    pub fn with_conditional(mut self, conditional: StepConditional) -> Self {
        self.conditional = Some(conditional);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn clone(&self) -> WizardStep {
        WizardStep {
            id: self.id.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            step_type: self.step_type,
            content: self.content.clone(),
            validation: self.validation.clone(),
            conditional: self.conditional.clone(),
            required: self.required,
            created_at: self.created_at,
        }
    }
}

impl Default fn default() -> Self {
        Self {
            enable_interactive: true,
            default_theme: "default".to_string(),
            timeout: None,
            auto_confirm: false,
            show_help: true,
            show_defaults: true,
            max_attempts: 3,
            clear_screen: false,
        }
}

impl Default fn default() -> Self {
        Self {
            success: true,
            value: String::new(),
            cancelled: false,
            timeout: false,
            attempts: 1,
        }
}

impl Default fn default() -> Self {
        Self {
            success: true,
            button_id: String::new(),
            cancelled: false,
            timeout: false,
        }
}

impl Default fn default() -> Self {
        Self {
            success: true,
            item_id: String::new(),
            cancelled: false,
            timeout: false,
        }
}

impl Default fn default() -> Self {
        Self {
            success: true,
            step_results: HashMap::new(),
            cancelled: false,
            completed: true,
        }
}

pub fn create_interactive_manager(config: InteractiveConfig) -> InteractiveManager {
    InteractiveManager::new(config)
}

pub fn create_interactive_config() -> InteractiveConfig {
    InteractiveConfig::default()
}

pub fn create_prompt(id: String, name: String, message: String, prompt_type: PromptType) -> InteractivePrompt {
    InteractivePrompt::new(id, name, message, prompt_type)
}

pub fn create_dialog(id: String, name: String, title: String, message: String, dialog_type: DialogType) -> InteractiveDialog {
    InteractiveDialog::new(id, name, title, message, dialog_type)
}

pub fn create_menu(id: String, name: String, title: String, menu_type: MenuType) -> InteractiveMenu {
    InteractiveMenu::new(id, name, title, menu_type)
}

pub fn create_wizard(id: String, name: String, title: String) -> InteractiveWizard {
    InteractiveWizard::new(id, name, title)
}
