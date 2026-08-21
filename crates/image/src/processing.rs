use crate::{
  ColorSpace,
  ImageData,
  ImageFormat,
  ImageProcessor,
  ResizeFilter,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use ellastic_utils::create_random_generator;
use rayon::prelude::*;

pub fn decode_image(buffer: &[u8], format: ImageFormat) -> Result<ImageData> {
  let image = match format {
    ImageFormat::PNG => image::load_from_memory(buffer)
      .map_err(|e| EllasticError::IoError(format!("PNG decode error: {}", e)))?,
    ImageFormat::JPEG => image::load_from_memory(buffer)
      .map_err(|e| EllasticError::IoError(format!("JPEG decode error: {}", e)))?,
    ImageFormat::BMP => image::load_from_memory(buffer)
      .map_err(|e| EllasticError::IoError(format!("BMP decode error: {}", e)))?,
    ImageFormat::GIF => image::load_from_memory(buffer)
      .map_err(|e| EllasticError::IoError(format!("GIF decode error: {}", e)))?,
    ImageFormat::TIFF => image::load_from_memory(buffer)
      .map_err(|e| EllasticError::IoError(format!("TIFF decode error: {}", e)))?,
    ImageFormat::WEBP => image::load_from_memory(buffer)
      .map_err(|e| EllasticError::IoError(format!("WEBP decode error: {}", e)))?,
    _ => {
      return Err(EllasticError::UnsupportedFormat(format!(
        "Unsupported format: {:?}",
        format
      )));
    }
  };

  let (width, height) = (image.width(), image.height());
  let channels = match image.color() {
    image::ColorType::L8 => 1,
    image::ColorType::La8 => 2,
    image::ColorType::Rgb8 => 3,
    image::ColorType::Rgba8 => 4,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported color type".to_string(),
      ));
    }
  };

  let data = image.to_rgba8().into_raw();
  let final_data = if channels == 4 {
    data
  } else if channels == 3 {
    data
      .chunks_exact(4)
      .flat_map(|rgba| [rgba[0], rgba[1], rgba[2]])
      .collect()
  } else if channels == 2 {
    data
      .chunks_exact(4)
      .flat_map(|rgba| [rgba[0], rgba[3]])
      .collect()
  } else {
    data.chunks_exact(4).map(|rgba| rgba[0]).collect()
  };

  ImageData::new(width, height, channels, final_data)
}

pub fn encode_image(
  image_data: &ImageData,
  format: ImageFormat,
  quality: Option<u8>,
) -> Result<Vec<u8>> {
  let width = image_data.width;
  let height = image_data.height;
  let channels = image_data.channels;

  let rgba_data = if channels == 4 {
    image_data.data.clone()
  } else if channels == 3 {
    let mut rgba = Vec::with_capacity(image_data.data.len() / 3 * 4);
    for chunk in image_data.data.chunks_exact(3) {
      rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
    }
    rgba
  } else if channels == 2 {
    let mut rgba = Vec::with_capacity(image_data.data.len() / 2 * 4);
    for chunk in image_data.data.chunks_exact(2) {
      rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
    }
    rgba
  } else {
    let mut rgba = Vec::with_capacity(image_data.data.len() * 4);
    for &gray in &image_data.data {
      rgba.extend_from_slice(&[gray, gray, gray, 255]);
    }
    rgba
  };

  let image_buffer = image::RgbaImage::from_raw(width, height, rgba_data)
    .ok_or_else(|| EllasticError::ProcessingError("Failed to create image buffer".to_string()))?;

  let mut buffer = Vec::new();
  {
    let mut cursor = std::io::Cursor::new(&mut buffer);
    match format {
      ImageFormat::PNG => {
        image_buffer
          .write_to(&mut cursor, image::ImageFormat::Png)
          .map_err(|e| EllasticError::IoError(format!("PNG encode error: {}", e)))?;
      }
      ImageFormat::JPEG => {
        let mut encoder =
          image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality.unwrap_or(90));
        encoder
          .encode(
            image::DynamicImage::ImageRgba8(image_buffer).as_bytes(),
            width,
            height,
            image::ColorType::Rgba8,
          )
          .map_err(|e| EllasticError::IoError(format!("JPEG encode error: {}", e)))?;
      }
      ImageFormat::BMP => {
        image_buffer
          .write_to(&mut cursor, image::ImageFormat::Bmp)
          .map_err(|e| EllasticError::IoError(format!("BMP encode error: {}", e)))?;
      }
      ImageFormat::TIFF => {
        image_buffer
          .write_to(&mut cursor, image::ImageFormat::Tiff)
          .map_err(|e| EllasticError::IoError(format!("TIFF encode error: {}", e)))?;
      }
      _ => {
        return Err(EllasticError::UnsupportedFormat(format!(
          "Unsupported output format: {:?}",
          format
        )));
      }
    }
  }

  Ok(buffer)
}

