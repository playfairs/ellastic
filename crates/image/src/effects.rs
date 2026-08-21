use crate::{
  ImageData,
  ImageProcessor,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use ellastic_utils::{
  NoiseGenerator,
  create_noise_generator_with_seed,
  create_random_generator,
};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct ImageEffectProcessor {
  image: ImageProcessor,
}

impl ImageEffectProcessor {
  pub fn new(image: ImageProcessor) -> Self {
    Self { image }
  }

  pub fn image(&self) -> &ImageProcessor {
    &self.image
  }

  pub fn image_mut(&mut self) -> &mut ImageProcessor {
    &mut self.image
  }

  pub fn into_image(self) -> ImageProcessor {
    self.image
  }

  pub fn apply_effect(&mut self, effect: &ImageEffect) -> Result<()> {
    match effect {
      ImageEffect::Grayscale => self.grayscale(),
      ImageEffect::Sepia => self.sepia(),
      ImageEffect::Invert => self.invert(),
      ImageEffect::Brightness { amount } => self.brightness(*amount),
      ImageEffect::Contrast { amount } => self.contrast(*amount),
      ImageEffect::Saturation { amount } => self.saturation(*amount),
      ImageEffect::HueRotate { degrees } => self.hue_rotate(*degrees),
      ImageEffect::Gamma { gamma } => self.gamma(*gamma),
      ImageEffect::Exposure { exposure } => self.exposure(*exposure),
      ImageEffect::Vibrance { amount } => self.vibrance(*amount),
      ImageEffect::Temperature { temperature } => self.temperature(*temperature),
      ImageEffect::Tint { color } => self.tint(color),
      ImageEffect::Blur { radius } => self.blur(*radius),
      ImageEffect::GaussianBlur { sigma } => self.gaussian_blur(*sigma),
      ImageEffect::MotionBlur { angle, distance } => self.motion_blur(*angle, *distance),
      ImageEffect::Sharpen { amount } => self.sharpen(*amount),
      ImageEffect::UnsharpMask {
        amount,
        radius,
        threshold,
      } => self.unsharp_mask(*amount, *radius, *threshold),
      ImageEffect::EdgeDetection => self.edge_detection(),
      ImageEffect::Emboss { strength } => self.emboss(*strength),
      ImageEffect::Noise { amount, noise_type } => self.noise(*amount, noise_type),
      ImageEffect::Pixelate { size } => self.pixelate(*size),
      ImageEffect::Mosaic { tile_size } => self.mosaic(*tile_size),
      ImageEffect::Glitch { intensity } => self.glitch(*intensity),
      ImageEffect::DataMosh { strength } => self.data_mosh(*strength),
      ImageEffect::ChromaticAberration { amount } => self.chromatic_aberration(*amount),
      ImageEffect::Vignette { amount, radius } => self.vignette(*amount, *radius),
      ImageEffect::LensDistortion { amount } => self.lens_distortion(*amount),
      ImageEffect::Wave {
        amplitude,
        frequency,
        phase,
      } => self.wave(*amplitude, *frequency, *phase),
      ImageEffect::Swirl { strength, center } => self.swirl(*strength, *center),
      ImageEffect::PolarCoordinates => self.polar_coordinates(),
      ImageEffect::Cartoon {
        threshold,
        line_thickness,
      } => self.cartoon(*threshold, *line_thickness),
      ImageEffect::OilPainting { radius, intensity } => self.oil_painting(*radius, *intensity),
      ImageEffect::Watercolor { wetness, pigment } => self.watercolor(*wetness, *pigment),
      ImageEffect::Halftone { dot_size, angle } => self.halftone(*dot_size, *angle),
      ImageEffect::Dithering { method } => self.dithering(*method),
      ImageEffect::Posterize { levels } => self.posterize(*levels),
      ImageEffect::Solarize { threshold } => self.solarize(*threshold),
      ImageEffect::Threshold { level } => self.threshold(*level),
    }
  }

  pub fn grayscale(&mut self) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    if channels == 1 {
      return Ok(());
    }

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, _)) = self.image.get_rgba_pixel(x, y) {
          let gray = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
          self.image.set_grayscale_pixel(x, y, gray)?;
        }
      }
    }

    Ok(())
  }

  pub fn sepia(&mut self) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, a)) = self.image.get_rgba_pixel(x, y) {
          let tr = (0.393 * r as f32 + 0.769 * g as f32 + 0.189 * b as f32).min(255.0) as u8;
          let tg = (0.349 * r as f32 + 0.686 * g as f32 + 0.168 * b as f32).min(255.0) as u8;
          let tb = (0.272 * r as f32 + 0.534 * g as f32 + 0.131 * b as f32).min(255.0) as u8;
          self.image.set_rgba_pixel(x, y, tr, tg, tb, a)?;
        }
      }
    }

    Ok(())
  }

  pub fn invert(&mut self) -> Result<()> {
    let data = self.image.data_mut();
    for byte in data.iter_mut() {
      *byte = !*byte;
    }
    Ok(())
  }

  pub fn brightness(&mut self, amount: f32) -> Result<()> {
    let data = self.image.data_mut();
    for byte in data.iter_mut() {
      let value = *byte as f32 + amount * 255.0;
      *byte = value.clamp(0.0, 255.0) as u8;
    }
    Ok(())
  }

  pub fn contrast(&mut self, amount: f32) -> Result<()> {
    let factor = (259.0 * (amount * 255.0 + 255.0)) / (255.0 * (259.0 - amount * 255.0));
    let data = self.image.data_mut();

    for byte in data.iter_mut() {
      let value = factor * (*byte as f32 - 128.0) + 128.0;
      *byte = value.clamp(0.0, 255.0) as u8;
    }
    Ok(())
  }

  pub fn saturation(&mut self, amount: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, a)) = self.image.get_rgba_pixel(x, y) {
          let gray = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
          let saturation_factor = amount + 1.0;

          let nr = (gray + saturation_factor * (r as f32 - gray)).clamp(0.0, 255.0) as u8;
          let ng = (gray + saturation_factor * (g as f32 - gray)).clamp(0.0, 255.0) as u8;
          let nb = (gray + saturation_factor * (b as f32 - gray)).clamp(0.0, 255.0) as u8;

          self.image.set_rgba_pixel(x, y, nr, ng, nb, a)?;
        }
      }
    }

    Ok(())
  }

  pub fn hue_rotate(&mut self, degrees: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let hue = degrees.to_radians();

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, a)) = self.image.get_rgba_pixel(x, y) {
          let (h, s, l) = rgb_to_hsl(r, g, b);
          let new_h = (h + hue) % (2.0 * std::f32::consts::PI);
          let (nr, ng, nb) = hsl_to_rgb(new_h, s, l);

          self.image.set_rgba_pixel(x, y, nr, ng, nb, a)?;
        }
      }
    }

    Ok(())
  }

  pub fn gamma(&mut self, gamma: f32) -> Result<()> {
    let gamma = gamma.max(0.1);
    let data = self.image.data_mut();

    for byte in data.iter_mut() {
      let value = 255.0 * (*byte as f32 / 255.0).powf(1.0 / gamma);
      *byte = value.clamp(0.0, 255.0) as u8;
    }
    Ok(())
  }

  pub fn exposure(&mut self, exposure: f32) -> Result<()> {
    let data = self.image.data_mut();

    for byte in data.iter_mut() {
      let value = *byte as f32 * (2.0f32).powf(exposure);
      *byte = value.clamp(0.0, 255.0) as u8;
    }
    Ok(())
  }

  pub fn vibrance(&mut self, amount: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, a)) = self.image.get_rgba_pixel(x, y) {
          let max = r.max(g).max(b) as f32;
          let avg = (r as f32 + g as f32 + b as f32) / 3.0;
          let amt = ((max - avg).abs() * 2.0 / 255.0) * amount * 255.0;

          let nr = (r as f32 + amt).clamp(0.0, 255.0) as u8;
          let ng = (g as f32 + amt).clamp(0.0, 255.0) as u8;
          let nb = (b as f32 + amt).clamp(0.0, 255.0) as u8;

          self.image.set_rgba_pixel(x, y, nr, ng, nb, a)?;
        }
      }
    }

    Ok(())
  }

  pub fn temperature(&mut self, temperature: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, a)) = self.image.get_rgba_pixel(x, y) {
          let tr = (r as f32 * (1.0 + temperature)).clamp(0.0, 255.0) as u8;
          let tb = (b as f32 * (1.0 - temperature)).clamp(0.0, 255.0) as u8;

          self.image.set_rgba_pixel(x, y, tr, g, tb, a)?;
        }
      }
    }

    Ok(())
  }

  pub fn tint(&mut self, color: &(u8, u8, u8)) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();

    for y in 0..height {
      for x in 0..width {
        if let Some((r, g, b, a)) = self.image.get_rgba_pixel(x, y) {
          let nr = (r as f32 * color.0 as f32 / 255.0).clamp(0.0, 255.0) as u8;
          let ng = (g as f32 * color.1 as f32 / 255.0).clamp(0.0, 255.0) as u8;
          let nb = (b as f32 * color.2 as f32 / 255.0).clamp(0.0, 255.0) as u8;

          self.image.set_rgba_pixel(x, y, nr, ng, nb, a)?;
        }
      }
    }

    Ok(())
  }

  pub fn blur(&mut self, radius: u32) -> Result<()> {
    let kernel_size = radius * 2 + 1;
    let mut kernel = Vec::with_capacity((kernel_size * kernel_size) as usize);

    for y in 0..kernel_size {
      for x in 0..kernel_size {
        let dx = x as f32 - radius as f32;
        let dy = y as f32 - radius as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        let value = (-distance * distance / (2.0 * radius as f32 * radius as f32)).exp();
        kernel.push(value);
      }
    }

    let sum: f32 = kernel.iter().sum();
    for value in kernel.iter_mut() {
      *value /= sum;
    }

    crate::processing::apply_convolution(self.image.data_mut(), &kernel, kernel_size)
  }

  pub fn gaussian_blur(&mut self, sigma: f32) -> Result<()> {
    let radius = (sigma * 2.0) as u32;
    let kernel_size = radius * 2 + 1;
    let mut kernel = Vec::with_capacity((kernel_size * kernel_size) as usize);

    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut sum = 0.0f32;

    for y in 0..kernel_size {
      for x in 0..kernel_size {
        let dx = x as f32 - radius as f32;
        let dy = y as f32 - radius as f32;
        let value = (-(dx * dx + dy * dy) / two_sigma_sq).exp();
        kernel.push(value);
        sum += value;
      }
    }

    for value in kernel.iter_mut() {
      *value /= sum;
    }

    crate::processing::apply_convolution(self.image.data_mut(), &kernel, kernel_size)
  }

  pub fn motion_blur(&mut self, angle: f32, distance: u32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();
    let data = self.image.data().to_vec();
    let mut new_data = data.clone();

    let angle_rad = angle.to_radians();
    let dx = angle_rad.cos();
    let dy = angle_rad.sin();

    for y in 0..height {
      for x in 0..width {
        for c in 0..channels {
          let mut sum = 0u32;
          let mut count = 0u32;

          for i in 0..=distance {
            let sx = (x as f32 + dx * i as f32)
              .round()
              .clamp(0.0, width as f32 - 1.0) as u32;
            let sy = (y as f32 + dy * i as f32)
              .round()
              .clamp(0.0, height as f32 - 1.0) as u32;

            if let Some(value) = self.image.get_pixel_value(sx, sy, c) {
              sum += value as u32;
              count += 1;
            }
          }

          let avg = (sum / count) as u8;
          let offset = ((y * width + x) * channels as u32 + c as u32) as usize;
          new_data[offset] = avg;
        }
      }
    }

    *self.image.data_mut() = new_data;
    Ok(())
  }

  pub fn sharpen(&mut self, amount: f32) -> Result<()> {
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

    crate::processing::apply_convolution(self.image.data_mut(), &kernel, 3)
  }

  pub fn unsharp_mask(&mut self, amount: f32, radius: u32, threshold: f32) -> Result<()> {
    let original_data = self.image.data().to_vec();
    self.gaussian_blur(radius as f32)?;

    let data = self.image.data_mut();
    for (i, &original_byte) in original_data.iter().enumerate() {
      let blurred_byte = data[i];
      let difference = original_byte as f32 - blurred_byte as f32;

      if difference.abs() > threshold * 255.0 {
        let sharpened = original_byte as f32 + amount * difference;
        data[i] = sharpened.clamp(0.0, 255.0) as u8;
      } else {
        data[i] = original_byte;
      }
    }

    Ok(())
  }

  pub fn edge_detection(&mut self) -> Result<()> {
    let kernel = [-1.0, -1.0, -1.0, -1.0, 8.0, -1.0, -1.0, -1.0, -1.0];

    crate::processing::apply_convolution(self.image.data_mut(), &kernel, 3)
  }

  pub fn emboss(&mut self, strength: f32) -> Result<()> {
    let kernel = [
      -2.0 * strength,
      -1.0 * strength,
      0.0,
      -1.0 * strength,
      1.0,
      1.0 * strength,
      0.0,
      1.0 * strength,
      2.0 * strength,
    ];

    crate::processing::apply_convolution(self.image.data_mut(), &kernel, 3)
  }

  pub fn noise(&mut self, amount: f32, noise_type: &NoiseType) -> Result<()> {
    let noise_gen = create_noise_generator_with_seed(42);
    noise_gen.apply_to_bytes(self.image.data_mut(), amount);
    Ok(())
  }

  pub fn pixelate(&mut self, size: u32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    for y in (0..height).step_by(size as usize) {
      for x in (0..width).step_by(size as usize) {
        let end_x = std::cmp::min(x + size, width);
        let end_y = std::cmp::min(y + size, height);

        let mut sum = vec![0u32; channels as usize];
        let mut count = 0u32;

        for py in y..end_y {
          for px in x..end_x {
            for c in 0..channels {
              if let Some(value) = self.image.get_pixel_value(px, py, c) {
                sum[c as usize] += value as u32;
              }
            }
            count += 1;
          }
        }

        for py in y..end_y {
          for px in x..end_x {
            for c in 0..channels {
              let avg = (sum[c as usize] / count) as u8;
              self.image.set_pixel_value(px, py, c, avg)?;
            }
          }
        }
      }
    }

    Ok(())
  }

  pub fn mosaic(&mut self, tile_size: u32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let tiles_x = (width + tile_size - 1) / tile_size;
    let tiles_y = (height + tile_size - 1) / tile_size;

    let mut tiles = Vec::new();
    for ty in 0..tiles_y {
      for tx in 0..tiles_x {
        let start_x = tx * tile_size;
        let start_y = ty * tile_size;
        let end_x = std::cmp::min(start_x + tile_size, width);
        let end_y = std::cmp::min(start_y + tile_size, height);

        let mut tile_data = Vec::new();
        for py in start_y..end_y {
          for px in start_x..end_x {
            if let Some(pixel) = self.image.get_pixel(px, py) {
              tile_data.push(pixel.to_vec());
            }
          }
        }
        tiles.push(tile_data);
      }
    }

    let mut rng = create_random_generator();
    rng.shuffle(&mut tiles);

    let mut tile_index = 0;
    for ty in 0..tiles_y {
      for tx in 0..tiles_x {
        let start_x = tx * tile_size;
        let start_y = ty * tile_size;
        let end_x = std::cmp::min(start_x + tile_size, width);
        let end_y = std::cmp::min(start_y + tile_size, height);

        if let Some(tile_data) = tiles.get(tile_index) {
          let mut data_index = 0;
          for py in start_y..end_y {
            for px in start_x..end_x {
              if let Some(pixel) = tile_data.get(data_index) {
                self.image.set_pixel(px, py, pixel)?;
                data_index += 1;
              }
            }
          }
        }
        tile_index += 1;
      }
    }

    Ok(())
  }

  pub fn glitch(&mut self, intensity: f32) -> Result<()> {
    let data = self.image.data_mut();
    let glitch_count = (data.len() as f32 * intensity) as usize;
    let mut rng = create_random_generator();

    for _ in 0..glitch_count {
      let start = rng.gen_range(0, data.len() as u64) as usize;
      let end = std::cmp::min(start + rng.gen_range(1, 100) as usize, data.len());

      for i in start..end {
        data[i] = rng.gen_u8();
      }
    }

    Ok(())
  }

  pub fn data_mosh(&mut self, strength: f32) -> Result<()> {
    let data = self.image.data_mut();
    let mosh_count = (data.len() as f32 * strength) as usize;
    let mut rng = create_random_generator();

    for _ in 0..mosh_count {
      let pos = rng.gen_range(0, data.len() as u64) as usize;
      let operation = rng.gen_range(0, 4);

      match operation {
        0 => data[pos] ^= rng.gen_u8(),
        1 => data[pos] = data[pos].wrapping_add(rng.gen_u8()),
        2 => data[pos] = data[pos].wrapping_mul(rng.gen_range(1, 10) as u8),
        3 => data[pos] = data[pos].rotate_left(rng.gen_range(1, 8)),
        _ => {}
      }
    }

    Ok(())
  }

  pub fn chromatic_aberration(&mut self, amount: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();

    if channels < 3 {
      return Ok(());
    }

    let original_data = self.image.data().to_vec();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        let offset = ((x as f32 - width as f32 / 2.0) / width as f32 * amount).round() as i32;

        for c in 0..3 {
          let channel_offset = match c {
            0 => -offset,
            1 => 0,
            2 => offset,
            _ => 0,
          };

          let sx = (x as i32 + channel_offset).max(0).min(width as i32 - 1) as u32;
          let sy = y;

          if let Some(value) = self.image.get_pixel_value(sx, sy, c) {
            let offset = ((y * width + x) * channels as u32 + c as u32) as usize;
            data[offset] = value;
          }
        }
      }
    }

    Ok(())
  }

  pub fn vignette(&mut self, amount: f32, radius: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_distance = (center_x * center_x + center_y * center_y).sqrt() * radius;

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let vignette = (1.0 - (distance / max_distance).clamp(0.0, 1.0)).powf(2.0);
        let factor = 1.0 - amount * (1.0 - vignette);

        for c in 0..self.image.channels() {
          if let Some(value) = self.image.get_pixel_value(x, y, c) {
            let new_value = (value as f32 * factor).clamp(0.0, 255.0) as u8;
            self.image.set_pixel_value(x, y, c, new_value)?;
          }
        }
      }
    }

    Ok(())
  }

  pub fn lens_distortion(&mut self, amount: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_radius = center_x.min(center_y);

    let original_data = self.image.data().to_vec();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        let distortion = 1.0 + amount * (distance / max_radius).powf(2.0);
        let src_x = (dx / distortion + center_x)
          .round()
          .clamp(0.0, width as f32 - 1.0) as u32;
        let src_y = (dy / distortion + center_y)
          .round()
          .clamp(0.0, height as f32 - 1.0) as u32;

        for c in 0..self.image.channels() {
          if let Some(value) = self.image.get_pixel_value(src_x, src_y, c) {
            let offset = ((y * width + x) * self.image.channels() as u32 + c as u32) as usize;
            data[offset] = value;
          }
        }
      }
    }

    Ok(())
  }

  pub fn wave(&mut self, amplitude: f32, frequency: f32, phase: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let original_data = self.image.data().to_vec();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        let wave = (x as f32 * frequency + phase).sin() * amplitude;
        let src_y = (y as f32 + wave).round().clamp(0.0, height as f32 - 1.0) as u32;

        for c in 0..self.image.channels() {
          if let Some(value) = self.image.get_pixel_value(x, src_y, c) {
            let offset = ((y * width + x) * self.image.channels() as u32 + c as u32) as usize;
            data[offset] = value;
          }
        }
      }
    }

    Ok(())
  }

  pub fn swirl(&mut self, strength: f32, center: (u32, u32)) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let center_x = center.0 as f32;
    let center_y = center.1 as f32;
    let max_radius = center_x
      .min(center_y)
      .min((width as f32 - center_x).min(height as f32 - center_y));

    let original_data = self.image.data().to_vec();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < max_radius {
          let angle = dy.atan2(dx);
          let swirl_angle = strength * (1.0 - distance / max_radius);
          let new_angle = angle + swirl_angle;

          let src_x = (distance * new_angle.cos() + center_x)
            .round()
            .clamp(0.0, width as f32 - 1.0) as u32;
          let src_y = (distance * new_angle.sin() + center_y)
            .round()
            .clamp(0.0, height as f32 - 1.0) as u32;

          for c in 0..self.image.channels() {
            if let Some(value) = self.image.get_pixel_value(src_x, src_y, c) {
              let offset = ((y * width + x) * self.image.channels() as u32 + c as u32) as usize;
              data[offset] = value;
            }
          }
        }
      }
    }

    Ok(())
  }

  pub fn polar_coordinates(&mut self) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_radius = center_x.min(center_y);

    let original_data = self.image.data().to_vec();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx);

        let src_x = ((angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI) * width as f32)
          .round()
          .clamp(0.0, width as f32 - 1.0) as u32;
        let src_y = (distance / max_radius * height as f32)
          .round()
          .clamp(0.0, height as f32 - 1.0) as u32;

        for c in 0..self.image.channels() {
          if let Some(value) = self.image.get_pixel_value(src_x, src_y, c) {
            let offset = ((y * width + x) * self.image.channels() as u32 + c as u32) as usize;
            data[offset] = value;
          }
        }
      }
    }

    Ok(())
  }

  pub fn cartoon(&mut self, threshold: f32, line_thickness: u32) -> Result<()> {
    self.edge_detection()?;
    self.posterize((threshold * 255.0) as u8)?;

    for _ in 0..line_thickness {
      self.edge_detection()?;
    }

    Ok(())
  }

  pub fn oil_painting(&mut self, radius: u32, intensity: f32) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();
    let original_data = self.image.data().to_vec();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        let mut intensity_sum = vec![0f32; channels as usize];
        let mut color_sum = vec![0f32; channels as usize];
        let mut max_intensity = 0f32;

        for dy in -(radius as i32)..=(radius as i32) {
          for dx in -(radius as i32)..=(radius as i32) {
            let sx = (x as i32 + dx).max(0).min(width as i32 - 1) as u32;
            let sy = (y as i32 + dy).max(0).min(height as i32 - 1) as u32;

            for c in 0..channels {
              if let Some(value) = self.image.get_pixel_value(sx, sy, c) {
                let intensity_value = value as f32;
                intensity_sum[c] += intensity_value;
                color_sum[c] += intensity_value * intensity_value;

                if intensity_value > max_intensity {
                  max_intensity = intensity_value;
                }
              }
            }
          }
        }

        for c in 0..channels {
          let avg_intensity = intensity_sum[c] / ((2 * radius + 1) * (2 * radius + 1)) as f32;
          let factor = (max_intensity - avg_intensity).abs() / 255.0;
          let new_value = (avg_intensity * (1.0 - intensity)
            + color_sum[c] / intensity_sum[c] * intensity * factor)
            .clamp(0.0, 255.0) as u8;

          let offset = ((y * width + x) * channels as u32 + c as u32) as usize;
          data[offset] = new_value;
        }
      }
    }

    Ok(())
  }

  pub fn watercolor(&mut self, wetness: f32, pigment: f32) -> Result<()> {
    self.gaussian_blur(wetness)?;
    self.oil_painting(3, pigment)?;
    self.contrast(0.2)
  }

  pub fn halftone(&mut self, dot_size: u32, angle: f32) -> Result<()> {
    self.grayscale()?;

    let width = self.image.width();
    let height = self.image.height();
    let data = self.image.data_mut();

    for y in (0..height).step_by(dot_size as usize) {
      for x in (0..width).step_by(dot_size as usize) {
        let mut sum = 0u32;
        let mut count = 0u32;

        for dy in 0..dot_size {
          for dx in 0..dot_size {
            let sx = x + dx;
            let sy = y + dy;

            if sx < width && sy < height {
              if let Some(value) = self.image.get_pixel_value(sx, sy, 0) {
                sum += value as u32;
                count += 1;
              }
            }
          }
        }

        let avg = if count > 0 { sum / count } else { 0 };
        let dot_value = if avg > 128 { 255 } else { 0 };

        for dy in 0..dot_size {
          for dx in 0..dot_size {
            let sx = x + dx;
            let sy = y + dy;

            if sx < width && sy < height {
              let offset = (sy * width + sx) as usize;
              data[offset] = dot_value;
            }
          }
        }
      }
    }

    Ok(())
  }

  pub fn dithering(&mut self, method: DitheringMethod) -> Result<()> {
    match method {
      DitheringMethod::FloydSteinberg => self.floyd_steinberg_dithering(),
      DitheringMethod::Ordered => self.ordered_dithering(),
      DitheringMethod::Random => self.random_dithering(),
    }
  }

  fn floyd_steinberg_dithering(&mut self) -> Result<()> {
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        for c in 0..channels {
          let offset = ((y * width + x) * channels as u32 + c as u32) as usize;
          let old_pixel = data[offset];
          let new_pixel = if old_pixel > 127 { 255 } else { 0 };
          let error = old_pixel as i32 - new_pixel as i32;

          data[offset] = new_pixel;

          if x + 1 < width {
            let right_offset = ((y * width + x + 1) * channels as u32 + c as u32) as usize;
            data[right_offset] = (data[right_offset] as i32 + error * 7 / 16).clamp(0, 255) as u8;
          }

          if y + 1 < height {
            if x > 0 {
              let bottom_left_offset =
                (((y + 1) * width + x - 1) * channels as u32 + c as u32) as usize;
              data[bottom_left_offset] =
                (data[bottom_left_offset] as i32 + error * 3 / 16).clamp(0, 255) as u8;
            }

            let bottom_offset = (((y + 1) * width + x) * channels as u32 + c as u32) as usize;
            data[bottom_offset] = (data[bottom_offset] as i32 + error * 5 / 16).clamp(0, 255) as u8;

            if x + 1 < width {
              let bottom_right_offset =
                (((y + 1) * width + x + 1) * channels as u32 + c as u32) as usize;
              data[bottom_right_offset] =
                (data[bottom_right_offset] as i32 + error * 1 / 16).clamp(0, 255) as u8;
            }
          }
        }
      }
    }

    Ok(())
  }

  fn ordered_dithering(&mut self) -> Result<()> {
    let matrix = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        let threshold = matrix[(y % 4) as usize][(x % 4) as usize] * 16 + 8;

        for c in 0..channels {
          let offset = ((y * width + x) * channels as u32 + c as u32) as usize;
          data[offset] = if data[offset] as u32 > threshold {
            255
          } else {
            0
          };
        }
      }
    }

    Ok(())
  }

  fn random_dithering(&mut self) -> Result<()> {
    let mut rng = create_random_generator();
    let width = self.image.width();
    let height = self.image.height();
    let channels = self.image.channels();
    let data = self.image.data_mut();

    for y in 0..height {
      for x in 0..width {
        for c in 0..channels {
          let offset = ((y * width + x) * channels as u32 + c as u32) as usize;
          let threshold = rng.gen_u8();
          data[offset] = if data[offset] > threshold { 255 } else { 0 };
        }
      }
    }

    Ok(())
  }

  pub fn posterize(&mut self, levels: u8) -> Result<()> {
    if levels < 2 {
      return Err(EllasticError::InvalidParameter(
        "Levels must be at least 2".to_string(),
      ));
    }

    let data = self.image.data_mut();
    let factor = 255.0 / (levels - 1) as f32;

    for byte in data.iter_mut() {
      *byte = ((*byte as f32 / factor).round() * factor).clamp(0.0, 255.0) as u8;
    }

    Ok(())
  }

  pub fn solarize(&mut self, threshold: u8) -> Result<()> {
    let data = self.image.data_mut();

    for byte in data.iter_mut() {
      *byte = if *byte > threshold {
        255 - *byte
      } else {
        *byte
      };
    }

    Ok(())
  }

  pub fn threshold(&mut self, level: u8) -> Result<()> {
    let data = self.image.data_mut();

    for byte in data.iter_mut() {
      *byte = if *byte > level { 255 } else { 0 };
    }

    Ok(())
  }
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
  let rf = r as f32 / 255.0;
  let gf = g as f32 / 255.0;
  let bf = b as f32 / 255.0;

  let max = rf.max(gf).max(bf);
  let min = rf.min(gf).min(bf);
  let l = (max + min) / 2.0;

  if max == min {
    return (0.0, 0.0, l);
  }

  let d = max - min;
  let s = if l > 0.5 {
    d / (2.0 - max - min)
  } else {
    d / (max + min)
  };

  let h = match max {
    x if x == rf => ((gf - bf) / d + if gf < bf { 6.0 } else { 0.0 }) / 6.0,
    x if x == gf => ((bf - rf) / d + 2.0) / 6.0,
    _ => ((rf - gf) / d + 4.0) / 6.0,
  };

  (h * 2.0 * std::f32::consts::PI, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
  if s == 0.0 {
    let gray = (l * 255.0).round() as u8;
    return (gray, gray, gray);
  }

  let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
  let x = c * (1.0 - ((h / (2.0 * std::f32::consts::PI / 3.0)) % 2.0 - 1.0).abs());
  let m = l - c / 2.0;

  let (r1, g1, b1) = if h < 2.0 * std::f32::consts::PI / 3.0 {
    (c, x, 0.0)
  } else if h < 4.0 * std::f32::consts::PI / 3.0 {
    (x, c, 0.0)
  } else if h < 6.0 * std::f32::consts::PI / 3.0 {
    (0.0, c, x)
  } else if h < 8.0 * std::f32::consts::PI / 3.0 {
    (0.0, x, c)
  } else if h < 10.0 * std::f32::consts::PI / 3.0 {
    (x, 0.0, c)
  } else {
    (c, 0.0, x)
  };

  let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
  let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
  let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;

  (r, g, b)
}

