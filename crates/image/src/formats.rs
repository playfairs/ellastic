use crate::{
  ImageData,
  ImageFormat,
  ImageProcessor,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use std::io::{
  Cursor,
  Read,
  Write,
};

pub fn load_png(data: &[u8]) -> Result<ImageData> {
  let decoder = image::codecs::png::PngDecoder::new(Cursor::new(data))
    .map_err(|e| EllasticError::IoError(format!("PNG decoder error: {}", e)))?;

  let (width, height) = decoder.dimensions();
  let color_type = decoder.color_type();
  let bit_depth = decoder.bit_depth();

  let mut reader = decoder.into_reader();
  let mut buffer = vec![0u8; (width * height * color_type.bytes_per_pixel()) as usize];
  reader
    .read_exact(&mut buffer)
    .map_err(|e| EllasticError::IoError(format!("PNG read error: {}", e)))?;

  let channels = match color_type {
    image::ColorType::L8 => 1,
    image::ColorType::La8 => 2,
    image::ColorType::Rgb8 => 3,
    image::ColorType::Rgba8 => 4,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported PNG color type".to_string(),
      ));
    }
  };

  ImageData::new(width, height, channels, buffer)
}

pub fn save_png(image_data: &ImageData, quality: Option<u8>) -> Result<Vec<u8>> {
  let color_type = match image_data.channels {
    1 => image::ColorType::L8,
    2 => image::ColorType::La8,
    3 => image::ColorType::Rgb8,
    4 => image::ColorType::Rgba8,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported channel count for PNG".to_string(),
      ));
    }
  };

  let image_buffer = match image_data.channels {
    1 => image::GrayImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create gray image buffer".to_string())
    })?,
    2 => {
      image::GrayAlphaImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
        .ok_or_else(|| {
          EllasticError::ProcessingError("Failed to create gray-alpha image buffer".to_string())
        })?
    }
    3 => image::RgbImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
        EllasticError::ProcessingError("Failed to create RGB image buffer".to_string())
      })?,
    4 => image::RgbaImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create RGBA image buffer".to_string())
    })?,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported channel count".to_string(),
      ));
    }
  };

  let mut buffer = Vec::new();
  {
    let mut cursor = Cursor::new(&mut buffer);
    image::codecs::png::PngEncoder::new(&mut cursor)
      .encode(
        image_buffer.into_raw(),
        image_data.width,
        image_data.height,
        color_type,
      )
      .map_err(|e| EllasticError::IoError(format!("PNG encode error: {}", e)))?;
  }

  Ok(buffer)
}

pub fn load_jpeg(data: &[u8]) -> Result<ImageData> {
  let decoder = image::codecs::jpeg::JpegDecoder::new(Cursor::new(data))
    .map_err(|e| EllasticError::IoError(format!("JPEG decoder error: {}", e)))?;

  let (width, height) = decoder.dimensions();
  let mut reader = decoder.into_reader();
  let mut buffer = vec![0u8; (width * height * 3) as usize];
  reader
    .read_exact(&mut buffer)
    .map_err(|e| EllasticError::IoError(format!("JPEG read error: {}", e)))?;

  ImageData::new(width, height, 3, buffer)
}

pub fn save_jpeg(image_data: &ImageData, quality: Option<u8>) -> Result<Vec<u8>> {
  let rgb_data = if image_data.channels == 3 {
    image_data.data.clone()
  } else if image_data.channels == 4 {
    let mut rgb = Vec::with_capacity(image_data.data.len() / 4 * 3);
    for chunk in image_data.data.chunks_exact(4) {
      rgb.extend_from_slice(&[chunk[0], chunk[1], chunk[2]]);
    }
    rgb
  } else if image_data.channels == 1 {
    let mut rgb = Vec::with_capacity(image_data.data.len() * 3);
    for &gray in &image_data.data {
      rgb.extend_from_slice(&[gray, gray, gray]);
    }
    rgb
  } else {
    return Err(EllasticError::UnsupportedFormat(
      "Unsupported channel count for JPEG".to_string(),
    ));
  };

  let image_buffer = image::RgbImage::from_raw(image_data.width, image_data.height, rgb_data)
    .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create RGB image buffer".to_string())
    })?;

  let mut buffer = Vec::new();
  {
    let mut cursor = Cursor::new(&mut buffer);
    let mut encoder =
      image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality.unwrap_or(90));
    encoder
      .encode(
        image_buffer.into_raw(),
        image_data.width,
        image_data.height,
        image::ColorType::Rgb8,
      )
      .map_err(|e| EllasticError::IoError(format!("JPEG encode error: {}", e)))?;
  }

  Ok(buffer)
}

