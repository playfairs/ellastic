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
pub struct ThemeManager {
    pub themes: Arc<RwLock<HashMap<String, Theme>>>,
    pub current_theme: Arc<RwLock<String>>,
    pub custom_themes: Arc<RwLock<HashMap<String, Theme>>>,
    pub config: ThemeManagerConfig,
}

#[derive(Debug, Clone)]
pub struct ThemeManagerConfig {
    pub default_theme: String,
    pub auto_switch: bool,
    pub follow_system: bool,
    pub enable_animations: bool,
    pub animation_speed: f32,
    pub cache_enabled: bool,
    pub cache_size: usize,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub sizes: ThemeSizes,
    pub spacing: ThemeSpacing,
    pub borders: ThemeBorders,
    pub shadows: ThemeShadows,
    pub animations: ThemeAnimations,
    pub transitions: ThemeTransitions,
    pub components: ComponentThemes,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub primary: egui::Color32,
    pub primary_variant: egui::Color32,
    pub secondary: egui::Color32,
    pub secondary_variant: egui::Color32,
    pub background: egui::Color32,
    pub surface: egui::Color32,
    pub surface_variant: egui::Color32,
    pub error: egui::Color32,
    pub warning: egui::Color32,
    pub success: egui::Color32,
    pub info: egui::Color32,
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
    pub text_disabled: egui::Color32,
    pub text_hint: egui::Color32,
    pub border: egui::Color32,
    pub divider: egui::Color32,
    pub outline: egui::Color32,
    pub shadow: egui::Color32,
    pub highlight: egui::Color32,
    pub accent: egui::Color32,
    pub overlay: egui::Color32,
    pub scrim: egui::Color32,
    pub inverse_surface: egui::Color32,
    pub inverse_on_surface: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct ThemeFonts {
    pub primary: ThemeFont,
    pub secondary: ThemeFont,
    pub monospace: ThemeFont,
    pub display: ThemeFont,
    pub headline: ThemeFont,
    pub title: ThemeFont,
    pub body: ThemeFont,
    pub caption: ThemeFont,
    pub label: ThemeFont,
    pub button: ThemeFont,
    pub input: ThemeFont,
}

#[derive(Debug, Clone)]
pub struct ThemeFont {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub color: egui::Color32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Custom(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone)]
pub struct ThemeSizes {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub xxxl: f32,
    pub button_height: f32,
    pub button_height_small: f32,
    pub button_height_large: f32,
    pub input_height: f32,
    pub input_height_small: f32,
    pub input_height_large: f32,
    pub icon_size: f32,
    pub icon_size_small: f32,
    pub icon_size_large: f32,
    pub avatar_size: f32,
    pub avatar_size_small: f32,
    pub avatar_size_large: f32,
    pub thumbnail_size: f32,
    pub preview_size: f32,
}

#[derive(Debug, Clone)]
pub struct ThemeSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub none: f32,
    pub tight: f32,
    pub normal: f32,
    pub wide: f32,
    pub extra_wide: f32,
}

#[derive(Debug, Clone)]
pub struct ThemeBorders {
    pub none: Border,
    pub thin: Border,
    pub normal: Border,
    pub thick: Border,
    pub rounded: Border,
    pub circle: Border,
}

#[derive(Debug, Clone)]
pub struct Border {
    pub width: f32,
    pub color: egui::Color32,
    pub style: BorderStyle,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Debug, Clone)]
pub struct ThemeShadows {
    pub none: Shadow,
    pub xs: Shadow,
    pub sm: Shadow,
    pub md: Shadow,
    pub lg: Shadow,
    pub xl: Shadow,
    pub xxl: Shadow,
}

#[derive(Debug, Clone)]
pub struct Shadow {
    pub color: egui::Color32,
    pub offset: egui::Vec2,
    pub blur: f32,
    pub spread: f32,
}