pub fn resize_image(
  image_data: &ImageData,
  new_width: u32,
  new_height: u32,
  filter: ResizeFilter,
) -> Result<ImageData> {
  let rgba_data = convert_to_rgba(image_data)?;
  let image_buffer = image::RgbaImage::from_raw(image_data.width, image_data.height, rgba_data)
    .ok_or_else(|| EllasticError::ProcessingError("Failed to create image buffer".to_string()))?;

  let filter_type = match filter {
    ResizeFilter::Nearest => image::imageops::FilterType::Nearest,
    ResizeFilter::Linear => image::imageops::FilterType::Triangle,
    ResizeFilter::Cubic => image::imageops::FilterType::CatmullRom,
    ResizeFilter::Lanczos => image::imageops::FilterType::Lanczos3,
  };

  let resized = image::imageops::resize(&image_buffer, new_width, new_height, filter_type);

  let channels = image_data.channels;
  let final_data = if channels == 4 {
    resized.into_raw()
  } else if channels == 3 {
    resized
      .into_raw()
      .chunks_exact(4)
      .flat_map(|rgba| [rgba[0], rgba[1], rgba[2]])
      .collect()
  } else if channels == 2 {
    resized
      .into_raw()
      .chunks_exact(4)
      .flat_map(|rgba| [rgba[0], rgba[3]])
      .collect()
  } else {
    resized
      .into_raw()
      .chunks_exact(4)
      .map(|rgba| rgba[0])
      .collect()
  };

  ImageData::new(new_width, new_height, channels, final_data)
}

pub fn crop_image(
  image_data: &ImageData,
  x: u32,
  y: u32,
  width: u32,
  height: u32,
) -> Result<ImageData> {
  if x + width > image_data.width || y + height > image_data.height {
    return Err(EllasticError::CoordinatesOutOfBounds { x, y });
  }

  let rgba_data = convert_to_rgba(image_data)?;
  let image_buffer = image::RgbaImage::from_raw(image_data.width, image_data.height, rgba_data)
    .ok_or_else(|| EllasticError::ProcessingError("Failed to create image buffer".to_string()))?;

  let cropped = image::imageops::crop(&image_buffer, x, y, width, height).to_image();

  let channels = image_data.channels;
  let final_data = if channels == 4 {
    cropped.into_raw()
  } else if channels == 3 {
    cropped
      .into_raw()
      .chunks_exact(4)
      .flat_map(|rgba| [rgba[0], rgba[1], rgba[2]])
      .collect()
  } else if channels == 2 {
    cropped
      .into_raw()
      .chunks_exact(4)
      .flat_map(|rgba| [rgba[0], rgba[3]])
      .collect()
  } else {
    cropped
      .into_raw()
      .chunks_exact(4)
      .map(|rgba| rgba[0])
      .collect()
  };

  ImageData::new(width, height, channels, final_data)
}

pub fn flip_image_horizontal(image_data: &mut ImageData) -> Result<()> {
  let channels = image_data.channels;
  let width = image_data.width;
  let height = image_data.height;

  for y in 0..height {
    for x in 0..width / 2 {
      let left_pixel = image_data.get_pixel(x, y).unwrap();
      let right_pixel = image_data.get_pixel(width - 1 - x, y).unwrap();

      image_data.set_pixel(x, y, right_pixel)?;
      image_data.set_pixel(width - 1 - x, y, left_pixel)?;
    }
  }

  Ok(())
}

pub fn flip_image_vertical(image_data: &mut ImageData) -> Result<()> {
  let channels = image_data.channels;
  let width = image_data.width;
  let height = image_data.height;

  for y in 0..height / 2 {
    for x in 0..width {
      let top_pixel = image_data.get_pixel(x, y).unwrap();
      let bottom_pixel = image_data.get_pixel(x, height - 1 - y).unwrap();

      image_data.set_pixel(x, y, bottom_pixel)?;
      image_data.set_pixel(x, height - 1 - y, top_pixel)?;
    }
  }

  Ok(())
}