pub fn load_bmp(data: &[u8]) -> Result<ImageData> {
  let decoder = image::codecs::bmp::BmpDecoder::new(Cursor::new(data))
    .map_err(|e| EllasticError::IoError(format!("BMP decoder error: {}", e)))?;

  let (width, height) = decoder.dimensions();
  let color_type = decoder.color_type();
  let mut reader = decoder.into_reader();
  let mut buffer = vec![0u8; (width * height * color_type.bytes_per_pixel()) as usize];
  reader
    .read_exact(&mut buffer)
    .map_err(|e| EllasticError::IoError(format!("BMP read error: {}", e)))?;

  let channels = match color_type {
    image::ColorType::L8 => 1,
    image::ColorType::Rgb8 => 3,
    image::ColorType::Rgba8 => 4,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported BMP color type".to_string(),
      ));
    }
  };

  ImageData::new(width, height, channels, buffer)
}

pub fn save_bmp(image_data: &ImageData, _quality: Option<u8>) -> Result<Vec<u8>> {
  let color_type = match image_data.channels {
    1 => image::ColorType::L8,
    3 => image::ColorType::Rgb8,
    4 => image::ColorType::Rgba8,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported channel count for BMP".to_string(),
      ));
    }
  };

  let image_buffer = match image_data.channels {
    1 => image::GrayImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create gray image buffer".to_string())
    })?,
    3 => image::RgbImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
        EllasticError::ProcessingError("Failed to create RGB image buffer".to_string())
      })?,
    4 => image::RgbaImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create RGBA image buffer".to_string())
    })?,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported channel count".to_string(),
      ));
    }
  };

  let mut buffer = Vec::new();
  {
    let mut cursor = Cursor::new(&mut buffer);
    image_buffer
      .write_to(&mut cursor, image::ImageFormat::Bmp)
      .map_err(|e| EllasticError::IoError(format!("BMP encode error: {}", e)))?;
  }

  Ok(buffer)
}

pub fn load_gif(data: &[u8]) -> Result<ImageData> {
  let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(data))
    .map_err(|e| EllasticError::IoError(format!("GIF decoder error: {}", e)))?;

  let (width, height) = decoder.dimensions();
  let first_frame = decoder
    .into_frames()
    .next()
    .ok_or_else(|| EllasticError::ProcessingError("No frames in GIF".to_string()))?;

  let frame = first_frame.into_buffer();
  let channels = match frame.color() {
    image::ColorType::L8 => 1,
    image::ColorType::La8 => 2,
    image::ColorType::Rgb8 => 3,
    image::ColorType::Rgba8 => 4,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported GIF color type".to_string(),
      ));
    }
  };

  ImageData::new(width, height, channels, frame.into_raw())
}

pub fn save_gif(image_data: &ImageData, _quality: Option<u8>) -> Result<Vec<u8>> {
  let color_type = match image_data.channels {
    1 => image::ColorType::L8,
    3 => image::ColorType::Rgb8,
    4 => image::ColorType::Rgba8,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported channel count for GIF".to_string(),
      ));
    }
  };

  let image_buffer = match image_data.channels {
    1 => image::GrayImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create gray image buffer".to_string())
    })?,
    3 => image::RgbImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
        EllasticError::ProcessingError("Failed to create RGB image buffer".to_string())
      })?,
    4 => image::RgbaImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create RGBA image buffer".to_string())
    })?,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported channel count".to_string(),
      ));
    }
  };

  let mut buffer = Vec::new();
  {
    let mut cursor = Cursor::new(&mut buffer);
    image_buffer
      .write_to(&mut cursor, image::ImageFormat::Gif)
      .map_err(|e| EllasticError::IoError(format!("GIF encode error: {}", e)))?;
  }

  Ok(buffer)
}

