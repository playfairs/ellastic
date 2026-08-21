use ellastic_core::ImageData;
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_image::{
  ImageFormat,
  ImageProcessor,
};

#[derive(Debug, Clone)]
pub struct PNGCodec {
  compression_level: u8,
}

impl PNGCodec {
  pub fn new(compression_level: u8) -> Self {
    Self { compression_level }
  }

  pub fn compression_level(&self) -> u8 {
    self.compression_level
  }

  pub fn set_compression_level(&mut self, level: u8) {
    self.compression_level = level.clamp(0, 9);
  }

  pub fn encode(&self, image_processor: &ImageProcessor) -> Result<Vec<u8>> {
    image_processor.encode(ImageFormat::PNG, Some(self.compression_level))
  }

  pub fn decode(&self, data: &[u8]) -> Result<ImageProcessor> {
    let image_data = ellastic_image::decode_image(data, ImageFormat::PNG)?;
    Ok(ImageProcessor::from_image_data(image_data))
  }

  pub fn supports_progressive(&self) -> bool {
    true
  }

  pub fn supports_transparency(&self) -> bool {
    true
  }

  pub fn supports_animation(&self) -> bool {
    true
  }

  pub fn get_supported_color_depths(&self) -> Vec<u8> {
    vec![8, 16, 24, 32]
  }

  pub fn estimate_output_size(&self, image_processor: &ImageProcessor) -> usize {
    let image_data = image_processor.data();
    let base_size = image_data.byte_size();

    match self.compression_level {
      0 => base_size,
      1 => (base_size as f32 * 0.9) as usize,
      2 => (base_size as f32 * 0.8) as usize,
      3 => (base_size as f32 * 0.7) as usize,
      4 => (base_size as f32 * 0.6) as usize,
      5 => (base_size as f32 * 0.5) as usize,
      6 => (base_size as f32 * 0.4) as usize,
      7 => (base_size as f32 * 0.3) as usize,
      8 => (base_size as f32 * 0.2) as usize,
      9 => (base_size as f32 * 0.1) as usize,
      _ => base_size,
    }
  }
}

#[derive(Debug, Clone)]
pub struct JPEGCodec {
  quality: u8,
  chroma_subsampling: ChromaSubsampling,
}

impl JPEGCodec {
  pub fn new(quality: u8, chroma_subsampling: ChromaSubsampling) -> Self {
    Self {
      quality,
      chroma_subsampling,
    }
  }