pub fn rotate_image_90(image_data: &ImageData) -> Result<ImageData> {
  let new_width = image_data.height;
  let new_height = image_data.width;
  let channels = image_data.channels;
  let mut new_data = vec![0u8; (new_width * new_height * channels as u32) as usize];

  for y in 0..image_data.height {
    for x in 0..image_data.width {
      let old_pixel = image_data.get_pixel(x, y).unwrap();
      let new_x = new_width - 1 - y;
      let new_y = x;

      let offset = ((new_y * new_width + new_x) * channels as u32) as usize;
      new_data[offset..offset + channels as usize].copy_from_slice(old_pixel);
    }
  }

  ImageData::new(new_width, new_height, channels, new_data)
}

pub fn rotate_image_180(image_data: &ImageData) -> Result<ImageData> {
  let width = image_data.width;
  let height = image_data.height;
  let channels = image_data.channels;
  let mut new_data = vec![0u8; image_data.data.len()];

  for y in 0..height {
    for x in 0..width {
      let old_pixel = image_data.get_pixel(x, y).unwrap();
      let new_x = width - 1 - x;
      let new_y = height - 1 - y;

      let offset = ((new_y * width + new_x) * channels as u32) as usize;
      new_data[offset..offset + channels as usize].copy_from_slice(old_pixel);
    }
  }

  ImageData::new(width, height, channels, new_data)
}

pub fn rotate_image_270(image_data: &ImageData) -> Result<ImageData> {
  let new_width = image_data.height;
  let new_height = image_data.width;
  let channels = image_data.channels;
  let mut new_data = vec![0u8; (new_width * new_height * channels as u32) as usize];

  for y in 0..image_data.height {
    for x in 0..image_data.width {
      let old_pixel = image_data.get_pixel(x, y).unwrap();
      let new_x = y;
      let new_y = new_height - 1 - x;

      let offset = ((new_y * new_width + new_x) * channels as u32) as usize;
      new_data[offset..offset + channels as usize].copy_from_slice(old_pixel);
    }
  }

  ImageData::new(new_width, new_height, channels, new_data)
}

pub fn convert_colorspace(
  image_data: &ImageData,
  from: ColorSpace,
  to: ColorSpace,
) -> Result<ImageData> {
  if from == to {
    return Ok(image_data.clone());
  }

  let rgba_data = convert_to_rgba(image_data)?;
  let image_buffer = image::RgbaImage::from_raw(image_data.width, image_data.height, rgba_data)
    .ok_or_else(|| EllasticError::ProcessingError("Failed to create image buffer".to_string()))?;

  let converted = match to {
    ColorSpace::RGB => {
      let rgb_buffer = image::DynamicImage::ImageRgba8(image_buffer).to_rgb8();
      rgb_buffer.into_raw()
    }
    ColorSpace::GRAYSCALE => {
      let gray_buffer = image::DynamicImage::ImageRgba8(image_buffer).to_luma8();
      gray_buffer.into_raw()
    }
    _ => {
      return Err(EllasticError::UnsupportedFormat(format!(
        "Unsupported target colorspace: {:?}",
        to
      )));
    }
  };

  let channels = match to {
    ColorSpace::RGB => 3,
    ColorSpace::GRAYSCALE => 1,
    _ => 4,
  };

  ImageData::new(image_data.width, image_data.height, channels, converted)
}

pub fn convert_to_grayscale(image_data: &ImageData) -> Result<ImageData> {
  convert_colorspace(image_data, ColorSpace::RGBA, ColorSpace::GRAYSCALE)
}

pub fn convert_to_rgb(image_data: &ImageData) -> Result<ImageData> {
  convert_colorspace(image_data, ColorSpace::RGBA, ColorSpace::RGB)
}

pub fn convert_to_rgba(image_data: &ImageData) -> Vec<u8> {
  let channels = image_data.channels;
  let width = image_data.width;
  let height = image_data.height;

  if channels == 4 {
    image_data.data.clone()
  } else if channels == 3 {
    let mut rgba = Vec::with_capacity(image_data.data.len() / 3 * 4);
    for chunk in image_data.data.chunks_exact(3) {
      rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
    }
    rgba
  } else if channels == 2 {
    let mut rgba = Vec::with_capacity(image_data.data.len() / 2 * 4);
    for chunk in image_data.data.chunks_exact(2) {
      rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
    }
    rgba
  } else {
    let mut rgba = Vec::with_capacity(image_data.data.len() * 4);
    for &gray in &image_data.data {
      rgba.extend_from_slice(&[gray, gray, gray, 255]);
    }
    rgba
  }
}