#[derive(Debug, Clone)]
pub struct ThemeAnimations {
    pub duration: f32,
    pub easing: EasingFunction,
    pub enabled: bool,
    pub fast: Animation,
    pub normal: Animation,
    pub slow: Animation,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub duration: f32,
    pub easing: EasingFunction,
    pub delay: f32,
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
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

#[derive(Debug, Clone)]
pub struct ThemeTransitions {
    pub none: Transition,
    pub fast: Transition,
    pub normal: Transition,
    pub slow: Transition,
    pub slide: Transition,
    pub fade: Transition,
    pub scale: Transition,
    pub rotate: Transition,
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub duration: f32,
    pub easing: EasingFunction,
    pub delay: f32,
    pub property: TransitionProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionProperty {
    All,
    Opacity,
    Transform,
    Color,
    Background,
    Border,
    Shadow,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ComponentThemes {
    pub button: ButtonTheme,
    pub input: InputTheme,
    pub checkbox: CheckboxTheme,
    pub radio: RadioTheme,
    pub slider: SliderTheme,
    pub switch: SwitchTheme,
    pub progress_bar: ProgressBarTheme,
    pub card: CardTheme,
    pub panel: PanelTheme,
    pub dialog: DialogTheme,
    pub menu: MenuTheme,
    pub toolbar: ToolbarTheme,
    pub status_bar: StatusBarTheme,
    pub tab_bar: TabBarTheme,
    pub table: TableTheme,
    pub list: ListTheme,
    pub tree: TreeTheme,
    pub accordion: AccordionTheme,
    pub carousel: CarouselTheme,
    pub tooltip: TooltipTheme,
    pub badge: BadgeTheme,
    pub avatar: AvatarTheme,
}

#[derive(Debug, Clone)]
pub struct ButtonTheme {
    pub primary: ButtonVariantTheme,
    pub secondary: ButtonVariantTheme,
    pub success: ButtonVariantTheme,
    pub warning: ButtonVariantTheme,
    pub danger: ButtonVariantTheme,
    pub info: ButtonVariantTheme,
    pub light: ButtonVariantTheme,
    pub dark: ButtonVariantTheme,
    pub link: ButtonVariantTheme,
    pub outline: ButtonVariantTheme,
    pub ghost: ButtonVariantTheme,
}

#[derive(Debug, Clone)]
pub struct ButtonVariantTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub hover_background: egui::Color32,
    pub hover_foreground: egui::Color32,
    pub hover_border: egui::Color32,
    pub active_background: egui::Color32,
    pub active_foreground: egui::Color32,
    pub active_border: egui::Color32,
    pub disabled_background: egui::Color32,
    pub disabled_foreground: egui::Color32,
    pub disabled_border: egui::Color32,
    pub shadow: Option<Shadow>,
    pub border_radius: f32,
    pub font: ThemeFont,
}

#[derive(Debug, Clone)]
pub struct InputTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub placeholder: egui::Color32,
    pub focus_background: egui::Color32,
    pub focus_border: egui::Color32,
    pub error_background: egui::Color32,
    pub error_border: egui::Color32,
    pub disabled_background: egui::Color32,
    pub disabled_foreground: egui::Color32,
    pub disabled_border: egui::Color32,
    pub border_radius: f32,
    pub border_width: f32,
    pub font: ThemeFont,
}

#[derive(Debug, Clone)]
pub struct CheckboxTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub check_color: egui::Color32,
    pub hover_background: egui::Color32,
    pub hover_border: egui::Color32,
    pub disabled_background: egui::Color32,
    pub disabled_foreground: egui::Color32,
    pub disabled_border: egui::Color32,
    pub border_radius: f32,
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct RadioTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub dot_color: egui::Color32,
    pub hover_background: egui::Color32,
    pub hover_border: egui::Color32,
    pub disabled_background: egui::Color32,
    pub disabled_foreground: egui::Color32,
    pub disabled_border: egui::Color32,
    pub border_radius: f32,
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct SliderTheme {
    pub track_background: egui::Color32,
    pub track_fill: egui::Color32,
    pub handle_background: egui::Color32,
    pub handle_border: egui::Color32,
    pub handle_hover_background: egui::Color32,
    pub handle_active_background: egui::Color32,
    pub disabled_track_background: egui::Color32,
    pub disabled_track_fill: egui::Color32,
    pub disabled_handle_background: egui::Color32,
    pub border_radius: f32,
    pub track_height: f32,
    pub handle_size: f32,
}

#[derive(Debug, Clone)]
pub struct SwitchTheme {
    pub track_background: egui::Color32,
    pub track_active_background: egui::Color32,
    pub thumb_background: egui::Color32,
    pub thumb_active_background: egui::Color32,
    pub thumb_hover_background: egui::Color32,
    pub disabled_track_background: egui::Color32,
    pub disabled_thumb_background: egui::Color32,
    pub border_radius: f32,
    pub track_width: f32,
    pub track_height: f32,
    pub thumb_size: f32,
}

#[derive(Debug, Clone)]
pub struct ProgressBarTheme {
    pub background: egui::Color32,
    pub fill: egui::Color32,
    pub text: egui::Color32,
    pub border: egui::Color32,
    pub border_radius: f32,
    pub height: f32,
    pub font: ThemeFont,
}

#[derive(Debug, Clone)]
pub struct CardTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub shadow: Shadow,
    pub border_radius: f32,
    pub padding: f32,
    pub header_background: egui::Color32,
    pub header_foreground: egui::Color32,
    pub header_border: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct PanelTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub shadow: Shadow,
    pub border_radius: f32,
    pub padding: f32,
    pub header_background: egui::Color32,
    pub header_foreground: egui::Color32,
    pub header_border: egui::Color32,
    pub header_height: f32,
}

#[derive(Debug, Clone)]
pub struct DialogTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub shadow: Shadow,
    pub border_radius: f32,
    pub padding: f32,
    pub header_background: egui::Color32,
    pub header_foreground: egui::Color32,
    pub header_border: egui::Color32,
    pub overlay_background: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct MenuTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub shadow: Shadow,
    pub border_radius: f32,
    pub padding: f32,
    pub item_background: egui::Color32,
    pub item_foreground: egui::Color32,
    pub item_hover_background: egui::Color32,
    pub item_hover_foreground: egui::Color32,
    pub item_active_background: egui::Color32,
    pub item_active_foreground: egui::Color32,
    pub separator: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct ToolbarTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub shadow: Shadow,
    pub border_radius: f32,
    pub padding: f32,
    pub button_background: egui::Color32,
    pub button_foreground: egui::Color32,
    pub button_hover_background: egui::Color32,
    pub button_hover_foreground: egui::Color32,
    pub button_active_background: egui::Color32,
    pub button_active_foreground: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct StatusBarTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub padding: f32,
    pub font: ThemeFont,
}

#[derive(Debug, Clone)]
pub struct TabBarTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub border_radius: f32,
    pub padding: f32,
    pub tab_background: egui::Color32,
    pub tab_foreground: egui::Color32,
    pub tab_hover_background: egui::Color32,
    pub tab_hover_foreground: egui::Color32,
    pub tab_active_background: egui::Color32,
    pub tab_active_foreground: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct TableTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub header_background: egui::Color32,
    pub header_foreground: egui::Color32,
    pub header_border: egui::Color32,
    pub row_background: egui::Color32,
    pub row_foreground: egui::Color32,
    pub row_hover_background: egui::Color32,
    pub row_hover_foreground: egui::Color32,
    pub row_selected_background: egui::Color32,
    pub row_selected_foreground: egui::Color32,
    pub border_radius: f32,
    pub padding: f32,
}

