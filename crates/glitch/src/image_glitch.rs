use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_image::{
  ImageData,
  ImageProcessor,
};
use ellastic_utils::{
  NoiseGenerator,
  create_noise_generator_with_seed,
  create_random_generator,
};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ImageGlitch {
  PixelSort {
    threshold: f32,
    mode: PixelSortMode,
  },
  ChannelShift {
    channel: u8,
    amount: i32,
  },
  SliceAndReorder {
    slice_size: u32,
    reorder: ReorderMode,
  },
  DataMosh {
    intensity: f32,
    preserve_size: bool,
  },
  GlitchArt {
    style: GlitchStyle,
    intensity: f32,
  },
  ColorCorruption {
    corruption_type: ColorCorruptionType,
    amount: f32,
  },
  GeometricDistortion {
    distortion_type: GeometricDistortionType,
    strength: f32,
  },
  CompressionArtifacts {
    artifact_type: CompressionArtifactType,
    quality: u8,
  },
  BitManipulation {
    manipulation_type: BitManipulationType,
    bits: u8,
  },
  NoiseInjection {
    noise_type: NoiseType,
    intensity: f32,
  },
  Custom {
    custom_function: Box<dyn Fn(&mut ImageProcessor) -> Result<()> + Send + Sync>,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelSortMode {
  Brightness,
  Hue,
  Saturation,
  Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReorderMode {
  Random,
  Reverse,
  Rotate,
  Shuffle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlitchStyle {
  Digital,
  Analog,
  Compression,
  DataLoss,
  ColorShift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCorruptionType {
  ChannelSwap,
  ColorShift,
  HueRotation,
  SaturationShift,
  BrightnessShift,
  ContrastShift,
  Inversion,
  Posterization,
  Solarization,
  Threshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometricDistortionType {
  Wave,
  Ripple,
  Swirl,
  Fisheye,
  Barrel,
  Pinch,
  Perspective,
  Shear,
  Skew,
  Twist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionArtifactType {
  JPEG,
  PNG,
  WebP,
  GIF,
  BMP,
  TIFF,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitManipulationType {
  BitFlip,
  BitShift,
  BitRotate,
  BitSwap,
  BitInvert,
  BitMask,
  BitXOR,
  BitOR,
  BitAND,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseType {
  Gaussian,
  Uniform,
  SaltPepper,
  Perlin,
  Simplex,
  Cellular,
  Fractal,
  Custom,
}

#[derive(Debug, Clone)]
pub struct ImageGlitchProcessor {
  image_processor: ImageProcessor,
  random_seed: Option<u64>,
}

impl ImageGlitchProcessor {
  pub fn new(image_processor: ImageProcessor) -> Self {
    Self {
      image_processor,
      random_seed: None,
    }
  }

  pub fn image_processor(&self) -> &ImageProcessor {
    &self.image_processor
  }

  pub fn image_processor_mut(&mut self) -> &mut ImageProcessor {
    &mut self.image_processor
  }

  pub fn into_image_processor(self) -> ImageProcessor {
    self.image_processor
  }

  pub fn random_seed(&self) -> Option<u64> {
    self.random_seed
  }

  pub fn set_random_seed(&mut self, seed: u64) {
    self.random_seed = Some(seed);
  }

  pub fn apply_glitch(&mut self, glitch: &ImageGlitch) -> Result<()> {
    match glitch {
      ImageGlitch::PixelSort { threshold, mode } => {
        self.pixel_sort(*threshold, *mode)?;
      }
      ImageGlitch::ChannelShift { channel, amount } => {
        self.channel_shift(*channel, *amount)?;
      }
      ImageGlitch::SliceAndReorder {
        slice_size,
        reorder,
      } => {
        self.slice_and_reorder(*slice_size, *reorder)?;
      }
      ImageGlitch::DataMosh {
        intensity,
        preserve_size,
      } => {
        self.data_mosh(*intensity, *preserve_size)?;
      }
      ImageGlitch::GlitchArt { style, intensity } => {
        self.glitch_art(*style, *intensity)?;
      }
      ImageGlitch::ColorCorruption {
        corruption_type,
        amount,
      } => {
        self.color_corruption(*corruption_type, *amount)?;
      }
      ImageGlitch::GeometricDistortion {
        distortion_type,
        strength,
      } => {
        self.geometric_distortion(*distortion_type, *strength)?;
      }
      ImageGlitch::CompressionArtifacts {
        artifact_type,
        quality,
      } => {
        self.compression_artifacts(*artifact_type, *quality)?;
      }
      ImageGlitch::BitManipulation {
        manipulation_type,
        bits,
      } => {
        self.bit_manipulation(*manipulation_type, *bits)?;
      }
      ImageGlitch::NoiseInjection {
        noise_type,
        intensity,
      } => {
        self.noise_injection(*noise_type, *intensity)?;
      }
      ImageGlitch::Custom { custom_function } => {
        custom_function(&mut self.image_processor)?;
      }
    }
    Ok(())
  }

  pub fn apply_glitch_batch(&mut self, glitches: &[ImageGlitch]) -> Result<Vec<ImageProcessor>> {
    let mut results = Vec::new();

    for glitch in glitches {
      let mut temp_processor = self.image_processor.clone();
      let mut temp_glitcher = ImageGlitchProcessor::new(temp_processor);

      if let Some(seed) = self.random_seed {
        temp_glitcher.set_random_seed(seed);
      }

      temp_glitcher.apply_glitch(glitch)?;
      results.push(temp_glitcher.into_image_processor());
    }

    Ok(results)
  }

  pub fn pixel_sort(&mut self, threshold: f32, mode: PixelSortMode) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut pixels = Vec::new();

    for y in 0..image_data.height {
      for x in 0..image_data.width {
        if let Some(pixel) = self.image_processor.get_pixel(x, y) {
          let brightness = pixel.iter().sum::<u8>() as f32 / pixel.len() as f32;
          if brightness >= threshold {
            pixels.push((x, y, pixel));
          }
        }
      }
    }

    match mode {
      PixelSortMode::Brightness => {
        pixels.sort_by(|_, a, b| {
          let brightness_a = a.2.iter().sum::<u8>() as f32 / a.2.len() as f32;
          let brightness_b = b.2.iter().sum::<u8>() as f32 / b.2.len() as f32;
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
        rng.shuffle(&mut pixels);
      }
    }

    let mut new_image_data = image_data.clone();
    for (x, y, pixel) in pixels {
      if let Some(_) = new_image_data.set_pixel(x, y, &pixel) {}
    }

    Ok(())
  }

  pub fn channel_shift(&mut self, channel: u8, amount: i32) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();

    for y in 0..image_data.height {
      for x in 0..image_data.width {
        if let Some(mut pixel) = self.image_processor.get_pixel(x, y) {
          if channel < pixel.len() as u8 {
            let shifted_value = pixel[channel as usize] as i32;
            let shifted_value = (shifted_value + amount).clamp(0, 255) as u8;
            pixel[channel as usize] = shifted_value;
          }

          if let Some(_) = new_image_data.set_pixel(x, y, &pixel) {}
        }
      }
    }

    Ok(())
  }

  pub fn slice_and_reorder(&mut self, slice_size: u32, reorder: ReorderMode) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();
    let mut slices = Vec::new();

    for y in (0..image_data.height).step_by(slice_size) {
      for x in (0..image_data.width).step_by(slice_size) {
        let slice = self
          .image_processor
          .get_region(x, y, slice_size, slice_size)?;
        slices.push(slice);
      }
    }

    match reorder {
      ReorderMode::Random => {
        let mut rng = create_random_generator();
        rng.shuffle(&mut slices);
      }
      ReorderMode::Reverse => {
        slices.reverse();
      }
      ReorderMode::Rotate => {
        slices.rotate_left(1);
      }
      ReorderMode::Shuffle => {
        let mut rng = create_random_generator();
        rng.shuffle(&mut slices);
      }
    }

    let mut offset_y = 0;
    for slice in slices {
      if let Some(_) =
        new_image_data.set_region(0, offset_y, slice.width(), slice.height(), &slice.data())
      {
        offset_y += slice.height();
      }
    }

    Ok(())
  }

  pub fn data_mosh(&mut self, intensity: f32, preserve_size: bool) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();
    let data = &image_data.data;

    let corruption_count = (data.len() as f32 * intensity) as usize;
    let mut rng = create_random_generator();

    for _ in 0..corruption_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      if pos < data.len() {
        new_image_data.data[pos] = rng.gen_range(0, 256) as u8;
      }
    }

    Ok(())
  }

  pub fn glitch_art(&mut self, style: GlitchStyle, intensity: f32) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();

    match style {
      GlitchStyle::Digital => {
        self.digital_glitch(&mut new_image_data, intensity)?;
      }
      GlitchStyle::Analog => {
        self.analog_glitch(&mut new_image_data, intensity)?;
      }
      GlitchStyle::Compression => {
        self.compression_glitch(&mut new_image_data, intensity)?;
      }
      GlitchStyle::DataLoss => {
        self.data_loss_glitch(&mut new_image_data, intensity)?;
      }
      GlitchStyle::ColorShift => {
        self.color_shift_glitch(&mut new_image_data, intensity)?;
      }
    }

    Ok(())
  }

  pub fn color_corruption(
    &mut self,
    corruption_type: ColorCorruptionType,
    amount: f32,
  ) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();

    match corruption_type {
      ColorCorruptionType::ChannelSwap => {
        self.channel_swap_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::ColorShift => {
        self.color_shift_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::HueRotation => {
        self.hue_rotation_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::SaturationShift => {
        self.saturation_shift_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::BrightnessShift => {
        self.brightness_shift_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::ContrastShift => {
        self.contrast_shift_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::Inversion => {
        self.inversion_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::Posterization => {
        self.posterization_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::Solarization => {
        self.solarization_corruption(&mut new_image_data, amount)?;
      }
      ColorCorruptionType::Threshold => {
        self.threshold_corruption(&mut new_image_data, amount)?;
      }
    }

    Ok(())
  }

  pub fn geometric_distortion(
    &mut self,
    distortion_type: GeometricDistortionType,
    strength: f32,
  ) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();

    match distortion_type {
      GeometricDistortionType::Wave => {
        self.wave_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Ripple => {
        self.ripple_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Swirl => {
        self.swirl_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Fisheye => {
        self.fisheye_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Barrel => {
        self.barrel_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Pinch => {
        self.pinch_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Perspective => {
        self.perspective_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Shear => {
        self.shear_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Skew => {
        self.skew_distortion(&mut new_image_data, strength)?;
      }
      GeometricDistortionType::Twist => {
        self.twist_distortion(&mut new_image_data, strength)?;
      }
    }

    Ok(())
  }

  pub fn compression_artifacts(
    &mut self,
    artifact_type: CompressionArtifactType,
    quality: u8,
  ) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();

    match artifact_type {
      CompressionArtifactType::JPEG => {
        self.jpeg_artifacts(&mut new_image_data, quality)?;
      }
      CompressionArtifactType::PNG => {
        self.png_artifacts(&mut new_image_data, quality)?;
      }
      CompressionArtifactType::WebP => {
        self.webp_artifacts(&mut new_image_data, quality)?;
      }
      CompressionArtifactType::GIF => {
        self.gif_artifacts(&mut new_image_data, quality)?;
      }
      CompressionArtifactType::BMP => {
        self.bmp_artifacts(&mut new_image_data, quality)?;
      }
      CompressionArtifactType::TIFF => {
        self.tiff_artifacts(&mut new_image_data, quality)?;
      }
      CompressionArtifactType::Custom => {
        self.custom_artifacts(&mut new_image_data, quality)?;
      }
    }

    Ok(())
  }

  pub fn bit_manipulation(
    &mut self,
    manipulation_type: BitManipulationType,
    bits: u8,
  ) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    match manipulation_type {
      BitManipulationType::BitFlip => {
        self.bit_flip(data, bits)?;
      }
      BitManipulationType::BitShift => {
        self.bit_shift(data, bits)?;
      }
      BitManipulationType::BitRotate => {
        self.bit_rotate(data, bits)?;
      }
      BitManipulationType::BitSwap => {
        self.bit_swap(data, bits)?;
      }
      BitManipulationType::BitInvert => {
        self.bit_invert(data, bits)?;
      }
      BitManipulationType::BitMask => {
        self.bit_mask(data, bits)?;
      }
      BitManipulationType::BitXOR => {
        self.bit_xor(data, bits)?;
      }
      BitManipulationType::BitOR => {
        self.bit_or(data, bits)?;
      }
      BitManipulationType::BitAND => {
        self.bit_and(data, bits)?;
      }
    }

    Ok(())
  }

  pub fn noise_injection(&mut self, noise_type: NoiseType, intensity: f32) -> Result<()> {
    let image_data = self.image_processor.data();
    let mut new_image_data = image_data.clone();
    let data = &mut new_image_data.data;

    match noise_type {
      NoiseType::Gaussian => {
        self.gaussian_noise(data, intensity)?;
      }
      NoiseType::Uniform => {
        self.uniform_noise(data, intensity)?;
      }
      NoiseType::SaltPepper => {
        self.salt_pepper_noise(data, intensity)?;
      }
      NoiseType::Perlin => {
        self.perlin_noise(data, intensity)?;
      }
      NoiseType::Simplex => {
        self.simplex_noise(data, intensity)?;
      }
      NoiseType::Cellular => {
        self.cellular_noise(data, intensity)?;
      }
      NoiseType::Fractal => {
        self.fractal_noise(data, intensity)?;
      }
      NoiseType::Custom => {
        self.custom_noise(data, intensity)?;
      }
    }

    Ok(())
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
      (g - b) / delta
    } else if max == g {
      2.0 + (b - r) / delta
    } else {
      4.0 + (r - g) / delta
    };

    (hue * 360.0 / 6.0).fract() * 360.0
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
      delta / (2.0 - lightness * 2.0)
    } else {
      delta / (lightness * 2.0 - 1.0)
    };

    saturation
  }

  fn digital_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
    let data = &mut image_data.data;
    let glitch_count = (data.len() as f32 * intensity) as usize;
    let mut rng = create_random_generator();

    for _ in 0..glitch_count {
      let start = rng.gen_range(0, data.len() as u64) as usize;
      let end = (start + 1024).min(data.len());

      if end > start {
        for i in start..end {
          data[i] = rng.gen_range(0, 256) as u8;
        }
      }
    }

    Ok(())
  }

  fn analog_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
    let data = &mut image_data.data;
    let glitch_count = (data.len() as f32 * intensity) as usize;
    let mut rng = create_random_generator();

    for _ in 0..glitch_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      if pos < data.len() {
        let original = data[pos];
        let noise = rng.gen_range(-50, 50) as i8;
        let corrupted = (original as i16).saturating_add(noise) as u8;
        data[pos] = corrupted;
      }
    }

    Ok(())
  }

  fn compression_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
    let data = &mut image_data.data;
    let chunk_size = (1024.0 * intensity) as usize;
    let mut rng = create_random_generator();

    for chunk in data.chunks_mut(chunk_size) {
      if rng.gen_range(0.0, 1.0) < 0.5 {
        chunk.fill(0);
      }
    }

    Ok(())
  }

  fn data_loss_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
    let data = &mut image_data.data;
    let loss_count = (data.len() as f32 * intensity) as usize;
    let mut rng = create_random_generator();

    for _ in 0..loss_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      if pos < data.len() {
        data[pos] = 0;
      }
    }

    Ok(())
  }

  fn color_shift_glitch(&mut self, image_data: &mut ImageData, intensity: f32) -> Result<()> {
    let data = &mut image_data.data;
    let shift_count = (data.len() / 4) as usize;
    let mut rng = create_random_generator();

    for _ in 0..shift_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      if pos + 3 < data.len() {
        data[pos..pos + 4].rotate_right(1);
      }
    }

    Ok(())
  }

  fn channel_swap_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let swap_count = (data.len() as f32 * amount / 4.0) as usize;
    let mut rng = create_random_generator();

    for _ in 0..swap_count {
      let pos = rng.gen_range(0, data.len() as u64 - 3) as usize;
      if pos + 2 < data.len() {
        data.swap(pos, pos + 1);
        data.swap(pos + 1, pos + 2);
      }
    }

    Ok(())
  }

  fn color_shift_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let shift_amount = (amount * 255.0) as i32;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = (pixel[0] as i32 + shift_amount).clamp(0, 255) as u8;
        pixel[1] = (pixel[1] as i32 + shift_amount).clamp(0, 255) as u8;
        pixel[2] = (pixel[2] as i32 + shift_amount).clamp(0, 255) as u8;
      }
    }

    Ok(())
  }

  fn hue_rotation_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let rotation = amount * 360.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        let (h, s, l) = self.rgb_to_hsl(r, g, b);
        let (r_new, g_new, b_new) = self.hsl_to_rgb((h + rotation).fract(), s, l);

        pixel[0] = (r_new * 255.0) as u8;
        pixel[1] = (g_new * 255.0) as u8;
        pixel[2] = (b_new * 255.0) as u8;
      }
    }

    Ok(())
  }

  fn saturation_shift_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let shift = amount * 2.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        let (h, s, l) = self.rgb_to_hsl(r, g, b);
        let (r_new, g_new, b_new) = self.hsl_to_rgb(h, (s + shift).clamp(0.0, 1.0), l);

        pixel[0] = (r_new * 255.0) as u8;
        pixel[1] = (g_new * 255.0) as u8;
        pixel[2] = (b_new * 255.0) as u8;
      }
    }

    Ok(())
  }

  fn brightness_shift_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let shift = amount * 255.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = (pixel[0] as f32 + shift).clamp(0.0, 255.0) as u8;
        pixel[1] = (pixel[1] as f32 + shift).clamp(0.0, 255.0) as u8;
        pixel[2] = (pixel[2] as f32 + shift).clamp(0.0, 255.0) as u8;
      }
    }

    Ok(())
  }

  fn contrast_shift_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let factor = amount * 2.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = ((pixel[0] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
        pixel[1] = ((pixel[1] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
        pixel[2] = ((pixel[2] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
      }
    }

    Ok(())
  }

  fn inversion_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let invert_factor = amount;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = (pixel[0] as f32 * (1.0 - invert_factor)
          + (255.0 - pixel[0] as f32) * invert_factor) as u8;
        pixel[1] = (pixel[1] as f32 * (1.0 - invert_factor)
          + (255.0 - pixel[1] as f32) * invert_factor) as u8;
        pixel[2] = (pixel[2] as f32 * (1.0 - invert_factor)
          + (255.0 - pixel[2] as f32) * invert_factor) as u8;
      }
    }

    Ok(())
  }

  fn posterization_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let levels = (1.0 + amount * 15.0) as u8;
    let factor = 255.0 / levels as f32;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = ((pixel[0] as f32 / factor).round() * factor).clamp(0.0, 255.0) as u8;
        pixel[1] = ((pixel[1] as f32 / factor).round() * factor).clamp(0.0, 255.0) as u8;
        pixel[2] = ((pixel[2] as f32 / factor).round() * factor).clamp(0.0, 255.0) as u8;
      }
    }

    Ok(())
  }

  fn solarization_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let threshold = amount * 255.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = if pixel[0] as f32 > threshold {
          255 - pixel[0]
        } else {
          pixel[0]
        };
        pixel[1] = if pixel[1] as f32 > threshold {
          255 - pixel[1]
        } else {
          pixel[1]
        };
        pixel[2] = if pixel[2] as f32 > threshold {
          255 - pixel[2]
        } else {
          pixel[2]
        };
      }
    }

    Ok(())
  }

  fn threshold_corruption(&mut self, image_data: &mut ImageData, amount: f32) -> Result<()> {
    let data = &mut image_data.data;
    let threshold = amount * 255.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        let value = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
        let binary = if value > threshold { 255 } else { 0 };
        pixel[0] = binary;
        pixel[1] = binary;
        pixel[2] = binary;
      }
    }

    Ok(())
  }

  fn wave_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;

    for y in 0..height {
      for x in 0..width {
        let offset =
          (strength * 10.0 * ((x as f32 / width as f32) * 2.0 * std::f32::consts::PI).sin()) as i32;
        let pixel_index = ((y as usize * width as usize + x as usize) * 4).min(data.len() - 1);

        if offset != 0 && pixel_index + offset < data.len() {
          let source_pixel = data[pixel_index];
          data[pixel_index] = data[pixel_index + offset];
        }
      }
    }

    Ok(())
  }

  fn ripple_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;

    for y in 0..height {
      for x in 0..width {
        let distance = ((x as f32 - width as f32 / 2.0).powi(2)
          + (y as f32 - height as f32 / 2.0).powi(2))
        .sqrt();
        let wave_height = (strength * 20.0 * (distance / 100.0)).sin();
        let offset = wave_height as i32;

        let pixel_index = ((y as usize * width as usize + x as usize) * 4).min(data.len() - 1);

        if offset != 0 && pixel_index + offset < data.len() {
          let source_pixel = data[pixel_index];
          data[pixel_index] = data[pixel_index + offset];
        }
      }
    }

    Ok(())
  }

  fn swirl_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let angle = strength * distance / 10.0;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        let source_x = (center_x + dx * cos_angle - dy * sin_angle) as i32;
        let source_y = (center_y + dx * sin_angle + dy * cos_angle) as i32;

        if source_x >= 0 && source_x < width as i32 && source_y >= 0 && source_y < height as i32 {
          let source_index = (source_y as usize * width as usize + source_x as usize) * 4;
          let pixel_index = (y as usize * width as usize + x as usize) * 4;

          if source_index < data.len() && pixel_index < data.len() {
            data[pixel_index] = data[source_index];
          }
        }
      }
    }

    Ok(())
  }

  fn fisheye_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let radius = strength * 100.0;

        if distance < radius {
          let source_x = (center_x + dx / (1.0 + distance / radius)) as i32;
          let source_y = (center_y + dy / (1.0 + distance / radius)) as i32;

          if source_x >= 0 && source_x < width as i32 && source_y >= 0 && source_y < height as i32 {
            let source_index = (source_y as usize * width as usize + source_x as usize) * 4;
            let pixel_index = (y as usize * width as usize + x as usize) * 4;

            if source_index < data.len() && pixel_index < data.len() {
              data[pixel_index] = data[source_index];
            }
          }
        }
      }
    }

    Ok(())
  }

  fn barrel_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_radius = (center_x * center_x + center_y * center_y).sqrt();

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let normalized_distance = distance / max_radius;
        let distortion_factor = 1.0 + strength * normalized_distance * normalized_distance;

        let source_x = (center_x + dx * distortion_factor) as i32;
        let source_y = (center_y + dy * distortion_factor) as i32;

        if source_x >= 0 && source_x < width as i32 && source_y >= 0 && source_y < height as i32 {
          let source_index = (source_y as usize * width as usize + source_x as usize) * 4;
          let pixel_index = (y as usize * width as usize + x as usize) * 4;

          if source_index < data.len() && pixel_index < data.len() {
            data[pixel_index] = data[source_index];
          }
        }
      }
    }

    Ok(())
  }

  fn pinch_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let radius = strength * 100.0;

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < radius {
          let factor = 1.0 - (distance / radius) * strength;
          let source_x = (center_x + dx * factor) as i32;
          let source_y = (center_y + dy * factor) as i32;

          if source_x >= 0 && source_x < width as i32 && source_y >= 0 && source_y < height as i32 {
            let source_index = (source_y as usize * width as usize + source_x as usize) * 4;
            let pixel_index = (y as usize * width as usize + x as usize) * 4;

            if source_index < data.len() && pixel_index < data.len() {
              data[pixel_index] = data[source_index];
            }
          }
        }
      }
    }

    Ok(())
  }

  fn perspective_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let perspective_factor = strength;

    for y in 0..height {
      for x in 0..width {
        let normalized_x = x as f32 / width as f32;
        let normalized_y = y as f32 / height as f32;

        let perspective_x = normalized_x * (1.0 + perspective_factor * normalized_y);
        let perspective_y = normalized_y * (1.0 + perspective_factor * normalized_x);

        let source_x = (perspective_x * width as f32) as i32;
        let source_y = (perspective_y * height as f32) as i32;

        if source_x >= 0 && source_x < width as i32 && source_y >= 0 && source_y < height as i32 {
          let source_index = (source_y as usize * width as usize + source_x as usize) * 4;
          let pixel_index = (y as usize * width as usize + x as usize) * 4;

          if source_index < data.len() && pixel_index < data.len() {
            data[pixel_index] = data[source_index];
          }
        }
      }
    }

    Ok(())
  }

  fn shear_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let shear_x = strength;
    let shear_y = strength;

    for y in 0..height {
      for x in 0..width {
        let source_x = (x as f32 + y as f32 * shear_x) as i32;
        let source_y = (y as f32 + x as f32 * shear_y) as i32;

        if source_x >= 0 && source_x < width as i32 && source_y >= 0 && source_y < height as i32 {
          let source_index = (source_y as usize * width as usize + source_x as usize) * 4;
          let pixel_index = (y as usize * width as usize + x as usize) * 4;

          if source_index < data.len() && pixel_index < data.len() {
            data[pixel_index] = data[source_index];
          }
        }
      }
    }

    Ok(())
  }

  fn skew_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let skew_x = strength;
    let skew_y = strength;

    for y in 0..height {
      for x in 0..width {
        let source_x = (x as f32 + y as f32 * skew_x) as i32;
        let source_y = y as i32;

        if source_x >= 0 && source_x < width as i32 {
          let source_index = (source_y as usize * width as usize + source_x as usize) * 4;
          let pixel_index = (y as usize * width as usize + x as usize) * 4;

          if source_index < data.len() && pixel_index < data.len() {
            data[pixel_index] = data[source_index];
          }
        }
      }
    }

    Ok(())
  }

  fn twist_distortion(&mut self, image_data: &mut ImageData, strength: f32) -> Result<()> {
    let data = &mut image_data.data;
    let width = image_data.width;
    let height = image_data.height;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let angle = strength * distance / 100.0;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        let source_x = (center_x + dx * cos_angle - dy * sin_angle) as i32;
        let source_y = (center_y + dx * sin_angle + dy * cos_angle) as i32;

        if source_x >= 0 && source_x < width as i32 && source_y >= 0 && source_y < height as i32 {
          let source_index = (source_y as usize * width as usize + source_x as usize) * 4;
          let pixel_index = (y as usize * width as usize + x as usize) * 4;

          if source_index < data.len() && pixel_index < data.len() {
            data[pixel_index] = data[source_index];
          }
        }
      }
    }

    Ok(())
  }

  fn jpeg_artifacts(&mut self, image_data: &mut ImageData, quality: u8) -> Result<()> {
    let data = &mut image_data.data;
    let block_size = 8;
    let quality_factor = (100 - quality) as f32 / 100.0;

    for chunk in data.chunks_mut(block_size * block_size * 4) {
      for pixel in chunk.chunks_mut(4) {
        if pixel.len() >= 3 {
          pixel[0] = ((pixel[0] as f32 / 8.0).round() * 8.0 * (1.0 - quality_factor)
            + pixel[0] as f32 * quality_factor) as u8;
          pixel[1] = ((pixel[1] as f32 / 8.0).round() * 8.0 * (1.0 - quality_factor)
            + pixel[1] as f32 * quality_factor) as u8;
          pixel[2] = ((pixel[2] as f32 / 8.0).round() * 8.0 * (1.0 - quality_factor)
            + pixel[2] as f32 * quality_factor) as u8;
        }
      }
    }

    Ok(())
  }

  fn png_artifacts(&mut self, image_data: &mut ImageData, quality: u8) -> Result<()> {
    let data = &mut image_data.data;
    let compression_factor = (100 - quality) as f32 / 100.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = (pixel[0] as f32 * (1.0 - compression_factor * 0.1)) as u8;
        pixel[1] = (pixel[1] as f32 * (1.0 - compression_factor * 0.1)) as u8;
        pixel[2] = (pixel[2] as f32 * (1.0 - compression_factor * 0.1)) as u8;
      }
    }

    Ok(())
  }

  fn webp_artifacts(&mut self, image_data: &mut ImageData, quality: u8) -> Result<()> {
    let data = &mut image_data.data;
    let artifact_factor = (100 - quality) as f32 / 100.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = ((pixel[0] as f32 / 4.0).round() * 4.0 * (1.0 - artifact_factor)
          + pixel[0] as f32 * artifact_factor) as u8;
        pixel[1] = ((pixel[1] as f32 / 4.0).round() * 4.0 * (1.0 - artifact_factor)
          + pixel[1] as f32 * artifact_factor) as u8;
        pixel[2] = ((pixel[2] as f32 / 4.0).round() * 4.0 * (1.0 - artifact_factor)
          + pixel[2] as f32 * artifact_factor) as u8;
      }
    }

    Ok(())
  }

  fn gif_artifacts(&mut self, image_data: &mut ImageData, quality: u8) -> Result<()> {
    let data = &mut image_data.data;
    let color_reduction = (100 - quality) as f32 / 100.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = ((pixel[0] as f32 / 256.0 * (1.0 - color_reduction * 255.0)).round() * 256.0
          / (1.0 - color_reduction * 255.0)) as u8;
        pixel[1] = ((pixel[1] as f32 / 256.0 * (1.0 - color_reduction * 255.0)).round() * 256.0
          / (1.0 - color_reduction * 255.0)) as u8;
        pixel[2] = ((pixel[2] as f32 / 256.0 * (1.0 - color_reduction * 255.0)).round() * 256.0
          / (1.0 - color_reduction * 255.0)) as u8;
      }
    }

    Ok(())
  }

  fn bmp_artifacts(&mut self, image_data: &mut ImageData, quality: u8) -> Result<()> {
    let data = &mut image_data.data;
    let artifact_factor = (100 - quality) as f32 / 100.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = (pixel[0] as f32 * (1.0 - artifact_factor * 0.05)) as u8;
        pixel[1] = (pixel[1] as f32 * (1.0 - artifact_factor * 0.05)) as u8;
        pixel[2] = (pixel[2] as f32 * (1.0 - artifact_factor * 0.05)) as u8;
      }
    }

    Ok(())
  }

  fn tiff_artifacts(&mut self, image_data: &mut ImageData, quality: u8) -> Result<()> {
    let data = &mut image_data.data;
    let artifact_factor = (100 - quality) as f32 / 100.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = (pixel[0] as f32 * (1.0 - artifact_factor * 0.02)) as u8;
        pixel[1] = (pixel[1] as f32 * (1.0 - artifact_factor * 0.02)) as u8;
        pixel[2] = (pixel[2] as f32 * (1.0 - artifact_factor * 0.02)) as u8;
      }
    }

    Ok(())
  }

  fn custom_artifacts(&mut self, image_data: &mut ImageData, quality: u8) -> Result<()> {
    let data = &mut image_data.data;
    let artifact_factor = (100 - quality) as f32 / 100.0;

    for pixel in data.chunks_mut(4) {
      if pixel.len() >= 3 {
        pixel[0] = (pixel[0] as f32 * (1.0 - artifact_factor * 0.1)) as u8;
        pixel[1] = (pixel[1] as f32 * (1.0 - artifact_factor * 0.1)) as u8;
        pixel[2] = (pixel[2] as f32 * (1.0 - artifact_factor * 0.1)) as u8;
      }
    }

    Ok(())
  }

  fn bit_flip(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let flip_count = (data.len() as f32 * bits as f32 / 100.0) as usize;
    let mut rng = create_random_generator();

    for _ in 0..flip_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      let bit_pos = rng.gen_range(0, 8);
      data[pos] ^= 1 << bit_pos;
    }

    Ok(())
  }

  fn bit_shift(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let shift_amount = bits % 8;

    for byte in data.iter_mut() {
      *byte = (*byte << shift_amount) | (*byte >> (8 - shift_amount));
    }

    Ok(())
  }

  fn bit_rotate(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let rotate_amount = bits % 8;

    for byte in data.iter_mut() {
      let high_bits = *byte & (0xFF << rotate_amount);
      let low_bits = *byte & (0xFF >> (8 - rotate_amount));
      *byte = (high_bits >> rotate_amount) | (low_bits << (8 - rotate_amount));
    }

    Ok(())
  }

  fn bit_swap(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let swap_count = (data.len() as f32 * bits as f32 / 100.0) as usize;
    let mut rng = create_random_generator();

    for _ in 0..swap_count {
      let pos1 = rng.gen_range(0, data.len() as u64) as usize;
      let pos2 = rng.gen_range(0, data.len() as u64) as usize;
      data.swap(pos1, pos2);
    }

    Ok(())
  }

  fn bit_invert(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let invert_count = (data.len() as f32 * bits as f32 / 100.0) as usize;
    let mut rng = create_random_generator();

    for _ in 0..invert_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      data[pos] = !data[pos];
    }

    Ok(())
  }

  fn bit_mask(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let mask = bits;

    for byte in data.iter_mut() {
      *byte &= mask;
    }

    Ok(())
  }

  fn bit_xor(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let xor_value = bits;

    for byte in data.iter_mut() {
      *byte ^= xor_value;
    }

    Ok(())
  }

  fn bit_or(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let or_value = bits;

    for byte in data.iter_mut() {
      *byte |= or_value;
    }

    Ok(())
  }

  fn bit_and(&mut self, data: &mut [u8], bits: u8) -> Result<()> {
    let and_value = bits;

    for byte in data.iter_mut() {
      *byte &= and_value;
    }

    Ok(())
  }

  fn gaussian_noise(&mut self, data: &mut [u8], intensity: f32) -> Result<()> {
    let mut rng = create_random_generator();
    let noise_factor = intensity * 50.0;

    for byte in data.iter_mut() {
      let noise: f32 = rng.gen_range(-1.0, 1.0) * noise_factor;
      *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
    }

    Ok(())
  }

  fn uniform_noise(&mut self, data: &mut [u8], intensity: f32) -> Result<()> {
    let mut rng = create_random_generator();
    let noise_factor = intensity * 50.0;

    for byte in data.iter_mut() {
      let noise: f32 = rng.gen_range(-noise_factor, noise_factor);
      *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
    }

    Ok(())
  }

  fn salt_pepper_noise(&mut self, data: &mut [u8], intensity: f32) -> Result<()> {
    let mut rng = create_random_generator();
    let noise_probability = intensity;

    for byte in data.iter_mut() {
      if rng.gen_range(0.0, 1.0) < noise_probability {
        *byte = if rng.gen_range(0.0, 1.0) < 0.5 {
          0
        } else {
          255
        };
      }
    }

    Ok(())
  }

  fn perlin_noise(&mut self, data: &mut [u8], intensity: f32) -> Result<()> {
    let mut noise_gen = create_noise_generator_with_seed(42);
    let noise_factor = intensity * 50.0;

    for (i, byte) in data.iter_mut().enumerate() {
      let noise = noise_gen.perlin(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
      *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
    }

    Ok(())
  }

  fn simplex_noise(&mut self, data: &mut [u8], intensity: f32) -> Result<()> {
    let mut noise_gen = create_noise_generator_with_seed(42);
    let noise_factor = intensity * 50.0;

    for (i, byte) in data.iter_mut().enumerate() {
      let noise = noise_gen.simplex(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
      *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
    }

    Ok(())
  }

  fn cellular_noise(&mut self, data: &mut [u8], intensity: f32) -> Result<()> {
    let mut noise_gen = create_noise_generator_with_seed(42);
    let noise_factor = intensity * 50.0;

    for (i, byte) in data.iter_mut().enumerate() {
      let noise = noise_gen.cellular(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
      *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
    }

    Ok(())
  }

  fn fractal_noise(&mut self, data: &mut [u8], intensity: f32) -> Result<()> {
    let mut noise_gen = create_noise_generator_with_seed(42);
    let noise_factor = intensity * 50.0;

    for (i, byte) in data.iter_mut().enumerate() {
      let noise = noise_gen.fractal(i as f32 / 100.0, 0.0, 0.0) * noise_factor;
      *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
    }

    Ok(())
  }

  fn custom_noise(&mut self, data: &mut [u8], intensity: f32) -> Result<()> {
    let mut rng = create_random_generator();
    let noise_factor = intensity * 50.0;

    for byte in data.iter_mut() {
      let noise: f32 = rng.gen_range(-noise_factor, noise_factor);
      *byte = (*byte as f32 + noise).clamp(0.0, 255.0) as u8;
    }

    Ok(())
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

  pub fn clone(&self) -> ImageGlitchProcessor {
    ImageGlitchProcessor {
      image_processor: self.image_processor.clone(),
      random_seed: self.random_seed,
    }
  }
}

pub fn create_image_glitch_processor(image_processor: ImageProcessor) -> ImageGlitchProcessor {
  ImageGlitchProcessor::new(image_processor)
}

pub fn create_pixel_sort_glitch(threshold: f32, mode: PixelSortMode) -> ImageGlitch {
  ImageGlitch::PixelSort { threshold, mode }
}

pub fn create_channel_shift_glitch(channel: u8, amount: i32) -> ImageGlitch {
  ImageGlitch::ChannelShift { channel, amount }
}

pub fn create_slice_reorder_glitch(slice_size: u32, reorder: ReorderMode) -> ImageGlitch {
  ImageGlitch::SliceAndReorder {
    slice_size,
    reorder,
  }
}

pub fn create_data_mosh_glitch(intensity: f32, preserve_size: bool) -> ImageGlitch {
  ImageGlitch::DataMosh {
    intensity,
    preserve_size,
  }
}

pub fn create_glitch_art_glitch(style: GlitchStyle, intensity: f32) -> ImageGlitch {
  ImageGlitch::GlitchArt { style, intensity }
}

pub fn create_color_corruption_glitch(
  corruption_type: ColorCorruptionType,
  amount: f32,
) -> ImageGlitch {
  ImageGlitch::ColorCorruption {
    corruption_type,
    amount,
  }
}

pub fn create_geometric_distortion_glitch(
  distortion_type: GeometricDistortionType,
  strength: f32,
) -> ImageGlitch {
  ImageGlitch::GeometricDistortion {
    distortion_type,
    strength,
  }
}

pub fn create_compression_artifacts_glitch(
  artifact_type: CompressionArtifactType,
  quality: u8,
) -> ImageGlitch {
  ImageGlitch::CompressionArtifacts {
    artifact_type,
    quality,
  }
}

pub fn create_bit_manipulation_glitch(
  manipulation_type: BitManipulationType,
  bits: u8,
) -> ImageGlitch {
  ImageGlitch::BitManipulation {
    manipulation_type,
    bits,
  }
}

pub fn create_noise_injection_glitch(noise_type: NoiseType, intensity: f32) -> ImageGlitch {
  ImageGlitch::NoiseInjection {
    noise_type,
    intensity,
  }
}