pub fn apply_image_filter(image_data: &mut ImageData, filter: &crate::ImageFilter) -> Result<()> {
  match filter.filter_type {
    crate::FilterType::Brightness => adjust_brightness(image_data, filter.parameters[0]),
    crate::FilterType::Contrast => adjust_contrast(image_data, filter.parameters[0]),
    crate::FilterType::Saturation => adjust_saturation(image_data, filter.parameters[0]),
    crate::FilterType::Gamma => apply_gamma(image_data, filter.parameters[0]),
    crate::FilterType::Blur => apply_blur(image_data, filter.parameters[0] as u32),
    crate::FilterType::Sharpen => apply_sharpen(image_data, filter.parameters[0] as u32),
    crate::FilterType::Noise => add_noise(image_data, filter.parameters[0]),
    crate::FilterType::Pixelate => pixelate(image_data, filter.parameters[0] as u32),
    _ => Err(EllasticError::UnsupportedFormat(
      "Filter not implemented".to_string(),
    )),
  }
}

pub fn apply_convolution(
  image_data: &mut ImageData,
  kernel: &[f32],
  kernel_size: u32,
) -> Result<()> {
  if kernel_size * kernel_size != kernel.len() as u32 {
    return Err(EllasticError::InvalidParameter(
      "Kernel size mismatch".to_string(),
    ));
  }

  let channels = image_data.channels;
  let width = image_data.width;
  let height = image_data.height;
  let mut new_data = image_data.data.clone();

  let kernel_offset = (kernel_size / 2) as isize;
  let kernel_size = kernel_size as usize;

  for y in 0..height {
    for x in 0..width {
      for c in 0..channels {
        let mut sum = 0.0f32;

        for ky in 0..kernel_size {
          for kx in 0..kernel_size {
            let px = (x as isize + kx as isize - kernel_offset)
              .max(0)
              .min(width as isize - 1) as u32;
            let py = (y as isize + ky as isize - kernel_offset)
              .max(0)
              .min(height as isize - 1) as u32;

            let pixel_value = image_data.get_pixel_value(px, py, c).unwrap_or(0) as f32;
            let kernel_value = kernel[ky * kernel_size + kx];
            sum += pixel_value * kernel_value;
          }
        }

        let result = sum.clamp(0.0, 255.0) as u8;
        let offset = ((y * width + x) * channels as u32 + c as u32) as usize;
        new_data[offset] = result;
      }
    }
  }

  image_data.data = new_data;
  Ok(())
}

pub fn blend_images(
  image1: &ImageData,
  image2: &ImageData,
  mode: crate::BlendMode,
  opacity: f32,
) -> Result<ImageData> {
  if image1.width != image2.width || image1.height != image2.height {
    return Err(EllasticError::InvalidParameter(
      "Image dimensions must match".to_string(),
    ));
  }

  let channels = std::cmp::max(image1.channels, image2.channels);
  let width = image1.width;
  let height = image1.height;
  let mut result_data = vec![0u8; (width * height * channels as u32) as usize];

  for y in 0..height {
    for x in 0..width {
      for c in 0..channels {
        let val1 = image1.get_pixel_value(x, y, c).unwrap_or(0) as f32;
        let val2 = image2.get_pixel_value(x, y, c).unwrap_or(0) as f32;

        let blended = match mode {
          crate::BlendMode::Normal => val1 * (1.0 - opacity) + val2 * opacity,
          crate::BlendMode::Multiply => (val1 * val2 / 255.0) * opacity + val1 * (1.0 - opacity),
          crate::BlendMode::Screen => {
            (1.0 - (1.0 - val1 / 255.0) * (1.0 - val2 / 255.0)) * 255.0 * opacity
              + val1 * (1.0 - opacity)
          }
          crate::BlendMode::Overlay => {
            if val1 < 128.0 {
              (2.0 * val1 * val2 / 255.0) * opacity + val1 * (1.0 - opacity)
            } else {
              (255.0 - 2.0 * (255.0 - val1) * (255.0 - val2) / 255.0) * opacity
                + val1 * (1.0 - opacity)
            }
          }
          _ => val1,
        };

        let result = blended.clamp(0.0, 255.0) as u8;
        let offset = ((y * width + x) * channels as u32 + c as u32) as usize;
        result_data[offset] = result;
      }
    }
  }

  ImageData::new(width, height, channels, result_data)
}

