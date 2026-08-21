use ellastic_core::{
  ImageData,
  MediaData,
  MediaType,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use std::path::Path;

pub mod analysis;
pub mod corruption;
pub mod effects;
pub mod formats;
pub mod processing;

pub use analysis::*;
pub use corruption::*;
pub use effects::*;
pub use formats::*;
pub use processing::*;

#[derive(Debug, Clone)]
pub struct ImageProcessor {
  data: ImageData,
}

impl ImageProcessor {
  pub fn new(width: u32, height: u32, channels: u8, data: Vec<u8>) -> Result<Self> {
    let image_data = ImageData::new(width, height, channels, data)?;
    Ok(Self { data: image_data })
  }

  pub fn from_image_data(data: ImageData) -> Self {
    Self { data }
  }

  pub fn from_buffer(buffer: Vec<u8>, format: ImageFormat) -> Result<Self> {
    let image_data = decode_image(&buffer, format)?;
    Ok(Self { data: image_data })
  }

  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    let buffer = std::fs::read(path)
      .map_err(|e| EllasticError::IoError(format!("Failed to read image file: {}", e)))?;

    let format = detect_format(&buffer)?;
    Self::from_buffer(buffer, format)
  }

  pub fn data(&self) -> &ImageData {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut ImageData {
    &mut self.data
  }

  pub fn into_data(self) -> ImageData {
    self.data
  }

  pub fn width(&self) -> u32 {
    self.data.width
  }

  pub fn height(&self) -> u32 {
    self.data.height
  }

  pub fn channels(&self) -> u8 {
    self.data.channels
  }

  pub fn pixel_count(&self) -> usize {
    self.data.pixel_count()
  }

  pub fn byte_size(&self) -> usize {
    self.data.byte_size()
  }

  pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
    self.data.get_pixel(x, y)
  }

  pub fn set_pixel(&mut self, x: u32, y: u32, pixel: &[u8]) -> Result<()> {
    self.data.set_pixel(x, y, pixel)
  }

  pub fn get_pixel_value(&self, x: u32, y: u32, channel: u8) -> Option<u8> {
    self
      .get_pixel(x, y)
      .and_then(|pixel| pixel.get(channel as usize).copied())
  }

  pub fn set_pixel_value(&mut self, x: u32, y: u32, channel: u8, value: u8) -> Result<()> {
    if let Some(pixel) = self.get_pixel(x, y) {
      let mut new_pixel = pixel.to_vec();
      if let Some(target) = new_pixel.get_mut(channel as usize) {
        *target = value;
        return self.set_pixel(x, y, &new_pixel);
      }
    }
    Err(EllasticError::CoordinatesOutOfBounds { x, y })
  }

  pub fn get_rgb_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8)> {
    self.get_pixel(x, y).and_then(|pixel| {
      if pixel.len() >= 3 {
        Some((pixel[0], pixel[1], pixel[2]))
      } else {
        None
      }
    })
  }

  pub fn set_rgb_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) -> Result<()> {
    let pixel = match self.channels {
      1 => vec![r],
      2 => vec![r, g],
      3 => vec![r, g, b],
      4 => vec![r, g, b, 255],
      _ => {
        return Err(EllasticError::InvalidPixelSize {
          expected: self.channels as usize,
          actual: 3,
        });
      }
    };
    self.set_pixel(x, y, &pixel)
  }

  pub fn get_rgba_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
    self.get_pixel(x, y).and_then(|pixel| {
      if pixel.len() >= 4 {
        Some((pixel[0], pixel[1], pixel[2], pixel[3]))
      } else if pixel.len() == 3 {
        Some((pixel[0], pixel[1], pixel[2], 255))
      } else {
        None
      }
    })
  }

  pub fn set_rgba_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) -> Result<()> {
    let pixel = match self.channels {
      1 => vec![r],
      2 => vec![r, g],
      3 => vec![r, g, b],
      4 => vec![r, g, b, a],
      _ => {
        return Err(EllasticError::InvalidPixelSize {
          expected: self.channels as usize,
          actual: 4,
        });
      }
    };
    self.set_pixel(x, y, &pixel)
  }

  pub fn get_grayscale_pixel(&self, x: u32, y: u32) -> Option<u8> {
    self
      .get_pixel(x, y)
      .and_then(|pixel| pixel.first().copied())
  }

  pub fn set_grayscale_pixel(&mut self, x: u32, y: u32, value: u8) -> Result<()> {
    let pixel = vec![value; self.channels as usize];
    self.set_pixel(x, y, &pixel)
  }

  pub fn get_row(&self, y: u32) -> Option<&[u8]> {
    if y >= self.height {
      return None;
    }

    let start = (y * self.width * self.channels as u32) as usize;
    let end = start + (self.width * self.channels as u32) as usize;
    self.data.data.get(start..end)
  }

  pub fn get_row_mut(&mut self, y: u32) -> Option<&mut [u8]> {
    if y >= self.height {
      return None;
    }

    let start = (y * self.width * self.channels as u32) as usize;
    let end = start + (self.width * self.channels as u32) as usize;
    self.data.data.get_mut(start..end)
  }

  pub fn get_column(&self, x: u32) -> Vec<u8> {
    let mut column = Vec::with_capacity(self.height as usize);

    for y in 0..self.height {
      if let Some(pixel) = self.get_pixel(x, y) {
        column.extend_from_slice(pixel);
      }
    }

    column
  }

  pub fn set_column(&mut self, x: u32, column_data: &[u8]) -> Result<()> {
    if column_data.len() != self.height as usize * self.channels as usize {
      return Err(EllasticError::InvalidParameter(
        "Column data size mismatch".to_string(),
      ));
    }

    for y in 0..self.height {
      let start = (y * self.channels as usize) as usize;
      let end = start + self.channels as usize;
      let pixel = &column_data[start..end];
      self.set_pixel(x, y, pixel)?;
    }

    Ok(())
  }

  pub fn get_channel(&self, channel: u8) -> Result<Vec<u8>> {
    if channel >= self.channels {
      return Err(EllasticError::InvalidParameter(
        "Channel out of range".to_string(),
      ));
    }

    let mut channel_data = Vec::with_capacity(self.pixel_count());
    for y in 0..self.height {
      for x in 0..self.width {
        if let Some(value) = self.get_pixel_value(x, y, channel) {
          channel_data.push(value);
        }
      }
    }

    Ok(channel_data)
  }

  pub fn set_channel(&mut self, channel: u8, data: &[u8]) -> Result<()> {
    if channel >= self.channels {
      return Err(EllasticError::InvalidParameter(
        "Channel out of range".to_string(),
      ));
    }

    if data.len() != self.pixel_count() {
      return Err(EllasticError::InvalidParameter(
        "Channel data size mismatch".to_string(),
      ));
    }

    let mut index = 0;
    for y in 0..self.height {
      for x in 0..self.width {
        self.set_pixel_value(x, y, channel, data[index])?;
        index += 1;
      }
    }

    Ok(())
  }

  pub fn extract_channel(&mut self, channel: u8) -> Result<ImageProcessor> {
    let channel_data = self.get_channel(channel)?;
    let mut new_data = vec![0u8; self.pixel_count()];

    for (i, &value) in channel_data.iter().enumerate() {
      new_data[i] = value;
    }

    ImageProcessor::new(self.width, self.height, 1, new_data)
  }

  pub fn combine_channels(&self, channels: &[ImageProcessor]) -> Result<ImageProcessor> {
    if channels.is_empty() {
      return Err(EllasticError::InvalidParameter(
        "No channels to combine".to_string(),
      ));
    }

    let first_channel = &channels[0];
    if first_channel.width != self.width || first_channel.height != self.height {
      return Err(EllasticError::InvalidParameter(
        "Channel size mismatch".to_string(),
      ));
    }

    let total_channels = channels.len();
    let mut combined_data = Vec::with_capacity(self.pixel_count() * total_channels);

    for y in 0..self.height {
      for x in 0..self.width {
        for channel in channels {
          if let Some(value) = channel.get_grayscale_pixel(x, y) {
            combined_data.push(value);
          }
        }
      }
    }

    ImageProcessor::new(self.width, self.height, total_channels as u8, combined_data)
  }

  pub fn resize(
    &self,
    new_width: u32,
    new_height: u32,
    filter: ResizeFilter,
  ) -> Result<ImageProcessor> {
    let resized_data = resize_image(&self.data, new_width, new_height, filter)?;
    Ok(ImageProcessor::from_image_data(resized_data))
  }

  pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Result<ImageProcessor> {
    let cropped_data = crop_image(&self.data, x, y, width, height)?;
    Ok(ImageProcessor::from_image_data(cropped_data))
  }

  pub fn flip_horizontal(&mut self) -> Result<()> {
    flip_image_horizontal(&mut self.data)
  }

  pub fn flip_vertical(&mut self) -> Result<()> {
    flip_image_vertical(&mut self.data)
  }

  pub fn rotate_90(&self) -> Result<ImageProcessor> {
    let rotated_data = rotate_image_90(&self.data)?;
    Ok(ImageProcessor::from_image_data(rotated_data))
  }

  pub fn rotate_180(&self) -> Result<ImageProcessor> {
    let rotated_data = rotate_image_180(&self.data)?;
    Ok(ImageProcessor::from_image_data(rotated_data))
  }

  pub fn rotate_270(&self) -> Result<ImageProcessor> {
    let rotated_data = rotate_image_270(&self.data)?;
    Ok(ImageProcessor::from_image_data(rotated_data))
  }

  pub fn convert_colorspace(&self, from: ColorSpace, to: ColorSpace) -> Result<ImageProcessor> {
    let converted_data = convert_colorspace(&self.data, from, to)?;
    Ok(ImageProcessor::from_image_data(converted_data))
  }

  pub fn to_grayscale(&self) -> Result<ImageProcessor> {
    let grayscale_data = convert_to_grayscale(&self.data)?;
    Ok(ImageProcessor::from_image_data(grayscale_data))
  }

  pub fn to_rgb(&self) -> Result<ImageProcessor> {
    let rgb_data = convert_to_rgb(&self.data)?;
    Ok(ImageProcessor::from_image_data(rgb_data))
  }

  pub fn to_rgba(&self) -> Result<ImageProcessor> {
    let rgba_data = convert_to_rgba(&self.data)?;
    Ok(ImageProcessor::from_image_data(rgba_data))
  }

  pub fn apply_filter(&mut self, filter: &ImageFilter) -> Result<()> {
    apply_image_filter(&mut self.data, filter)
  }

  pub fn apply_convolution(&mut self, kernel: &[f32], kernel_size: u32) -> Result<()> {
    apply_convolution(&mut self.data, kernel, kernel_size)
  }

  pub fn blend_with(
    &self,
    other: &ImageProcessor,
    mode: BlendMode,
    opacity: f32,
  ) -> Result<ImageProcessor> {
    let blended_data = blend_images(&self.data, &other.data, mode, opacity)?;
    Ok(ImageProcessor::from_image_data(blended_data))
  }

  pub fn histogram(&self) -> Vec<u32> {
    calculate_histogram(&self.data)
  }

  pub fn channel_histogram(&self, channel: u8) -> Result<Vec<u32>> {
    if channel >= self.channels {
      return Err(EllasticError::InvalidParameter(
        "Channel out of range".to_string(),
      ));
    }
    Ok(calculate_channel_histogram(&self.data, channel))
  }

  pub fn calculate_statistics(&self) -> ImageStatistics {
    calculate_image_statistics(&self.data)
  }

  pub fn calculate_channel_statistics(&self, channel: u8) -> Result<ChannelStatistics> {
    if channel >= self.channels {
      return Err(EllasticError::InvalidParameter(
        "Channel out of range".to_string(),
      ));
    }
    Ok(calculate_channel_statistics(&self.data, channel))
  }

  pub fn encode(&self, format: ImageFormat, quality: Option<u8>) -> Result<Vec<u8>> {
    encode_image(&self.data, format, quality)
  }

  pub fn save<P: AsRef<Path>>(
    &self,
    path: P,
    format: ImageFormat,
    quality: Option<u8>,
  ) -> Result<()> {
    let encoded = self.encode(format, quality)?;
    std::fs::write(path, encoded)
      .map_err(|e| EllasticError::IoError(format!("Failed to save image: {}", e)))
  }

  pub fn clone(&self) -> ImageProcessor {
    ImageProcessor {
      data: ImageData::new(
        self.width,
        self.height,
        self.channels,
        self.data.data.clone(),
      )
      .unwrap(),
    }
  }

  pub fn to_media_data(self) -> MediaData {
    MediaData::Image(self.data)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
  PNG,
  JPEG,
  BMP,
  GIF,
  TIFF,
  WEBP,
  PPM,
  PGM,
  PBM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
  RGB,
  RGBA,
  GRAYSCALE,
  HSL,
  HSV,
  LAB,
  CMYK,
  YUV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeFilter {
  Nearest,
  Linear,
  Cubic,
  Lanczos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
  Normal,
  Multiply,
  Screen,
  Overlay,
  SoftLight,
  HardLight,
  ColorDodge,
  ColorBurn,
  Darken,
  Lighten,
  Difference,
  Exclusion,
}

#[derive(Debug, Clone)]
pub struct ImageFilter {
  pub filter_type: FilterType,
  pub parameters: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum FilterType {
  Brightness,
  Contrast,
  Saturation,
  Hue,
  Gamma,
  Blur,
  Sharpen,
  EdgeDetect,
  Emboss,
  Noise,
  Pixelate,
}

#[derive(Debug, Clone)]
pub struct ImageStatistics {
  pub width: u32,
  pub height: u32,
  pub channels: u8,
  pub min_value: u8,
  pub max_value: u8,
  pub mean_value: f64,
  pub median_value: f64,
  pub standard_deviation: f64,
  pub entropy: f64,
}

#[derive(Debug, Clone)]
pub struct ChannelStatistics {
  pub channel: u8,
  pub min_value: u8,
  pub max_value: u8,
  pub mean_value: f64,
  pub median_value: f64,
  pub standard_deviation: f64,
  pub histogram: Vec<u32>,
}

pub fn detect_format(buffer: &[u8]) -> Result<ImageFormat> {
  if buffer.len() < 8 {
    return Err(EllasticError::UnsupportedFormat(
      "Insufficient data".to_string(),
    ));
  }

  if buffer.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
    Ok(ImageFormat::PNG)
  } else if buffer.starts_with(&[0xFF, 0xD8, 0xFF]) {
    Ok(ImageFormat::JPEG)
  } else if buffer.starts_with(&[0x42, 0x4D]) {
    Ok(ImageFormat::BMP)
  } else if buffer.starts_with(b"GIF87a") || buffer.starts_with(b"GIF89a") {
    Ok(ImageFormat::GIF)
  } else if buffer.starts_with(&[0x49, 0x49, 0x2A, 0x00])
    || buffer.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
  {
    Ok(ImageFormat::TIFF)
  } else if buffer.starts_with(b"RIFF") && buffer.len() > 8 && buffer[8..12] == b"WEBP" {
    Ok(ImageFormat::WEBP)
  } else if buffer.starts_with(b"P1") || buffer.starts_with(b"P2") || buffer.starts_with(b"P3") {
    Ok(ImageFormat::PPM)
  } else if buffer.starts_with(b"P2") || buffer.starts_with(b"P5") {
    Ok(ImageFormat::PGM)
  } else if buffer.starts_with(b"P1") || buffer.starts_with(b"P4") {
    Ok(ImageFormat::PBM)
  } else {
    Err(EllasticError::UnsupportedFormat(
      "Unknown image format".to_string(),
    ))
  }
}

pub fn create_image_processor(
  width: u32,
  height: u32,
  channels: u8,
  data: Vec<u8>,
) -> Result<ImageProcessor> {
  ImageProcessor::new(width, height, channels, data)
}

pub fn load_image<P: AsRef<Path>>(path: P) -> Result<ImageProcessor> {
  ImageProcessor::from_file(path)
}

pub fn create_blank_image(
  width: u32,
  height: u32,
  channels: u8,
  fill_value: u8,
) -> Result<ImageProcessor> {
  let data = vec![fill_value; (width * height * channels as u32) as usize];
  ImageProcessor::new(width, height, channels, data)
}

pub fn create_gradient_image(
  width: u32,
  height: u32,
  channels: u8,
  start_color: &[u8],
  end_color: &[u8],
) -> Result<ImageProcessor> {
  if start_color.len() != channels as usize || end_color.len() != channels as usize {
    return Err(EllasticError::InvalidParameter(
      "Color size mismatch".to_string(),
    ));
  }

  let mut data = Vec::with_capacity((width * height * channels as u32) as usize);

  for y in 0..height {
    for x in 0..width {
      let t = (x + y) as f32 / (width + height - 2) as f32;
      for c in 0..channels {
        let start = start_color[c as usize] as f32;
        let end = end_color[c as usize] as f32;
        let value = start + t * (end - start);
        data.push(value.clamp(0.0, 255.0) as u8);
      }
    }
  }

  ImageProcessor::new(width, height, channels, data)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_image_processor_creation() {
    let processor = create_image_processor(100, 100, 3, vec![255u8; 30000]).unwrap();
    assert_eq!(processor.width(), 100);
    assert_eq!(processor.height(), 100);
    assert_eq!(processor.channels(), 3);
    assert_eq!(processor.pixel_count(), 10000);
  }

  #[test]
  fn test_pixel_operations() {
    let mut processor = create_blank_image(10, 10, 3, 0).unwrap();

    processor.set_rgb_pixel(5, 5, 255, 0, 0).unwrap();
    assert_eq!(processor.get_rgb_pixel(5, 5), Some((255, 0, 0)));

    processor.set_pixel_value(5, 5, 1, 128).unwrap();
    assert_eq!(processor.get_pixel_value(5, 5, 1), Some(128));
  }

  #[test]
  fn test_channel_operations() {
    let mut processor = create_blank_image(10, 10, 3, 0).unwrap();

    for y in 0..10 {
      for x in 0..10 {
        processor.set_rgb_pixel(x, y, x as u8, y as u8, 0).unwrap();
      }
    }

    let red_channel = processor.get_channel(0).unwrap();
    assert_eq!(red_channel.len(), 100);
  }

  #[test]
  fn test_format_detection() {
    let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(detect_format(&png_header).unwrap(), ImageFormat::PNG);

    let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0];
    assert_eq!(detect_format(&jpeg_header).unwrap(), ImageFormat::JPEG);
  }
}