pub fn load_tiff(data: &[u8]) -> Result<ImageData> {
  let decoder = image::codecs::tiff::TiffDecoder::new(Cursor::new(data))
    .map_err(|e| EllasticError::IoError(format!("TIFF decoder error: {}", e)))?;

  let (width, height) = decoder.dimensions();
  let color_type = decoder.color_type();
  let mut reader = decoder.into_reader();
  let mut buffer = vec![0u8; (width * height * color_type.bytes_per_pixel()) as usize];
  reader
    .read_exact(&mut buffer)
    .map_err(|e| EllasticError::IoError(format!("TIFF read error: {}", e)))?;

  let channels = match color_type {
    image::ColorType::L8 => 1,
    image::ColorType::La8 => 2,
    image::ColorType::Rgb8 => 3,
    image::ColorType::Rgba8 => 4,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported TIFF color type".to_string(),
      ));
    }
  };

  ImageData::new(width, height, channels, buffer)
}

pub fn save_tiff(image_data: &ImageData, _quality: Option<u8>) -> Result<Vec<u8>> {
  let color_type = match image_data.channels {
    1 => image::ColorType::L8,
    2 => image::ColorType::La8,
    3 => image::ColorType::Rgb8,
    4 => image::ColorType::Rgba8,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported channel count for TIFF".to_string(),
      ));
    }
  };

  let image_buffer = match image_data.channels {
    1 => image::GrayImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create gray image buffer".to_string())
    })?,
    2 => {
      image::GrayAlphaImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
        .ok_or_else(|| {
          EllasticError::ProcessingError("Failed to create gray-alpha image buffer".to_string())
        })?
    }
    3 => image::RgbImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
        EllasticError::ProcessingError("Failed to create RGB image buffer".to_string())
      })?,
    4 => image::RgbaImage::from_raw(image_data.width, image_data.height, image_data.data.clone())
      .ok_or_else(|| {
      EllasticError::ProcessingError("Failed to create RGBA image buffer".to_string())
    })?,
    _ => {
      return Err(EllasticError::UnsupportedFormat(
        "Unsupported channel count".to_string(),
      ));
    }
  };

  let mut buffer = Vec::new();
  {
    let mut cursor = Cursor::new(&mut buffer);
    image_buffer
      .write_to(&mut cursor, image::ImageFormat::Tiff)
      .map_err(|e| EllasticError::IoError(format!("TIFF encode error: {}", e)))?;
  }

  Ok(buffer)
}

pub fn load_ppm(data: &[u8]) -> Result<ImageData> {
  let content = std::str::from_utf8(data)
    .map_err(|_| EllasticError::IoError("Invalid UTF-8 in PPM".to_string()))?;

  let mut lines = content.lines();
  let header = lines
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing PPM header".to_string()))?;

  if !header.starts_with("P6") {
    return Err(EllasticError::UnsupportedFormat(
      "Only P6 PPM format supported".to_string(),
    ));
  }

  let dimensions = lines
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing PPM dimensions".to_string()))?;
  let mut parts = dimensions.split_whitespace();
  let width: u32 = parts
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing width".to_string()))?
    .parse()
    .map_err(|_| EllasticError::IoError("Invalid width".to_string()))?;
  let height: u32 = parts
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing height".to_string()))?
    .parse()
    .map_err(|_| EllasticError::IoError("Invalid height".to_string()))?;

  let max_val = lines
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing max value".to_string()))?;
  let _max_val: u32 = max_val
    .parse()
    .map_err(|_| EllasticError::IoError("Invalid max value".to_string()))?;

  let mut binary_data = Vec::new();
  for line in lines {
    binary_data.extend_from_slice(line.as_bytes());
  }

  let expected_size = (width * height * 3) as usize;
  if binary_data.len() < expected_size {
    return Err(EllasticError::IoError(
      "Insufficient binary data".to_string(),
    ));
  }

  ImageData::new(width, height, 3, binary_data[..expected_size].to_vec())
}

