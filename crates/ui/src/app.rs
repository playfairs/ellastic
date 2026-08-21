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

use crate::{
    UIManager, UIManagerConfig, ThemeManager, IconManager, EventBus, StateManager,
    UIState, AppConfig, PanelType, DialogType, Selection, Clipboard, History,
    Notification, Shortcut, Theme, IconSet, Event, State,
};

#[derive(Debug, Clone)]
pub struct ElasticApp {
    pub project_manager: Arc<RwLock<ProjectManager>>,
    pub export_manager: Arc<RwLock<ExportManager>>,
    pub current_project: Arc<RwLock<Option<Project>>>,
    pub ui_state: Arc<RwLock<UIState>>,
    pub app_config: AppConfig,
    pub running: bool,
    pub title: String,
    pub window_state: WindowState,
    pub performance_stats: PerformanceStats,
}

#[derive(Debug, Clone)]
pub struct WindowState {
    pub size: egui::Vec2,
    pub position: egui::Pos2,
    pub maximized: bool,
    pub fullscreen: bool,
    pub focused: bool,
    pub dpi_scale: f32,
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub fps: f64,
    pub frame_time: f64,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub render_time: f64,
    pub update_time: f64,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AppEvent {
    pub id: Uuid,
    pub event_type: AppEventType,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEventType {
    ProjectCreated,
    ProjectOpened,
    ProjectSaved,
    ProjectClosed,
    MediaAdded,
    MediaRemoved,
    EffectApplied,
    ExportStarted,
    ExportCompleted,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct AppCommand {
    pub id: Uuid,
    pub command_type: AppCommandType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCommandType {
    NewProject,
    OpenProject,
    SaveProject,
    CloseProject,
    AddMedia,
    RemoveMedia,
    ApplyEffect,
    Export,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Quit,
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub ui: UISettings,
    pub performance: PerformanceSettings,
    pub shortcuts: ShortcutSettings,
    pub themes: ThemeSettings,
    pub plugins: PluginSettings,
}

#[derive(Debug, Clone)]
pub struct GeneralSettings {
    pub language: String,
    pub auto_save: bool,
    pub auto_save_interval: u64,
    pub backup_enabled: bool,
    pub backup_count: u32,
    pub temp_directory: String,
    pub recent_projects: Vec<String>,
    pub max_recent_projects: usize,
}

#[derive(Debug, Clone)]
pub struct UISettings {
    pub theme: String,
    pub font_size: f32,
    pub ui_scale: f32,
    pub animations_enabled: bool,
    pub tooltips_enabled: bool,
    pub shortcuts_enabled: bool,
    pub show_menubar: bool,
    pub show_toolbar: bool,
    pub show_statusbar: bool,
    pub show_sidebar: bool,
    pub show_bottom_panel: bool,
}

#[derive(Debug, Clone)]
pub struct PerformanceSettings {
    pub vsync: bool,
    pub multisampling: u16,
    pub max_fps: Option<u32>,
    pub render_backend: RenderBackend,
    pub gpu_acceleration: bool,
    pub max_memory_mb: Option<usize>,
    pub thread_count: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    Auto,
    OpenGL,
    Vulkan,
    Metal,
    DirectX11,
    DirectX12,
    WebGPU,
}

#[derive(Debug, Clone)]
pub struct ShortcutSettings {
    pub shortcuts: HashMap<String, Shortcut>,
    pub enabled: bool,
    pub show_in_tooltips: bool,
}

#[derive(Debug, Clone)]
pub struct ThemeSettings {
    pub current_theme: String,
    pub custom_themes: Vec<Theme>,
    pub auto_switch: bool,
    pub follow_system: bool,
}

#[derive(Debug, Clone)]
pub struct PluginSettings {
    pub enabled: bool,
    pub auto_load: bool,
    pub plugin_directories: Vec<String>,
    pub loaded_plugins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AppContext {
    pub ctx: egui::Context,
    pub frame: eframe::Frame,
    pub delta_time: f32,
    pub screen_rect: egui::Rect,
    pub available_rect: egui::Rect,
    pub pointer_pos: Option<egui::Pos2>,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub struct AppResources {
    pub textures: HashMap<String, egui::TextureHandle>,
    pub fonts: HashMap<String, egui::FontId>,
    pub icons: HashMap<String, IconSet>,
    pub themes: HashMap<String, Theme>,
    pub sounds: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct AppPlugins {
    pub plugins: HashMap<String, AppPlugin>,
    pub load_order: Vec<String>,
    pub enabled_plugins: HashMap<String, bool>,
}

#[derive(Debug, Clone)]
pub struct AppPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub loaded: bool,
    pub api_version: String,
    pub dependencies: Vec<String>,
    pub provides: Vec<String>,
    pub commands: Vec<AppCommand>,
    pub settings: HashMap<String, serde_json::Value>,
}

impl ElasticApp {
    pub fn new() -> Self {
        Self {
            project_manager: Arc::new(RwLock::new(ProjectManager::new(ellastic_project::ProjectManagerConfig::new()).unwrap()),
            export_manager: Arc::new(RwLock::new(ExportManager::new(ellastic_export::ExportManagerConfig::new()).unwrap(),
            current_project: Arc::new(RwLock::new(None)),
            ui_state: Arc::new(RwLock::new(UIState::new()),
            app_config: AppConfig::new(),
            running: true,
            title: "Ellastic".to_string(),
            window_state: WindowState::new(),
            performance_stats: PerformanceStats::new(),
        }
    }

    pub fn new_with_config(config: AppConfig) -> Self {
        let mut app = Self::new();
        app.app_config = config;
        app
    }

    pub fn initialize(&mut self, ctx: &egui::Context) -> Result<()> {
        let mut ui_state = self.ui_state.write();
        ui_state.initialize()?;

        self.setup_fonts(ctx)?;

        self.setup_textures(ctx)?;

        self.load_settings()?;

        self.initialize_plugins()?;

        self.setup_event_handlers(ctx)?;

        Ok(())
    }

    pub fn update(&mut self, ctx: &mut egui::Context, frame: &mut eframe::Frame) {
        if !self.running {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.update_performance_stats();

        let app_context = self.create_app_context(ctx, frame);

        self.process_events(&app_context);

        self.update_state(&app_context);

        self.render_ui(&app_context);

        self.update_window_state(ctx);

        self.handle_auto_save();

        self.update_title(ctx);
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.save_settings()?;

        self.save_current_project()?;

        self.cleanup_resources()?;

        self.shutdown_plugins()?;

        Ok(())
    }

    fn setup_fonts(&mut self, ctx: &egui::Context) -> Result<()> {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "inter".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf")).into(),
        );

        fonts.font_data.insert(
            "inter_bold".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Bold.ttf")).into(),
        );

        fonts.font_data.insert(
            "jetbrains_mono".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")).into(),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_insert_with(|| egui::FontFamily::default())
            .insert(0, "inter".into());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_insert_with(|| egui::FontFamily::default())
            .insert(0, "jetbrains_mono".into());

        ctx.set_fonts(fonts);
        Ok(())
    }

    fn setup_textures(&mut self, ctx: &egui::Context) -> Result<()> {
        Ok(())
    }

    fn load_settings(&mut self) -> Result<()> {
        Ok(())
    }

    fn save_settings(&self) -> Result<()> {
        Ok(())
    }

    fn initialize_plugins(&mut self) -> Result<()> {
        Ok(())
    }

    fn shutdown_plugins(&mut self) -> Result<()> {
        Ok(())
    }

    fn setup_event_handlers(&mut self, ctx: &egui::Context) -> Result<()> {
        Ok(())
    }

    fn create_app_context(&self, ctx: &egui::Context, frame: &eframe::Frame) -> AppContext {
        AppContext {
            ctx: ctx.clone(),
            frame: frame.clone(),
            delta_time: frame.info().cpu_usage.unwrap_or(0.0),
            screen_rect: ctx.screen_rect(),
            available_rect: ctx.available_rect(),
            pointer_pos: ctx.pointer_hover_pos(),
            focused: ctx.has_focus(),
        }
    }

    fn process_events(&mut self, context: &AppContext) {
    }

    fn update_state(&mut self, context: &AppContext) {
    }

    fn render_ui(&mut self, context: &AppContext) {
        self.render_main_ui(context);

        self.render_dialogs(context);

        self.render_notifications(context);

        self.render_tooltips(context);
    }

    fn render_main_ui(&mut self, context: &AppContext) {
        let ctx = &context.ctx;
        let ui_state = self.ui_state.read();

        if ui_state.show_menubar {
            egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
                self.render_menubar(ui, context);
            });
        }

        if ui_state.show_toolbar {
            egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                self.render_toolbar(ui, context);
            });
        }

        if ui_state.show_sidebar {
            egui::SidePanel::left("sidebar", ui_state.sidebar_width, true).show(ctx, |ui| {
                self.render_sidebar(ui, context);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_content(ui, context);
        });

        if ui_state.show_bottom_panel {
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                self.render_bottom_panel(ui, context);
            });
        }

        if ui_state.show_statusbar {
            egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
                self.render_statusbar(ui, context);
            });
        }
    }

    fn render_menubar(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project").clicked() {
                    self.execute_command(AppCommandType::NewProject);
                }
                if ui.button("Open Project").clicked() {
                    self.execute_command(AppCommandType::OpenProject);
                }
                if ui.button("Save Project").clicked() {
                    self.execute_command(AppCommandType::SaveProject);
                }
                if ui.button("Close Project").clicked() {
                    self.execute_command(AppCommandType::CloseProject);
                }
                ui.separator();
                if ui.button("Export").clicked() {
                    self.execute_command(AppCommandType::Export);
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    self.execute_command(AppCommandType::Quit);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    self.execute_command(AppCommandType::Undo);
                }
                if ui.button("Redo").clicked() {
                    self.execute_command(AppCommandType::Redo);
                }
                ui.separator();
                if ui.button("Cut").clicked() {
                    self.execute_command(AppCommandType::Cut);
                }
                if ui.button("Copy").clicked() {
                    self.execute_command(AppCommandType::Copy);
                }
                if ui.button("Paste").clicked() {
                    self.execute_command(AppCommandType::Paste);
                }
            });

            ui.menu_button("View", |ui| {
                if ui.checkbox("Toolbar", &mut self.ui_state.write().show_toolbar).clicked() {}
                if ui.checkbox("Status Bar", &mut self.ui_state.write().show_statusbar).clicked() {}
                if ui.checkbox("Sidebar", &mut self.ui_state.write().show_sidebar).clicked() {}
                if ui.checkbox("Bottom Panel", &mut self.ui_state.write().show_bottom_panel).clicked() {}
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Settings").clicked() {
                    self.show_dialog(DialogType::Settings);
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    self.show_dialog(DialogType::About);
                }
            });
        });
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.horizontal(|ui| {
            if ui.button("📁 Open").clicked() {
                self.execute_command(AppCommandType::OpenProject);
            }
            if ui.button("💾 Save").clicked() {
                self.execute_command(AppCommandType::SaveProject);
            }
            if ui.button("📤 Export").clicked() {
                self.execute_command(AppCommandType::Export);
            }
            ui.separator();
            if ui.button("↶ Undo").clicked() {
                self.execute_command(AppCommandType::Undo);
            }
            if ui.button("↷ Redo").clicked() {
                self.execute_command(AppCommandType::Redo);
            }
            ui.separator();
            if ui.button("🔍 Search").clicked() {
            }
        });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.vertical(|ui| {
            ui.heading("Panels");
            ui.separator();

            let ui_state = self.ui_state.read();
            let current_panel = ui_state.current_panel;

            if ui.button("Project").selected(current_panel == PanelType::Project).clicked() {
                self.set_current_panel(PanelType::Project);
            }
            if ui.button("Media").selected(current_panel == PanelType::Media).clicked() {
                self.set_current_panel(PanelType::Media);
            }
            if ui.button("Effects").selected(current_panel == PanelType::Effects).clicked() {
                self.set_current_panel(PanelType::Effects);
            }
            if ui.button("Pipeline").selected(current_panel == PanelType::Pipeline).clicked() {
                self.set_current_panel(PanelType::Pipeline);
            }
            if ui.button("Export").selected(current_panel == PanelType::Export).clicked() {
                self.set_current_panel(PanelType::Export);
            }
            if ui.button("Settings").selected(current_panel == PanelType::Settings).clicked() {
                self.set_current_panel(PanelType::Settings);
            }
        });
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        let ui_state = self.ui_state.read();

        match ui_state.current_panel {
            PanelType::Project => self.render_project_panel(ui, context),
            PanelType::Media => self.render_media_panel(ui, context),
            PanelType::Effects => self.render_effects_panel(ui, context),
            PanelType::Pipeline => self.render_pipeline_panel(ui, context),
            PanelType::Export => self.render_export_panel(ui, context),
            PanelType::Settings => self.render_settings_panel(ui, context),
            PanelType::Help => self.render_help_panel(ui, context),
        }
    }

    fn render_bottom_panel(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.horizontal(|ui| {
            ui.label("Bottom Panel");
        });
    }

    fn render_statusbar(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.horizontal(|ui| {
            ui.label("Ready");
            ui.separator();
            ui.label(format!("FPS: {:.1}", self.performance_stats.fps));
            ui.separator();
            ui.label(format!("Zoom: {:.0}%", self.ui_state.read().zoom_level * 100.0));
            ui.separator();
            ui.label(format!("Position: ({:.0}, {:.0})", self.ui_state.read().pan_offset.0, self.ui_state.read().pan_offset.1));
        });
    }

    fn render_dialogs(&mut self, context: &AppContext) {
        let ui_state = self.ui_state.read();

        if let Some(dialog_type) = ui_state.active_dialog {
            self.render_dialog(context, dialog_type);
        }
    }

    fn render_dialog(&mut self, context: &AppContext, dialog_type: DialogType) {
        let ctx = &context.ctx;

        match dialog_type {
            DialogType::NewProject => self.render_new_project_dialog(ctx),
            DialogType::OpenProject => self.render_open_project_dialog(ctx),
            DialogType::SaveProject => self.render_save_project_dialog(ctx),
            DialogType::Export => self.render_export_dialog(ctx),
            DialogType::Settings => self.render_settings_dialog(ctx),
            DialogType::About => self.render_about_dialog(ctx),
            DialogType::Confirm => self.render_confirm_dialog(ctx),
            DialogType::Error => self.render_error_dialog(ctx),
            DialogType::Progress => self.render_progress_dialog(ctx),
        }
    }

    fn render_notifications(&mut self, context: &AppContext) {
        let ctx = &context.ctx;
        let ui_state = self.ui_state.read();

        for notification in &ui_state.notifications {
            egui::Window::new(&notification.title)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&notification.message);

                    for action in &notification.actions {
                        if ui.button(&action.label).clicked() {
                        }
                    }
                });
        }
    }

    fn render_tooltips(&mut self, context: &AppContext) {
        let ctx = &context.ctx;
        let ui_state = self.ui_state.read();

        let mouse_pos = context.pointer_pos;
        if let Some(pos) = mouse_pos {
            for (widget_id, tooltip) in &ui_state.tooltips {
                egui::show_tooltip_at(ctx, pos, egui::Id::new(widget_id), |ui| {
                    ui.label(tooltip);
                });
            }
        }
    }

    fn render_project_panel(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.heading("Project");
        ui.label("Project panel content here");
    }

    fn render_media_panel(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.heading("Media");
        ui.label("Media panel content here");
    }

    fn render_effects_panel(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.heading("Effects");
        ui.label("Effects panel content here");
    }

    fn render_pipeline_panel(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.heading("Pipeline");
        ui.label("Pipeline panel content here");
    }

    fn render_export_panel(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.heading("Export");
        ui.label("Export panel content here");
    }

    fn render_settings_panel(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.heading("Settings");
        ui.label("Settings panel content here");
    }

    fn render_help_panel(&mut self, ui: &mut egui::Ui, context: &AppContext) {
        ui.heading("Help");
        ui.label("Help panel content here");
    }

    fn render_new_project_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("New Project")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Create new project");
                if ui.button("Create").clicked() {
                    self.hide_dialog();
                }
                if ui.button("Cancel").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn render_open_project_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Open Project")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Open existing project");
                if ui.button("Open").clicked() {
                    self.hide_dialog();
                }
                if ui.button("Cancel").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn render_save_project_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Save Project")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Save project");
                if ui.button("Save").clicked() {
                    self.hide_dialog();
                }
                if ui.button("Cancel").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn render_export_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Export")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Export project");
                if ui.button("Export").clicked() {
                    self.hide_dialog();
                }
                if ui.button("Cancel").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Application settings");
                if ui.button("Save").clicked() {
                    self.hide_dialog();
                }
                if ui.button("Cancel").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("About Ellastic")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Ellastic v0.1.0");
                ui.label("Multimedia databending toolkit");
                ui.hyperlink_to("https://ellastic.dev", "https://ellastic.dev");
                if ui.button("OK").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn render_confirm_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Confirm")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Are you sure?");
                if ui.button("Yes").clicked() {
                    self.hide_dialog();
                }
                if ui.button("No").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn render_error_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("An error occurred");
                if ui.button("OK").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn render_progress_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Progress")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Processing...");
                ui.spinner();
                if ui.button("Cancel").clicked() {
                    self.hide_dialog();
                }
            });
    }

    fn update_window_state(&mut self, ctx: &egui::Context) {
        let viewport = ctx.viewport();
        if let Some(viewport) = viewport {
            self.window_state.size = egui::vec2(viewport.inner_rect.width(), viewport.inner_rect.height());
            self.window_state.position = egui::pos2(viewport.inner_rect.min.x, viewport.inner_rect.min.y);
            self.window_state.focused = ctx.has_focus();
            self.window_state.dpi_scale = ctx.pixels_per_point();
        }
    }

    fn update_performance_stats(&mut self) {
        self.performance_stats.last_update = Utc::now();
    }

    fn handle_auto_save(&mut self) {
        if self.app_config.auto_save {
        }
    }

    fn save_current_project(&mut self) -> Result<()> {
        Ok(())
    }

    fn cleanup_resources(&mut self) -> Result<()> {
        Ok(())
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = if let Some(project) = self.current_project.read().as_ref() {
            format!("{} - {}", project.name, self.title)
        } else {
            self.title.clone()
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::Title { title });
    }

    pub fn execute_command(&mut self, command_type: AppCommandType) {
        let command = AppCommand {
            id: Uuid::new_v4(),
            command_type,
            parameters: HashMap::new(),
            timestamp: Utc::now(),
        };

        self.handle_command(command);
    }

    fn handle_command(&mut self, command: AppCommand) {
        match command.command_type {
            AppCommandType::NewProject => self.new_project(),
            AppCommandType::OpenProject => self.open_project(),
            AppCommandType::SaveProject => self.save_project(),
            AppCommandType::CloseProject => self.close_project(),
            AppCommandType::AddMedia => self.add_media(),
            AppCommandType::RemoveMedia => self.remove_media(),
            AppCommandType::ApplyEffect => self.apply_effect(),
            AppCommandType::Export => self.export(),
            AppCommandType::Undo => self.undo(),
            AppCommandType::Redo => self.redo(),
            AppCommandType::Cut => self.cut(),
            AppCommandType::Copy => self.copy(),
            AppCommandType::Paste => self.paste(),
            AppCommandType::Quit => self.quit(),
        }
    }

    fn new_project(&mut self) {
        self.show_dialog(DialogType::NewProject);
    }

    fn open_project(&mut self) {
        self.show_dialog(DialogType::OpenProject);
    }

    fn save_project(&mut self) {
        self.show_dialog(DialogType::SaveProject);
    }

    fn close_project(&mut self) {
        self.show_dialog(DialogType::Confirm);
    }

    fn add_media(&mut self) {
    }

    fn remove_media(&mut self) {
    }

    fn apply_effect(&mut self) {
    }

    fn export(&mut self) {
        self.show_dialog(DialogType::Export);
    }

    fn undo(&mut self) {
    }

    fn redo(&mut self) {
    }

    fn cut(&mut self) {
    }

    fn copy(&mut self) {
    }

    fn paste(&mut self) {
    }

    fn quit(&mut self) {
        self.running = false;
    }

    pub fn show_dialog(&mut self, dialog_type: DialogType) {
        self.ui_state.write().active_dialog = Some(dialog_type);
    }

    pub fn hide_dialog(&mut self) {
        self.ui_state.write().active_dialog = None;
    }

    pub fn set_current_panel(&mut self, panel_type: PanelType) {
        self.ui_state.write().current_panel = panel_type;
    }

    pub fn add_notification(&mut self, notification: Notification) {
        self.ui_state.write().notifications.push(notification);
    }

    pub fn clone(&self) -> ElasticApp {
        ElasticApp {
            project_manager: self.project_manager.clone(),
            export_manager: self.export_manager.clone(),
            current_project: self.current_project.clone(),
            ui_state: self.ui_state.clone(),
            app_config: self.app_config.clone(),
            running: self.running,
            title: self.title.clone(),
            window_state: self.window_state.clone(),
            performance_stats: self.performance_stats.clone(),
        }
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            size: egui::vec2(1200.0, 800.0),
            position: egui::pos2(0.0, 0.0),
            maximized: false,
            fullscreen: false,
            focused: true,
            dpi_scale: 1.0,
        }
    }
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            fps: 60.0,
            frame_time: 16.67,
            cpu_usage: 0.0,
            memory_usage: 0,
            render_time: 0.0,
            update_time: 0.0,
            last_update: Utc::now(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            ui: UISettings::default(),
            performance: PerformanceSettings::default(),
            shortcuts: ShortcutSettings::default(),
            themes: ThemeSettings::default(),
            plugins: PluginSettings::default(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            auto_save: true,
            auto_save_interval: 300,
            backup_enabled: true,
            backup_count: 5,
            temp_directory: "./temp".to_string(),
            recent_projects: Vec::new(),
            max_recent_projects: 10,
        }
    }
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            font_size: 14.0,
            ui_scale: 1.0,
            animations_enabled: true,
            tooltips_enabled: true,
            shortcuts_enabled: true,
            show_menubar: true,
            show_toolbar: true,
            show_statusbar: true,
            show_sidebar: true,
            show_bottom_panel: false,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            multisampling: 4,
            max_fps: None,
            render_backend: RenderBackend::Auto,
            gpu_acceleration: true,
            max_memory_mb: None,
            thread_count: None,
        }
    }
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            shortcuts: HashMap::new(),
            enabled: true,
            show_in_tooltips: true,
        }
    }
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            current_theme: "default".to_string(),
            custom_themes: Vec::new(),
            auto_switch: false,
            follow_system: false,
        }
    }
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_load: false,
            plugin_directories: Vec::new(),
            loaded_plugins: Vec::new(),
        }
    }
}

impl Default for AppResources {
    fn default() -> Self {
        Self {
            textures: HashMap::new(),
            fonts: HashMap::new(),
            icons: HashMap::new(),
            themes: HashMap::new(),
            sounds: HashMap::new(),
        }
    }
}

impl Default for AppPlugins {
    fn default() -> Self {
        Self {
            plugins: HashMap::new(),
            load_order: Vec::new(),
            enabled_plugins: HashMap::new(),
        }
    }
}

pub fn create_elastic_app() -> ElasticApp {
    ElasticApp::new()
}

pub fn create_elastic_app_with_config(config: AppConfig) -> ElasticApp {
    ElasticApp::new_with_config(config)
}