#[derive(Debug, Clone)]
pub struct ListTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub item_background: egui::Color32,
    pub item_foreground: egui::Color32,
    pub item_hover_background: egui::Color32,
    pub item_hover_foreground: egui::Color32,
    pub item_active_background: egui::Color32,
    pub item_active_foreground: egui::Color32,
    pub item_selected_background: egui::Color32,
    pub item_selected_foreground: egui::Color32,
    pub border_radius: f32,
    pub padding: f32,
}

#[derive(Debug, Clone)]
pub struct TreeTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub node_background: egui::Color32,
    pub node_foreground: egui::Color32,
    pub node_hover_background: egui::Color32,
    pub node_hover_foreground: egui::Color32,
    pub node_active_background: egui::Color32,
    pub node_active_foreground: egui::Color32,
    pub node_selected_background: egui::Color32,
    pub node_selected_foreground: egui::Color32,
    pub expand_icon_color: egui::Color32,
    pub border_radius: f32,
    pub padding: f32,
}

#[derive(Debug, Clone)]
pub struct AccordionTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub header_background: egui::Color32,
    pub header_foreground: egui::Color32,
    pub header_border: egui::Color32,
    pub header_hover_background: egui::Color32,
    pub header_hover_foreground: egui::Color32,
    pub content_background: egui::Color32,
    pub content_foreground: egui::Color32,
    pub border_radius: f32,
    pub padding: f32,
}

#[derive(Debug, Clone)]
pub struct CarouselTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub shadow: Shadow,
    pub border_radius: f32,
    pub padding: f32,
    pub control_background: egui::Color32,
    pub control_foreground: egui::Color32,
    pub control_hover_background: egui::Color32,
    pub control_hover_foreground: egui::Color32,
    pub indicator_background: egui::Color32,
    pub indicator_active_background: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct TooltipTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub shadow: Shadow,
    pub border_radius: f32,
    pub padding: f32,
    pub font: ThemeFont,
}

#[derive(Debug, Clone)]
pub struct BadgeTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub border_radius: f32,
    pub padding: f32,
    pub font: ThemeFont,
}

#[derive(Debug, Clone)]
pub struct AvatarTheme {
    pub background: egui::Color32,
    pub foreground: egui::Color32,
    pub border: egui::Color32,
    pub border_radius: f32,
    pub font: ThemeFont,
}