#[derive(Debug, Clone)]
pub enum ImageEffect {
  Grayscale,
  Sepia,
  Invert,
  Brightness {
    amount: f32,
  },
  Contrast {
    amount: f32,
  },
  Saturation {
    amount: f32,
  },
  HueRotate {
    degrees: f32,
  },
  Gamma {
    gamma: f32,
  },
  Exposure {
    exposure: f32,
  },
  Vibrance {
    amount: f32,
  },
  Temperature {
    temperature: f32,
  },
  Tint {
    color: (u8, u8, u8),
  },
  Blur {
    radius: u32,
  },
  GaussianBlur {
    sigma: f32,
  },
  MotionBlur {
    angle: f32,
    distance: u32,
  },
  Sharpen {
    amount: f32,
  },
  UnsharpMask {
    amount: f32,
    radius: u32,
    threshold: f32,
  },
  EdgeDetection,
  Emboss {
    strength: f32,
  },
  Noise {
    amount: f32,
    noise_type: crate::corruption::NoiseType,
  },
  Pixelate {
    size: u32,
  },
  Mosaic {
    tile_size: u32,
  },
  Glitch {
    intensity: f32,
  },
  DataMosh {
    strength: f32,
  },
  ChromaticAberration {
    amount: f32,
  },
  Vignette {
    amount: f32,
    radius: f32,
  },
  LensDistortion {
    amount: f32,
  },
  Wave {
    amplitude: f32,
    frequency: f32,
    phase: f32,
  },
  Swirl {
    strength: f32,
    center: (u32, u32),
  },
  PolarCoordinates,
  Cartoon {
    threshold: f32,
    line_thickness: u32,
  },
  OilPainting {
    radius: u32,
    intensity: f32,
  },
  Watercolor {
    wetness: f32,
    pigment: f32,
  },
  Halftone {
    dot_size: u32,
    angle: f32,
  },
  Dithering {
    method: DitheringMethod,
  },
  Posterize {
    levels: u8,
  },
  Solarize {
    threshold: u8,
  },
  Threshold {
    level: u8,
  },
}

#[derive(Debug, Clone)]
pub enum DitheringMethod {
  FloydSteinberg,
  Ordered,
  Random,
}

pub fn create_effect_processor(image: ImageProcessor) -> ImageEffectProcessor {
  ImageEffectProcessor::new(image)
}