  pub fn quality(&self) -> u8 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: u8) {
    self.quality = quality.clamp(1, 100);
  }

  pub fn chroma_subsampling(&self) -> ChromaSubsampling {
    self.chroma_subsampling
  }

  pub fn set_chroma_subsampling(&mut self, subsampling: ChromaSubsampling) {
    self.chroma_subsampling = subsampling;
  }

  pub fn encode(&self, image_processor: &ImageProcessor) -> Result<Vec<u8>> {
    image_processor.encode(ImageFormat::JPEG, Some(self.quality))
  }

  pub fn decode(&self, data: &[u8]) -> Result<ImageProcessor> {
    let image_data = ellastic_image::decode_image(data, ImageFormat::JPEG)?;
    Ok(ImageProcessor::from_image_data(image_data))
  }

  pub fn supports_progressive(&self) -> bool {
    true
  }

  pub fn supports_transparency(&self) -> bool {
    false
  }

  pub fn supports_animation(&self) -> bool {
    false
  }

  pub fn get_supported_color_depths(&self) -> Vec<u8> {
    vec![8, 12, 16, 24, 32]
  }

  pub fn estimate_output_size(&self, image_processor: &ImageProcessor) -> usize {
    let image_data = image_processor.data();
    let base_size = image_data.byte_size();

    let quality_factor = self.quality as f32 / 100.0;
    let subsampling_factor = match self.chroma_subsampling {
      ChromaSubsampling::None => 1.0,
      ChromaSubsampling::H1V1 => 0.7,
      ChromaSubsampling::H2V1 => 0.5,
      ChromaSubsampling::H2V2 => 0.4,
      ChromaSubsampling::H4V4 => 0.3,
    };

    (base_size as f32 * quality_factor * subsampling_factor) as usize
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaSubsampling {
  None,
  H1V1,
  H2V1,
  H2V2,
  H4V4,
}

#[derive(Debug, Clone)]
pub struct WebPCodec {
  quality: u8,
  method: WebPMethod,
  lossless: bool,
}

impl WebPCodec {
  pub fn new(quality: u8, method: WebPMethod, lossless: bool) -> Self {
    Self {
      quality,
      method,
      lossless,
    }
  }

  pub fn quality(&self) -> u8 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: u8) {
    self.quality = quality.clamp(0, 100);
  }

  pub fn method(&self) -> WebPMethod {
    self.method
  }

  pub fn set_method(&mut self, method: WebPMethod) {
    self.method = method;
  }

  pub fn lossless(&self) -> bool {
    self.lossless
  }

  pub fn set_lossless(&mut self, lossless: bool) {
    self.lossless = lossless;
  }

  pub fn encode(&self, image_processor: &ImageProcessor) -> Result<Vec<u8>> {
    image_processor.encode(ImageFormat::WEBP, Some(self.quality))
  }

  pub fn decode(&self, data: &[u8]) -> Result<ImageProcessor> {
    let image_data = ellastic_image::decode_image(data, ImageFormat::WEBP)?;
    Ok(ImageProcessor::from_image_data(image_data))
  }

  pub fn supports_progressive(&self) -> bool {
    false
  }

  pub fn supports_transparency(&self) -> bool {
    true
  }

  pub fn supports_animation(&self) -> bool {
    true
  }

  pub fn get_supported_color_depths(&self) -> Vec<u8> {
    vec![8, 16, 24, 32]
  }

  pub fn estimate_output_size(&self, image_processor: &ImageProcessor) -> usize {
    let image_data = image_processor.data();
    let base_size = image_data.byte_size();

    if self.lossless {
      base_size
    } else {
      let quality_factor = self.quality as f32 / 100.0;
      let method_factor = match self.method {
        WebPMethod::Lossless => 1.0,
        WebPMethod::Lossy => 0.6,
        WebPMethod::Mixed => 0.8,
      };

      (base_size as f32 * quality_factor * method_factor) as usize
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebPMethod {
  Lossless,
  Lossy,
  Mixed,
}

#[derive(Debug, Clone)]
pub struct BMPCoder {
  compression: BMPCompression,
}

impl BMPCoder {
  pub fn new(compression: BMPCompression) -> Self {
    Self { compression }
  }

  pub fn compression(&self) -> BMPCompression {
    self.compression
  }

  pub fn set_compression(&mut self, compression: BMPCompression) {
    self.compression = compression;
  }

  pub fn encode(&self, image_processor: &ImageProcessor) -> Result<Vec<u8>> {
    image_processor.encode(ImageFormat::BMP, None)
  }

  pub fn decode(&self, data: &[u8]) -> Result<ImageProcessor> {
    let image_data = ellastic_image::decode_image(data, ImageFormat::BMP)?;
    Ok(ImageProcessor::from_image_data(image_data))
  }

  pub fn supports_progressive(&self) -> bool {
    false
  }

  pub fn supports_transparency(&self) -> bool {
    true
  }

  pub fn supports_animation(&self) -> bool {
    false
  }

  pub fn get_supported_color_depths(&self) -> Vec<u8> {
    vec![1, 4, 8, 16, 24, 32]
  }

  pub fn estimate_output_size(&self, image_processor: &ImageProcessor) -> usize {
    let image_data = image_processor.data();
    let base_size = image_data.byte_size();

    match self.compression {
      BMPCompression::None => base_size,
      BMPCompression::RLE8 => (base_size as f32 * 0.5) as usize,
      BMPCompression::RLE4 => (base_size as f32 * 0.3) as usize,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BMPCompression {
  None,
  RLE8,
  RLE4,
}

#[derive(Debug, Clone)]
pub struct GIFCodec {
  dithering: bool,
  color_count: Option<u16>,
  frame_delay: Option<u16>,
}

impl GIFCodec {
  pub fn new(dithering: bool, color_count: Option<u16>, frame_delay: Option<u16>) -> Self {
    Self {
      dithering,
      color_count,
      frame_delay,
    }
  }

  pub fn dithering(&self) -> bool {
    self.dithering
  }

  pub fn set_dithering(&mut self, dithering: bool) {
    self.dithering = dithering;
  }

  pub fn color_count(&self) -> Option<u16> {
    self.color_count
  }

  pub fn set_color_count(&mut self, count: u16) {
    self.color_count = Some(count.clamp(2, 256));
  }

  pub fn frame_delay(&self) -> Option<u16> {
    self.frame_delay
  }

  pub fn set_frame_delay(&mut self, delay: u16) {
    self.frame_delay = Some(delay);
  }

  pub fn encode(&self, image_processor: &ImageProcessor) -> Result<Vec<u8>> {
    image_processor.encode(ImageFormat::GIF, None)
  }

  pub fn decode(&self, data: &[u8]) -> Result<ImageProcessor> {
    let image_data = ellastic_image::decode_image(data, ImageFormat::GIF)?;
    Ok(ImageProcessor::from_image_data(image_data))
  }

  pub fn supports_progressive(&self) -> bool {
    true
  }

  pub fn supports_transparency(&self) -> bool {
    true
  }

  pub fn supports_animation(&self) -> bool {
    true
  }

  pub fn get_supported_color_depths(&self) -> Vec<u8> {
    vec![1, 2, 4, 8]
  }

  pub fn estimate_output_size(&self, image_processor: &ImageProcessor) -> usize {
    let image_data = image_processor.data();
    let base_size = image_data.byte_size();

    let color_factor = match self.color_count {
      Some(count) => {
        if count <= 16 {
          0.5
        } else {
          0.3
        }
      }
      None => 0.5,
    };

    let dithering_factor = if self.dithering { 1.1 } else { 1.0 };

    (base_size as f32 * color_factor * dithering_factor) as usize
  }
}

#[derive(Debug, Clone)]
pub struct TIFFCodec {
  compression: TIFFCompression,
  predictor: TIFFPredictor,
}

impl TIFFCodec {
  pub fn new(compression: TIFFCompression, predictor: TIFFPredictor) -> Self {
    Self {
      compression,
      predictor,
    }
  }

  pub fn compression(&self) -> TIFFCompression {
    self.compression
  }

  pub fn set_compression(&mut self, compression: TIFFCompression) {
    self.compression = compression;
  }

  pub fn predictor(&self) -> TIFFPredictor {
    self.predictor
  }

  pub fn set_predictor(&mut self, predictor: TIFFPredictor) {
    self.predictor = predictor;
  }

  pub fn encode(&self, image_processor: &ImageProcessor) -> Result<Vec<u8>> {
    image_processor.encode(ImageFormat::TIFF, None)
  }

  pub fn decode(&self, data: &[u8]) -> Result<ImageProcessor> {
    let image_data = ellastic_image::decode_image(data, ImageFormat::TIFF)?;
    Ok(ImageProcessor::from_image_data(image_data))
  }

  pub fn supports_progressive(&self) -> bool {
    true
  }

  pub fn supports_transparency(&self) -> bool {
    true
  }

  pub fn supports_animation(&self) -> bool {
    true
  }

  pub fn get_supported_color_depths(&self) -> Vec<u8> {
    vec![1, 4, 8, 16, 24, 32, 64]
  }

  pub fn estimate_output_size(&self, image_processor: &ImageProcessor) -> usize {
    let image_data = image_processor.data();
    let base_size = image_data.byte_size();

    let compression_factor = match self.compression {
      TIFFCompression::None => 1.0,
      TIFFCompression::LZW => 0.6,
      TIFFCompression::Deflate => 0.7,
      TIFFCompression::Packbits => 0.8,
      TIFFCompression::JPEG => 0.4,
    };

    let predictor_factor = match self.predictor {
      TIFFPredictor::None => 1.0,
      TIFFPredictor::Horizontal => 0.9,
      TIFFPredictor::Vertical => 0.9,
      TIFFPredictor::Left => 0.85,
      TIFFPredictor::Upper => 0.85,
    };

    (base_size as f32 * compression_factor * predictor_factor) as usize
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TIFFCompression {
  None,
  LZW,
  Deflate,
  Packbits,
  JPEG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TIFFPredictor {
  None,
  Horizontal,
  Vertical,
  Left,
  Upper,
}

#[derive(Debug, Clone)]
pub struct ImageCodecRegistry {
  codecs: std::collections::HashMap<String, Box<dyn ImageCodec>>,
}

impl ImageCodecRegistry {
  pub fn new() -> Self {
    let mut registry = Self {
      codecs: std::collections::HashMap::new(),
    };

    registry.register_codec("png", Box::new(PNGCodec::new(6)));
    registry.register_codec(
      "jpeg",
      Box::new(JPEGCodec::new(85, ChromaSubsampling::H2V2)),
    );
    registry.register_codec(
      "webp",
      Box::new(WebPCodec::new(80, WebPMethod::Mixed, false)),
    );
    registry.register_codec("bmp", Box::new(BMPCoder::new(BMPCompression::None)));
    registry.register_codec("gif", Box::new(GIFCodec::new(false, Some(256), Some(100))));
    registry.register_codec(
      "tiff",
      Box::new(TIFFCodec::new(
        TIFFCompression::Deflate,
        TIFFPredictor::Horizontal,
      )),
    );

    registry
  }

  pub fn register_codec(&mut self, name: &str, codec: Box<dyn ImageCodec>) {
    self.codecs.insert(name.to_lowercase(), codec);
  }

  pub fn get_codec(&self, name: &str) -> Option<&dyn ImageCodec> {
    self
      .codecs
      .get(&name.to_lowercase())
      .map(|codec| codec.as_ref())
  }

  pub fn list_codecs(&self) -> Vec<String> {
    self.codecs.keys().cloned().collect()
  }

  pub fn get_supported_formats(&self) -> Vec<String> {
    vec!["png", "jpeg", "webp", "bmp", "gif", "tiff"]
  }

  pub fn auto_detect_codec(&self, data: &[u8]) -> Option<String> {
    if data.len() < 8 {
      return None;
    }

    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
      Some("png".to_string())
    } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
      Some("jpeg".to_string())
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
      Some("webp".to_string())
    } else if data.starts_with(b"BM") {
      Some("bmp".to_string())
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
      Some("gif".to_string())
    } else if data.starts_with(b"II*\0") || data.starts_with(b"MM\0*") {
      Some("tiff".to_string())
    } else {
      None
    }
  }

  pub fn create_encoder(
    &self,
    name: &str,
    options: &ImageEncodingOptions,
  ) -> Result<Box<dyn ImageCodec>> {
    match name.to_lowercase().as_str() {
      "png" => Ok(Box::new(PNGCodec::new(
        options.compression_level.unwrap_or(6),
      ))),
      "jpeg" => Ok(Box::new(JPEGCodec::new(
        options.quality.unwrap_or(85),
        options
          .chroma_subsampling
          .unwrap_or(ChromaSubsampling::H2V2),
      ))),
      "webp" => Ok(Box::new(WebPCodec::new(
        options.quality.unwrap_or(80),
        options.webp_method.unwrap_or(WebPMethod::Mixed),
        options.lossless.unwrap_or(false),
      ))),
      "bmp" => Ok(Box::new(BMPCoder::new(
        options.bmp_compression.unwrap_or(BMPCompression::None),
      ))),
      "gif" => Ok(Box::new(GIFCodec::new(
        options.dithering.unwrap_or(false),
        options.color_count,
        options.frame_delay,
      ))),
      "tiff" => Ok(Box::new(TIFFCodec::new(
        options.tiff_compression.unwrap_or(TIFFCompression::Deflate),
        options.tiff_predictor.unwrap_or(TIFFPredictor::Horizontal),
      ))),
      _ => Err(EllasticError::UnsupportedFormat(format!(
        "Unknown image codec: {}",
        name
      ))),
    }
  }

  pub fn create_decoder(&self, name: &str) -> Result<Box<dyn ImageCodec>> {
    match name.to_lowercase().as_str() {
      "png" => Ok(Box::new(PNGCodec::new(6))),
      "jpeg" => Ok(Box::new(JPEGCodec::new(85, ChromaSubsampling::H2V2))),
      "webp" => Ok(Box::new(WebPCodec::new(80, WebPMethod::Mixed, false))),
      "bmp" => Ok(Box::new(BMPCoder::new(BMPCompression::None))),
      "gif" => Ok(Box::new(GIFCodec::new(false, None, None))),
      "tiff" => Ok(Box::new(TIFFCodec::new(
        TIFFCompression::Deflate,
        TIFFPredictor::Horizontal,
      ))),
      _ => Err(EllasticError::UnsupportedFormat(format!(
        "Unknown image codec: {}",
        name
      ))),
    }
  }

  pub fn batch_encode(
    &self,
    images: &[ImageProcessor],
    codec_name: &str,
    options: &ImageEncodingOptions,
  ) -> Result<Vec<Vec<u8>>> {
    let codec = self.create_encoder(codec_name, options)?;
    let mut results = Vec::new();

    for image in images {
      let encoded = codec.encode(image)?;
      results.push(encoded);
    }

    Ok(results)
  }

  pub fn batch_decode(&self, data: &[u8], codec_name: &str) -> Result<Vec<ImageProcessor>> {
    let codec = self.create_decoder(codec_name)?;
    let mut images = Vec::new();

    if codec_name == "gif" {
      let mut offset = 0;
      while offset < data.len() {
        if let Ok(image) = codec.decode(&data[offset..]) {
          images.push(image);
          offset += image.data().byte_size();
        } else {
          break;
        }
      }
    } else {
      let image = codec.decode(data)?;
      images.push(image);
    }

    Ok(images)
  }
}

pub trait ImageCodec {
  fn encode(&self, image_processor: &ImageProcessor) -> Result<Vec<u8>>;
  fn decode(&self, data: &[u8]) -> Result<ImageProcessor>;
  fn supports_progressive(&self) -> bool;
  fn supports_transparency(&self) -> bool;
  fn supports_animation(&self) -> bool;
  fn get_supported_color_depths(&self) -> Vec<u8>;
  fn estimate_output_size(&self, image_processor: &ImageProcessor) -> usize;
}

#[derive(Debug, Clone)]
pub struct ImageEncodingOptions {
  pub quality: Option<u8>,
  pub compression_level: Option<u8>,
  pub chroma_subsampling: Option<ChromaSubsampling>,
  pub webp_method: Option<WebPMethod>,
  pub lossless: Option<bool>,
  pub bmp_compression: Option<BMPCompression>,
  pub dithering: Option<bool>,
  pub color_count: Option<u16>,
  pub frame_delay: Option<u16>,
  pub tiff_compression: Option<TIFFCompression>,
  pub tiff_predictor: Option<TIFFPredictor>,
}

impl Default for ImageEncodingOptions {
  fn default() -> Self {
    Self {
      quality: None,
      compression_level: None,
      chroma_subsampling: None,
      webp_method: None,
      lossless: None,
      bmp_compression: None,
      dithering: None,
      color_count: None,
      frame_delay: None,
      tiff_compression: None,
      tiff_predictor: None,
    }
  }
}

pub fn create_png_codec(compression_level: u8) -> PNGCodec {
  PNGCodec::new(compression_level)
}

pub fn create_jpeg_codec(quality: u8, chroma_subsampling: ChromaSubsampling) -> JPEGCodec {
  JPEGCodec::new(quality, chroma_subsampling)
}

pub fn create_webp_codec(quality: u8, method: WebPMethod, lossless: bool) -> WebPCodec {
  WebPCodec::new(quality, method, lossless)
}

pub fn create_bmp_codec(compression: BMPCompression) -> BMPCoder {
  BMPCoder::new(compression)
}

pub fn create_gif_codec(
  dithering: bool,
  color_count: Option<u16>,
  frame_delay: Option<u16>,
) -> GIFCodec {
  GIFCodec::new(dithering, color_count, frame_delay)
}

pub fn create_tiff_codec(compression: TIFFCompression, predictor: TIFFPredictor) -> TIFFCodec {
  TIFFCodec::new(compression, predictor)
}

pub fn create_image_codec_registry() -> ImageCodecRegistry {
  ImageCodecRegistry::new()
}
