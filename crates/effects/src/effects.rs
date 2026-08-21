use chrono::{
  DateTime,
  Utc,
};
use ellastic_audio::{
  AudioData,
  AudioProcessor,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_image::{
  ImageData,
  ImageProcessor,
};
use ellastic_media::MediaProcessor;
use ellastic_utils::{
  NoiseGenerator,
  create_noise_generator_with_seed,
  create_random_generator,
};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelSortMode {
  Brightness,
  Hue,
  Saturation,
  Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
  Add,
  Multiply,
  Screen,
  Overlay,
  Difference,
  ColorBurn,
  ColorDodge,
  HardLight,
  SoftLight,
  Exclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
  LowPass,
  HighPass,
  BandPass,
  Notch,
  Peak,
  LowShelf,
  HighShelf,
  AllPass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformType {
  Rotate,
  Scale,
  Translate,
  Skew,
  Perspective,
  Affine,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
  RGB,
  HSL,
  HSV,
  LAB,
  XYZ,
  CMYK,
  YUV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationType {
  Nearest,
  Linear,
  Cubic,
  Lanczos,
  Spline,
}

#[derive(Debug, Clone)]
pub struct ImageEffectProcessor {
  processor: ImageProcessor,
  random_seed: Option<u64>,
}

impl ImageEffectProcessor {
  pub fn new(processor: ImageProcessor) -> Self {
    Self {
      processor,
      random_seed: None,
    }
  }

  pub fn with_seed(processor: ImageProcessor, seed: u64) -> Self {
    Self {
      processor,
      random_seed: Some(seed),
    }
  }

  pub fn processor(&self) -> &ImageProcessor {
    &self.processor
  }

  pub fn processor_mut(&mut self) -> &mut ImageProcessor {
    &mut self.processor
  }

  pub fn into_processor(self) -> ImageProcessor {
    self.processor
  }

  pub fn random_seed(&self) -> Option<u64> {
    self.random_seed
  }

  pub fn set_random_seed(&mut self, seed: u64) {
    self.random_seed = Some(seed);
  }

  pub fn apply_brightness(&mut self, amount: f32) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      if pixel.len() >= 3 {
        for channel in pixel.iter_mut().take(3) {
          let adjusted = (*channel as f32 * (1.0 + amount)).clamp(0.0, 255.0);
          *channel = adjusted as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_contrast(&mut self, amount: f32) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    let factor = 1.0 + amount;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      if pixel.len() >= 3 {
        for channel in pixel.iter_mut().take(3) {
          let adjusted = ((*channel as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0);
          *channel = adjusted as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_saturation(&mut self, amount: f32) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    let factor = 1.0 + amount;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      if pixel.len() >= 3 {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        let gray = 0.299 * r + 0.587 * g + 0.114 * b;

        pixel[0] = (gray + (r - gray) * factor * 255.0).clamp(0.0, 255.0) as u8;
        pixel[1] = (gray + (g - gray) * factor * 255.0).clamp(0.0, 255.0) as u8;
        pixel[2] = (gray + (b - gray) * factor * 255.0).clamp(0.0, 255.0) as u8;
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_gamma(&mut self, gamma: f32) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    let gamma_correction = 1.0 / gamma;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      if pixel.len() >= 3 {
        for channel in pixel.iter_mut().take(3) {
          let normalized = *channel as f32 / 255.0;
          let corrected = normalized.powf(gamma_correction) * 255.0;
          *channel = corrected.clamp(0.0, 255.0) as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_blur(&mut self, radius: f32) -> Result<()> {
    let image_data = self.processor.data();
    let kernel_size = (radius * 2.0).ceil() as usize;
    let kernel = self.create_gaussian_kernel(kernel_size, radius);

    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let channels = image_data.channels as usize;

    for y in 0..height {
      for x in 0..width {
        let mut sum = [0.0; 4];
        let mut weight_sum = 0.0;

        for ky in 0..kernel_size {
          for kx in 0..kernel_size {
            let px =
              (x as i32 + kx as i32 - kernel_size as i32 / 2).clamp(0, width as i32 - 1) as u32;
            let py =
              (y as i32 + ky as i32 - kernel_size as i32 / 2).clamp(0, height as i32 - 1) as u32;

            let pixel_index = (py * width + px) * channels as u32;
            if pixel_index + channels as u32 <= data.len() as u32 {
              let pixel_start = pixel_index as usize;
              let weight = kernel[ky * kernel_size + kx];

              for c in 0..channels.min(4) {
                sum[c] += data[pixel_start + c] as f32 * weight;
              }
              weight_sum += weight;
            }
          }
        }

        let pixel_index = (y * width + x) * channels as u32;
        let pixel_start = pixel_index as usize;

        for c in 0..channels.min(4) {
          data[pixel_start + c] = (sum[c] / weight_sum).clamp(0.0, 255.0) as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_sharpen(&mut self, amount: f32) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let channels = image_data.channels as usize;

    let kernel = [
      0.0,
      -amount,
      0.0,
      -amount,
      1.0 + 4.0 * amount,
      -amount,
      0.0,
      -amount,
      0.0,
    ];

    for y in 1..height - 1 {
      for x in 1..width - 1 {
        let pixel_index = (y * width + x) * channels as u32;
        let pixel_start = pixel_index as usize;

        for c in 0..channels.min(3) {
          let mut sum = 0.0;

          for ky in 0..3 {
            for kx in 0..3 {
              let px = x + kx - 1;
              let py = y + ky - 1;
              let neighbor_index = (py * width + px) * channels as u32;
              let neighbor_start = neighbor_index as usize;

              sum += data[neighbor_start + c] as f32 * kernel[ky * 3 + kx];
            }
          }

          data[pixel_start + c] = sum.clamp(0.0, 255.0) as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_edge_detection(&mut self) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let channels = image_data.channels as usize;

    let sobel_x = [-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0];

    let sobel_y = [-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0];

    for y in 1..height - 1 {
      for x in 1..width - 1 {
        let pixel_index = (y * width + x) * channels as u32;
        let pixel_start = pixel_index as usize;

        for c in 0..channels.min(3) {
          let mut gx = 0.0;
          let mut gy = 0.0;

          for ky in 0..3 {
            for kx in 0..3 {
              let px = x + kx - 1;
              let py = y + ky - 1;
              let neighbor_index = (py * width + px) * channels as u32;
              let neighbor_start = neighbor_index as usize;

              gx += data[neighbor_start + c] as f32 * sobel_x[ky * 3 + kx];
              gy += data[neighbor_start + c] as f32 * sobel_y[ky * 3 + kx];
            }
          }

          let magnitude = (gx * gx + gy * gy).sqrt();
          data[pixel_start + c] = magnitude.clamp(0.0, 255.0) as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_emboss(&mut self) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let channels = image_data.channels as usize;

    let kernel = [-2.0, -1.0, 0.0, -1.0, 1.0, 1.0, 0.0, 1.0, 2.0];

    for y in 1..height - 1 {
      for x in 1..width - 1 {
        let pixel_index = (y * width + x) * channels as u32;
        let pixel_start = pixel_index as usize;

        for c in 0..channels.min(3) {
          let mut sum = 0.0;

          for ky in 0..3 {
            for kx in 0..3 {
              let px = x + kx - 1;
              let py = y + ky - 1;
              let neighbor_index = (py * width + px) * channels as u32;
              let neighbor_start = neighbor_index as usize;

              sum += data[neighbor_start + c] as f32 * kernel[ky * 3 + kx];
            }
          }

          data[pixel_start + c] = (sum + 128.0).clamp(0.0, 255.0) as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_pixel_sort(&mut self, threshold: f32, mode: PixelSortMode) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let channels = image_data.channels as usize;

    let mut pixels = Vec::new();

    for y in 0..height {
      for x in 0..width {
        let pixel_index = (y * width + x) * channels as u32;
        let pixel_start = pixel_index as usize;

        if pixel_start + channels <= data.len() {
          let pixel_data = data[pixel_start..pixel_start + channels].to_vec();
          let brightness = pixel_data.iter().take(3).sum::<u8>() as f32 / 3.0;

          if brightness >= threshold {
            pixels.push((x, y, pixel_data));
          }
        }
      }
    }

    match mode {
      PixelSortMode::Brightness => {
        pixels.sort_by(|_, a, b| {
          let brightness_a = a.2.iter().take(3).sum::<u8>() as f32 / 3.0;
          let brightness_b = b.2.iter().take(3).sum::<u8>() as f32 / 3.0;
          brightness_a
            .partial_cmp(&brightness_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        });
      }
      PixelSortMode::Hue => {
        pixels.sort_by(|_, a, b| {
          let hue_a = self.calculate_hue(&a.2);
          let hue_b = self.calculate_hue(&b.2);
          hue_a
            .partial_cmp(&hue_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        });
      }
      PixelSortMode::Saturation => {
        pixels.sort_by(|_, a, b| {
          let sat_a = self.calculate_saturation(&a.2);
          let sat_b = self.calculate_saturation(&b.2);
          sat_a
            .partial_cmp(&sat_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        });
      }
      PixelSortMode::Random => {
        let mut rng = create_random_generator();
        if let Some(seed) = self.random_seed {
          rng.set_seed(seed);
        }
        rng.shuffle(&mut pixels);
      }
    }

    for (x, y, pixel_data) in pixels {
      let pixel_index = (y * width + x) * channels as u32;
      let pixel_start = pixel_index as usize;

      if pixel_start + channels <= data.len() {
        data[pixel_start..pixel_start + channels].copy_from_slice(&pixel_data);
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_data_mosh(&mut self, intensity: f32, preserve_size: bool) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    let corruption_count = (data.len() as f32 * intensity) as usize;
    let mut rng = create_random_generator();

    if let Some(seed) = self.random_seed {
      rng.set_seed(seed);
    }

    for _ in 0..corruption_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      if pos < data.len() {
        data[pos] = rng.gen_range(0, 256) as u8;
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_grayscale(&mut self) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      if pixel.len() >= 3 {
        let gray =
          (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8;
        pixel[0] = gray;
        pixel[1] = gray;
        pixel[2] = gray;
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_sepia(&mut self) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      if pixel.len() >= 3 {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;

        pixel[0] = (r * 0.393 + g * 0.769 + b * 0.189).clamp(0.0, 255.0) as u8;
        pixel[1] = (r * 0.349 + g * 0.686 + b * 0.168).clamp(0.0, 255.0) as u8;
        pixel[2] = (r * 0.272 + g * 0.534 + b * 0.131).clamp(0.0, 255.0) as u8;
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_invert(&mut self) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      for channel in pixel.iter_mut().take(3) {
        *channel = 255 - *channel;
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_hue_rotate(&mut self, angle: f32) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      if pixel.len() >= 3 {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        let (h, s, l) = self.rgb_to_hsl(r, g, b);
        let (r_new, g_new, b_new) = self.hsl_to_rgb((h + angle / 360.0).fract(), s, l);

        pixel[0] = (r_new * 255.0) as u8;
        pixel[1] = (g_new * 255.0) as u8;
        pixel[2] = (b_new * 255.0) as u8;
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_colorize(&mut self, r: u8, g: u8, b: u8, a: u8) -> Result<()> {
    let image_data = self.processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    let color_r = r as f32 / 255.0;
    let color_g = g as f32 / 255.0;
    let color_b = b as f32 / 255.0;
    let color_a = a as f32 / 255.0;

    for pixel in data.chunks_mut(image_data.channels as usize) {
      if pixel.len() >= 3 {
        let gray = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0 / 255.0;

        pixel[0] = (gray * color_r * 255.0) as u8;
        pixel[1] = (gray * color_g * 255.0) as u8;
        pixel[2] = (gray * color_b * 255.0) as u8;

        if pixel.len() >= 4 {
          pixel[3] = (gray * color_a * 255.0) as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_blend(
    &mut self,
    overlay: &ImageProcessor,
    mode: BlendMode,
    mix_ratio: f32,
  ) -> Result<()> {
    let image_data = self.processor.data();
    let overlay_data = overlay.data();

    if image_data.width != overlay_data.width || image_data.height != overlay_data.height {
      return Err(EllasticError::InvalidParameter(
        "Image dimensions must match for blending".to_string(),
      ));
    }

    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let overlay_pixels = &overlay_data.data;

    for (i, pixel) in data.chunks_mut(image_data.channels as usize).enumerate() {
      let overlay_start = i * image_data.channels as usize;

      if overlay_start + image_data.channels as usize <= overlay_pixels.len() {
        let overlay_pixel =
          &overlay_pixels[overlay_start..overlay_start + image_data.channels as usize];

        for c in 0..pixel.len().min(overlay_pixel.len()).min(3) {
          let base = pixel[c] as f32 / 255.0;
          let overlay = overlay_pixel[c] as f32 / 255.0;

          let blended = match mode {
            BlendMode::Add => (base + overlay).min(1.0),
            BlendMode::Multiply => base * overlay,
            BlendMode::Screen => 1.0 - (1.0 - base) * (1.0 - overlay),
            BlendMode::Overlay => {
              if base < 0.5 {
                2.0 * base * overlay
              } else {
                1.0 - 2.0 * (1.0 - base) * (1.0 - overlay)
              }
            }
            BlendMode::Difference => (base - overlay).abs(),
            BlendMode::ColorBurn => {
              if overlay > 0.0 {
                1.0 - (1.0 - base) / overlay
              } else {
                0.0
              }
            }
            BlendMode::ColorDodge => {
              if overlay < 1.0 {
                base / (1.0 - overlay)
              } else {
                1.0
              }
            }
            BlendMode::HardLight => {
              if overlay < 0.5 {
                2.0 * base * overlay
              } else {
                1.0 - 2.0 * (1.0 - base) * (1.0 - overlay)
              }
            }
            BlendMode::SoftLight => {
              if overlay < 0.5 {
                base - (1.0 - 2.0 * overlay) * base * (1.0 - base)
              } else {
                base + (2.0 * overlay - 1.0) * (self.sqrt(base) - base)
              }
            }
            BlendMode::Exclusion => base + overlay - 2.0 * base * overlay,
          };

          pixel[c] = (blended * 255.0 * mix_ratio + base * 255.0 * (1.0 - mix_ratio)) as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_composite(&mut self, composite: &ImageProcessor) -> Result<()> {
    let image_data = self.processor.data();
    let composite_data = composite.data();

    if image_data.width != composite_data.width || image_data.height != composite_data.height {
      return Err(EllasticError::InvalidParameter(
        "Image dimensions must match for compositing".to_string(),
      ));
    }

    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let composite_pixels = &composite_data.data;

    for (i, pixel) in data.chunks_mut(image_data.channels as usize).enumerate() {
      let composite_start = i * image_data.channels as usize;

      if composite_start + image_data.channels as usize <= composite_pixels.len() {
        let composite_pixel =
          &composite_pixels[composite_start..composite_start + image_data.channels as usize];

        for c in 0..pixel.len().min(composite_pixel.len()) {
          if composite_pixel[c] > 0 {
            pixel[c] = composite_pixel[c];
          }
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  pub fn apply_mask(&mut self, mask: &ImageProcessor) -> Result<()> {
    let image_data = self.processor.data();
    let mask_data = mask.data();

    if image_data.width != mask_data.width || image_data.height != mask_data.height {
      return Err(EllasticError::InvalidParameter(
        "Image dimensions must match for masking".to_string(),
      ));
    }

    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;
    let mask_pixels = &mask_data.data;

    for (i, pixel) in data.chunks_mut(image_data.channels as usize).enumerate() {
      let mask_start = i * mask_data.channels as usize;

      if mask_start < mask_pixels.len() {
        let mask_value = mask_pixels[mask_start] as f32 / 255.0;

        for c in 0..pixel.len() {
          pixel[c] = (pixel[c] as f32 * mask_value) as u8;
        }
      }
    }

    self.processor = ImageProcessor::from_image_data(new_image_data);
    Ok(())
  }

  fn create_gaussian_kernel(&self, size: usize, sigma: f32) -> Vec<f32> {
    let mut kernel = vec![0.0; size * size];
    let center = size as f32 / 2.0;
    let mut sum = 0.0;

    for y in 0..size {
      for x in 0..size {
        let dx = x as f32 - center;
        let dy = y as f32 - center;
        let distance = (dx * dx + dy * dy) / (2.0 * sigma * sigma);
        kernel[y * size + x] = (-distance).exp();
        sum += kernel[y * size + x];
      }
    }

    for value in kernel.iter_mut() {
      *value /= sum;
    }

    kernel
  }

  fn calculate_hue(&self, pixel: &[u8]) -> f32 {
    if pixel.len() < 3 {
      return 0.0;
    }

    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta == 0.0 {
      return 0.0;
    }

    let hue = if max == r {
      ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
      ((b - r) / delta + 2.0) / 6.0
    } else {
      ((r - g) / delta + 4.0) / 6.0
    };

    hue
  }

  fn calculate_saturation(&self, pixel: &[u8]) -> f32 {
    if pixel.len() < 3 {
      return 0.0;
    }

    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta == 0.0 {
      return 0.0;
    }

    let lightness = (max + min) / 2.0;

    if lightness == 0.0 {
      return 0.0;
    }

    let saturation = if lightness <= 0.5 {
      delta / (max + min)
    } else {
      delta / (2.0 - max - min)
    };

    saturation
  }

  fn rgb_to_hsl(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
      0.0
    } else if max == r {
      ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
      ((b - r) / delta + 2.0) / 6.0
    } else {
      ((r - g) / delta + 4.0) / 6.0
    };

    let l = (max + min) / 2.0;
    let s = if delta == 0.0 {
      0.0
    } else if l <= 0.5 {
      delta / (max + min)
    } else {
      delta / (2.0 - max - min)
    };

    (h, s, l)
  }

  fn hsl_to_rgb(&self, h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0 / 6.0 {
      (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
      (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
      (0.0, c, x)
    } else if h < 4.0 / 6.0 {
      (0.0, x, c)
    } else if h < 5.0 / 6.0 {
      (x, 0.0, c)
    } else {
      (c, 0.0, x)
    };

    (r + m, g + m, b + m)
  }

  fn sqrt(&self, x: f32) -> f32 {
    if x < 0.0 { 0.0 } else { x.sqrt() }
  }

  pub fn clone(&self) -> ImageEffectProcessor {
    ImageEffectProcessor {
      processor: self.processor.clone(),
      random_seed: self.random_seed,
    }
  }
}

#[derive(Debug, Clone)]
pub struct AudioEffectProcessor {
  processor: AudioProcessor,
  random_seed: Option<u64>,
}

impl AudioEffectProcessor {
  pub fn new(processor: AudioProcessor) -> Self {
    Self {
      processor,
      random_seed: None,
    }
  }

  pub fn with_seed(processor: AudioProcessor, seed: u64) -> Self {
    Self {
      processor,
      random_seed: Some(seed),
    }
  }

  pub fn processor(&self) -> &AudioProcessor {
    &self.processor
  }

  pub fn processor_mut(&mut self) -> &mut AudioProcessor {
    &mut self.processor
  }

  pub fn into_processor(self) -> AudioProcessor {
    self.processor
  }

  pub fn random_seed(&self) -> Option<u64> {
    self.random_seed
  }

  pub fn set_random_seed(&mut self, seed: u64) {
    self.random_seed = Some(seed);
  }

  pub fn apply_reverb(&mut self, room_size: f32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let sample_rate = audio_data.sample_rate;

    let delay_samples = (room_size * sample_rate as f32) as usize;
    let decay = 0.5;

    for i in 0..samples.len() {
      if i >= delay_samples {
        samples[i] += samples[i - delay_samples] * decay;
      }
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn apply_echo(&mut self, delay: f32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let sample_rate = audio_data.sample_rate;

    let delay_samples = (delay * sample_rate as f32) as usize;
    let decay = 0.5;

    for i in 0..samples.len() {
      if i >= delay_samples {
        samples[i] += samples[i - delay_samples] * decay;
      }
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn apply_delay(&mut self, delay: f32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let sample_rate = audio_data.sample_rate;

    let delay_samples = (delay * sample_rate as f32) as usize;
    let decay = 0.3;

    for i in 0..samples.len() {
      if i >= delay_samples {
        samples[i] += samples[i - delay_samples] * decay;
      }
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn apply_distortion(&mut self, amount: f32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    for sample in samples.iter_mut() {
      let distorted = (*sample * amount).tanh();
      *sample = distorted;
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn apply_compressor(&mut self, ratio: f32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    let threshold = 0.7;
    let makeup_gain = 1.0 / ratio;

    for sample in samples.iter_mut() {
      if *sample > threshold {
        *sample = threshold + (*sample - threshold) / ratio;
      }
      *sample *= makeup_gain;
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn apply_bit_crush(&mut self, bit_depth: u8, sample_rate_reduction: u32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;

    let levels = 2.0_f32.powi(bit_depth as i32);
    let step = 2.0 / levels;

    for sample in samples.iter_mut() {
      *sample = (*sample / step).round() * step;
    }

    if sample_rate_reduction > 1 {
      let step = sample_rate_reduction as usize;
      for i in (0..samples.len()).step_by(step) {
        for j in 1..step {
          if i + j < samples.len() {
            samples[i + j] = samples[i];
          }
        }
      }
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn apply_low_pass_filter(&mut self, cutoff: f32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let sample_rate = audio_data.sample_rate;

    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff / sample_rate as f32);
    let mut prev_sample = 0.0f32;

    for sample in samples.iter_mut() {
      let filtered = *sample * rc + prev_sample * (1.0 - rc);
      *sample = filtered;
      prev_sample = *sample;
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn apply_high_pass_filter(&mut self, cutoff: f32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let sample_rate = audio_data.sample_rate;

    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff / sample_rate as f32);
    let mut prev_sample = 0.0f32;

    for sample in samples.iter_mut() {
      let filtered = *sample * rc + prev_sample * (1.0 - rc);
      *sample = *sample - filtered;
      prev_sample = filtered;
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn apply_band_pass_filter(&mut self, low_cutoff: f32, high_cutoff: f32) -> Result<()> {
    let audio_data = self.processor.data();
    let mut new_audio_data = audio_data.clone();
    let samples = &mut new_audio_data.samples;
    let sample_rate = audio_data.sample_rate;

    let rc_low = 1.0 / (2.0 * std::f32::consts::PI * low_cutoff / sample_rate as f32);
    let rc_high = 1.0 / (2.0 * std::f32::consts::PI * high_cutoff / sample_rate as f32);
    let mut prev_low_sample = 0.0f32;
    let mut prev_high_sample = 0.0f32;

    for sample in samples.iter_mut() {
      let low_filtered = *sample * rc_low + prev_low_sample * (1.0 - rc_low);
      let high_filtered = *sample * rc_high + prev_high_sample * (1.0 - rc_high);
      let band_passed = low_filtered - high_filtered;
      *sample = band_passed;
      prev_low_sample = low_filtered;
      prev_high_sample = high_filtered;
    }

    self.processor = AudioProcessor::from_audio_data(new_audio_data);
    Ok(())
  }

  pub fn clone(&self) -> AudioEffectProcessor {
    AudioEffectProcessor {
      processor: self.processor.clone(),
      random_seed: self.random_seed,
    }
  }
}

#[derive(Debug, Clone)]
pub struct VideoEffectProcessor {
  processor: ellastic_media::VideoProcessor,
  random_seed: Option<u64>,
}

impl VideoEffectProcessor {
  pub fn new(processor: ellastic_media::VideoProcessor) -> Self {
    Self {
      processor,
      random_seed: None,
    }
  }

  pub fn with_seed(processor: ellastic_media::VideoProcessor, seed: u64) -> Self {
    Self {
      processor,
      random_seed: Some(seed),
    }
  }

  pub fn processor(&self) -> &elastic_media::VideoProcessor {
    &self.processor
  }

  pub fn processor_mut(&mut self) -> &mut ellastic_media::VideoProcessor {
    &mut self.processor
  }

  pub fn into_processor(self) -> ellastic_media::VideoProcessor {
    self.processor
  }

  pub fn random_seed(&self) -> Option<u64> {
    self.random_seed
  }

  pub fn set_random_seed(&mut self, seed: u64) {
    self.random_seed = Some(seed);
  }

  pub fn apply_frame_duplication(&mut self, count: u32) -> Result<()> {
    let video_data = self.processor.data();
    let mut new_video_data = video_data.clone();
    let frames = &mut new_video_data.frames;

    let mut new_frames = Vec::new();

    for frame in frames.iter() {
      new_frames.push(frame.clone());
      for _ in 0..count {
        new_frames.push(frame.clone());
      }
    }

    frames.clear();
    frames.extend(new_frames);

    self.processor = ellastic_media::VideoProcessor::from_video_data(new_video_data);
    Ok(())
  }

  pub fn apply_frame_dropping(&mut self, count: u32) -> Result<()> {
    let video_data = self.processor.data();
    let mut new_video_data = video_data.clone();
    let frames = &mut new_video_data.frames;

    let mut new_frames = Vec::new();

    for (i, frame) in frames.iter().enumerate() {
      if i % (count + 1) == 0 {
        new_frames.push(frame.clone());
      }
    }

    frames.clear();
    frames.extend(new_frames);

    self.processor = ellastic_media::VideoProcessor::from_video_data(new_video_data);
    Ok(())
  }

  pub fn apply_time_stretch(&mut self, ratio: f32) -> Result<()> {
    let video_data = self.processor.data();
    let mut new_video_data = video_data.clone();
    let frames = &mut new_video_data.frames;

    let new_length = (frames.len() as f32 * ratio) as usize;
    let mut new_frames = Vec::with_capacity(new_length);

    for i in 0..new_length {
      let src_pos = i as f32 / ratio;
      let src_index = src_pos as usize;
      let fraction = src_pos - src_index as f32;

      if src_index < frames.len() {
        let frame = &frames[src_index];
        let mut new_frame = frame.clone();

        if fraction > 0.0 && src_index + 1 < frames.len() {
          let next_frame = &frames[src_index + 1];
          self.interpolate_frames(&mut new_frame, frame, next_frame, fraction)?;
        }

        new_frames.push(new_frame);
      }
    }

    frames.clear();
    frames.extend(new_frames);

    self.processor = ellastic_media::VideoProcessor::from_video_data(new_video_data);
    Ok(())
  }

  pub fn apply_reverse_playback(&mut self) -> Result<()> {
    let video_data = self.processor.data();
    let mut new_video_data = video_data.clone();
    let frames = &mut new_video_data.frames;

    frames.reverse();

    self.processor = ellastic_media::VideoProcessor::from_video_data(new_video_data);
    Ok(())
  }

  pub fn apply_data_mosh(&mut self, intensity: f32, preserve_duration: bool) -> Result<()> {
    let video_data = self.processor.data();
    let mut new_video_data = video_data.clone();
    let frames = &mut new_video_data.frames;

    for i in 1..frames.len() {
      let current_frame = &mut frames[i];
      let previous_frame = &frames[i - 1];

      let mosh_count = (current_frame.image_data.data.len() as f32 * intensity) as usize;
      let mut rng = create_random_generator();

      if let Some(seed) = self.random_seed {
        rng.set_seed(seed);
      }

      for _ in 0..mosh_count {
        let pos = rng.gen_range(0, current_frame.image_data.data.len() as u64) as usize;
        if pos < current_frame.image_data.data.len() && pos < previous_frame.image_data.data.len() {
          current_frame.image_data.data[pos] = previous_frame.image_data.data[pos];
        }
      }
    }

    self.processor = ellastic_media::VideoProcessor::from_video_data(new_video_data);
    Ok(())
  }

  fn interpolate_frames(
    &self,
    target_frame: &mut ellastic_media::VideoFrame,
    frame1: &ellastic_media::VideoFrame,
    frame2: &ellastic_media::VideoFrame,
    fraction: f32,
  ) -> Result<()> {
    let data1 = &frame1.image_data.data;
    let data2 = &frame2.image_data.data;
    let data_new = &mut target_frame.image_data.data;

    for i in 0..data_new.len().min(data1.len().min(data2.len())) {
      data_new[i] = (data1[i] as f32 * (1.0 - fraction) + data2[i] as f32 * fraction) as u8;
    }

    Ok(())
  }

  pub fn clone(&self) -> VideoEffectProcessor {
    VideoEffectProcessor {
      processor: self.processor.clone(),
      random_seed: self.random_seed,
    }
  }
}

pub fn create_image_effect_processor(processor: ImageProcessor) -> ImageEffectProcessor {
  ImageEffectProcessor::new(processor)
}

pub fn create_image_effect_processor_with_seed(
  processor: ImageProcessor,
  seed: u64,
) -> ImageEffectProcessor {
  ImageEffectProcessor::with_seed(processor, seed)
}

pub fn create_audio_effect_processor(processor: AudioProcessor) -> AudioEffectProcessor {
  AudioEffectProcessor::new(processor)
}

pub fn create_audio_effect_processor_with_seed(
  processor: AudioProcessor,
  seed: u64,
) -> AudioEffectProcessor {
  AudioEffectProcessor::with_seed(processor, seed)
}

pub fn create_video_effect_processor(
  processor: ellastic_media::VideoProcessor,
) -> VideoEffectProcessor {
  VideoEffectProcessor::new(processor)
}

pub fn create_video_effect_processor_with_seed(
  processor: ellastic_media::VideoProcessor,
  seed: u64,
) -> VideoEffectProcessor {
  VideoEffectProcessor::with_seed(processor, seed)
}