pub fn save_ppm(image_data: &ImageData, _quality: Option<u8>) -> Result<Vec<u8>> {
  if image_data.channels != 3 {
    return Err(EllasticError::UnsupportedFormat(
      "PPM only supports RGB".to_string(),
    ));
  }

  let mut buffer = Vec::new();
  buffer.extend_from_slice(b"P6\n");
  buffer.extend_from_slice(format!("{} {}\n255\n", image_data.width, image_data.height).as_bytes());
  buffer.extend_from_slice(&image_data.data);

  Ok(buffer)
}

pub fn load_pgm(data: &[u8]) -> Result<ImageData> {
  let content = std::str::from_utf8(data)
    .map_err(|_| EllasticError::IoError("Invalid UTF-8 in PGM".to_string()))?;

  let mut lines = content.lines();
  let header = lines
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing PGM header".to_string()))?;

  if !header.starts_with("P5") {
    return Err(EllasticError::UnsupportedFormat(
      "Only P5 PGM format supported".to_string(),
    ));
  }

  let dimensions = lines
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing PGM dimensions".to_string()))?;
  let mut parts = dimensions.split_whitespace();
  let width: u32 = parts
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing width".to_string()))?
    .parse()
    .map_err(|_| EllasticError::IoError("Invalid width".to_string()))?;
  let height: u32 = parts
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing height".to_string()))?
    .parse()
    .map_err(|_| EllasticError::IoError("Invalid height".to_string()))?;

  let max_val = lines
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing max value".to_string()))?;
  let _max_val: u32 = max_val
    .parse()
    .map_err(|_| EllasticError::IoError("Invalid max value".to_string()))?;

  let mut binary_data = Vec::new();
  for line in lines {
    binary_data.extend_from_slice(line.as_bytes());
  }

  let expected_size = (width * height) as usize;
  if binary_data.len() < expected_size {
    return Err(EllasticError::IoError(
      "Insufficient binary data".to_string(),
    ));
  }

  ImageData::new(width, height, 1, binary_data[..expected_size].to_vec())
}

pub fn save_pgm(image_data: &ImageData, _quality: Option<u8>) -> Result<Vec<u8>> {
  if image_data.channels != 1 {
    return Err(EllasticError::UnsupportedFormat(
      "PGM only supports grayscale".to_string(),
    ));
  }

  let mut buffer = Vec::new();
  buffer.extend_from_slice(b"P5\n");
  buffer.extend_from_slice(format!("{} {}\n255\n", image_data.width, image_data.height).as_bytes());
  buffer.extend_from_slice(&image_data.data);

  Ok(buffer)
}

pub fn load_pbm(data: &[u8]) -> Result<ImageData> {
  let content = std::str::from_utf8(data)
    .map_err(|_| EllasticError::IoError("Invalid UTF-8 in PBM".to_string()))?;

  let mut lines = content.lines();
  let header = lines
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing PBM header".to_string()))?;

  if !header.starts_with("P4") {
    return Err(EllasticError::UnsupportedFormat(
      "Only P4 PBM format supported".to_string(),
    ));
  }

  let dimensions = lines
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing PBM dimensions".to_string()))?;
  let mut parts = dimensions.split_whitespace();
  let width: u32 = parts
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing width".to_string()))?
    .parse()
    .map_err(|_| EllasticError::IoError("Invalid width".to_string()))?;
  let height: u32 = parts
    .next()
    .ok_or_else(|| EllasticError::IoError("Missing height".to_string()))?
    .parse()
    .map_err(|_| EllasticError::IoError("Invalid height".to_string()))?;

  let mut binary_data = Vec::new();
  for line in lines {
    binary_data.extend_from_slice(line.as_bytes());
  }

  let expected_size = ((width * height + 7) / 8) as usize;
  if binary_data.len() < expected_size {
    return Err(EllasticError::IoError(
      "Insufficient binary data".to_string(),
    ));
  }

  let mut grayscale_data = Vec::with_capacity((width * height) as usize);
  for &byte in &binary_data[..expected_size] {
    for i in 0..8 {
      if grayscale_data.len() < (width * height) as usize {
        grayscale_data.push(if (byte >> (7 - i)) & 1 == 1 { 255 } else { 0 });
      }
    }
  }

  ImageData::new(width, height, 1, grayscale_data)
}