fn adjust_brightness(image_data: &mut ImageData, brightness: f32) -> Result<()> {
  for pixel in image_data.data.iter_mut() {
    *pixel = (*pixel as f32 + brightness).clamp(0.0, 255.0) as u8;
  }
  Ok(())
}

fn adjust_contrast(image_data: &mut ImageData, contrast: f32) -> Result<()> {
  let factor = (259.0 * (contrast + 255.0)) / (255.0 * (259.0 - contrast));

  for pixel in image_data.data.iter_mut() {
    *pixel = (factor * (*pixel as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
  }
  Ok(())
}

fn adjust_saturation(image_data: &mut ImageData, saturation: f32) -> Result<()> {
  if image_data.channels < 3 {
    return Ok(());
  }

  for pixel in image_data
    .data
    .chunks_exact_mut(image_data.channels as usize)
  {
    let r = pixel[0] as f32;
    let g = pixel[1] as f32;
    let b = pixel[2] as f32;

    let gray = 0.299 * r + 0.587 * g + 0.114 * b;
    let saturation_factor = saturation + 1.0;

    pixel[0] = (gray + saturation_factor * (r - gray)).clamp(0.0, 255.0) as u8;
    pixel[1] = (gray + saturation_factor * (g - gray)).clamp(0.0, 255.0) as u8;
    pixel[2] = (gray + saturation_factor * (b - gray)).clamp(0.0, 255.0) as u8;
  }
  Ok(())
}

fn apply_gamma(image_data: &mut ImageData, gamma: f32) -> Result<()> {
  let gamma = gamma.max(0.1);

  for pixel in image_data.data.iter_mut() {
    *pixel = (255.0 * (*pixel as f32 / 255.0).powf(1.0 / gamma)).clamp(0.0, 255.0) as u8;
  }
  Ok(())
}

fn apply_blur(image_data: &mut ImageData, radius: u32) -> Result<()> {
  let kernel_size = radius * 2 + 1;
  let mut kernel = Vec::with_capacity(kernel_size as usize);

  for i in 0..kernel_size {
    let x = i as f32 - radius as f32;
    let value = (-x * x / (2.0 * radius as f32 * radius as f32)).exp();
    kernel.push(value);
  }

  let sum: f32 = kernel.iter().sum();
  for value in kernel.iter_mut() {
    *value /= sum;
  }

  apply_convolution(image_data, &kernel, kernel_size)?;
  apply_convolution(image_data, &kernel, kernel_size)?;
  Ok(())
}

fn apply_sharpen(image_data: &mut ImageData, amount: u32) -> Result<()> {
  let kernel = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];

  apply_convolution(image_data, &kernel, 3)
}

fn add_noise(image_data: &mut ImageData, intensity: f32) -> Result<()> {
  let mut rng = create_random_generator();

  for pixel in image_data.data.iter_mut() {
    let noise = (rng.gen_range(-1000, 1000) as f32 / 1000.0) * intensity * 255.0;
    *pixel = (*pixel as f32 + noise).clamp(0.0, 255.0) as u8;
  }
  Ok(())
}

fn pixelate(image_data: &mut ImageData, pixel_size: u32) -> Result<()> {
  let width = image_data.width;
  let height = image_data.height;
  let channels = image_data.channels;

  for y in (0..height).step_by(pixel_size as usize) {
    for x in (0..width).step_by(pixel_size as usize) {
      let mut avg = vec![0f32; channels as usize];
      let mut count = 0;

      for dy in 0..pixel_size.min(height - y) {
        for dx in 0..pixel_size.min(width - x) {
          for c in 0..channels {
            if let Some(value) = image_data.get_pixel_value(x + dx, y + dy, c) {
              avg[c as usize] += value as f32;
            }
          }
          count += 1;
        }
      }

      for c in 0..channels {
        avg[c as usize] /= count as f32;
      }

      for dy in 0..pixel_size.min(height - y) {
        for dx in 0..pixel_size.min(width - x) {
          for c in 0..channels {
            image_data.set_pixel_value(x + dx, y + dy, c, avg[c as usize] as u8)?;
          }
        }
      }
    }
  }

  Ok(())
}

pub fn calculate_histogram(image_data: &ImageData) -> Vec<u32> {
  let mut histogram = vec![0u32; 256];

  for &pixel in &image_data.data {
    histogram[pixel as usize] += 1;
  }

  histogram
}

pub fn calculate_channel_histogram(image_data: &ImageData, channel: u8) -> Result<Vec<u32>> {
  if channel >= image_data.channels {
    return Err(EllasticError::InvalidParameter(
      "Channel out of range".to_string(),
    ));
  }

  let mut histogram = vec![0u32; 256];
  let channels = image_data.channels as usize;

  for chunk in image_data.data.chunks_exact(channels) {
    histogram[chunk[channel as usize] as usize] += 1;
  }

  Ok(histogram)
}

pub fn calculate_image_statistics(image_data: &ImageData) -> crate::ImageStatistics {
  let width = image_data.width;
  let height = image_data.height;
  let channels = image_data.channels;

  let mut min = 255u8;
  let mut max = 0u8;
  let mut sum = 0u64;

  for &pixel in &image_data.data {
    min = min.min(pixel);
    max = max.max(pixel);
    sum += pixel as u64;
  }

  let mean = sum as f64 / image_data.data.len() as f64;

  let mut variance = 0.0f64;
  for &pixel in &image_data.data {
    variance += (pixel as f64 - mean).powi(2);
  }
  variance /= image_data.data.len() as f64;
  let std_dev = variance.sqrt();

  let mut sorted_data = image_data.data.clone();
  sorted_data.sort();
  let median = if sorted_data.is_empty() {
    0.0
  } else {
    let mid = sorted_data.len() / 2;
    if sorted_data.len() % 2 == 0 {
      (sorted_data[mid - 1] + sorted_data[mid]) as f64 / 2.0
    } else {
      sorted_data[mid] as f64
    }
  };

  let mode = calculate_histogram(image_data)
    .iter()
    .enumerate()
    .max_by_key(|(_, &count)| count)
    .map(|(index, _)| index as u8)
    .unwrap_or(0);

  let entropy = {
    let mut freq = [0u64; 256];
    for &pixel in &image_data.data {
      freq[pixel as usize] += 1;
    }

    let len = image_data.data.len() as f64;
    let mut entropy = 0.0;

    for &count in &freq {
      if count > 0 {
        let probability = count as f64 / len;
        entropy -= probability * probability.log2();
      }
    }

    entropy
  };

  crate::ImageStatistics {
    width,
    height,
    channels,
    min_value: min,
    max_value: max,
    mean_value: mean,
    median_value: median,
    standard_deviation: std_dev,
    entropy,
  }
}

pub fn calculate_channel_statistics(
  image_data: &ImageData,
  channel: u8,
) -> Result<crate::ChannelStatistics> {
  if channel >= image_data.channels {
    return Err(EllasticError::InvalidParameter(
      "Channel out of range".to_string(),
    ));
  }

  let channels = image_data.channels as usize;
  let mut channel_values = Vec::new();

  for chunk in image_data.data.chunks_exact(channels) {
    channel_values.push(chunk[channel as usize]);
  }

  let min = channel_values.iter().min().copied().unwrap_or(0);
  let max = channel_values.iter().max().copied().unwrap_or(0);
  let sum: u64 = channel_values.iter().map(|&v| v as u64).sum();
  let mean = sum as f64 / channel_values.len() as f64;

  let mut variance = 0.0f64;
  for &value in &channel_values {
    variance += (value as f64 - mean).powi(2);
  }
  variance /= channel_values.len() as f64;
  let std_dev = variance.sqrt();

  let mut sorted_values = channel_values.clone();
  sorted_values.sort();
  let median = if sorted_values.is_empty() {
    0.0
  } else {
    let mid = sorted_values.len() / 2;
    if sorted_values.len() % 2 == 0 {
      (sorted_values[mid - 1] + sorted_values[mid]) as f64 / 2.0
    } else {
      sorted_values[mid] as f64
    }
  };

  let mode = channel_values
    .iter()
    .fold(std::collections::HashMap::new(), |mut map, &value| {
      *map.entry(value).or_insert(0) += 1;
      map
    })
    .iter()
    .max_by_key(|(_, &count)| count)
    .map(|(&value, _)| value)
    .unwrap_or(0);

  let mut histogram = vec![0u32; 256];
  for &value in &channel_values {
    histogram[value as usize] += 1;
  }

  Ok(crate::ChannelStatistics {
    channel,
    min_value: min,
    max_value: max,
    mean_value: mean,
    median_value: median,
    standard_deviation: std_dev,
    histogram,
  })
}
