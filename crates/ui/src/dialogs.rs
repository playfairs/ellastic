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
pub struct DialogManager {
    pub dialogs: Arc<RwLock<HashMap<Uuid, Dialog>>>,
    pub active_dialogs: Arc<RwLock<Vec<Uuid>>>,
    pub dialog_history: Arc<RwLock<Vec<DialogHistoryEntry>>>,
    pub config: DialogManagerConfig,
}

#[derive(Debug, Clone)]
pub struct DialogManagerConfig {
    pub max_dialogs: usize,
    pub max_history: usize,
    pub auto_close_delay: u64,
    pub enable_animations: bool,
    pub animation_speed: f32,
    pub modal_background: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct Dialog {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub dialog_type: DialogType,
    pub content: DialogContent,
    pub buttons: Vec<DialogButton>,
    pub position: DialogPosition,
    pub size: DialogSize,
    pub state: DialogState,
    pub config: DialogConfig,
    pub result: Option<DialogResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    Info,
    Warning,
    Error,
    Confirm,
    Input,
    File,
    Directory,
    Color,
    Font,
    Progress,
    Settings,
    About,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogContent {
    pub message: String,
    pub details: Option<String>,
    pub icon: Option<DialogIcon>,
    pub progress: Option<DialogProgress>,
    pub form: Option<DialogForm>,
    pub preview: Option<DialogPreview>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogIcon {
    Info,
    Warning,
    Error,
    Success,
    Question,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogProgress {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub text: Option<String>,
    pub indeterminate: bool,
}

#[derive(Debug, Clone)]
pub struct DialogForm {
    pub fields: Vec<FormField>,
    pub validation: Option<FormValidation>,
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub id: String,
    pub name: String,
    pub field_type: FieldType,
    pub label: String,
    pub placeholder: Option<String>,
    pub value: Option<serde_json::Value>,
    pub options: Vec<FieldOption>,
    pub required: bool,
    pub readonly: bool,
    pub disabled: bool,
    pub validation: Option<FieldValidation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Text,
    TextArea,
    Number,
    Email,
    Password,
    Checkbox,
    Radio,
    Select,
    MultiSelect,
    File,
    Directory,
    Color,
    Date,
    Time,
    DateTime,
    Range,
    Custom,
}

#[derive(Debug, Clone)]
pub struct FieldOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
    pub group: Option<String>,
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
pub struct FormValidation {
    pub on_submit: bool,
    pub on_change: bool,
    pub show_errors: bool,
    pub error_style: ErrorDisplayStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDisplayStyle {
    Inline,
    Below,
    Tooltip,
    Summary,
}

#[derive(Debug, Clone)]
pub struct DialogPreview {
    pub preview_type: PreviewType,
    pub data: Vec<u8>,
    pub metadata: PreviewMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewType {
    Image,
    Video,
    Audio,
    Text,
    Code,
    Custom,
}

#[derive(Debug, Clone)]
pub struct PreviewMetadata {
    pub format: String,
    pub size: usize,
    pub dimensions: Option<(u32, u32)>,
    pub duration: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DialogButton {
    pub id: String,
    pub label: String,
    pub button_type: ButtonType,
    pub default: bool,
    pub cancel: bool,
    pub disabled: bool,
    pub action: DialogAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonType {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
    Info,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogAction {
    pub action_type: ActionType,
    pub target: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Close,
    Submit,
    Cancel,
    Reset,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogPosition {
    pub x: f32,
    pub y: f32,
    pub anchor: DialogAnchor,
    pub offset: egui::Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnchor {
    Center,
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogSize {
    pub width: DialogSizeValue,
    pub height: DialogSizeValue,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub resizable: bool,
}

#[derive(Debug, Clone)]
pub enum DialogSizeValue {
    Fixed(f32),
    Percent(f32),
    Auto,
    Content,
    MaxContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogState {
    Open,
    Closed,
    Minimized,
    Maximized,
    Loading,
    Error,
}

#[derive(Debug, Clone)]
pub struct DialogConfig {
    pub modal: bool,
    pub resizable: bool,
    pub movable: bool,
    pub minimizable: bool,
    pub maximizable: bool,
    pub closable: bool,
    pub auto_close: bool,
    pub auto_close_delay: u64,
    pub show_title_bar: bool,
    pub show_status_bar: bool,
    pub show_progress_bar: bool,
    pub escape_key_closes: bool,
    pub click_outside_closes: bool,
}

#[derive(Debug, Clone)]
pub struct DialogResult {
    pub dialog_id: Uuid,
    pub button_id: Option<String>,
    pub data: Option<HashMap<String, serde_json::Value>>,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DialogHistoryEntry {
    pub dialog_id: Uuid,
    pub dialog_type: DialogType,
    pub title: String,
    pub result: DialogResult,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DialogTheme {
    pub name: String,
    pub colors: DialogColors,
    pub fonts: DialogFonts,
    pub sizes: DialogSizes,
    pub spacing: DialogSpacing,
    pub animations: DialogAnimations,
}

#[derive(Debug, Clone)]
pub struct DialogColors {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub shadow: egui::Color32,
    pub title: egui::Color32,
    pub title_background: egui::Color32,
    pub content_background: egui::Color32,
    pub button_primary: egui::Color32,
    pub button_primary_text: egui::Color32,
    pub button_secondary: egui::Color32,
    pub button_secondary_text: egui::Color32,
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub error: egui::Color32,
    pub info: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct DialogFonts {
    pub title: egui::FontId,
    pub body: egui::FontId,
    pub button: egui::FontId,
    pub caption: egui::FontId,
    pub monospace: egui::FontId,
}

#[derive(Debug, Clone)]
pub struct DialogSizes {
    pub min_width: f32,
    pub min_height: f32,
    pub default_width: f32,
    pub default_height: f32,
    pub max_width: f32,
    pub max_height: f32,
    pub button_height: f32,
    pub button_min_width: f32,
    pub icon_size: f32,
}

#[derive(Debug, Clone)]
pub struct DialogSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

#[derive(Debug, Clone)]
pub struct DialogAnimations {
    pub duration: f32,
    pub easing: EasingFunction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
}

#[derive(Debug, Clone)]
pub struct DialogEvent {
    pub id: Uuid,
    pub event_type: DialogEventType,
    pub dialog_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogEventType {
    Created,
    Opened,
    Closed,
    Minimized,
    Maximized,
    Restored,
    ButtonClicked,
    FormSubmitted,
    FormValidated,
    ProgressUpdated,
    Error,
    Custom,
}

impl DialogManager {
    pub fn new(config: DialogManagerConfig) -> Self {
        Self {
            dialogs: Arc::new(RwLock::new(HashMap::new())),
            active_dialogs: Arc::new(RwLock::new(Vec::new())),
            dialog_history: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    pub fn create_dialog(&mut self, name: String, title: String, dialog_type: DialogType, content: DialogContent, buttons: Vec<DialogButton>, config: DialogConfig) -> Result<Uuid> {
        let dialog_id = Uuid::new_v4();
        let now = Utc::now();

        if self.dialogs.read().len() >= self.config.max_dialogs {
            return Err(EllasticError::LimitExceeded("Maximum dialog limit reached".to_string()));
        }

        let dialog = Dialog {
            id: dialog_id,
            name,
            title,
            dialog_type,
            content,
            buttons,
            position: DialogPosition::new(),
            size: DialogSize::new(),
            state: DialogState::Open,
            config,
            result: None,
            created_at: now,
            updated_at: now,
        };

        self.dialogs.write().insert(dialog_id, dialog);
        self.active_dialogs.write().push(dialog_id);
        Ok(dialog_id)
    }

    pub fn get_dialog(&self, dialog_id: Uuid) -> Option<&Dialog> {
        self.dialogs.read().get(&dialog_id)
    }

    pub fn list_dialogs(&self) -> Vec<&Dialog> {
        self.dialogs.read().values().collect()
    }

    pub fn list_active_dialogs(&self) -> Vec<&Dialog> {
        let active_dialogs = self.active_dialogs.read();
        let dialogs = self.dialogs.read();
        
        active_dialogs.iter()
            .filter_map(|id| dialogs.get(id))
            .collect()
    }

    pub fn update_dialog(&mut self, dialog_id: Uuid, content: Option<DialogContent>, buttons: Option<Vec<DialogButton>>, state: Option<DialogState>) -> Result<()> {
        let mut dialogs = self.dialogs.write();
        if let Some(dialog) = dialogs.get_mut(&dialog_id) {
            if let Some(content) = content {
                dialog.content = content;
            }
            if let Some(buttons) = buttons {
                dialog.buttons = buttons;
            }
            if let Some(state) = state {
                dialog.state = state;
            }
            dialog.updated_at = Utc::now();
            Ok(())
        } else {
            Err(EllasticError::InvalidParameter(format!("Dialog {} not found", dialog_id)))
        }
    }

    pub fn close_dialog(&mut self, dialog_id: Uuid, result: Option<DialogResult>) -> Result<()> {
        let mut dialogs = self.dialogs.write();
        if let Some(dialog) = dialogs.get_mut(&dialog_id) {
            dialog.state = DialogState::Closed;
            dialog.result = result.clone();
            dialog.updated_at = Utc::now();
            
            self.active_dialogs.write().retain(|&id| id != dialog_id);
            
            if let Some(result) = result {
                let history_entry = DialogHistoryEntry {
                    dialog_id,
                    dialog_type: dialog.dialog_type,
                    title: dialog.title.clone(),
                    result,
                    timestamp: Utc::now(),
                };
                
                let mut history = self.dialog_history.write();
                history.push(history_entry);
                
                if history.len() > self.config.max_history {
                    history.remove(0);
                }
            }
            
            Ok(())
        } else {
            Err(EllasticError::InvalidParameter(format!("Dialog {} not found", dialog_id)))
        }
    }

    pub fn render_dialogs(&self, ctx: &egui::Context) -> Result<()> {
        let active_dialogs = self.active_dialogs.read();
        let dialogs = self.dialogs.read();
        
        for dialog_id in active_dialogs.iter() {
            if let Some(dialog) = dialogs.get(dialog_id) {
                self.render_dialog(ctx, dialog)?;
            }
        }
        
        Ok(())
    }

    fn render_dialog(&self, ctx: &egui::Context, dialog: &Dialog) -> Result<()> {
        let window = egui::Window::new(&dialog.title)
            .collapsible(false)
            .resizable(dialog.config.resizable)
            .movable(dialog.config.movable);

        let window = if dialog.config.modal {
            window.fixed_size(dialog.size.width.to_f32(), dialog.size.height.to_f32())
        } else {
            window
        };

        window.show(ctx, |ui| {
            self.render_dialog_content(ui, dialog)?;
            
            self.render_dialog_buttons(ui, dialog)?;
        });

        Ok(())
    }

    fn render_dialog_content(&self, ui: &mut egui::Ui, dialog: &Dialog) -> Result<()> {
        if let Some(icon) = dialog.content.icon {
            self.render_dialog_icon(ui, icon)?;
        }

        ui.label(&dialog.content.message);

        if let Some(details) = &dialog.content.details {
            ui.separator();
            ui.label(details);
        }

        if let Some(progress) = &dialog.content.progress {
            ui.separator();
            self.render_dialog_progress(ui, progress)?;
        }

        if let Some(form) = &dialog.content.form {
            ui.separator();
            self.render_dialog_form(ui, form)?;
        }

        if let Some(preview) = &dialog.content.preview {
            ui.separator();
            self.render_dialog_preview(ui, preview)?;
        }

        Ok(())
    }

    fn render_dialog_icon(&self, ui: &mut egui::Ui, icon: DialogIcon) -> Result<()> {
        let icon_text = match icon {
            DialogIcon::Info => "ℹ️",
            DialogIcon::Warning => "⚠️",
            DialogIcon::Error => "❌",
            DialogIcon::Success => "✅",
            DialogIcon::Question => "❓",
            DialogIcon::Custom => "🔧",
        };
        
        ui.heading(icon_text);
        Ok(())
    }

    fn render_dialog_progress(&self, ui: &mut egui::Ui, progress: &DialogProgress) -> Result<()> {
        let mut progress_bar = egui::ProgressBar::new(progress.value)
            .range(progress.min..=progress.max);

        if progress.indeterminate {
            progress_bar = progress_bar.show_percentage();
        }

        if let Some(text) = &progress.text {
            progress_bar = progress_bar.text(text);
        }

        ui.add(progress_bar);
        Ok(())
    }

    fn render_dialog_form(&self, ui: &mut egui::Ui, form: &DialogForm) -> Result<()> {
        for field in &form.fields {
            self.render_form_field(ui, field)?;
        }
        Ok(())
    }

    fn render_form_field(&self, ui: &mut egui::Ui, field: &FormField) -> Result<()> {
        ui.label(&field.label);

        match field.field_type {
            FieldType::Text => {
                let placeholder = field.placeholder.as_deref().unwrap_or("");
                let value = field.value.as_ref().and_then(|v| v.as_str()).unwrap_or("");
                ui.text_edit_singleline(&mut value.to_string())
                    .hint_text(placeholder);
            }
            FieldType::TextArea => {
                let placeholder = field.placeholder.as_deref().unwrap_or("");
                let value = field.value.as_ref().and_then(|v| v.as_str()).unwrap_or("");
                ui.text_edit_multiline(&mut value.to_string())
                    .hint_text(placeholder);
            }
            FieldType::Number => {
                let value = field.value.as_ref().and_then(|v| v.as_f64()).unwrap_or(0.0);
                ui.add(egui::DragValue::new(&mut value.to_owned()));
            }
            FieldType::Email => {
                let placeholder = field.placeholder.as_deref().unwrap_or("email@example.com");
                let value = field.value.as_ref().and_then(|v| v.as_str()).unwrap_or("");
                ui.text_edit_singleline(&mut value.to_string())
                    .hint_text(placeholder);
            }
            FieldType::Password => {
                let placeholder = field.placeholder.as_deref().unwrap_or("password");
                let value = field.value.as_ref().and_then(|v| v.as_str()).unwrap_or("");
                ui.text_edit_singleline(&mut value.to_string())
                    .password(true)
                    .hint_text(placeholder);
            }
            FieldType::Checkbox => {
                let value = field.value.as_ref().and_then(|v| v.as_bool()).unwrap_or(false);
                ui.checkbox(&mut value.to_owned(), "");
            }
            FieldType::Radio => {
                let value = field.value.as_ref().and_then(|v| v.as_bool()).unwrap_or(false);
                ui.radio(&mut value.to_owned(), &field.label);
            }
            FieldType::Select => {
                let selected = field.value.as_ref().and_then(|v| v.as_str()).unwrap_or("");
                egui::ComboBox::from_label(&field.label)
                    .selected_text(selected)
                    .show_ui(ui, |ui| {
                        for option in &field.options {
                            ui.selectable_label(&option.label, option.value == selected);
                        }
                    });
            }
            FieldType::MultiSelect => {
                ui.label("Multi-select field");
            }
            FieldType::File => {
                ui.button("Choose File");
            }
            FieldType::Directory => {
                ui.button("Choose Directory");
            }
            FieldType::Color => {
                ui.button("Choose Color");
            }
            FieldType::Date => {
                ui.button("Choose Date");
            }
            FieldType::Time => {
                ui.button("Choose Time");
            }
            FieldType::DateTime => {
                ui.button("Choose Date & Time");
            }
            FieldType::Range => {
                let value = field.value.as_ref().and_then(|v| v.as_f64()).unwrap_or(0.0);
                ui.add(egui::Slider::new(&mut value.to_owned(), 0.0..=100.0));
            }
            FieldType::Custom => {
                ui.label("Custom field");
            }
        }

        Ok(())
    }

    fn render_dialog_preview(&self, ui: &mut egui::Ui, preview: &DialogPreview) -> Result<()> {
        match preview.preview_type {
            PreviewType::Image => {
                ui.label("Image Preview");
            }
            PreviewType::Video => {
                ui.label("Video Preview");
            }
            PreviewType::Audio => {
                ui.label("Audio Preview");
            }
            PreviewType::Text => {
                ui.label("Text Preview");
            }
            PreviewType::Code => {
                ui.label("Code Preview");
            }
            PreviewType::Custom => {
                ui.label("Custom Preview");
            }
        }

        Ok(())
    }

    fn render_dialog_buttons(&self, ui: &mut egui::Ui, dialog: &Dialog) -> Result<()> {
        ui.horizontal(|ui| {
            for button in &dialog.buttons {
                let mut button_widget = egui::Button::new(&button.label);
                
                match button.button_type {
                    ButtonType::Primary => {
                        button_widget = button_widget.fill(egui::Color32::from_rgb(0, 123, 255));
                    }
                    ButtonType::Secondary => {
                        button_widget = button_widget.fill(egui::Color32::from_rgb(108, 117, 125));
                    }
                    ButtonType::Success => {
                        button_widget = button_widget.fill(egui::Color32::from_rgb(40, 167, 69));
                    }
                    ButtonType::Warning => {
                        button_widget = button_widget.fill(egui::Color32::from_rgb(255, 193, 7));
                    }
                    ButtonType::Danger => {
                        button_widget = button_widget.fill(egui::Color32::from_rgb(220, 53, 69));
                    }
                    ButtonType::Info => {
                        button_widget = button_widget.fill(egui::Color32::from_rgb(23, 162, 184));
                    }
                    ButtonType::Custom => {
                    }
                }

                if button.disabled {
                    button_widget = button_widget.disabled();
                }

                if ui.add(button_widget).clicked() {
                    self.handle_button_click(dialog.id, &button);
                }
            }
        });

        Ok(())
    }

    fn handle_button_click(&self, dialog_id: Uuid, button: &DialogButton) {
        match button.action.action_type {
            ActionType::Close => {
                let result = DialogResult {
                    dialog_id,
                    button_id: Some(button.id.clone()),
                    data: None,
                    success: true,
                    error: None,
                    timestamp: Utc::now(),
                };
                
            }
            ActionType::Submit => {
            }
            ActionType::Cancel => {
                let result = DialogResult {
                    dialog_id,
                    button_id: Some(button.id.clone()),
                    data: None,
                    success: false,
                    error: None,
                    timestamp: Utc::now(),
                };
                
            }
            ActionType::Reset => {
            }
            ActionType::Custom => {
            }
        }
    }

    pub fn create_info_dialog(&mut self, title: String, message: String, details: Option<String>) -> Result<Uuid> {
        let content = DialogContent {
            message,
            details,
            icon: Some(DialogIcon::Info),
            progress: None,
            form: None,
            preview: None,
        };

        let buttons = vec![
            DialogButton {
                id: "ok".to_string(),
                label: "OK".to_string(),
                button_type: ButtonType::Primary,
                default: true,
                cancel: false,
                disabled: false,
                action: DialogAction {
                    action_type: ActionType::Close,
                    target: "".to_string(),
                    parameters: HashMap::new(),
                },
            },
        ];

        let config = DialogConfig {
            modal: true,
            resizable: false,
            movable: false,
            minimizable: false,
            maximizable: false,
            closable: true,
            auto_close: false,
            auto_close_delay: 0,
            show_title_bar: true,
            show_status_bar: false,
            show_progress_bar: false,
            escape_key_closes: true,
            click_outside_closes: true,
        };

        self.create_dialog("info".to_string(), title, DialogType::Info, content, buttons, config)
    }

    pub fn create_warning_dialog(&mut self, title: String, message: String, details: Option<String>) -> Result<Uuid> {
        let content = DialogContent {
            message,
            details,
            icon: Some(DialogIcon::Warning),
            progress: None,
            form: None,
            preview: None,
        };

        let buttons = vec![
            DialogButton {
                id: "ok".to_string(),
                label: "OK".to_string(),
                button_type: ButtonType::Warning,
                default: true,
                cancel: false,
                disabled: false,
                action: DialogAction {
                    action_type: ActionType::Close,
                    target: "".to_string(),
                    parameters: HashMap::new(),
                },
            },
        ];

        let config = DialogConfig {
            modal: true,
            resizable: false,
            movable: false,
            minimizable: false,
            maximizable: false,
            closable: true,
            auto_close: false,
            auto_close_delay: 0,
            show_title_bar: true,
            show_status_bar: false,
            show_progress_bar: false,
            escape_key_closes: true,
            click_outside_closes: true,
        };

        self.create_dialog("warning".to_string(), title, DialogType::Warning, content, buttons, config)
    }

    pub fn create_error_dialog(&mut self, title: String, message: String, details: Option<String>) -> Result<Uuid> {
        let content = DialogContent {
            message,
            details,
            icon: Some(DialogIcon::Error),
            progress: None,
            form: None,
            preview: None,
        };

        let buttons = vec![
            DialogButton {
                id: "ok".to_string(),
                label: "OK".to_string(),
                button_type: ButtonType::Danger,
                default: true,
                cancel: false,
                disabled: false,
                action: DialogAction {
                    action_type: ActionType::Close,
                    target: "".to_string(),
                    parameters: HashMap::new(),
                },
            },
        ];

        let config = DialogConfig {
            modal: true,
            resizable: false,
            movable: false,
            minimizable: false,
            maximizable: false,
            closable: true,
            auto_close: false,
            auto_close_delay: 0,
            show_title_bar: true,
            show_status_bar: false,
            show_progress_bar: false,
            escape_key_closes: true,
            click_outside_closes: true,
        };

        self.create_dialog("error".to_string(), title, DialogType::Error, content, buttons, config)
    }

    pub fn create_confirm_dialog(&mut self, title: String, message: String, details: Option<String>) -> Result<Uuid> {
        let content = DialogContent {
            message,
            details,
            icon: Some(DialogIcon::Question),
            progress: None,
            form: None,
            preview: None,
        };

        let buttons = vec![
            DialogButton {
                id: "yes".to_string(),
                label: "Yes".to_string(),
                button_type: ButtonType::Primary,
                default: true,
                cancel: false,
                disabled: false,
                action: DialogAction {
                    action_type: ActionType::Close,
                    target: "".to_string(),
                    parameters: HashMap::new(),
                },
            },
            DialogButton {
                id: "no".to_string(),
                label: "No".to_string(),
                button_type: ButtonType::Secondary,
                default: false,
                cancel: true,
                disabled: false,
                action: DialogAction {
                    action_type: ActionType::Cancel,
                    target: "".to_string(),
                    parameters: HashMap::new(),
                },
            },
        ];

        let config = DialogConfig {
            modal: true,
            resizable: false,
            movable: false,
            minimizable: false,
            maximizable: false,
            closable: true,
            auto_close: false,
            auto_close_delay: 0,
            show_title_bar: true,
            show_status_bar: false,
            show_progress_bar: false,
            escape_key_closes: true,
            click_outside_closes: false,
        };

        self.create_dialog("confirm".to_string(), title, DialogType::Confirm, content, buttons, config)
    }

    pub fn create_progress_dialog(&mut self, title: String, message: String, progress: DialogProgress) -> Result<Uuid> {
        let content = DialogContent {
            message,
            details: None,
            icon: None,
            progress: Some(progress),
            form: None,
            preview: None,
        };

        let buttons = vec![
            DialogButton {
                id: "cancel".to_string(),
                label: "Cancel".to_string(),
                button_type: ButtonType::Secondary,
                default: false,
                cancel: true,
                disabled: false,
                action: DialogAction {
                    action_type: ActionType::Cancel,
                    target: "".to_string(),
                    parameters: HashMap::new(),
                },
            },
        ];

        let config = DialogConfig {
            modal: true,
            resizable: false,
            movable: false,
            minimizable: false,
            maximizable: false,
            closable: false,
            auto_close: false,
            auto_close_delay: 0,
            show_title_bar: true,
            show_status_bar: false,
            show_progress_bar: true,
            escape_key_closes: false,
            click_outside_closes: false,
        };

        self.create_dialog("progress".to_string(), title, DialogType::Progress, content, buttons, config)
    }

    pub fn create_about_dialog(&mut self, title: String, message: String) -> Result<Uuid> {
        let content = DialogContent {
            message,
            details: None,
            icon: Some(DialogIcon::Info),
            progress: None,
            form: None,
            preview: None,
        };

        let buttons = vec![
            DialogButton {
                id: "ok".to_string(),
                label: "OK".to_string(),
                button_type: ButtonType::Primary,
                default: true,
                cancel: false,
                disabled: false,
                action: DialogAction {
                    action_type: ActionType::Close,
                    target: "".to_string(),
                    parameters: HashMap::new(),
                },
            },
        ];

        let config = DialogConfig {
            modal: true,
            resizable: false,
            movable: false,
            minimizable: false,
            maximizable: false,
            closable: true,
            auto_close: false,
            auto_close_delay: 0,
            show_title_bar: true,
            show_status_bar: false,
            show_progress_bar: false,
            escape_key_closes: true,
            click_outside_closes: true,
        };

        self.create_dialog("about".to_string(), title, DialogType::About, content, buttons, config)
    }

    pub fn clone(&self) -> DialogManager {
        DialogManager {
            dialogs: self.dialogs.clone(),
            active_dialogs: self.active_dialogs.clone(),
            dialog_history: self.dialog_history.clone(),
            config: self.config.clone(),
        }
    }
}

impl DialogPosition {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            anchor: DialogAnchor::Center,
            offset: egui::vec2(0.0, 0.0),
        }
    }

    pub fn clone(&self) -> DialogPosition {
        DialogPosition {
            x: self.x,
            y: self.y,
            anchor: self.anchor,
            offset: self.offset,
        }
    }
}

impl DialogSize {
    pub fn new() -> Self {
        Self {
            width: DialogSizeValue::Content,
            height: DialogSizeValue::Content,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            resizable: false,
        }
    }

    pub fn clone(&self) -> DialogSize {
        DialogSize {
            width: self.width.clone(),
            height: self.height.clone(),
            min_width: self.min_width,
            min_height: self.min_height,
            max_width: self.max_width,
            max_height: self.max_height,
            resizable: self.resizable,
        }
    }
}

impl DialogSizeValue {
    pub fn to_f32(&self) -> f32 {
        match self {
            DialogSizeValue::Fixed(value) => *value,
            DialogSizeValue::Percent(value) => 400.0 * value,
            DialogSizeValue::Auto => 400.0,
            DialogSizeValue::Content => 400.0,
            DialogSizeValue::MaxContent => 400.0,
        }
    }

    pub fn clone(&self) -> DialogSizeValue {
        match self {
            DialogSizeValue::Fixed(value) => DialogSizeValue::Fixed(*value),
            DialogSizeValue::Percent(value) => DialogSizeValue::Percent(*value),
            DialogSizeValue::Auto => DialogSizeValue::Auto,
            DialogSizeValue::Content => DialogSizeValue::Content,
            DialogSizeValue::MaxContent => DialogSizeValue::MaxContent,
        }
    }
}

impl Default fn default() -> Self {
        Self {
            modal: true,
            resizable: false,
            movable: false,
            minimizable: false,
            maximizable: false,
            closable: true,
            auto_close: false,
            auto_close_delay: 0,
            show_title_bar: true,
            show_status_bar: false,
            show_progress_bar: false,
            escape_key_closes: true,
            click_outside_closes: true,
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            shadow: egui::Color32::from_rgba(0, 0, 0, 128),
            title: egui::Color32::from_rgb(33, 37, 41),
            title_background: egui::Color32::from_rgb(233, 236, 239),
            content_background: egui::Color32::from_rgb(255, 255, 255),
            button_primary: egui::Color32::from_rgb(0, 123, 255),
            button_primary_text: egui::Color32::from_rgb(255, 255, 255),
            button_secondary: egui::Color32::from_rgb(108, 117, 125),
            button_secondary_text: egui::Color32::from_rgb(255, 255, 255),
            success: egui::Color32::from_rgb(40, 167, 69),
            warning: egui::Color32::from_rgb(255, 193, 7),
            error: egui::Color32::from_rgb(220, 53, 69),
            info: egui::Color32::from_rgb(23, 162, 184),
        }
}

impl Default fn default() -> Self {
        Self {
            title: egui::FontId::default(),
            body: egui::FontId::default(),
            button: egui::FontId::default(),
            caption: egui::FontId::default(),
            monospace: egui::FontId::monospace(),
        }
}

impl Default fn default() -> Self {
        Self {
            min_width: 300.0,
            min_height: 200.0,
            default_width: 400.0,
            default_height: 300.0,
            max_width: 800.0,
            max_height: 600.0,
            button_height: 32.0,
            button_min_width: 80.0,
            icon_size: 16.0,
        }
}

impl Default fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
        }
}

impl Default fn default() -> Self {
        Self {
            duration: 0.2,
            easing: EasingFunction::EaseInOut,
            enabled: true,
        }
}

impl Default fn default() -> Self {
        Self {
            max_dialogs: 10,
            max_history: 100,
            auto_close_delay: 5000,
            enable_animations: true,
            animation_speed: 1.0,
            modal_background: egui::Color32::from_rgba(0, 0, 0, 128),
        }
}

pub fn create_dialog_manager(config: DialogManagerConfig) -> DialogManager {
    DialogManager::new(config)
}

pub fn create_dialog_manager_config() -> DialogManagerConfig {
    DialogManagerConfig::default()
}