pub fn save_pbm(image_data: &ImageData, _quality: Option<u8>) -> Result<Vec<u8>> {
  if image_data.channels != 1 {
    return Err(EllasticError::UnsupportedFormat(
      "PBM only supports grayscale".to_string(),
    ));
  }

  let mut buffer = Vec::new();
  buffer.extend_from_slice(b"P4\n");
  buffer.extend_from_slice(format!("{} {}\n", image_data.width, image_data.height).as_bytes());

  for chunk in image_data.data.chunks(8) {
    let mut byte = 0u8;
    for (i, &pixel) in chunk.iter().enumerate() {
      if i < 8 && pixel > 127 {
        byte |= 1 << (7 - i);
      }
    }
    buffer.push(byte);
  }

  Ok(buffer)
}

pub fn get_format_info(format: ImageFormat) -> FormatInfo {
  match format {
    ImageFormat::PNG => FormatInfo {
      name: "PNG",
      supports_transparency: true,
      supports_animation: false,
      max_channels: 4,
      supports_lossless: true,
      supports_lossy: false,
    },
    ImageFormat::JPEG => FormatInfo {
      name: "JPEG",
      supports_transparency: false,
      supports_animation: false,
      max_channels: 3,
      supports_lossless: false,
      supports_lossy: true,
    },
    ImageFormat::BMP => FormatInfo {
      name: "BMP",
      supports_transparency: true,
      supports_animation: false,
      max_channels: 4,
      supports_lossless: true,
      supports_lossy: false,
    },
    ImageFormat::GIF => FormatInfo {
      name: "GIF",
      supports_transparency: true,
      supports_animation: true,
      max_channels: 3,
      supports_lossless: true,
      supports_lossy: false,
    },
    ImageFormat::TIFF => FormatInfo {
      name: "TIFF",
      supports_transparency: true,
      supports_animation: false,
      max_channels: 4,
      supports_lossless: true,
      supports_lossy: true,
    },
    ImageFormat::WEBP => FormatInfo {
      name: "WEBP",
      supports_transparency: true,
      supports_animation: true,
      max_channels: 4,
      supports_lossless: true,
      supports_lossy: true,
    },
    ImageFormat::PPM => FormatInfo {
      name: "PPM",
      supports_transparency: false,
      supports_animation: false,
      max_channels: 3,
      supports_lossless: true,
      supports_lossy: false,
    },
    ImageFormat::PGM => FormatInfo {
      name: "PGM",
      supports_transparency: false,
      supports_animation: false,
      max_channels: 1,
      supports_lossless: true,
      supports_lossy: false,
    },
    ImageFormat::PBM => FormatInfo {
      name: "PBM",
      supports_transparency: false,
      supports_animation: false,
      max_channels: 1,
      supports_lossless: true,
      supports_lossy: false,
    },
  }
}

#[derive(Debug, Clone)]
pub struct FormatInfo {
  pub name: &'static str,
  pub supports_transparency: bool,
  pub supports_animation: bool,
  pub max_channels: u8,
  pub supports_lossless: bool,
  pub supports_lossy: bool,
}

pub fn get_supported_formats() -> Vec<ImageFormat> {
  vec![
    ImageFormat::PNG,
    ImageFormat::JPEG,
    ImageFormat::BMP,
    ImageFormat::GIF,
    ImageFormat::TIFF,
    ImageFormat::WEBP,
    ImageFormat::PPM,
    ImageFormat::PGM,
    ImageFormat::PBM,
  ]
}

pub fn get_formats_with_transparency() -> Vec<ImageFormat> {
  get_supported_formats()
    .into_iter()
    .filter(|format| get_format_info(*format).supports_transparency)
    .collect()
}

pub fn get_formats_with_animation() -> Vec<ImageFormat> {
  get_supported_formats()
    .into_iter()
    .filter(|format| get_format_info(*format).supports_animation)
    .collect()
}

pub fn get_formats_with_lossless() -> Vec<ImageFormat> {
  get_supported_formats()
    .into_iter()
    .filter(|format| get_format_info(*format).supports_lossless)
    .collect()
}

pub fn get_formats_with_lossy() -> Vec<ImageFormat> {
  get_supported_formats()
    .into_iter()
    .filter(|format| get_format_info(*format).supports_lossy)
    .collect()
}
