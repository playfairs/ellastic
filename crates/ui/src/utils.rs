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
use ellastic_export::{
  ExportManager,
  ExportRequest,
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
use ellastic_project::{
  Project,
  ProjectManager,
};
use ellastic_utils::create_random_generator;
use parking_lot::RwLock;
use rayon::prelude::*;
use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UIUtils {
  pub config: UIUtilsConfig,
}

#[derive(Debug, Clone)]
pub struct UIUtilsConfig {
  pub default_font_size: f32,
  pub default_spacing: f32,
  pub default_border_radius: f32,
  pub default_animation_duration: f32,
  pub enable_animations: bool,
  pub enable_tooltips: bool,
  pub enable_shortcuts: bool,
}

impl UIUtils {
  pub fn new(config: UIUtilsConfig) -> Self {
    Self { config }
  }

  pub fn config(&self) -> &UIUtilsConfig {
    &self.config
  }

  pub fn calculate_layout(&self, available_size: egui::Vec2, items: &[LayoutItem]) -> LayoutResult {
    let mut layout = LayoutResult::new();
    let mut current_pos = egui::pos2(0.0, 0.0);
    let mut row_height = 0.0;
    let mut current_row_items = Vec::new();

    for item in items {
      let item_size = self.calculate_item_size(item, available_size);

      if current_pos.x + item_size.x > available_size.x && !current_row_items.is_empty() {
        layout.rows.push(LayoutRow {
          items: current_row_items.clone(),
          y: current_pos.y,
          height: row_height,
        });
        current_pos.x = 0.0;
        current_pos.y += row_height + self.config.default_spacing;
        current_row_items.clear();
        row_height = 0.0;
      }

      let layout_item = LayoutItemResult {
        item: item.clone(),
        position: current_pos,
        size: item_size,
      };

      current_row_items.push(layout_item);
      row_height = row_height.max(item_size.y);
      current_pos.x += item_size.x + self.config.default_spacing;
    }

    if !current_row_items.is_empty() {
      layout.rows.push(LayoutRow {
        items: current_row_items,
        y: current_pos.y,
        height: row_height,
      });
    }

    layout.total_size = egui::vec2(available_size.x, current_pos.y + row_height);
    layout
  }

  fn calculate_item_size(&self, item: &LayoutItem, available_size: egui::Vec2) -> egui::Vec2 {
    match item.item_type {
      LayoutItemType::Button => {
        let width = item.width.unwrap_or(80.0);
        let height = item.height.unwrap_or(32.0);
        egui::vec2(width, height)
      }
      LayoutItemType::Input => {
        let width = item.width.unwrap_or(200.0);
        let height = item.height.unwrap_or(32.0);
        egui::vec2(width, height)
      }
      LayoutItemType::Label => {
        let width = item.width.unwrap_or(100.0);
        let height = item.height.unwrap_or(20.0);
        egui::vec2(width, height)
      }
      LayoutItemType::Icon => {
        let size = item.size.unwrap_or(16.0);
        egui::vec2(size, size)
      }
      LayoutItemType::Image => {
        let width = item.width.unwrap_or(128.0);
        let height = item.height.unwrap_or(128.0);
        egui::vec2(width, height)
      }
      LayoutItemType::Custom => {
        let width = item.width.unwrap_or(100.0);
        let height = item.height.unwrap_or(100.0);
        egui::vec2(width, height)
      }
    }
  }

  pub fn calculate_text_size(&self, text: &str, font_size: f32, max_width: f32) -> egui::Vec2 {
    let char_count = text.chars().count() as f32;
    let estimated_width = char_count * font_size * 0.6;

    let width = estimated_width.min(max_width);
    let height = font_size * 1.2;

    egui::vec2(width, height)
  }

  pub fn truncate_text(&self, text: &str, max_width: f32, font_size: f32) -> String {
    let ellipsis = "...";
    let ellipsis_width = ellipsis.len() as f32 * font_size * 0.6;

    if self.calculate_text_size(text, font_size, f32::MAX).x <= max_width {
      return text.to_string();
    }

    let mut truncated = String::new();
    let mut current_width = 0.0;

    for ch in text.chars() {
      let char_width = font_size * 0.6;
      if current_width + char_width + ellipsis_width > max_width {
        break;
      }
      truncated.push(ch);
      current_width += char_width;
    }

    truncated + ellipsis
  }

  pub fn format_file_size(&self, bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
      size /= 1024.0;
      unit_index += 1;
    }

    if unit_index == 0 {
      format!("{} {}", bytes, UNITS[unit_index])
    } else {
      format!("{:.1} {}", size, UNITS[unit_index])
    }
  }

  pub fn format_duration(&self, seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;

    if hours > 0 {
      format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else if minutes > 0 {
      format!("{:02}:{:02}", minutes, secs)
    } else {
      format!("{}s", secs)
    }
  }

  pub fn format_number(&self, number: f64, decimals: u32) -> String {
    if number.abs() >= 1_000_000.0 {
      format!("{:.1}M", number / 1_000_000.0)
    } else if number.abs() >= 1_000.0 {
      format!("{:.1}K", number / 1_000.0)
    } else {
      format!("{:.1$}", number, decimals)
    }
  }

  pub fn get_contrast_color(&self, background: egui::Color32) -> egui::Color32 {
    let luminance = self.calculate_luminance(background);
    if luminance > 0.5 {
      egui::Color32::BLACK
    } else {
      egui::Color32::WHITE
    }
  }

  fn calculate_luminance(&self, color: egui::Color32) -> f32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    0.2126 * r + 0.7152 * g + 0.0722 * b
  }

  pub fn blend_colors(
    &self,
    color1: egui::Color32,
    color2: egui::Color32,
    factor: f32,
  ) -> egui::Color32 {
    let r = (color1.r() as f32 * (1.0 - factor) + color2.r() as f32 * factor) as u8;
    let g = (color1.g() as f32 * (1.0 - factor) + color2.g() as f32 * factor) as u8;
    let b = (color1.b() as f32 * (1.0 - factor) + color2.b() as f32 * factor) as u8;
    let a = (color1.a() as f32 * (1.0 - factor) + color2.a() as f32 * factor) as u8;

    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
  }

  pub fn adjust_brightness(&self, color: egui::Color32, factor: f32) -> egui::Color32 {
    let r = (color.r() as f32 * factor).min(255.0) as u8;
    let g = (color.g() as f32 * factor).min(255.0) as u8;
    let b = (color.b() as f32 * factor).min(255.0) as u8;

    egui::Color32::from_rgb(r, g, b)
  }

  pub fn adjust_saturation(&self, color: egui::Color32, factor: f32) -> egui::Color32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let sum = max + min;

    let h = if delta == 0.0 {
      0.0
    } else if max == r {
      ((g - b) / delta) % 6.0
    } else if max == g {
      ((b - r) / delta) + 2.0
    } else {
      ((r - g) / delta) + 4.0
    };

    let l = sum / 2.0;
    let s = if sum == 0.0 {
      0.0
    } else {
      delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let new_s = (s * factor).min(1.0);
    let new_delta = if l <= 0.5 {
      2.0 * l * new_s
    } else {
      (2.0 - 2.0 * l) * new_s
    };

    let new_sum = sum;
    let new_max = (new_sum + new_delta) / 2.0;
    let new_min = (new_sum - new_delta) / 2.0;

    let (new_r, new_g, new_b) = if h < 1.0 {
      (new_max, new_min, new_min + new_delta * h)
    } else if h < 2.0 {
      (new_min + new_delta * (2.0 - h), new_max, new_min)
    } else if h < 3.0 {
      (new_min, new_min + new_delta * (h - 2.0), new_max)
    } else if h < 4.0 {
      (new_min + new_delta * (4.0 - h), new_min, new_max)
    } else if h < 5.0 {
      (new_max, new_min + new_delta * (h - 4.0), new_min)
    } else {
      (new_min, new_max, new_min + new_delta * (6.0 - h))
    };

    egui::Color32::from_rgb(
      (new_r * 255.0) as u8,
      (new_g * 255.0) as u8,
      (new_b * 255.0) as u8,
    )
  }

  pub fn hex_to_color(&self, hex: &str) -> Result<egui::Color32> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 && hex.len() != 8 {
      return Err(EllasticError::InvalidParameter(
        "Invalid hex color format".to_string(),
      ));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
      .map_err(|_| EllasticError::InvalidParameter("Invalid hex color".to_string()))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
      .map_err(|_| EllasticError::InvalidParameter("Invalid hex color".to_string()))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
      .map_err(|_| EllasticError::InvalidParameter("Invalid hex color".to_string()))?;

    let a = if hex.len() == 8 {
      u8::from_str_radix(&hex[6..8], 16)
        .map_err(|_| EllasticError::InvalidParameter("Invalid hex color".to_string()))?
    } else {
      255
    };

    Ok(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
  }

  pub fn color_to_hex(&self, color: egui::Color32) -> String {
    format!(
      "#{:02X}{:02X}{:02X}{:02X}",
      color.r(),
      color.g(),
      color.b(),
      color.a()
    )
  }

  pub fn interpolate_color(
    &self,
    color1: egui::Color32,
    color2: egui::Color32,
    t: f32,
  ) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    self.blend_colors(color1, color2, t)
  }

  pub fn generate_gradient(
    &self,
    start_color: egui::Color32,
    end_color: egui::Color32,
    steps: usize,
  ) -> Vec<egui::Color32> {
    let mut gradient = Vec::new();

    for i in 0..steps {
      let t = i as f32 / (steps - 1) as f32;
      gradient.push(self.interpolate_color(start_color, end_color, t));
    }

    gradient
  }

  pub fn calculate_optimal_text_scale(
    &self,
    text: &str,
    available_width: f32,
    available_height: f32,
    font_size: f32,
  ) -> f32 {
    let text_size = self.calculate_text_size(text, font_size, available_width);

    if text_size.x <= available_width && text_size.y <= available_height {
      return 1.0;
    }

    let width_scale = available_width / text_size.x;
    let height_scale = available_height / text_size.y;

    width_scale.min(height_scale).max(0.5)
  }

  pub fn wrap_text(&self, text: &str, max_width: f32, font_size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0.0;
    let char_width = font_size * 0.6;

    for word in text.split_whitespace() {
      let word_width = word.len() as f32 * char_width;

      if current_width + word_width > max_width && !current_line.is_empty() {
        lines.push(current_line.clone());
        current_line.clear();
        current_width = 0.0;
      }

      if !current_line.is_empty() {
        current_line.push(' ');
        current_width += char_width;
      }

      current_line.push_str(word);
      current_width += word_width;
    }

    if !current_line.is_empty() {
      lines.push(current_line);
    }

    lines
  }

  pub fn calculate_scrollbar_size(&self, content_size: f32, viewport_size: f32) -> f32 {
    if content_size <= viewport_size {
      return viewport_size;
    }

    let scrollbar_size = (viewport_size * viewport_size / content_size).max(20.0);
    scrollbar_size.min(viewport_size)
  }

  pub fn calculate_scroll_position(
    &self,
    content_size: f32,
    viewport_size: f32,
    scroll_position: f32,
  ) -> f32 {
    if content_size <= viewport_size {
      return 0.0;
    }

    let max_scroll = content_size - viewport_size;
    scroll_position.clamp(0.0, max_scroll)
  }

  pub fn point_in_rect(&self, point: egui::Pos2, rect: egui::Rect) -> bool {
    point.x >= rect.min.x && point.x <= rect.max.x && point.y >= rect.min.y && point.y <= rect.max.y
  }

  pub fn rect_contains_rect(&self, outer: egui::Rect, inner: egui::Rect) -> bool {
    outer.min.x <= inner.min.x
      && outer.max.x >= inner.max.x
      && outer.min.y <= inner.min.y
      && outer.max.y >= inner.max.y
  }

  pub fn rect_intersect(&self, rect1: egui::Rect, rect2: egui::Rect) -> Option<egui::Rect> {
    let min_x = rect1.min.x.max(rect2.min.x);
    let min_y = rect1.min.y.max(rect2.min.y);
    let max_x = rect1.max.x.min(rect2.max.x);
    let max_y = rect1.max.y.min(rect2.max.y);

    if min_x <= max_x && min_y <= max_y {
      Some(egui::Rect::from_min_max(
        egui::pos2(min_x, min_y),
        egui::pos2(max_x, max_y),
      ))
    } else {
      None
    }
  }

  pub fn rect_union(&self, rect1: egui::Rect, rect2: egui::Rect) -> egui::Rect {
    let min_x = rect1.min.x.min(rect2.min.x);
    let min_y = rect1.min.y.min(rect2.min.y);
    let max_x = rect1.max.x.max(rect2.max.x);
    let max_y = rect1.max.y.max(rect2.max.y);

    egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y))
  }

  pub fn inflate_rect(&self, rect: egui::Rect, amount: f32) -> egui::Rect {
    egui::Rect::from_min_max(
      egui::pos2(rect.min.x - amount, rect.min.y - amount),
      egui::pos2(rect.max.x + amount, rect.max.y + amount),
    )
  }

  pub fn deflate_rect(&self, rect: egui::Rect, amount: f32) -> egui::Rect {
    egui::Rect::from_min_max(
      egui::pos2(rect.min.x + amount, rect.min.y + amount),
      egui::pos2(rect.max.x - amount, rect.max.y - amount),
    )
  }

  pub fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
  }

  pub fn smooth_step(&self, edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
  }

  pub fn clamp(&self, value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
  }

  pub fn map_range(
    &self,
    value: f32,
    from_min: f32,
    from_max: f32,
    to_min: f32,
    to_max: f32,
  ) -> f32 {
    let normalized = (value - from_min) / (from_max - from_min);
    to_min + normalized * (to_max - to_min)
  }

  pub fn ease_in_out(&self, t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
  }

  pub fn ease_in(&self, t: f32) -> f32 {
    t * t
  }

  pub fn ease_out(&self, t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
  }

  pub fn ease_in_cubic(&self, t: f32) -> f32 {
    t * t * t
  }

  pub fn ease_out_cubic(&self, t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
  }

  pub fn ease_in_out_cubic(&self, t: f32) -> f32 {
    if t < 0.5 {
      4.0 * t * t * t
    } else {
      1.0 - ((-2.0 * t + 2.0).powi(3) / 2.0)
    }
  }

  pub fn random_color(&self) -> egui::Color32 {
    let mut rng = create_random_generator();
    let r = rng.gen_range(0..256);
    let g = rng.gen_range(0..256);
    let b = rng.gen_range(0..256);

    egui::Color32::from_rgb(r as u8, g as u8, b as u8)
  }

  pub fn random_color_with_alpha(&self, alpha: u8) -> egui::Color32 {
    let mut rng = create_random_generator();
    let r = rng.gen_range(0..256);
    let g = rng.gen_range(0..256);
    let b = rng.gen_range(0..256);

    egui::Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, alpha)
  }

  pub fn generate_palette(&self, base_color: egui::Color32, count: usize) -> Vec<egui::Color32> {
    let mut palette = Vec::new();

    for i in 0..count {
      let t = i as f32 / (count - 1) as f32;
      let hue_shift = t * 360.0;
      let adjusted_color = self.adjust_hue(base_color, hue_shift);
      palette.push(adjusted_color);
    }

    palette
  }

  fn adjust_hue(&self, color: egui::Color32, hue_shift: f32) -> egui::Color32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let sum = max + min;

    let h = if delta == 0.0 {
      0.0
    } else if max == r {
      ((g - b) / delta) % 6.0
    } else if max == g {
      ((b - r) / delta) + 2.0
    } else {
      ((r - g) / delta) + 4.0
    };

    let l = sum / 2.0;
    let s = if sum == 0.0 {
      0.0
    } else {
      delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let new_h = (h + hue_shift / 60.0) % 6.0;

    let new_delta = if l <= 0.5 {
      2.0 * l * s
    } else {
      (2.0 - 2.0 * l) * s
    };

    let new_sum = sum;
    let new_max = (new_sum + new_delta) / 2.0;
    let new_min = (new_sum - new_delta) / 2.0;

    let (new_r, new_g, new_b) = if new_h < 1.0 {
      (new_max, new_min, new_min + new_delta * new_h)
    } else if new_h < 2.0 {
      (new_min + new_delta * (2.0 - new_h), new_max, new_min)
    } else if new_h < 3.0 {
      (new_min, new_min + new_delta * (new_h - 2.0), new_max)
    } else if new_h < 4.0 {
      (new_min + new_delta * (4.0 - new_h), new_min, new_max)
    } else if new_h < 5.0 {
      (new_max, new_min + new_delta * (new_h - 4.0), new_min)
    } else {
      (new_min, new_max, new_min + new_delta * (6.0 - new_h))
    };

    egui::Color32::from_rgb(
      (new_r * 255.0) as u8,
      (new_g * 255.0) as u8,
      (new_b * 255.0) as u8,
    )
  }

  pub fn create_animation(&self, duration: f32, easing: AnimationEasing) -> Animation {
    Animation {
      duration,
      easing,
      start_time: Utc::now(),
      current_time: Utc::now(),
      value: 0.0,
      running: false,
      completed: false,
    }
  }

  pub fn update_animation(&self, animation: &mut Animation) {
    if !animation.running {
      return;
    }

    let now = Utc::now();
    let elapsed = (now - animation.start_time).num_milliseconds() as f32 / 1000.0;

    if elapsed >= animation.duration {
      animation.value = 1.0;
      animation.completed = true;
      animation.running = false;
    } else {
      let t = elapsed / animation.duration;
      animation.value = match animation.easing {
        AnimationEasing::Linear => t,
        AnimationEasing::EaseIn => self.ease_in(t),
        AnimationEasing::EaseOut => self.ease_out(t),
        AnimationEasing::EaseInOut => self.ease_in_out(t),
        AnimationEasing::EaseInCubic => self.ease_in_cubic(t),
        AnimationEasing::EaseOutCubic => self.ease_out_cubic(t),
        AnimationEasing::EaseInOutCubic => self.ease_in_out_cubic(t),
      };
    }

    animation.current_time = now;
  }

  pub fn start_animation(&self, animation: &mut Animation) {
    animation.start_time = Utc::now();
    animation.current_time = Utc::now();
    animation.value = 0.0;
    animation.running = true;
    animation.completed = false;
  }

  pub fn reset_animation(&self, animation: &mut Animation) {
    animation.start_time = Utc::now();
    animation.current_time = Utc::now();
    animation.value = 0.0;
    animation.running = false;
    animation.completed = false;
  }

  pub fn clone(&self) -> UIUtils {
    UIUtils {
      config: self.config.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
  pub rows: Vec<LayoutRow>,
  pub total_size: egui::Vec2,
}

#[derive(Debug, Clone)]
pub struct LayoutRow {
  pub items: Vec<LayoutItemResult>,
  pub y: f32,
  pub height: f32,
}

#[derive(Debug, Clone)]
pub struct LayoutItemResult {
  pub item: LayoutItem,
  pub position: egui::Pos2,
  pub size: egui::Vec2,
}

#[derive(Debug, Clone)]
pub struct LayoutItem {
  pub id: String,
  pub item_type: LayoutItemType,
  pub content: String,
  pub width: Option<f32>,
  pub height: Option<f32>,
  pub size: Option<f32>,
  pub flex: Option<f32>,
  pub margin: Option<f32>,
  pub padding: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutItemType {
  Button,
  Input,
  Label,
  Icon,
  Image,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationEasing {
  Linear,
  EaseIn,
  EaseOut,
  EaseInOut,
  EaseInCubic,
  EaseOutCubic,
  EaseInOutCubic,
}

#[derive(Debug, Clone)]
pub struct Animation {
  pub duration: f32,
  pub easing: AnimationEasing,
  pub start_time: DateTime<Utc>,
  pub current_time: DateTime<Utc>,
  pub value: f32,
  pub running: bool,
  pub completed: bool,
}

impl LayoutResult {
  pub fn new() -> Self {
    Self {
      rows: Vec::new(),
      total_size: egui::vec2(0.0, 0.0),
    }
  }

  pub fn clone(&self) -> LayoutResult {
    LayoutResult {
      rows: self.rows.clone(),
      total_size: self.total_size,
    }
  }
}

impl Animation {
  pub fn clone(&self) -> Animation {
    Animation {
      duration: self.duration,
      easing: self.easing,
      start_time: self.start_time,
      current_time: self.current_time,
      value: self.value,
      running: self.running,
      completed: self.completed,
    }
  }
}

impl Default for UIUtilsConfig {
  fn default() -> Self {
    Self {
      default_font_size: 14.0,
      default_spacing: 8.0,
      default_border_radius: 4.0,
      default_animation_duration: 0.2,
      enable_animations: true,
      enable_tooltips: true,
      enable_shortcuts: true,
    }
  }
}

pub fn create_ui_utils(config: UIUtilsConfig) -> UIUtils {
  UIUtils::new(config)
}

pub fn create_ui_utils_config() -> UIUtilsConfig {
  UIUtilsConfig::default()
}

pub fn create_layout_item(
  id: String,
  item_type: LayoutItemType,
  content: String,
  width: Option<f32>,
  height: Option<f32>,
) -> LayoutItem {
  LayoutItem {
    id,
    item_type,
    content,
    width,
    height,
    size: None,
    flex: None,
    margin: None,
    padding: None,
  }
}

pub fn create_animation(duration: f32, easing: AnimationEasing) -> Animation {
  Animation {
    duration,
    easing,
    start_time: Utc::now(),
    current_time: Utc::now(),
    value: 0.0,
    running: false,
    completed: false,
  }
}
