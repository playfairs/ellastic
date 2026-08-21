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
pub struct IconManager {
    pub icons: Arc<RwLock<HashMap<String, Icon>>>,
    pub icon_sets: Arc<RwLock<HashMap<String, IconSet>>>,
    pub current_set: Arc<RwLock<String>>,
    pub config: IconManagerConfig,
}

#[derive(Debug, Clone)]
pub struct IconManagerConfig {
    pub default_set: String,
    pub default_size: f32,
    pub default_color: egui::Color32,
    pub cache_enabled: bool,
    pub cache_size: usize,
    pub load_on_demand: bool,
}

#[derive(Debug, Clone)]
pub struct Icon {
    pub name: String,
    pub data: IconData,
    pub size: f32,
    pub color: egui::Color32,
    pub metadata: IconMetadata,
}

#[derive(Debug, Clone)]
pub enum IconData {
    Unicode(char),
    Svg(String),
    Png(Vec<u8>),
    Custom(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct IconMetadata {
    pub category: IconCategory,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconCategory {
    Action,
    Alert,
    Av,
    Communication,
    Content,
    Device,
    Editor,
    File,
    Hardware,
    Image,
    Maps,
    Navigation,
    Notification,
    Places,
    Social,
    Toggle,
    Custom,
}

#[derive(Debug, Clone)]
pub struct IconSet {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub icons: HashMap<String, Icon>,
    pub default_size: f32,
    pub default_color: egui::Color32,
    pub metadata: IconSetMetadata,
}

#[derive(Debug, Clone)]
pub struct IconSetMetadata {
    pub icon_count: usize,
    pub categories: Vec<IconCategory>,
    pub supported_sizes: Vec<f32>,
    pub supported_colors: Vec<egui::Color32>,
    pub file_format: IconFormat,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconFormat {
    Unicode,
    Svg,
    Png,
    Custom,
}

#[derive(Debug, Clone)]
pub struct IconRenderer {
    pub renderer_type: IconRendererType,
    pub config: IconRendererConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconRendererType {
    Unicode,
    Svg,
    Png,
    Custom,
}

#[derive(Debug, Clone)]
pub struct IconRendererConfig {
    pub antialiasing: bool,
    pub smoothing: bool,
    pub cache_enabled: bool,
    pub max_cache_size: usize,
}

#[derive(Debug, Clone)]
pub struct IconTheme {
    pub name: String,
    pub colors: IconColors,
    pub sizes: IconSizes,
    pub styles: IconStyles,
}

#[derive(Debug, Clone)]
pub struct IconColors {
    pub primary: egui::Color32,
    pub secondary: egui::Color32,
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub error: egui::Color32,
    pub info: egui::Color32,
    pub light: egui::Color32,
    pub dark: egui::Color32,
    pub muted: egui::Color32,
    pub disabled: egui::Color32,
}

#[derive(Debug, Clone)]
pub struct IconSizes {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

#[derive(Debug, Clone)]
pub struct IconStyles {
    pub regular: IconStyle,
    pub filled: IconStyle,
    pub outlined: IconStyle,
    pub rounded: IconStyle,
    pub sharp: IconStyle,
    pub two_tone: IconStyle,
}

#[derive(Debug, Clone)]
pub struct IconStyle {
    pub weight: f32,
    pub fill: f32,
    pub grade: f32,
    pub optical_size: f32,
}

#[derive(Debug, Clone)]
pub struct IconAnimation {
    pub name: String,
    pub duration: f32,
    pub easing: IconEasingFunction,
    pub keyframes: Vec<AnimationFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconEasingFunction {
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
pub struct AnimationFrame {
    pub time: f32,
    pub transform: IconTransform,
    pub opacity: f32,
    pub color: Option<egui::Color32>,
}

#[derive(Debug, Clone)]
pub struct IconTransform {
    pub translate: egui::Vec2,
    pub rotate: f32,
    pub scale: f32,
    pub skew: egui::Vec2,
}

#[derive(Debug, Clone)]
pub struct IconCache {
    pub cache: HashMap<String, CachedIcon>,
    pub config: IconCacheConfig,
    pub stats: IconCacheStats,
}

#[derive(Debug, Clone)]
pub struct CachedIcon {
    pub icon: Icon,
    pub rendered_data: Option<Vec<u8>>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
}

#[derive(Debug, Clone)]
pub struct IconCacheConfig {
    pub max_size: usize,
    pub ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct IconCacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub last_cleanup: DateTime<Utc>,
}

impl IconManager {
    pub fn new(config: IconManagerConfig) -> Self {
        Self {
            icons: Arc::new(RwLock::new(HashMap::new())),
            icon_sets: Arc::new(RwLock::new(HashMap::new())),
            current_set: Arc::new(RwLock::new(config.default_set.clone())),
            config,
        }
    }

    pub fn load_default_icons(&mut self) -> Result<()> {
        self.load_unicode_icons()?;
        self.load_material_icons()?;
        self.load_font_awesome_icons()?;
        self.load_bootstrap_icons()?;
        self.load_heroicons()?;
        Ok(())
    }

    fn load_unicode_icons(&mut self) -> Result<()> {
        let mut icon_set = IconSet {
            name: "unicode".to_string(),
            description: "Unicode icons".to_string(),
            version: "1.0".to_string(),
            author: "Unicode".to_string(),
            license: "Unicode".to_string(),
            icons: HashMap::new(),
            default_size: 16.0,
            default_color: egui::Color32::BLACK,
            metadata: IconSetMetadata::new(),
        };

        let unicode_icons = vec![
            ("home", '🏠', IconCategory::Action),
            ("settings", '⚙', IconCategory::Action),
            ("user", '👤', IconCategory::Social),
            ("search", '🔍', IconCategory::Action),
            ("menu", '☰', IconCategory::Navigation),
            ("close", '✕', IconCategory::Action),
            ("check", '✓', IconCategory::Action),
            ("warning", '⚠', IconCategory::Alert),
            ("error", '❌', IconCategory::Alert),
            ("info", 'ℹ', IconCategory::Alert),
            ("success", '✅', IconCategory::Alert),
            ("star", '⭐', IconCategory::Action),
            ("heart", '❤', IconCategory::Social),
            ("trash", '🗑', IconCategory::Action),
            ("edit", '✏', IconCategory::Editor),
            ("save", '💾', IconCategory::Action),
            ("download", '📥', IconCategory::Action),
            ("upload", '📤', IconCategory::Action),
            ("folder", '📁', IconCategory::File),
            ("file", '📄', IconCategory::File),
            ("image", '🖼', IconCategory::Image),
            ("video", '🎥', IconCategory::Av),
            ("audio", '🎵', IconCategory::Av),
            ("play", '▶', IconCategory::Av),
            ("pause", '⏸', IconCategory::Av),
            ("stop", '⏹', IconCategory::Av),
            ("next", '⏭', IconCategory::Av),
            ("previous", '⏮', IconCategory::Av),
            ("volume_up", '🔊', IconCategory::Av),
            ("volume_down", '🔉', IconCategory::Av),
            ("volume_mute", '🔇', IconCategory::Av),
            ("fullscreen", '⛶', IconCategory::Action),
            ("minimize", '⊟', IconCategory::Action),
            ("maximize", '⊞', IconCategory::Action),
            ("refresh", '🔄', IconCategory::Action),
            ("copy", '📋', IconCategory::Action),
            ("paste", '📋', IconCategory::Action),
            ("cut", '✂', IconCategory::Action),
            ("undo", '↶', IconCategory::Editor),
            ("redo", '↷', IconCategory::Editor),
            ("bold", 'B', IconCategory::Editor),
            ("italic", 'I', IconCategory::Editor),
            ("underline", 'U', IconCategory::Editor),
            ("link", '🔗', IconCategory::Communication),
            ("mail", '✉', IconCategory::Communication),
            ("phone", '📞', IconCategory::Communication),
            ("calendar", '📅', IconCategory::Action),
            ("clock", '🕐', IconCategory::Action),
            ("location", '📍', IconCategory::Places),
            ("map", '🗺', IconCategory::Maps),
            ("camera", '📷', IconCategory::Device),
            ("computer", '💻', IconCategory::Device),
            ("mobile", '📱', IconCategory::Device),
            ("tablet", '📱', IconCategory::Device),
            ("printer", '🖨', IconCategory::Device),
            ("wifi", '📶', IconCategory::Device),
            ("bluetooth", '📶', IconCategory::Device),
            ("battery", '🔋', IconCategory::Device),
            ("power", '🔌', IconCategory::Device),
            ("lock", '🔒', IconCategory::Action),
            ("unlock", '🔓', IconCategory::Action),
            ("key", '🔑', IconCategory::Action),
            ("eye", '👁', IconCategory::Action),
            ("eye_off", '👁‍🗨', IconCategory::Action),
            ("bell", '🔔', IconCategory::Notification),
            ("bell_off", '🔕', IconCategory::Notification),
            ("bookmark", '🔖', IconCategory::Action),
            ("flag", '🏳', IconCategory::Action),
            ("tag", '🏷', IconCategory::Action),
            ("filte", '🔽', IconCategory::Action),
            ("sortv", '🔽', IconCategory::Action),
            ("grid", '⚏', IconCategory::Action),
            ("list", '☰', IconCategory::Action),
            ("card", '🃏', IconCategory::Content),
            ("chart", '📊', IconCategory::Content),
            ("graph", '📈', IconCategory::Content),
            ("database", '🗄', IconCategory::Device),
            ("server", '🖥', IconCategory::Device),
            ("cloud", '☁', IconCategory::Device),
            ("sun", '☀', IconCategory::Action),
            ("moon", '🌙', IconCategory::Action),
            ("cloud_rain", '🌧', IconCategory::Action),
            ("cloud_snow", '❄', IconCategory::Action),
            ("cloud_sun", '⛅', IconCategory::Action),
            ("umbrella", '☂', IconCategory::Action),
            ("glasses", '👓', IconCategory::Device),
            ("headphones", '🎧', IconCategory::Device),
            ("microphone", '🎤', IconCategory::Device),
            ("speaker", '🔊', IconCategory::Device),
            ("gamepad", '🎮', IconCategory::Device),
            ("rocket", '🚀', IconCategory::Action),
            ("airplane", '✈', IconCategory::Action),
            ("car", '🚗', IconCategory::Action),
            ("bus", '🚌', IconCategory::Action),
            ("train", '🚂', IconCategory::Action),
            ("bike", '🚲', IconCategory::Action),
            ("walk", '🚶', IconCategory::Action),
            ("run", '🏃', IconCategory::Action),
            ("food", '🍔', IconCategory::Action),
            ("coffee", '☕', IconCategory::Action),
            ("pizza", '🍕', IconCategory::Action),
            ("gift", '🎁', IconCategory::Action),
            ("party", '🎉', IconCategory::Action),
            ("music", '🎵', IconCategory::Av),
            ("palette", '🎨', IconCategory::Action),
            ("brush", '🖌', IconCategory::Action),
            ("pencil", '✏', IconCategory::Editor),
            ("eraser", '🧹', IconCategory::Editor),
            ("scissors", '✂', IconCategory::Editor),
            ("ruler", '📏', IconCategory::Editor),
            ("compass", '🧭', IconCategory::Maps),
            ("magnet", '🧲', IconCategory::Device),
            ("atom", '⚛', IconCategory::Science),
            ("flask", '🧪', IconCategory::Science),
            ("microscope", '🔬', IconCategory::Science),
            ("telescope", '🔭', IconCategory::Science),
            ("dna", '🧬', IconCategory::Science),
            ("pill", '💊', IconCategory::Medical),
            ("stethoscope", '🩺', IconCategory::Medical),
            ("hospital", '🏥', IconCategory::Places),
            ("school", '🏫', IconCategory::Places),
            ("book", '📚', IconCategory::Content),
            ("graduation_cap", '🎓', IconCategory::Education),
            ("trophy", '🏆', IconCategory::Action),
            ("medal", '🏅', IconCategory::Action),
            ("crown", '👑', IconCategory::Action),
            ("diamond", '💎', IconCategory::Action),
            ("money", '💰', IconCategory::Action),
            ("credit_card", '💳', IconCategory::Action),
            ("shopping_cart", '🛒', IconCategory::Action),
            ("shopping_bag", '🛍', IconCategory::Action),
            ("store", '🏪', IconCategory::Places),
            ("cart", '🛒', IconCategory::Action),
            ("tag", '🏷', IconCategory::Action),
            ("barcode", '📊', IconCategory::Action),
            ("qrcode", '📱', IconCategory::Action),
            ("fingerprint", '👆', IconCategory::Action),
            ("face", '😊', IconCategory::Social),
            ("smile", '😄', IconCategory::Social),
            ("frown", '😢', IconCategory::Social),
            ("angry", '😠', IconCategory::Social),
            ("surprise", '😲', IconCategory::Social),
            ("wink", '😉', IconCategory::Social),
            ("love", '❤', IconCategory::Social),
            ("like", '👍', IconCategory::Social),
            ("dislike", '👎', IconCategory::Social),
            ("thumbs_up", '👍', IconCategory::Social),
            ("thumbs_down", '👎', IconCategory::Social),
            ("hand", '✋', IconCategory::Action),
            ("wave", '👋', IconCategory::Social),
            ("ok", '👌', IconCategory::Action),
            ("no", '🚫', IconCategory::Action),
            ("prohibited", '🚫', IconCategory::Action),
            ("stop", '🛑', IconCategory::Action),
            ("arrow_up", '↑', IconCategory::Navigation),
            ("arrow_down", '↓', IconCategory::Navigation),
            ("arrow_left", '←', IconCategory::Navigation),
            ("arrow_right", '→', IconCategory::Navigation),
            ("arrow_up_left", '↖', IconCategory::Navigation),
            ("arrow_up_right", '↗', IconCategory::Navigation),
            ("arrow_down_left", '↙', IconCategory::Navigation),
            ("arrow_down_right", '↘', IconCategory::Navigation),
            ("chevron_up", '↑', IconCategory::Navigation),
            ("chevron_down", '↓', IconCategory::Navigation),
            ("chevron_left", '←', IconCategory::Navigation),
            ("chevron_right", '→', IconCategory::Navigation),
            ("angle_up", '↑', IconCategory::Navigation),
            ("angle_down", '↓', IconCategory::Navigation),
            ("angle_left", '←', IconCategory::Navigation),
            ("angle_right", '→', IconCategory::Navigation),
            ("caret_up", '▲', IconCategory::Navigation),
            ("caret_down", '▼', IconCategory::Navigation),
            ("caret_left", '◀', IconCategory::Navigation),
            ("caret_right", '▶', IconCategory::Navigation),
            ("plus", '+', IconCategory::Action),
            ("minus", '-', IconCategory::Action),
            ("multiply", '×', IconCategory::Action),
            ("divide", '÷', IconCategory::Action),
            ("equals", '=', IconCategory::Action),
            ("not_equal", '≠', IconCategory::Action),
            ("less_than", '<', IconCategory::Action),
            ("greater_than", '>', IconCategory::Action),
            ("less_equal", '≤', IconCategory::Action),
            ("greater_equal", '≥', IconCategory::Action),
            ("infinity", '∞', IconCategory::Action),
            ("pi", 'π', IconCategory::Action),
            ("omega", 'Ω', IconCategory::Action),
            ("alpha", 'α', IconCategory::Action),
            ("beta", 'β', IconCategory::Action),
            ("gamma", 'γ', IconCategory::Action),
            ("delta", 'δ', IconCategory::Action),
            ("epsilon", 'ε', IconCategory::Action),
            ("zeta", 'ζ', IconCategory::Action),
            ("eta", 'η', IconCategory::Action),
            ("theta", 'θ', IconCategory::Action),
            ("iota", 'ι', IconCategory::Action),
            ("kappa", 'κ', IconCategory::Action),
            ("lambda", 'λ', IconCategory::Action),
            ("mu", 'μ', IconCategory::Action),
            ("nu", 'ν', IconCategory::Action),
            ("xi", 'ξ', IconCategory::Action),
            ("omicron", 'ο', IconCategory::Action),
            ("rho", 'ρ', IconCategory::Action),
            ("sigma", 'σ', IconCategory::Action),
            ("tau", 'τ', IconCategory::Action),
            ("upsilon", 'υ', IconCategory::Action),
            ("phi", 'φ', IconCategory::Action),
            ("chi", 'χ', IconCategory::Action),
            ("psi", 'ψ', IconCategory::Action),
            ("omega", 'ω', IconCategory::Action),
        ];

        for (name, unicode_char, category) in unicode_icons {
            let icon = Icon {
                name: name.to_string(),
                data: IconData::Unicode(unicode_char),
                size: 16.0,
                color: egui::Color32::BLACK,
                metadata: IconMetadata::new(category),
            };
            icon_set.icons.insert(name.to_string(), icon);
        }

        self.icon_sets.write().insert("unicode".to_string(), icon_set);
        Ok(())
    }

    fn load_material_icons(&mut self) -> Result<()> {
        let mut icon_set = IconSet {
            name: "material".to_string(),
            description: "Material Design icons".to_string(),
            version: "4.0".to_string(),
            author: "Google".to_string(),
            license: "Apache 2.0".to_string(),
            icons: HashMap::new(),
            default_size: 24.0,
            default_color: egui::Color32::BLACK,
            metadata: IconSetMetadata::new(),
        };

\        let material_icons = vec![
            ("home", IconCategory::Action),
            ("settings", IconCategory::Action),
            ("search", IconCategory::Action),
            ("menu", IconCategory::Navigation),
            ("close", IconCategory::Action),
            ("check", IconCategory::Action),
            ("warning", IconCategory::Alert),
            ("error", IconCategory::Alert),
            ("info", IconCategory::Alert),
            ("star", IconCategory::Action),
            ("favorite", IconCategory::Action),
            ("delete", IconCategory::Action),
            ("edit", IconCategory::Editor),
            ("save", IconCategory::Action),
            ("download", IconCategory::Action),
            ("upload", IconCategory::Action),
            ("folder", IconCategory::File),
            ("file", IconCategory::File),
            ("image", IconCategory::Image),
            ("video", IconCategory::Av),
            ("audio", IconCategory::Av),
            ("play_arrow", IconCategory::Av),
            ("pause", IconCategory::Av),
            ("stop", IconCategory::Av),
            ("skip_next", IconCategory::Av),
            ("skip_previous", IconCategory::Av),
            ("volume_up", IconCategory::Av),
            ("volume_down", IconCategory::Av),
            ("volume_off", IconCategory::Av),
            ("fullscreen", IconCategory::Action),
            ("fullscreen_exit", IconCategory::Action),
            ("refresh", IconCategory::Action),
            ("content_copy", IconCategory::Action),
            ("content_paste", IconCategory::Action),
            ("content_cut", IconCategory::Action),
            ("undo", IconCategory::Editor),
            ("redo", IconCategory::Editor),
            ("format_bold", IconCategory::Editor),
            ("format_italic", IconCategory::Editor),
            ("format_underline", IconCategory::Editor),
            ("link", IconCategory::Communication),
            ("email", IconCategory::Communication),
            ("phone", IconCategory::Communication),
            ("calendar_today", IconCategory::Action),
            ("access_time", IconCategory::Action),
            ("location_on", IconCategory::Places),
            ("map", IconCategory::Maps),
            ("photo_camera", IconCategory::Device),
            ("computer", IconCategory::Device),
            ("smartphone", IconCategory::Device),
            ("tablet", IconCategory::Device),
            ("print", IconCategory::Device),
            ("wifi", IconCategory::Device),
            ("bluetooth", IconCategory::Device),
            ("battery_full", IconCategory::Device),
            ("power", IconCategory::Device),
            ("lock", IconCategory::Action),
            ("lock_open", IconCategory::Action),
            ("vpn_key", IconCategory::Action),
            ("visibility", IconCategory::Action),
            ("visibility_off", IconCategory::Action),
            ("notifications", IconCategory::Notification),
            ("notifications_off", IconCategory::Notification),
            ("bookmark", IconCategory::Action),
            ("flag", IconCategory::Action),
            ("label", IconCategory::Action),
            ("filter_list", IconCategory::Action),
            ("sort", IconCategory::Action),
            ("grid_view", IconCategory::Action),
            ("view_list", IconCategory::Action),
            ("view_module", IconCategory::Action),
            ("card_giftcard", IconCategory::Action),
            ("insert_chart", IconCategory::Content),
            ("show_chart", IconCategory::Content),
            ("storage", IconCategory::Device),
            ("dns", IconCategory::Device),
            ("cloud", IconCategory::Device),
            ("wb_sunny", IconCategory::Action),
            ("nights_stay", IconCategory::Action),
            ("grain", IconCategory::Action),
            ("beach_access", IconCategory::Action),
            ("umbrella", IconCategory::Action),
            ("visibility", IconCategory::Action),
            ("headset", IconCategory::Device),
            ("mic", IconCategory::Device),
            ("speaker", IconCategory::Device),
            ("sports_esports", IconCategory::Device),
            ("rocket_launch", IconCategory::Action),
            ("flight", IconCategory::Action),
            ("directions_car", IconCategory::Action),
            ("directions_bus", IconCategory::Action),
            ("directions_railway", IconCategory::Action),
            ("directions_bike", IconCategory::Action),
            ("directions_walk", IconCategory::Action),
            ("directions_run", IconCategory::Action),
            ("restaurant", IconCategory::Action),
            ("local_cafe", IconCategory::Action),
            ("local_pizza", IconCategory::Action),
            ("card_giftcard", IconCategory::Action),
            ("celebration", IconCategory::Action),
            ("music_note", IconCategory::Av),
            ("palette", IconCategory::Action),
            ("brush", IconCategory::Action),
            ("edit", IconCategory::Editor),
            ("cleaning_services", IconCategory::Editor),
            ("content_cut", IconCategory::Editor),
            ("straighten", IconCategory::Editor),
            ("explore", IconCategory::Maps),
            "science": IconCategory::Science,
            ("biotech", IconCategory::Science),
            ("health_and_safety", IconCategory::Medical),
            ("local_hospital", IconCategory::Places),
            ("school", IconCategory::Places),
            ("menu_book", IconCategory::Content),
            ("emoji_events", IconCategory::Education),
            ("emoji_events", IconCategory::Action),
            ("workspace_premium", IconCategory::Action),
            ("diamond", IconCategory::Action),
            ("payments", IconCategory::Action),
            ("credit_card", IconCategory::Action),
            ("shopping_cart", IconCategory::Action),
            ("shopping_bag", IconCategory::Action),
            ("store", IconCategory::Places),
            ("local_offer", IconCategory::Action),
            ("qr_code_scanner", IconCategory::Action),
            ("fingerprint", IconCategory::Action),
            ("face", IconCategory::Social),
            ("sentiment_very_satisfied", IconCategory::Social),
            ("sentiment_dissatisfied", IconCategory::Social),
            ("thumb_up", IconCategory::Social),
            ("thumb_down", IconCategory::Social),
            ("waving_hand", IconCategory::Social),
            ("thumb_up_off_alt", IconCategory::Social),
            ("block", IconCategory::Action),
            ("pan_tool", IconCategory::Action),
            ("back_hand", IconCategory::Action),
            ("front_hand", IconCategory::Action),
            ("keyboard_arrow_up", IconCategory::Navigation),
            ("keyboard_arrow_down", IconCategory::Navigation),
            ("keyboard_arrow_left", IconCategory::Navigation),
            ("keyboard_arrow_right", IconCategory::Navigation),
            ("arrow_drop_up", IconCategory::Navigation),
            ("arrow_drop_down", IconCategory::Navigation),
            ("arrow_left", IconCategory::Navigation),
            ("arrow_right", IconCategory::Navigation),
            ("expand_less", IconCategory::Navigation),
            ("expand_more", IconCategory::Navigation),
            ("add", IconCategory::Action),
            ("remove", IconCategory::Action),
            ("close", IconCategory::Action),
            ("check_circle", IconCategory::Action),
            ("cancel", IconCategory::Action),
            ("priority_high", IconCategory::Action),
            ("remove_circle", IconCategory::Action),
            ("add_circle", IconCategory::Action),
            ("do_not_disturb", IconCategory::Action),
            ("remove_circle_outline", IconCategory::Action),
            ("add_circle_outline", IconCategory::Action),
        ];

        for (name, category) in material_icons {
            let icon = Icon {
                name: name.to_string(),
                data: IconData::Custom(name.to_string()),
                size: 24.0,
                color: egui::Color32::BLACK,
                metadata: IconMetadata::new(category),
            };
            icon_set.icons.insert(name.to_string(), icon);
        }

        self.icon_sets.write().insert("material".to_string(), icon_set);
        Ok(())
    }

    fn load_font_awesome_icons(&mut self) -> Result<()> {
        let mut icon_set = IconSet {
            name: "fontawesome".to_string(),
            description: "Font Awesome icons".to_string(),
            version: "6.0".to_string(),
            author: "Font Awesome".to_string(),
            license: "CC BY 4.0".to_string(),
            icons: HashMap::new(),
            default_size: 16.0,
            default_color: egui::Color32::BLACK,
            metadata: IconSetMetadata::new(),
        };

        let font_awesome_icons = vec![
            ("home", IconCategory::Action),
            ("cog", IconCategory::Action),
            ("search", IconCategory::Action),
            ("bars", IconCategory::Navigation),
            ("times", IconCategory::Action),
            ("check", IconCategory::Action),
            ("exclamation-triangle", IconCategory::Alert),
            ("exclamation-circle", IconCategory::Alert),
            ("info-circle", IconCategory::Alert),
            ("star", IconCategory::Action),
            ("heart", IconCategory::Social),
            ("trash", IconCategory::Action),
            ("edit", IconCategory::Editor),
            ("save", IconCategory::Action),
            ("download", IconCategory::Action),
            ("upload", IconCategory::Action),
            ("folder", IconCategory::File),
            ("file", IconCategory::File),
            ("image", IconCategory::Image),
            ("video", IconCategory::Av),
            ("music", IconCategory::Av),
            ("play", IconCategory::Av),
            ("pause", IconCategory::Av),
            ("stop", IconCategory::Av),
            ("step-forward", IconCategory::Av),
            ("step-backward", IconCategory::Av),
            ("volume-up", IconCategory::Av),
            ("volume-down", IconCategory::Av),
            ("volume-mute", IconCategory::Av),
            ("expand", IconCategory::Action),
            ("compress", IconCategory::Action),
            ("sync", IconCategory::Action),
            ("copy", IconCategory::Action),
            ("paste", IconCategory::Action),
            ("cut", IconCategory::Action),
            ("undo", IconCategory::Editor),
            ("redo", IconCategory::Editor),
            ("bold", IconCategory::Editor),
            ("italic", IconCategory::Editor),
            ("underline", IconCategory::Editor),
            ("link", IconCategory::Communication),
            ("envelope", IconCategory::Communication),
            ("phone", IconCategory::Communication),
            ("calendar", IconCategory::Action),
            ("clock", IconCategory::Action),
            ("map-marker", IconCategory::Places),
            ("map", IconCategory::Maps),
            ("camera", IconCategory::Device),
            ("desktop", IconCategory::Device),
            ("mobile", IconCategory::Device),
            ("tablet", IconCategory::Device),
            ("print", IconCategory::Device),
            ("wifi", IconCategory::Device),
            ("bluetooth", IconCategory::Device),
            ("battery-full", IconCategory::Device),
            ("power-off", IconCategory::Device),
            ("lock", IconCategory::Action),
            ("unlock", IconCategory::Action),
            ("key", IconCategory::Action),
            ("eye", IconCategory::Action),
            ("eye-slash", IconCategory::Action),
            ("bell", IconCategory::Notification),
            ("bell-slash", IconCategory::Notification),
            ("bookmark", IconCategory::Action),
            ("flag", IconCategory::Action),
            ("tag", IconCategory::Action),
            ("filter", IconCategory::Action),
            ("sort", IconCategory::Action),
            ("th", IconCategory::Action),
            ("list", IconCategory::Action),
            ("th-large", IconCategory::Action),
            ("gift", IconCategory::Action),
            ("chart-bar", IconCategory::Content),
            ("chart-line", IconCategory::Content),
            ("database", IconCategory::Device),
            ("server", IconCategory::Device),
            ("cloud", IconCategory::Device),
            ("sun", IconCategory::Action),
            ("moon", IconCategory::Action),
            ("cloud-rain", IconCategory::Action),
            ("cloud-snow", IconCategory::Action),
            ("cloud-sun", IconCategory::Action),
            ("umbrella", IconCategory::Action),
            ("glasses", IconCategory::Device),
            ("headphones", IconCategory::Device),
            ("microphone", IconCategory::Device),
            ("speaker", IconCategory::Device),
            ("gamepad", IconCategory::Device),
            ("rocket", IconCategory::Action),
            ("plane", IconCategory::Action),
            ("car", IconCategory::Action),
            ("bus", IconCategory::Action),
            ("train", IconCategory::Action),
            ("bicycle", IconCategory::Action),
            ("walking", IconCategory::Action),
            ("running", IconCategory::Action),
            ("utensils", IconCategory::Action),
            ("coffee", IconCategory::Action),
            ("pizza-slice", IconCategory::Action),
            ("gift", IconCategory::Action),
            ("birthday-cake", IconCategory::Action),
            ("music", IconCategory::Av),
            ("palette", IconCategory::Action),
            ("paint-brush", IconCategory::Action),
            ("pen", IconCategory::Editor),
            ("eraser", IconCategory::Editor),
            ("scissors", IconCategory::Editor),
            ("ruler", IconCategory::Editor),
            ("compass", IconCategory::Maps),
            ("magnet", IconCategory::Device),
            ("atom", IconCategory::Science),
            ("flask", IconCategory::Science),
            ("microscope", IconCategory::Science),
            ("telescope", IconCategory::Science),
            ("dna", IconCategory::Science),
            ("pills", IconCategory::Medical),
            ("stethoscope", IconCategory::Medical),
            ("hospital", IconCategory::Places),
            ("graduation-cap", IconCategory::Education),
            ("book", IconCategory::Content),
            ("trophy", IconCategory::Action),
            ("medal", IconCategory::Action),
            ("crown", IconCategory::Action),
            ("gem", IconCategory::Action),
            ("money-bill", IconCategory::Action),
            ("credit-card", IconCategory::Action),
            ("shopping-cart", IconCategory::Action),
            ("shopping-bag", IconCategory::Action),
            ("store", IconCategory::Places),
            ("tag", IconCategory::Action),
            ("barcode", IconCategory::Action),
            ("qrcode", IconCategory::Action),
            ("fingerprint", IconCategory::Action),
            ("user", IconCategory::Social),
            ("smile", IconCategory::Social),
            ("frown", IconCategory::Social),
            ("angry", IconCategory::Social),
            ("surprise", IconCategory::Social),
            ("wink", IconCategory::Social),
            ("heart", IconCategory::Social),
            ("thumbs-up", IconCategory::Social),
            ("thumbs-down", IconCategory::Social),
            ("hand-paper", IconCategory::Action),
            ("wave", IconCategory::Social),
            ("thumbs-up", IconCategory::Social),
            ("thumbs-down", IconCategory::Social),
            ("hand-point-up", IconCategory::Action),
            ("ban", IconCategory::Action),
            ("hand", IconCategory::Action),
            ("hand-point-left", IconCategory::Action),
            ("hand-point-right", IconCategory::Action),
            ("angle-up", IconCategory::Navigation),
            ("angle-down", IconCategory::Navigation),
            ("angle-left", IconCategory::Navigation),
            ("angle-right", IconCategory::Navigation),
            ("caret-up", IconCategory::Navigation),
            ("caret-down", IconCategory::Navigation),
            ("caret-left", IconCategory::Navigation),
            ("caret-right", IconCategory::Navigation),
            ("plus", IconCategory::Action),
            ("minus", IconCategory::Action),
            ("times", IconCategory::Action),
            ("check-circle", IconCategory::Action),
            ("times-circle", IconCategory::Action),
            ("exclamation-triangle", IconCategory::Alert),
            ("minus-circle", IconCategory::Action),
            ("plus-circle", IconCategory::Action),
            ("ban", IconCategory::Action),
            ("times-circle", IconCategory::Action),
            ("plus-circle", IconCategory::Action),
        ];

        for (name, category) in font_awesome_icons {
            let icon = Icon {
                name: name.to_string(),
                data: IconData::Custom(name.to_string()),
                size: 16.0,
                color: egui::Color32::BLACK,
                metadata: IconMetadata::new(category),
            };
            icon_set.icons.insert(name.to_string(), icon);
        }

        self.icon_sets.write().insert("fontawesome".to_string(), icon_set);
        Ok(())
    }

    fn load_bootstrap_icons(&mut self) -> Result<()> {
        let mut icon_set = IconSet {
            name: "bootstrap".to_string(),
            description: "Bootstrap icons".to_string(),
            version: "1.0".to_string(),
            author: "Bootstrap".to_string(),
            license: "MIT".to_string(),
            icons: HashMap::new(),
            default_size: 16.0,
            default_color: egui::Color32::BLACK,
            metadata: IconSetMetadata::new(),
        };

        let bootstrap_icons = vec![
            ("house", IconCategory::Action),
            ("gear", IconCategory::Action),
            ("search", IconCategory::Action),
            ("list", IconCategory::Navigation),
            ("x", IconCategory::Action),
            ("check", IconCategory::Action),
            ("exclamation-triangle", IconCategory::Alert),
            ("exclamation-circle", IconCategory::Alert),
            ("info-circle", IconCategory::Alert),
            ("star", IconCategory::Action),
            ("heart", IconCategory::Social),
            ("trash", IconCategory::Action),
            ("pencil", IconCategory::Editor),
            ("save", IconCategory::Action),
            ("download", IconCategory::Action),
            ("upload", IconCategory::Action),
            ("folder", IconCategory::File),
            ("file", IconCategory::File),
            ("image", IconCategory::Image),
            ("play", IconCategory::Av),
            ("pause", IconCategory::Av),
            ("stop", IconCategory::Av),
            ("skip-forward", IconCategory::Av),
            ("skip-backward", IconCategory::Av),
            ("volume-up", IconCategory::Av),
            ("volume-down", IconCategory::Av),
            ("volume-mute", IconCategory::Av),
            ("arrows-fullscreen", IconCategory::Action),
            ("arrows-collapse", IconCategory::Action),
            ("arrow-clockwise", IconCategory::Action),
            ("files", IconCategory::Action),
            ("clipboard", IconCategory::Action),
            ("scissors", IconCategory::Editor),
            ("arrow-counterclockwise", IconCategory::Editor),
            ("type-bold", IconCategory::Editor),
            ("type-italic", IconCategory::Editor),
            ("type-underline", IconCategory::Editor),
            ("link", IconCategory::Communication),
            ("envelope", IconCategory::Communication),
            ("telephone", IconCategory::Communication),
            ("calendar", IconCategory::Action),
            ("clock", IconCategory::Action),
            ("geo-alt", IconCategory::Places),
            ("map", IconCategory::Maps),
            ("camera", IconCategory::Device),
            ("pc-display", IconCategory::Device),
            ("phone", IconCategory::Device),
            ("tablet", IconCategory::Device),
            ("printer", IconCategory::Device),
            ("wifi", IconCategory::Device),
            ("bluetooth", IconCategory::Device),
            ("battery-charging", IconCategory::Device),
            ("power", IconCategory::Device),
            ("lock", IconCategory::Action),
            ("unlock", IconCategory::Action),
            ("key", IconCategory::Action),
            ("eye", IconCategory::Action),
            ("eye-slash", IconCategory::Action),
            ("bell", IconCategory::Notification),
            ("bell-slash", IconCategory::Notification),
            ("bookmark", IconCategory::Action),
            ("flag", IconCategory::Action),
            ("tag", IconCategory::Action),
            ("funnel", IconCategory::Action),
            ("sort-down", IconCategory::Action),
            ("grid", IconCategory::Action),
            ("list-ul", IconCategory::Action),
            ("grid-3x3", IconCategory::Action),
            ("gift", IconCategory::Action),
            ("bar-chart", IconCategory::Content),
            ("graph-up", IconCategory::Content),
            ("server", IconCategory::Device),
            ("hdd", IconCategory::Device),
            ("cloud", IconCategory::Device),
            ("sun", IconCategory::Action),
            ("moon", IconCategory::Action),
            ("cloud-rain", IconCategory::Action),
            ("cloud-snow", IconCategory::Action),
            ("cloud-sun", IconCategory::Action),
            ("umbrella", IconCategory::Action),
            ("sunglasses", IconCategory::Device),
            ("headphones", IconCategory::Device),
            ("mic", IconCategory::Device),
            ("speaker", IconCategory::Device),
            ("controller", IconCategory::Device),
            ("rocket-takeoff", IconCategory::Action),
            ("airplane", IconCategory::Action),
            ("car-front", IconCategory::Action),
            ("bus-front", IconCategory::Action),
            ("train-front", IconCategory::Action),
            ("bicycle", IconCategory::Action),
            ("person-walking", IconCategory::Action),
            ("person-running", IconCategory::Action),
            ("cup-hot", IconCategory::Action),
            ("cup-straw", IconCategory::Action),
            ("pizza", IconCategory::Action),
            ("gift", IconCategory::Action),
            ("balloon", IconCategory::Action),
            ("music-note", IconCategory::Av),
            ("palette", IconCategory::Action),
            ("brush", IconCategory::Action),
            ("pen", IconCategory::Editor),
            ("eraser", IconCategory::Editor),
            ("scissors", IconCategory::Editor),
            ("ruler", IconCategory::Editor),
            ("compass", IconCategory::Maps),
            ("magnet", IconCategory::Device),
            ("cpu", IconCategory::Device),
            ("vial", IconCategory::Science),
            ("microscope", IconCategory::Science),
            ("binoculars", IconCategory::Science),
            ("dna", IconCategory::Science),
            ("capsule", IconCategory::Medical),
            ("stethoscope", IconCategory::Medical),
            ("hospital", IconCategory::Places),
            ("mortarboard", IconCategory::Education),
            ("book", IconCategory::Content),
            ("trophy", IconCategory::Action),
            ("award", IconCategory::Action),
            ("crown", IconCategory::Action),
            ("diamond", IconCategory::Action),
            ("cash", IconCategory::Action),
            ("credit-card", IconCategory::Action),
            ("cart", IconCategory::Action),
            ("bag", IconCategory::Action),
            ("shop", IconCategory::Places),
            ("tag", IconCategory::Action),
            ("upc", IconCategory::Action),
            ("qr-code", IconCategory::Action),
            ["fingerprint", IconCategory::Action],
            ["person", IconCategory::Social],
            ["emoji-smile", IconCategory::Social],
            ["emoji-frown", IconCategory::Social],
            ["emoji-angry", IconCategory::Social],
            ["emoji-surprise", IconCategory::Social],
            ["emoji-wink", IconCategory::Social],
            ["heart", IconCategory::Social],
            ["hand-thumbs-up", IconCategory::Social],
            ["hand-thumbs-down", IconCategory::Social],
            ["hand-index", IconCategory::Action],
            ["hand-wave", IconCategory::Social],
            ["hand-thumbs-up", IconCategory::Social],
            ["hand-thumbs-down", IconCategory::Social],
            ["hand-index", IconCategory::Action],
            ["shield-x", IconCategory::Action],
            ["hand-index", IconCategory::Action],
            ["hand-index", IconCategory::Action],
            ["caret-up", IconCategory::Navigation),
            ["caret-down", IconCategory::Navigation],
            ["caret-left", IconCategory::Navigation],
            ["caret-right", IconCategory::Navigation],
            ["plus", IconCategory::Action],
            ["dash", IconCategory::Action],
            ["x", IconCategory::Action],
            ["check-circle", IconCategory::Action],
            ["x-circle", IconCategory::Action],
            ["exclamation-triangle", IconCategory::Alert],
            ["dash-circle", IconCategory::Action],
            ["plus-circle", IconCategory::Action],
            ["shield-x", IconCategory::Action],
            ["x-circle", IconCategory::Action],
            ["plus-circle", IconCategory::Action],
        ];

        for (name, category) in bootstrap_icons {
            let icon = Icon {
                name: name.to_string(),
                data: IconData::Custom(name.to_string()),
                size: 16.0,
                color: egui::Color32::BLACK,
                metadata: IconMetadata::new(category),
            };
            icon_set.icons.insert(name.to_string(), icon);
        }

        self.icon_sets.write().insert("bootstrap".to_string(), icon_set);
        Ok(())
    }

    fn load_heroicons(&mut self) -> Result<()> {
        let mut icon_set = IconSet {
            name: "heroicons".to_string(),
            description: "Heroicons".to_string(),
            version: "2.0".to_string(),
            author: "Tailwind Labs".to_string(),
            license: "MIT".to_string(),
            icons: HashMap::new(),
            default_size: 24.0,
            default_color: egui::Color32::BLACK,
            metadata: IconSetMetadata::new(),
        };

        let heroicons = vec![
            ("home", IconCategory::Action),
            ("cog", IconCategory::Action),
            ("magnifying-glass", IconCategory::Action),
            ("bars", IconCategory::Navigation),
            ("x-mark", IconCategory::Action),
            ("check", IconCategory::Action),
            ("exclamation-triangle", IconCategory::Alert),
            ("exclamation-circle", IconCategory::Alert),
            ("information-circle", IconCategory::Alert),
            ("star", IconCategory::Action),
            ("heart", IconCategory::Social),
            ("trash", IconCategory::Action),
            ("pencil", IconCategory::Editor),
            ("bookmark", IconCategory::Action),
            ("arrow-down-tray", IconCategory::Action),
            ("arrow-up-tray", IconCategory::Action),
            ("folder", IconCategory::File),
            ("document", IconCategory::File),
            ("photo", IconCategory::Image),
            ("play", IconCategory::Av),
            ("pause", IconCategory::Av),
            ("stop", IconCategory::Av),
            ("forward", IconCategory::Av),
            ("backward", IconCategory::Av),
            ("speaker-wave", IconCategory::Av),
            ("speaker-x-mark", IconCategory::Av),
            ("arrows-pointing-out", IconCategory::Action),
            ("arrows-pointing-in", IconCategory::Action),
            ("arrow-path", IconCategory::Action),
            ("clipboard", IconCategory::Action),
            ("scissors", IconCategory::Editor),
            ("arrow-uturn-left", IconCategory::Editor),
            ("bold", IconCategory::Editor),
            ("italic", IconCategory::Editor),
            ("underline", IconCategory::Editor),
            ("link", IconCategory::Communication),
            ("envelope", IconCategory::Communication),
            ("phone", IconCategory::Communication),
            ("calendar", IconCategory::Action),
            ("clock", IconCategory::Action),
            ("map-pin", IconCategory::Places),
            ("map", IconCategory::Maps),
            ("camera", IconCategory::Device),
            ("computer-desktop", IconCategory::Device),
            ("device-phone-mobile", IconCategory::Device),
            ("device-tablet", IconCategory::Device),
            ("printer", IconCategory::Device),
            ["wifi", IconCategory::Device],
            ["bluetooth", IconCategory::Device],
            ["battery-charging", IconCategory::Device],
            ["power", IconCategory::Device],
            ["lock-closed", IconCategory::Action],
            ["lock-open", IconCategory::Action],
            ["key", IconCategory::Action],
            ["eye", IconCategory::Action],
            ["eye-slash", IconCategory::Action],
            ["bell", IconCategory::Notification],
            ["bell-slash", IconCategory::Notification],
            ["bookmark", IconCategory::Action],
            ["flag", IconCategory::Action],
            ["tag", IconCategory::Action],
            ["funnel", IconCategory::Action],
            ["bars-arrow-down", IconCategory::Action],
            ["squares", IconCategory::Action],
            ["list", IconCategory::Action],
            ["squares-2x2", IconCategory::Action),
            ["gift", IconCategory::Action],
            ["chart-bar", IconCategory::Content],
            ["chart-line", IconCategory::Content],
            ["server", IconCategory::Device],
            ["hard-drive", IconCategory::Device],
            ["cloud", IconCategory::Device],
            ["sun", IconCategory::Action],
            ["moon", IconCategory::Action],
            ["cloud-rain", IconCategory::Action],
            ["snowflake", IconCategory::Action],
            ["cloud-sun", IconCategory::Action],
            ["umbrella", IconCategory::Action],
            ["sunglasses", IconCategory::Device],
            ["headphones", IconCategory::Device],
            ["microphone", IconCategory::Device],
            ["speaker", IconCategory::Device],
            ["gamepad", IconCategory::Device],
            ["rocket", IconCategory::Action],
            ["airplane", IconCategory::Action],
            ["car", IconCategory::Action],
            ["bus", IconCategory::Action],
            ["train", IconCategory::Action],
            ["bicycle", IconCategory::Action],
            ["person-walking", IconCategory::Action],
            ["person-running", IconCategory::Action],
            ["coffee", IconCategory::Action],
            ["cake", IconCategory::Action],
            ["pizza", IconCategory::Action],
            ["gift", IconCategory::Action],
            ["balloon", IconCategory::Action],
            ["musical-note", IconCategory::Av],
            ["palette", IconCategory::Action],
            ["paint-brush", IconCategory::Action],
            ["pencil", IconCategory::Editor],
            ["eraser", IconCategory::Editor],
            ["scissors", IconCategory::Editor],
            ["ruler", IconCategory::Editor],
            ["compass", IconCategory::Maps],
            ["magnet", IconCategory::Device],
            ["cpu-chip", IconCategory::Device],
            ["test-tube", IconCategory::Science],
            ["microscope", IconCategory::Science],
            ["binoculars", IconCategory::Science],
            ["dna", IconCategory::Science],
            ["pills", IconCategory::Medical],
            ["stethoscope", IconCategory::Medical],
            ["hospital", IconCategory::Places],
            ["academic-cap", IconCategory::Education),
            ["book", IconCategory::Content],
            ["trophy", IconCategory::Action],
            ["award", IconCategory::Action],
            ["crown", IconCategory::Action],
            ["gem", IconCategory::Action],
            ["banknotes", IconCategory::Action],
            ["credit-card", IconCategory::Action],
            ["shopping-cart", IconCategory::Action],
            ["shopping-bag", IconCategory::Action],
            ["store", IconCategory::Places),
            ["tag", IconCategory::Action],
            ["qr-code", IconCategory::Action],
            ["fingerprint", IconCategory::Action],
            ["user", IconCategory::Social],
            ["face-smile", IconCategory::Social],
            ["face-frown", IconCategory::Social],
            ["face-angry", IconCategory::Social],
            ["face-surprise", IconCategory::Social],
            ["face-wink", IconCategory::Social],
            ["heart", IconCategory::Social],
            ["thumb-up", IconCategory::Social],
            ["thumb-down", IconCategory::Social],
            ["hand", IconCategory::Action],
            ["wave", IconCategory::Social],
            ["thumb-up", IconCategory::Social],
            ["thumb-down", IconCategory::Social],
            ["hand", IconCategory::Action],
            ["hand", IconCategory::Action],
            ["chevron-up", IconCategory::Navigation],
            ["chevron-down", IconCategory::Navigation),
            ["chevron-left", IconCategory::Navigation],
            ["chevron-right", IconCategory::Navigation],
            ["plus", IconCategory::Action],
            ["minus", IconCategory::Action],
            ["x-mark", IconCategory::Action],
            ["check-circle", IconCategory::Action],
            ["x-circle", IconCategory::Action],
            ["exclamation-triangle", IconCategory::Alert],
            ["minus-circle", IconCategory::Action],
            ["plus-circle", IconCategory::Action],
            ["no-symbol", IconCategory::Action],
            ["x-circle", IconCategory::Action],
            ["plus-circle", IconCategory::Action],
        ];

        for (name, category) in heroicons {
            let icon = Icon {
                name: name.to_string(),
                data: IconData::Custom(name.to_string()),
                size: 24.0,
                color: egui::Color32::BLACK,
                metadata: IconMetadata::new(category),
            };
            icon_set.icons.insert(name.to_string(), icon);
        }

        self.icon_sets.write().insert("heroicons".to_string(), icon_set);
        Ok(())
    }

    pub fn set_current_set(&mut self, set_name: &str) -> Result<()> {
        if self.icon_sets.read().contains_key(set_name) {
            *self.current_set.write() = set_name.to_string();
            Ok(())
        } else {
            Err(EllasticError::InvalidParameter(format!("Icon set '{}' not found", set_name)))
        }
    }

    pub fn get_current_set(&self) -> Option<String> {
        self.current_set.read().clone()
    }

    pub fn get_icon(&self, icon_name: &str) -> Option<Icon> {
        let current_set = self.current_set.read();
        if let Some(icon_set) = self.icon_sets.read().get(&*current_set) {
            icon_set.icons.get(icon_name).cloned()
        } else {
            None
        }
    }

    pub fn get_icon_from_set(&self, set_name: &str, icon_name: &str) -> Option<Icon> {
        if let Some(icon_set) = self.icon_sets.read().get(set_name) {
            icon_set.icons.get(icon_name).cloned()
        } else {
            None
        }
    }

    pub fn list_icons(&self) -> Vec<String> {
        let current_set = self.current_set.read();
        if let Some(icon_set) = self.icon_sets.read().get(&*current_set) {
            icon_set.icons.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn list_icons_from_set(&self, set_name: &str) -> Vec<String> {
        if let Some(icon_set) = self.icon_sets.read().get(set_name) {
            icon_set.icons.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn list_icon_sets(&self) -> Vec<String> {
        self.icon_sets.read().keys().cloned().collect()
    }

    pub fn render_icon(&self, ui: &mut egui::Ui, icon_name: &str, size: Option<f32>, color: Option<egui::Color32>) -> Result<()> {
        let icon = self.get_icon(icon_name)
            .ok_or_else(|| EllasticError::InvalidParameter(format!("Icon '{}' not found", icon_name)))?;

        let icon_size = size.unwrap_or(icon.size);
        let icon_color = color.unwrap_or(icon.color);

        match &icon.data {
            IconData::Unicode(unicode_char) => {
                let font_id = egui::FontId::monospace(icon_size);
                ui.label(egui::RichText::new(unicode_char.to_string()).font(font_id).color(icon_color));
            }
            IconData::Svg(svg_data) => {
                ui.label(&icon.name);
            }
            IconData::Png(png_data) => {
                ui.label(&icon.name);
            }
            IconData::Custom(custom_data) => {
                ui.label(&icon.name);
            }
        }

        Ok(())
    }

    pub fn render_icon_with_tooltip(&self, ui: &mut egui::Ui, icon_name: &str, tooltip: &str, size: Option<f32>, color: Option<egui::Color32>) -> Result<()> {
        let response = ui.allocate_exact_size(egui::vec2(size.unwrap_or(16.0), size.unwrap_or(16.0)), egui::Id::new(icon_name));

        if response.hovered() {
            egui::show_tooltip_at(ui, response.rect.center(), egui::Id::new(format!("tooltip_{}", icon_name)), |ui| {
                ui.label(tooltip);
            });
        }

        self.render_icon(ui, icon_name, size, color)
    }

    pub fn clone(&self) -> IconManager {
        IconManager {
            icons: self.icons.clone(),
            icon_sets: self.icon_sets.clone(),
            current_set: self.current_set.clone(),
            config: self.config.clone(),
        }
    }
}

impl IconMetadata {
    pub fn new(category: IconCategory) -> Self {
        Self {
            category,
            tags: Vec::new(),
            description: None,
            author: None,
            license: None,
            version: "1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn clone(&self) -> IconMetadata {
        IconMetadata {
            category: self.category,
            tags: self.tags.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            license: self.license.clone(),
            version: self.version.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl IconSetMetadata {
    pub fn new() -> Self {
        Self {
            icon_count: 0,
            categories: Vec::new(),
            supported_sizes: vec![12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 32.0, 48.0],
            supported_colors: vec![egui::Color32::BLACK, egui::Color32::WHITE],
            file_format: IconFormat::Unicode,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn clone(&self) -> IconSetMetadata {
        IconSetMetadata {
            icon_count: self.icon_count,
            categories: self.categories.clone(),
            supported_sizes: self.supported_sizes.clone(),
            supported_colors: self.supported_colors.clone(),
            file_format: self.file_format,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl Default for IconManagerConfig {
    fn default() -> Self {
        Self {
            default_set: "unicode".to_string(),
            default_size: 16.0,
            default_color: egui::Color32::BLACK,
            cache_enabled: true,
            cache_size: 1000,
            load_on_demand: false,
        }
    }
}

pub fn create_icon_manager(config: IconManagerConfig) -> IconManager {
    IconManager::new(config)
}

pub fn create_icon_manager_config() -> IconManagerConfig {
    IconManagerConfig::default()
}