impl ThemeManager {
    pub fn new(config: ThemeManagerConfig) -> Self {
        Self {
            themes: Arc::new(RwLock::new(HashMap::new())),
            current_theme: Arc::new(RwLock::new(config.default_theme.clone())),
            custom_themes: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn load_default_themes(&mut self) -> Result<()> {
        self.themes.write().insert("default".to_string(), Theme::default());
        self.themes.write().insert("dark".to_string(), Theme::dark());
        self.themes.write().insert("light".to_string(), Theme::light());
        self.themes.write().insert("blue".to_string(), Theme::blue());
        self.themes.write().insert("green".to_string(), Theme::green());
        self.themes.write().insert("purple".to_string(), Theme::purple());
        self.themes.write().insert("orange".to_string(), Theme::orange());
        self.themes.write().insert("red".to_string(), Theme::red());
        Ok(())
    }

    pub fn set_current_theme(&mut self, theme_name: &str) -> Result<()> {
        if self.themes.read().contains_key(theme_name) || self.custom_themes.read().contains_key(theme_name) {
            *self.current_theme.write() = theme_name.to_string();
            Ok(())
        } else {
            Err(EllasticError::InvalidParameter(format!("Theme '{}' not found", theme_name)))
        }
    }

    pub fn get_current_theme(&self) -> Option<Theme> {
        let current_theme = self.current_theme.read();
        if let Some(theme) = self.themes.read().get(&*current_theme) {
            Some(theme.clone())
        } else if let Some(theme) = self.custom_themes.read().get(&*current_theme) {
            Some(theme.clone())
        } else {
            None
        }
    }

    pub fn get_theme(&self, theme_name: &str) -> Option<Theme> {
        if let Some(theme) = self.themes.read().get(theme_name) {
            Some(theme.clone())
        } else if let Some(theme) = self.custom_themes.read().get(theme_name) {
            Some(theme.clone())
        } else {
            None
        }
    }

    pub fn list_themes(&self) -> Vec<String> {
        let mut themes = Vec::new();
        themes.extend(self.themes.read().keys().cloned());
        themes.extend(self.custom_themes.read().keys().cloned());
        themes
    }

    pub fn create_custom_theme(&mut self, name: String, theme: Theme) -> Result<()> {
        self.custom_themes.write().insert(name, theme);
        Ok(())
    }

    pub fn update_custom_theme(&mut self, name: &str, theme: Theme) -> Result<()> {
        if self.custom_themes.read().contains_key(name) {
            self.custom_themes.write().insert(name.to_string(), theme);
            Ok(())
        } else {
            Err(EllasticError::InvalidParameter(format!("Custom theme '{}' not found", name)))
        }
    }

    pub fn delete_custom_theme(&mut self, name: &str) -> Result<()> {
        let mut custom_themes = self.custom_themes.write();
        if custom_themes.remove(name).is_some() {
            Ok(())
        } else {
            Err(EllasticError::InvalidParameter(format!("Custom theme '{}' not found", name)))
        }
    }

    pub fn export_theme(&self, theme_name: &str) -> Result<String> {
        let theme = self.get_theme(theme_name)
            .ok_or_else(|| EllasticError::InvalidParameter(format!("Theme '{}' not found", theme_name)))?;

        serde_json::to_string_pretty(&theme)
            .map_err(|e| EllasticError::SerializationError(format!("Failed to serialize theme: {}", e)))
    }

    pub fn import_theme(&mut self, theme_data: &str) -> Result<String> {
        let theme: Theme = serde_json::from_str(theme_data)
            .map_err(|e| EllasticError::SerializationError(format!("Failed to deserialize theme: {}", e)))?;

        let theme_name = theme.id.clone();
        self.create_custom_theme(theme_name.clone(), theme)?;
        Ok(theme_name)
    }

    pub fn apply_theme_to_context(&self, ctx: &mut egui::Context) -> Result<()> {
        let theme = self.get_current_theme()
            .ok_or_else(|| EllasticError::InvalidParameter("No current theme set".to_string()))?;

        let mut style = ctx.style().clone();

        style.visuals.window_fill = theme.colors.background;
        style.visuals.panel_fill = theme.colors.surface;
        style.visuals.noninteractive = theme.colors.text_primary;
        style.visuals.weak_text_color = theme.colors.text_secondary;
        style.visuals.strong_text_color = theme.colors.text_primary;
        style.visuals.text_cursor = theme.colors.primary;
        style.visuals.selection.bg_fill = theme.colors.highlight;
        style.visuals.selection.stroke = theme.colors.primary;
        style.visuals.hyperlink_color = theme.colors.primary;
        style.visuals.warn_fg_color = theme.colors.warning;
        style.visuals.error_fg_color = theme.colors.error;
        style.visuals.code_bg_color = theme.colors.surface_variant;

        style.text_styles = [
            (egui::TextStyle::Heading, theme.fonts.display),
            (egui::TextStyle::Body, theme.fonts.body),
            (egui::TextStyle::Monospace, theme.fonts.monospace),
            (egui::TextStyle::Button, theme.fonts.button),
        ]
        .into_iter()
        .collect();

        ctx.set_style(style);
        Ok(())
    }

    pub fn clone(&self) -> ThemeManager {
        ThemeManager {
            themes: self.themes.clone(),
            current_theme: self.current_theme.clone(),
            custom_themes: self.custom_themes.clone(),
            config: self.config.clone(),
        }
    }
}

impl Default for ThemeManagerConfig {
    fn default() -> Self {
        Self {
            default_theme: "default".to_string(),
            auto_switch: false,
            follow_system: false,
            enable_animations: true,
            animation_speed: 1.0,
            cache_enabled: true,
            cache_size: 100,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            description: "Default theme".to_string(),
            version: "1.0".to_string(),
            author: "Ellastic".to_string(),
            colors: ThemeColors::default(),
            fonts: ThemeFonts::default(),
            sizes: ThemeSizes::default(),
            spacing: ThemeSpacing::default(),
            borders: ThemeBorders::default(),
            shadows: ThemeShadows::default(),
            animations: ThemeAnimations::default(),
            transitions: ThemeTransitions::default(),
            components: ComponentThemes::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Theme {
    pub fn dark() -> Self {
        let mut theme = Theme::default();
        theme.id = "dark".to_string();
        theme.name = "Dark".to_string();
        theme.description = "Dark theme".to_string();
        theme.colors = ThemeColors::dark();
        theme
    }

    pub fn light() -> Self {
        let mut theme = Theme::default();
        theme.id = "light".to_string();
        theme.name = "Light".to_string();
        theme.description = "Light theme".to_string();
        theme.colors = ThemeColors::light();
        theme
    }

    pub fn blue() -> Self {
        let mut theme = Theme::default();
        theme.id = "blue".to_string();
        theme.name = "Blue".to_string();
        theme.description = "Blue theme".to_string();
        theme.colors = ThemeColors::blue();
        theme
    }

    pub fn green() -> Self {
        let mut theme = Theme::default();
        theme.id = "green".to_string();
        theme.name = "Green".to_string();
        theme.description = "Green theme".to_string();
        theme.colors = ThemeColors::green();
        theme
    }

    pub fn purple() -> Self {
        let mut theme = Theme::default();
        theme.id = "purple".to_string();
        theme.name = "Purple".to_string();
        theme.description = "Purple theme".to_string();
        theme.colors = ThemeColors::purple();
        theme
    }

    pub fn orange() -> Self {
        let mut theme = Theme::default();
        theme.id = "orange".to_string();
        theme.name = "Orange".to_string();
        theme.description = "Orange theme".to_string();
        theme.colors = ThemeColors::orange();
        theme
    }

    pub fn red() -> Self {
        let mut theme = Theme::default();
        theme.id = "red".to_string();
        theme.name = "Red".to_string();
        theme.description = "Red theme".to_string();
        theme.colors = ThemeColors::red();
        theme
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: egui::Color32::from_rgb(59, 130, 246),
            primary_variant: egui::Color32::from_rgb(37, 99, 235),
            secondary: egui::Color32::from_rgb(108, 117, 125),
            secondary_variant: egui::Color32::from_rgb(73, 80, 87),
            background: egui::Color32::from_rgb(248, 249, 250),
            surface: egui::Color32::from_rgb(255, 255, 255),
            surface_variant: egui::Color32::from_rgb(233, 236, 239),
            error: egui::Color32::from_rgb(220, 53, 69),
            warning: egui::Color32::from_rgb(255, 193, 7),
            success: egui::Color32::from_rgb(40, 167, 69),
            info: egui::Color32::from_rgb(23, 162, 184),
            text_primary: egui::Color32::from_rgb(33, 37, 41),
            text_secondary: egui::Color32::from_rgb(108, 117, 125),
            text_disabled: egui::Color32::from_rgb(189, 189, 189),
            text_hint: egui::Color32::from_rgb(173, 181, 189),
            border: egui::Color32::from_rgb(222, 226, 230),
            divider: egui::Color32::from_rgb(233, 236, 239),
            outline: egui::Color32::from_rgb(59, 130, 246),
            shadow: egui::Color32::from_rgba(0, 0, 0, 128),
            highlight: egui::Color32::from_rgb(52, 152, 219),
            accent: egui::Color32::from_rgb(231, 76, 60),
            overlay: egui::Color32::from_rgba(0, 0, 0, 64),
            scrim: egui::Color32::from_rgba(0, 0, 0, 128),
            inverse_surface: egui::Color32::from_rgb(33, 37, 41),
            inverse_on_surface: egui::Color32::from_rgb(255, 255, 255),
        }
    }
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            primary: egui::Color32::from_rgb(66, 165, 245),
            primary_variant: egui::Color32::from_rgb(30, 136, 229),
            secondary: egui::Color32::from_rgb(158, 158, 158),
            secondary_variant: egui::Color32::from_rgb(117, 117, 117),
            background: egui::Color32::from_rgb(18, 18, 18),
            surface: egui::Color32::from_rgb(33, 33, 33),
            surface_variant: egui::Color32::from_rgb(49, 49, 49),
            error: egui::Color32::from_rgb(244, 67, 54),
            warning: egui::Color32::from_rgb(255, 235, 59),
            success: egui::Color32::from_rgb(76, 175, 80),
            info: egui::Color32::from_rgb(0, 188, 212),
            text_primary: egui::Color32::from_rgb(255, 255, 255),
            text_secondary: egui::Color32::from_rgb(189, 189, 189),
            text_disabled: egui::Color32::from_rgb(117, 117, 117),
            text_hint: egui::Color32::from_rgb(158, 158, 158),
            border: egui::Color32::from_rgb(66, 66, 66),
            divider: egui::Color32::from_rgb(84, 84, 84),
            outline: egui::Color32::from_rgb(66, 165, 245),
            shadow: egui::Color32::from_rgba(0, 0, 0, 128),
            highlight: egui::Color32::from_rgb(66, 165, 245),
            accent: egui::Color32::from_rgb(244, 67, 54),
            overlay: egui::Color32::from_rgba(0, 0, 0, 64),
            scrim: egui::Color32::from_rgba(0, 0, 0, 128),
            inverse_surface: egui::Color32::from_rgb(255, 255, 255),
            inverse_on_surface: egui::Color32::from_rgb(33, 33, 33),
        }
    }

    pub fn light() -> Self {
        Self::default()
    }

    pub fn blue() -> Self {
        let mut colors = Self::default();
        colors.primary = egui::Color32::from_rgb(33, 150, 243);
        colors.primary_variant = egui::Color32::from_rgb(25, 118, 210);
        colors.secondary = egui::Color32::from_rgb(255, 193, 7);
        colors.secondary_variant = egui::Color32::from_rgb(255, 152, 0);
        colors
    }

    pub fn green() -> Self {
        let mut colors = Self::default();
        colors.primary = egui::Color32::from_rgb(76, 175, 80);
        colors.primary_variant = egui::Color32::from_rgb(56, 142, 60);
        colors.secondary = egui::Color32::from_rgb(255, 193, 7);
        colors.secondary_variant = egui::Color32::from_rgb(255, 152, 0);
        colors
    }

    pub fn purple() -> Self {
        let mut colors = Self::default();
        colors.primary = egui::Color32::from_rgb(156, 39, 176);
        colors.primary_variant = egui::Color32::from_rgb(123, 31, 162);
        colors.secondary = egui::Color32::from_rgb(255, 193, 7);
        colors.secondary_variant = egui::Color32::from_rgb(255, 152, 0);
        colors
    }

    pub fn orange() -> Self {
        let mut colors = Self::default();
        colors.primary = egui::Color32::from_rgb(255, 152, 0);
        colors.primary_variant = egui::Color32::from_rgb(245, 124, 0);
        colors.secondary = egui::Color32::from_rgb(255, 193, 7);
        colors.secondary_variant = egui::Color32::from_rgb(255, 152, 0);
        colors
    }

    pub fn red() -> Self {
        let mut colors = Self::default();
        colors.primary = egui::Color32::from_rgb(244, 67, 54);
        colors.primary_variant = egui::Color32::from_rgb(229, 57, 53);
        colors.secondary = egui::Color32::from_rgb(255, 193, 7);
        colors.secondary_variant = egui::Color32::from_rgb(255, 152, 0);
        colors
    }
}

impl Default fn default() -> Self {
        Self {
            primary: ThemeFont::default(),
            secondary: ThemeFont::default(),
            monospace: ThemeFont::default(),
            display: ThemeFont::default(),
            headline: ThemeFont::default(),
            title: ThemeFont::default(),
            body: ThemeFont::default(),
            caption: ThemeFont::default(),
            label: ThemeFont::default(),
            button: ThemeFont::default(),
            input: ThemeFont::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            family: "Inter".to_string(),
            size: 14.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            line_height: 1.4,
            letter_spacing: 0.0,
            color: egui::Color32::from_rgb(33, 37, 41),
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
            xxxl: 64.0,
            button_height: 32.0,
            button_height_small: 24.0,
            button_height_large: 40.0,
            input_height: 32.0,
            input_height_small: 24.0,
            input_height_large: 40.0,
            icon_size: 16.0,
            icon_size_small: 12.0,
            icon_size_large: 24.0,
            avatar_size: 32.0,
            avatar_size_small: 24.0,
            avatar_size_large: 48.0,
            thumbnail_size: 128.0,
            preview_size: 256.0,
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
            none: 0.0,
            tight: 4.0,
            normal: 8.0,
            wide: 16.0,
            extra_wide: 24.0,
        }
}

impl Default fn default() -> Self {
        Self {
            none: Border::new(),
            thin: Border::new_with_width(1.0),
            normal: Border::new_with_width(2.0),
            thick: Border::new_with_width(4.0),
            rounded: Border::new_with_radius(4.0),
            circle: Border::new_with_radius(f32::MAX),
        }
}

impl Border {
    pub fn new() -> Self {
        Self {
            width: 0.0,
            color: egui::Color32::TRANSPARENT,
            style: BorderStyle::None,
            radius: 0.0,
        }
    }

    pub fn new_with_width(width: f32) -> Self {
        Self {
            width,
            color: egui::Color32::from_rgb(222, 226, 230),
            style: BorderStyle::Solid,
            radius: 0.0,
        }
    }

    pub fn new_with_radius(radius: f32) -> Self {
        Self {
            width: 1.0,
            color: egui::Color32::from_rgb(222, 226, 230),
            style: BorderStyle::Solid,
            radius,
        }
    }

    pub fn clone(&self) -> Border {
        Border {
            width: self.width,
            color: self.color,
            style: self.style,
            radius: self.radius,
        }
    }
}

impl Default fn default() -> Self {
        Self {
            none: Shadow::new(),
            xs: Shadow::new_with_values(0.0, 0.0, 0.0, 0.0),
            sm: Shadow::new_with_values(0.0, 1.0, 2.0, 0.0),
            md: Shadow::new_with_values(0.0, 4.0, 6.0, -1.0),
            lg: Shadow::new_with_values(0.0, 10.0, 15.0, -3.0),
            xl: Shadow::new_with_values(0.0, 20.0, 25.0, -5.0),
            xxl: Shadow::new_with_values(0.0, 25.0, 50.0, -12.0),
        }
}

impl Shadow {
    pub fn new() -> Self {
        Self {
            color: egui::Color32::TRANSPARENT,
            offset: egui::vec2(0.0, 0.0),
            blur: 0.0,
            spread: 0.0,
        }
    }

    pub fn new_with_values(x: f32, y: f32, blur: f32, spread: f32) -> Self {
        Self {
            color: egui::Color32::from_rgba(0, 0, 0, 128),
            offset: egui::vec2(x, y),
            blur,
            spread,
        }
    }

    pub fn clone(&self) -> Shadow {
        Shadow {
            color: self.color,
            offset: self.offset,
            blur: self.blur,
            spread: self.spread,
        }
    }
}

impl Default fn default() -> Self {
        Self {
            duration: 0.2,
            easing: EasingFunction::EaseInOut,
            enabled: true,
            fast: Animation::new_with_duration(0.1),
            normal: Animation::new_with_duration(0.2),
            slow: Animation::new_with_duration(0.4),
        }
}

impl Animation {
    pub fn new() -> Self {
        Self {
            duration: 0.2,
            easing: EasingFunction::EaseInOut,
            delay: 0.0,
        }
    }

    pub fn new_with_duration(duration: f32) -> Self {
        Self {
            duration,
            easing: EasingFunction::EaseInOut,
            delay: 0.0,
        }
    }

    pub fn clone(&self) -> Animation {
        Animation {
            duration: self.duration,
            easing: self.easing,
            delay: self.delay,
        }
    }
}

impl Default fn default() -> Self {
        Self {
            none: Transition::new(),
            fast: Transition::new_with_duration(0.1),
            normal: Transition::new_with_duration(0.2),
            slow: Transition::new_with_duration(0.4),
            slide: Transition::new_with_property(TransitionProperty::Transform),
            fade: Transition::new_with_property(TransitionProperty::Opacity),
            scale: Transition::new_with_property(TransitionProperty::Transform),
            rotate: Transition::new_with_property(TransitionProperty::Transform),
        }
}

impl Transition {
    pub fn new() -> Self {
        Self {
            duration: 0.2,
            easing: EasingFunction::EaseInOut,
            delay: 0.0,
            property: TransitionProperty::All,
        }
    }

    pub fn new_with_duration(duration: f32) -> Self {
        Self {
            duration,
            easing: EasingFunction::EaseInOut,
            delay: 0.0,
            property: TransitionProperty::All,
        }
    }

    pub fn new_with_property(property: TransitionProperty) -> Self {
        Self {
            duration: 0.2,
            easing: EasingFunction::EaseInOut,
            delay: 0.0,
            property,
        }
    }

    pub fn clone(&self) -> Transition {
        Transition {
            duration: self.duration,
            easing: self.easing,
            delay: self.delay,
            property: self.property,
        }
    }
}

impl Default fn default() -> Self {
        Self {
            button: ButtonTheme::default(),
            input: InputTheme::default(),
            checkbox: CheckboxTheme::default(),
            radio: RadioTheme::default(),
            slider: SliderTheme::default(),
            switch: SwitchTheme::default(),
            progress_bar: ProgressBarTheme::default(),
            card: CardTheme::default(),
            panel: PanelTheme::default(),
            dialog: DialogTheme::default(),
            menu: MenuTheme::default(),
            toolbar: ToolbarTheme::default(),
            status_bar: StatusBarTheme::default(),
            tab_bar: TabBarTheme::default(),
            table: TableTheme::default(),
            list: ListTheme::default(),
            tree: TreeTheme::default(),
            accordion: AccordionTheme::default(),
            carousel: CarouselTheme::default(),
            tooltip: TooltipTheme::default(),
            badge: BadgeTheme::default(),
            avatar: AvatarTheme::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            primary: ButtonVariantTheme::default(),
            secondary: ButtonVariantTheme::default(),
            success: ButtonVariantTheme::default(),
            warning: ButtonVariantTheme::default(),
            danger: ButtonVariantTheme::default(),
            info: ButtonVariantTheme::default(),
            light: ButtonVariantTheme::default(),
            dark: ButtonVariantTheme::default(),
            link: ButtonVariantTheme::default(),
            outline: ButtonVariantTheme::default(),
            ghost: ButtonVariantTheme::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(59, 130, 246),
            foreground: egui::Color32::from_rgb(255, 255, 255),
            border: egui::Color32::from_rgb(59, 130, 246),
            hover_background: egui::Color32::from_rgb(37, 99, 235),
            hover_foreground: egui::Color32::from_rgb(255, 255, 255),
            hover_border: egui::Color32::from_rgb(37, 99, 235),
            active_background: egui::Color32::from_rgb(29, 78, 216),
            active_foreground: egui::Color32::from_rgb(255, 255, 255),
            active_border: egui::Color32::from_rgb(29, 78, 216),
            disabled_background: egui::Color32::from_rgb(233, 236, 239),
            disabled_foreground: egui::Color32::from_rgb(108, 117, 125),
            disabled_border: egui::Color32::from_rgb(233, 236, 239),
            shadow: None,
            border_radius: 4.0,
            font: ThemeFont::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            placeholder: egui::Color32::from_rgb(189, 189, 189),
            focus_background: egui::Color32::from_rgb(255, 255, 255),
            focus_border: egui::Color32::from_rgb(59, 130, 246),
            error_background: egui::Color32::from_rgb(255, 255, 255),
            error_border: egui::Color32::from_rgb(220, 53, 69),
            disabled_background: egui::Color32::from_rgb(233, 236, 239),
            disabled_foreground: egui::Color32::from_rgb(108, 117, 125),
            disabled_border: egui::Color32::from_rgb(233, 236, 239),
            border_radius: 4.0,
            border_width: 1.0,
            font: ThemeFont::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            check_color: egui::Color32::from_rgb(59, 130, 246),
            hover_background: egui::Color32::from_rgb(248, 249, 250),
            hover_border: egui::Color32::from_rgb(59, 130, 246),
            disabled_background: egui::Color32::from_rgb(233, 236, 239),
            disabled_foreground: egui::Color32::from_rgb(108, 117, 125),
            disabled_border: egui::Color32::from_rgb(233, 236, 239),
            border_radius: 4.0,
            size: 16.0,
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            dot_color: egui::Color32::from_rgb(59, 130, 246),
            hover_background: egui::Color32::from_rgb(248, 249, 250),
            hover_border: egui::Color32::from_rgb(59, 130, 246),
            disabled_background: egui::Color32::from_rgb(233, 236, 239),
            disabled_foreground: egui::Color32::from_rgb(108, 117, 125),
            disabled_border: egui::Color32::from_rgb(233, 236, 239),
            border_radius: 50.0,
            size: 16.0,
        }
}

impl Default fn default() -> Self {
        Self {
            track_background: egui::Color32::from_rgb(233, 236, 239),
            track_fill: egui::Color32::from_rgb(59, 130, 246),
            handle_background: egui::Color32::from_rgb(255, 255, 255),
            handle_border: egui::Color32::from_rgb(59, 130, 246),
            handle_hover_background: egui::Color32::from_rgb(248, 249, 250),
            handle_active_background: egui::Color32::from_rgb(255, 255, 255),
            disabled_track_background: egui::Color32::from_rgb(233, 236, 239),
            disabled_track_fill: egui::Color32::from_rgb(189, 189, 189),
            disabled_handle_background: egui::Color32::from_rgb(233, 236, 239),
            border_radius: 4.0,
            track_height: 4.0,
            handle_size: 16.0,
        }
}

impl Default fn default() -> Self {
        Self {
            track_background: egui::Color32::from_rgb(233, 236, 239),
            track_active_background: egui::Color32::from_rgb(59, 130, 246),
            thumb_background: egui::Color32::from_rgb(255, 255, 255),
            thumb_active_background: egui::Color32::from_rgb(255, 255, 255),
            thumb_hover_background: egui::Color32::from_rgb(248, 249, 250),
            disabled_track_background: egui::Color32::from_rgb(233, 236, 239),
            disabled_thumb_background: egui::Color32::from_rgb(189, 189, 189),
            border_radius: 50.0,
            track_width: 36.0,
            track_height: 20.0,
            thumb_size: 16.0,
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(233, 236, 239),
            fill: egui::Color32::from_rgb(59, 130, 246),
            text: egui::Color32::from_rgb(255, 255, 255),
            border: egui::Color32::from_rgb(59, 130, 246),
            border_radius: 4.0,
            height: 8.0,
            font: ThemeFont::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            shadow: Shadow::new_with_values(0.0, 4.0, 6.0, -1.0),
            border_radius: 8.0,
            padding: 16.0,
            header_background: egui::Color32::from_rgb(248, 249, 250),
            header_foreground: egui::Color32::from_rgb(33, 37, 41),
            header_border: egui::Color32::from_rgb(222, 226, 230),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            shadow: Shadow::new_with_values(0.0, 4.0, 6.0, -1.0),
            border_radius: 8.0,
            padding: 16.0,
            header_background: egui::Color32::from_rgb(248, 249, 250),
            header_foreground: egui::Color32::from_rgb(33, 37, 41),
            header_border: egui::Color32::from_rgb(222, 226, 230),
            header_height: 32.0,
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            shadow: Shadow::new_with_values(0.0, 10.0, 15.0, -3.0),
            border_radius: 8.0,
            padding: 24.0,
            header_background: egui::Color32::from_rgb(248, 249, 250),
            header_foreground: egui::Color32::from_rgb(33, 37, 41),
            header_border: egui::Color32::from_rgb(222, 226, 230),
            overlay_background: egui::Color32::from_rgba(0, 0, 0, 64),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            shadow: Shadow::new_with_values(0.0, 4.0, 6.0, -1.0),
            border_radius: 4.0,
            padding: 8.0,
            item_background: egui::Color32::from_rgb(255, 255, 255),
            item_foreground: egui::Color32::from_rgb(33, 37, 41),
            item_hover_background: egui::Color32::from_rgb(248, 249, 250),
            item_hover_foreground: egui::Color32::from_rgb(33, 37, 41),
            item_active_background: egui::Color32::from_rgb(59, 130, 246),
            item_active_foreground: egui::Color32::from_rgb(255, 255, 255),
            separator: egui::Color32::from_rgb(222, 226, 230),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(248, 249, 250),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            shadow: Shadow::new(),
            border_radius: 4.0,
            padding: 8.0,
            button_background: egui::Color32::from_rgb(255, 255, 255),
            button_foreground: egui::Color32::from_rgb(33, 37, 41),
            button_hover_background: egui::Color32::from_rgb(248, 249, 250),
            button_hover_foreground: egui::Color32::from_rgb(33, 37, 41),
            button_active_background: egui::Color32::from_rgb(233, 236, 239),
            button_active_foreground: egui::Color32::from_rgb(33, 37, 41),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(248, 249, 250),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            padding: 8.0,
            font: ThemeFont::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            border_radius: 4.0,
            padding: 8.0,
            tab_background: egui::Color32::from_rgb(248, 249, 250),
            tab_foreground: egui::Color32::from_rgb(33, 37, 41),
            tab_hover_background: egui::Color32::from_rgb(233, 236, 239),
            tab_hover_foreground: egui::Color32::from_rgb(33, 37, 41),
            tab_active_background: egui::Color32::from_rgb(59, 130, 246),
            tab_active_foreground: egui::Color32::from_rgb(255, 255, 255),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            header_background: egui::Color32::from_rgb(248, 249, 250),
            header_foreground: egui::Color32::from_rgb(33, 37, 41),
            header_border: egui::Color32::from_rgb(222, 226, 230),
            row_background: egui::Color32::from_rgb(255, 255, 255),
            row_foreground: egui::Color32::from_rgb(33, 37, 41),
            row_hover_background: egui::Color32::from_rgb(248, 249, 250),
            row_hover_foreground: egui::Color32::from_rgb(33, 37, 41),
            row_selected_background: egui::Color32::from_rgb(59, 130, 246),
            row_selected_foreground: egui::Color32::from_rgb(255, 255, 255),
            border_radius: 4.0,
            padding: 8.0,
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            item_background: egui::Color32::from_rgb(255, 255, 255),
            item_foreground: egui::Color32::from_rgb(33, 37, 41),
            item_hover_background: egui::Color32::from_rgb(248, 249, 250),
            item_hover_foreground: egui::Color32::from_rgb(33, 37, 41),
            item_active_background: egui::Color32::from_rgb(59, 130, 246),
            item_active_foreground: egui::Color32::from_rgb(255, 255, 255),
            item_selected_background: egui::Color32::from_rgb(59, 130, 246),
            item_selected_foreground: egui::Color32::from_rgb(255, 255, 255),
            border_radius: 4.0,
            padding: 8.0,
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            node_background: egui::Color32::from_rgb(255, 255, 255),
            node_foreground: egui::Color32::from_rgb(33, 37, 41),
            node_hover_background: egui::Color32::from_rgb(248, 249, 250),
            node_hover_foreground: egui::Color32::from_rgb(33, 37, 41),
            node_active_background: egui::Color32::from_rgb(59, 130, 246),
            node_active_foreground: egui::Color32::from_rgb(255, 255, 255),
            node_selected_background: egui::Color32::from_rgb(59, 130, 246),
            node_selected_foreground: egui::Color32::from_rgb(255, 255, 255),
            expand_icon_color: egui::Color32::from_rgb(108, 117, 125),
            border_radius: 4.0,
            padding: 8.0,
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            header_background: egui::Color32::from_rgb(248, 249, 250),
            header_foreground: egui::Color32::from_rgb(33, 37, 41),
            header_border: egui::Color32::from_rgb(222, 226, 230),
            header_hover_background: egui::Color32::from_rgb(233, 236, 239),
            header_hover_foreground: egui::Color32::from_rgb(33, 37, 41),
            content_background: egui::Color32::from_rgb(255, 255, 255),
            content_foreground: egui::Color32::from_rgb(33, 37, 41),
            border_radius: 4.0,
            padding: 8.0,
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(255, 255, 255),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            shadow: Shadow::new_with_values(0.0, 4.0, 6.0, -1.0),
            border_radius: 8.0,
            padding: 16.0,
            control_background: egui::Color32::from_rgb(255, 255, 255),
            control_foreground: egui::Color32::from_rgb(33, 37, 41),
            control_hover_background: egui::Color32::from_rgb(248, 249, 250),
            control_hover_foreground: egui::Color32::from_rgb(33, 37, 41),
            indicator_background: egui::Color32::from_rgb(233, 236, 239),
            indicator_active_background: egui::Color32::from_rgb(59, 130, 246),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(33, 37, 41),
            foreground: egui::Color32::from_rgb(255, 255, 255),
            border: egui::Color32::from_rgb(222, 226, 230),
            shadow: Shadow::new_with_values(0.0, 4.0, 6.0, -1.0),
            border_radius: 4.0,
            padding: 8.0,
            font: ThemeFont::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(59, 130, 246),
            foreground: egui::Color32::from_rgb(255, 255, 255),
            border: egui::Color32::from_rgb(59, 130, 246),
            border_radius: 4.0,
            padding: 4.0,
            font: ThemeFont::default(),
        }
}

impl Default fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(233, 236, 239),
            foreground: egui::Color32::from_rgb(33, 37, 41),
            border: egui::Color32::from_rgb(222, 226, 230),
            border_radius: 50.0,
            font: ThemeFont::default(),
        }
}

pub fn create_theme_manager(config: ThemeManagerConfig) -> ThemeManager {
    ThemeManager::new(config)
}

pub fn create_theme_manager_config() -> ThemeManagerConfig {
    ThemeManagerConfig::default()
}
