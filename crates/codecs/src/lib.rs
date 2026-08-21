use ellastic_audio::{
  AudioFormat,
  AudioProcessor,
};
use ellastic_core::{
  MediaData,
  MediaType,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_image::{
  ImageFormat,
  ImageProcessor,
};
use ellastic_media::{
  MediaProcessor,
  VideoFormat,
  VideoProcessor,
};
use std::path::Path;

pub mod audio_codecs;
pub mod codec_registry;
pub mod compression;
pub mod decompression;
pub mod image_codecs;
pub mod transcoding;
pub mod video_codecs;

pub use audio_codecs::*;
pub use codec_registry::*;
pub use compression::*;
pub use decompression::*;
pub use image_codecs::*;
pub use transcoding::*;
pub use video_codecs::*;

#[derive(Debug, Clone)]
pub struct CodecProcessor {
  media_processor: MediaProcessor,
}

impl CodecProcessor {
  pub fn new(media_processor: MediaProcessor) -> Self {
    Self { media_processor }
  }

  pub fn media_processor(&self) -> &MediaProcessor {
    &self.media_processor
  }

  pub fn media_processor_mut(&mut self) -> &mut MediaProcessor {
    &mut self.media_processor
  }

  pub fn into_media_processor(self) -> MediaProcessor {
    self.media_processor
  }

  pub fn detect_codec(&self) -> Result<CodecInfo> {
    let media_type = self.media_processor.media_type();
    match media_type {
      MediaType::Image => {
        if let Some(image_processor) = self.media_processor.as_image_processor() {
          detect_image_codec(image_processor)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an image processor".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.media_processor.as_audio_processor() {
          detect_audio_codec(audio_processor)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an audio processor".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.media_processor.as_video_processor() {
          detect_video_codec(video_processor)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not a video processor".to_string(),
          ))
        }
      }
    }
  }

  pub fn transcode(
    &mut self,
    target_codec: &str,
    options: &TranscodeOptions,
  ) -> Result<MediaProcessor> {
    let media_type = self.media_processor.media_type();
    match media_type {
      MediaType::Image => {
        if let Some(image_processor) = self.media_processor.as_image_processor() {
          transcode_image(image_processor, target_codec, options)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an image processor".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.media_processor.as_audio_processor() {
          transcode_audio(audio_processor, target_codec, options)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an audio processor".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.media_processor.as_video_processor() {
          transcode_video(video_processor, target_codec, options)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not a video processor".to_string(),
          ))
        }
      }
    }
  }

  pub fn compress(
    &mut self,
    compression_type: CompressionType,
    level: u8,
  ) -> Result<MediaProcessor> {
    let media_type = self.media_processor.media_type();
    match media_type {
      MediaType::Image => {
        if let Some(image_processor) = self.media_processor.as_image_processor() {
          compress_image(image_processor, compression_type, level)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an image processor".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.media_processor.as_audio_processor() {
          compress_audio(audio_processor, compression_type, level)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an audio processor".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.media_processor.as_video_processor() {
          compress_video(video_processor, compression_type, level)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not a video processor".to_string(),
          ))
        }
      }
    }
  }

  pub fn decompress(&mut self) -> Result<MediaProcessor> {
    let media_type = self.media_processor.media_type();
    match media_type {
      MediaType::Image => {
        if let Some(image_processor) = self.media_processor.as_image_processor() {
          decompress_image(image_processor)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an image processor".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.media_processor.as_audio_processor() {
          decompress_audio(audio_processor)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an audio processor".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.media_processor.as_video_processor() {
          decompress_video(video_processor)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not a video processor".to_string(),
          ))
        }
      }
    }
  }

  pub fn optimize_codec(&mut self, optimization_type: OptimizationType) -> Result<MediaProcessor> {
    let media_type = self.media_processor.media_type();
    match media_type {
      MediaType::Image => {
        if let Some(image_processor) = self.media_processor.as_image_processor() {
          optimize_image_codec(image_processor, optimization_type)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an image processor".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.media_processor.as_audio_processor() {
          optimize_audio_codec(audio_processor, optimization_type)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an audio processor".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.media_processor.as_video_processor() {
          optimize_video_codec(video_processor, optimization_type)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not a video processor".to_string(),
          ))
        }
      }
    }
  }

  pub fn get_codec_capabilities(&self, codec: &str) -> Result<CodecCapabilities> {
    get_codec_info(codec).map(|info| info.capabilities)
  }

  pub fn list_available_codecs(&self, media_type: MediaType) -> Vec<String> {
    list_codecs_for_media_type(media_type)
  }

  pub fn validate_codec_support(&self, codec: &str) -> Result<bool> {
    let capabilities = self.get_codec_capabilities(codec)?;
    Ok(capabilities.is_supported)
  }

  pub fn get_codec_parameters(&self, codec: &str) -> Result<Vec<CodecParameter>> {
    get_codec_info(codec).map(|info| info.parameters)
  }

  pub fn set_codec_parameter(
    &mut self,
    codec: &str,
    parameter: &str,
    value: CodecParameterValue,
  ) -> Result<()> {
    set_codec_parameter_value(&mut self.media_processor, codec, parameter, value)
  }

  pub fn batch_transcode(
    &mut self,
    target_codecs: &[String],
    options: &TranscodeOptions,
  ) -> Result<Vec<MediaProcessor>> {
    let mut results = Vec::new();

    for target_codec in target_codecs {
      let mut temp_processor = self.media_processor.clone();
      let codec_processor = CodecProcessor::new(temp_processor);
      let transcoded = codec_processor.transcode(target_codec, options)?;
      results.push(transcoded);
    }

    Ok(results)
  }

  pub fn create_codec_pipeline(&self, pipeline_steps: &[CodecPipelineStep]) -> CodecPipeline {
    CodecPipeline::new(self.media_processor.clone(), pipeline_steps.to_vec())
  }

  pub fn stream_transcode(
    &mut self,
    target_codec: &str,
    options: &TranscodeOptions,
  ) -> Result<MediaStream> {
    let media_type = self.media_processor.media_type();
    match media_type {
      MediaType::Image => Err(EllasticError::UnsupportedOperation(
        "Streaming not supported for images".to_string(),
      )),
      MediaType::Audio => {
        if let Some(audio_processor) = self.media_processor.as_audio_processor() {
          stream_transcode_audio(audio_processor, target_codec, options)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not an audio processor".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.media_processor.as_video_processor() {
          stream_transcode_video(video_processor, target_codec, options)
        } else {
          Err(EllasticError::InvalidParameter(
            "Not a video processor".to_string(),
          ))
        }
      }
    }
  }

  pub fn clone(&self) -> CodecProcessor {
    CodecProcessor::new(self.media_processor.clone())
  }
}

#[derive(Debug, Clone)]
pub struct CodecInfo {
  pub name: String,
  pub version: String,
  pub media_type: MediaType,
  pub capabilities: CodecCapabilities,
  pub parameters: Vec<CodecParameter>,
  pub supported_formats: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodecCapabilities {
  pub is_supported: bool,
  pub can_encode: bool,
  pub can_decode: bool,
  pub can_compress: bool,
  pub can_decompress: bool,
  pub supports_streaming: bool,
  pub supports_lossless: bool,
  pub supports_lossy: bool,
  pub max_quality: u8,
  pub max_bitrate: Option<u32>,
  pub max_resolution: Option<(u32, u32)>,
  pub max_sample_rate: Option<u32>,
  pub max_channels: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct CodecParameter {
  pub name: String,
  pub parameter_type: ParameterType,
  pub default_value: CodecParameterValue,
  pub min_value: Option<CodecParameterValue>,
  pub max_value: Option<CodecParameterValue>,
  pub description: String,
}

#[derive(Debug, Clone)]
pub enum ParameterType {
  Integer,
  Float,
  String,
  Boolean,
  Enum(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum CodecParameterValue {
  Integer(i64),
  Float(f64),
  String(String),
  Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct TranscodeOptions {
  pub quality: Option<u8>,
  pub bitrate: Option<u32>,
  pub sample_rate: Option<u32>,
  pub channels: Option<u8>,
  pub resolution: Option<(u32, u32)>,
  pub frame_rate: Option<f64>,
  pub compression_level: Option<u8>,
  pub preset: Option<String>,
  pub custom_options: std::collections::HashMap<String, CodecParameterValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
  Lossless,
  Lossy,
  Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationType {
  Quality,
  Size,
  Speed,
  Balanced,
}

#[derive(Debug, Clone)]
pub struct CodecPipeline {
  source: MediaProcessor,
  steps: Vec<CodecPipelineStep>,
}

impl CodecPipeline {
  pub fn new(source: MediaProcessor, steps: Vec<CodecPipelineStep>) -> Self {
    Self { source, steps }
  }

  pub fn source(&self) -> &MediaProcessor {
    &self.source
  }

  pub fn steps(&self) -> &[CodecPipelineStep] {
    &self.steps
  }

  pub fn execute(&self) -> Result<MediaProcessor> {
    let mut current_processor = self.source.clone();

    for step in &self.steps {
      current_processor = self.execute_step(&current_processor, step)?;
    }

    Ok(current_processor)
  }

  fn execute_step(
    &self,
    processor: &MediaProcessor,
    step: &CodecPipelineStep,
  ) -> Result<MediaProcessor> {
    match step {
      CodecPipelineStep::Transcode {
        target_codec,
        options,
      } => {
        let mut codec_processor = CodecProcessor::new(processor.clone());
        codec_processor.transcode(target_codec, options)
      }
      CodecPipelineStep::Compress {
        compression_type,
        level,
      } => {
        let mut codec_processor = CodecProcessor::new(processor.clone());
        codec_processor.compress(*compression_type, *level)
      }
      CodecPipelineStep::Decompress => {
        let mut codec_processor = CodecProcessor::new(processor.clone());
        codec_processor.decompress()
      }
      CodecPipelineStep::Optimize { optimization_type } => {
        let mut codec_processor = CodecProcessor::new(processor.clone());
        codec_processor.optimize_codec(*optimization_type)
      }
      CodecPipelineStep::Filter {
        filter_type,
        parameters,
      } => apply_codec_filter(processor, filter_type, parameters),
    }
  }
}

#[derive(Debug, Clone)]
pub enum CodecPipelineStep {
  Transcode {
    target_codec: String,
    options: TranscodeOptions,
  },
  Compress {
    compression_type: CompressionType,
    level: u8,
  },
  Decompress,
  Optimize {
    optimization_type: OptimizationType,
  },
  Filter {
    filter_type: String,
    parameters: Vec<CodecParameterValue>,
  },
}

#[derive(Debug, Clone)]
pub struct MediaStream {
  pub media_type: MediaType,
  pub codec: String,
  pub data: Vec<u8>,
  pub metadata: std::collections::HashMap<String, String>,
}

fn detect_image_codec(image_processor: &ImageProcessor) -> Result<CodecInfo> {
  let image_data = image_processor.data();
  let format = ellastic_image::detect_format(&image_data.data)?;

  let codec_name = match format {
    ImageFormat::PNG => "PNG",
    ImageFormat::JPEG => "JPEG",
    ImageFormat::BMP => "BMP",
    ImageFormat::GIF => "GIF",
    ImageFormat::TIFF => "TIFF",
    ImageFormat::WEBP => "WebP",
  };

  let capabilities = get_image_codec_capabilities(format);
  let parameters = get_image_codec_parameters(format);

  Ok(CodecInfo {
    name: codec_name.to_string(),
    version: "1.0".to_string(),
    media_type: MediaType::Image,
    capabilities,
    parameters,
    supported_formats: vec![format.to_string()],
  })
}

fn detect_audio_codec(audio_processor: &AudioProcessor) -> Result<CodecInfo> {
  let audio_data = audio_processor.data();
  let format = ellastic_audio::detect_format(
    &audio_data
      .samples
      .iter()
      .flat_map(|&s| s.to_le_bytes())
      .collect::<Vec<_>>(),
  )?;

  let codec_name = match format {
    AudioFormat::WAV => "WAV",
    AudioFormat::MP3 => "MP3",
    AudioFormat::FLAC => "FLAC",
    AudioFormat::OGG => "OGG Vorbis",
    AudioFormat::AAC => "AAC",
  };

  let capabilities = get_audio_codec_capabilities(format);
  let parameters = get_audio_codec_parameters(format);

  Ok(CodecInfo {
    name: codec_name.to_string(),
    version: "1.0".to_string(),
    media_type: MediaType::Audio,
    capabilities,
    parameters,
    supported_formats: vec![format.to_string()],
  })
}

fn detect_video_codec(video_processor: &VideoProcessor) -> Result<CodecInfo> {
  let video_data = video_processor.data();
  let format = detect_video_format_from_data(
    &video_data
      .frames
      .first()
      .map(|f| f.data.clone())
      .unwrap_or_default(),
  )?;

  let codec_name = match format {
    VideoFormat::MP4 => "H.264",
    VideoFormat::AVI => "XVID",
    VideoFormat::MOV => "H.264",
    VideoFormat::WEBM => "VP9",
    VideoFormat::MKV => "H.265",
  };

  let capabilities = get_video_codec_capabilities(format);
  let parameters = get_video_codec_parameters(format);

  Ok(CodecInfo {
    name: codec_name.to_string(),
    version: "1.0".to_string(),
    media_type: MediaType::Video,
    capabilities,
    parameters,
    supported_formats: vec![format.to_string()],
  })
}

fn transcode_image(
  image_processor: &ImageProcessor,
  target_codec: &str,
  options: &TranscodeOptions,
) -> Result<MediaProcessor> {
  let image_data = image_processor.data();

  let target_format = match target_codec {
    "PNG" => ImageFormat::PNG,
    "JPEG" => ImageFormat::JPEG,
    "BMP" => ImageFormat::BMP,
    "GIF" => ImageFormat::GIF,
    "TIFF" => ImageFormat::TIFF,
    "WEBP" => ImageFormat::WEBP,
    _ => {
      return Err(EllasticError::UnsupportedFormat(format!(
        "Unsupported image codec: {}",
        target_codec
      )));
    }
  };

  let quality = options.quality.unwrap_or(85);
  let processor = ImageProcessor::from_image_data(image_data.clone());
  let encoded = processor.encode(target_format, Some(quality))?;
  let decoded = ellastic_image::decode_image(&encoded, target_format)?;

  Ok(MediaProcessor::new(MediaData::Image(decoded)))
}

fn transcode_audio(
  audio_processor: &AudioProcessor,
  target_codec: &str,
  options: &TranscodeOptions,
) -> Result<MediaProcessor> {
  let audio_data = audio_processor.data();

  let target_format = match target_codec {
    "WAV" => AudioFormat::WAV,
    "MP3" => AudioFormat::MP3,
    "FLAC" => AudioFormat::FLAC,
    "OGG" => AudioFormat::OGG,
    "AAC" => AudioFormat::AAC,
    _ => {
      return Err(EllasticError::UnsupportedFormat(format!(
        "Unsupported audio codec: {}",
        target_codec
      )));
    }
  };

  let quality = options.quality.unwrap_or(128);
  let bitrate = options.bitrate.unwrap_or(128000);
  let sample_rate = options.sample_rate.unwrap_or(audio_data.sample_rate);
  let channels = options.channels.unwrap_or(audio_data.channels);

  let processor = AudioProcessor::from_audio_data(audio_data.clone());
  let encoded = processor.encode(target_format, Some(quality))?;
  let decoded = ellastic_audio::decode_audio(&encoded, target_format)?;

  Ok(MediaProcessor::new(MediaData::Audio(decoded)))
}

fn transcode_video(
  video_processor: &VideoProcessor,
  target_codec: &str,
  options: &TranscodeOptions,
) -> Result<MediaProcessor> {
  let video_data = video_processor.data();

  let target_format = match target_codec {
    "MP4" => VideoFormat::MP4,
    "AVI" => VideoFormat::AVI,
    "MOV" => VideoFormat::MOV,
    "WEBM" => VideoFormat::WEBM,
    "MKV" => VideoFormat::MKV,
    _ => {
      return Err(EllasticError::UnsupportedFormat(format!(
        "Unsupported video codec: {}",
        target_codec
      )));
    }
  };

  let quality = options.quality.unwrap_or(75);
  let bitrate = options.bitrate.unwrap_or(5000000);
  let resolution = options
    .resolution
    .unwrap_or((video_data.width, video_data.height));
  let frame_rate = options.frame_rate.unwrap_or(video_data.frame_rate);

  let processor = VideoProcessor::new_with_data(video_data.clone());
  let encoded = processor.encode(target_format, Some(quality))?;
  let decoded = decode_video(&encoded, target_format)?;

  Ok(MediaProcessor::new(MediaData::Video(decoded)))
}

fn compress_image(
  image_processor: &ImageProcessor,
  compression_type: CompressionType,
  level: u8,
) -> Result<MediaProcessor> {
  let image_data = image_processor.data();

  match compression_type {
    CompressionType::Lossless => {
      let processor = ImageProcessor::from_image_data(image_data.clone());
      let encoded = processor.encode(ImageFormat::PNG, None)?;
      let decoded = ellastic_image::decode_image(&encoded, ImageFormat::PNG)?;
      Ok(MediaProcessor::new(MediaData::Image(decoded)))
    }
    CompressionType::Lossy => {
      let processor = ImageProcessor::from_image_data(image_data.clone());
      let encoded = processor.encode(ImageFormat::JPEG, Some(level))?;
      let decoded = ellastic_image::decode_image(&encoded, ImageFormat::JPEG)?;
      Ok(MediaProcessor::new(MediaData::Image(decoded)))
    }
    CompressionType::Hybrid => {
      let processor = ImageProcessor::from_image_data(image_data.clone());
      let encoded = processor.encode(ImageFormat::WEBP, Some(level))?;
      let decoded = ellastic_image::decode_image(&encoded, ImageFormat::WEBP)?;
      Ok(MediaProcessor::new(MediaData::Image(decoded)))
    }
  }
}

fn compress_audio(
  audio_processor: &AudioProcessor,
  compression_type: CompressionType,
  level: u8,
) -> Result<MediaProcessor> {
  let audio_data = audio_processor.data();

  match compression_type {
    CompressionType::Lossless => {
      let processor = AudioProcessor::from_audio_data(audio_data.clone());
      let encoded = processor.encode(AudioFormat::FLAC, None)?;
      let decoded = ellastic_audio::decode_audio(&encoded, AudioFormat::FLAC)?;
      Ok(MediaProcessor::new(MediaData::Audio(decoded)))
    }
    CompressionType::Lossy => {
      let processor = AudioProcessor::from_audio_data(audio_data.clone());
      let encoded = processor.encode(AudioFormat::MP3, Some(level))?;
      let decoded = ellastic_audio::decode_audio(&encoded, AudioFormat::MP3)?;
      Ok(MediaProcessor::new(MediaData::Audio(decoded)))
    }
    CompressionType::Hybrid => {
      let processor = AudioProcessor::from_audio_data(audio_data.clone());
      let encoded = processor.encode(AudioFormat::OGG, Some(level))?;
      let decoded = ellastic_audio::decode_audio(&encoded, AudioFormat::OGG)?;
      Ok(MediaProcessor::new(MediaData::Audio(decoded)))
    }
  }
}

fn compress_video(
  video_processor: &VideoProcessor,
  compression_type: CompressionType,
  level: u8,
) -> Result<MediaProcessor> {
  let video_data = video_processor.data();

  match compression_type {
    CompressionType::Lossless => {
      let processor = VideoProcessor::new_with_data(video_data.clone());
      let encoded = processor.encode(VideoFormat::MP4, None)?;
      let decoded = decode_video(&encoded, VideoFormat::MP4)?;
      Ok(MediaProcessor::new(MediaData::Video(decoded)))
    }
    CompressionType::Lossy => {
      let processor = VideoProcessor::new_with_data(video_data.clone());
      let encoded = processor.encode(VideoFormat::MP4, Some(level))?;
      let decoded = decode_video(&encoded, VideoFormat::MP4)?;
      Ok(MediaProcessor::new(MediaData::Video(decoded)))
    }
    CompressionType::Hybrid => {
      let processor = VideoProcessor::new_with_data(video_data.clone());
      let encoded = processor.encode(VideoFormat::WEBM, Some(level))?;
      let decoded = decode_video(&encoded, VideoFormat::WEBM)?;
      Ok(MediaProcessor::new(MediaData::Video(decoded)))
    }
  }
}

fn decompress_image(image_processor: &ImageProcessor) -> Result<MediaProcessor> {
  let image_data = image_processor.data();
  let processor = ImageProcessor::from_image_data(image_data.clone());
  let encoded = processor.encode(ImageFormat::PNG, None)?;
  let decoded = ellastic_image::decode_image(&encoded, ImageFormat::PNG)?;
  Ok(MediaProcessor::new(MediaData::Image(decoded)))
}

fn decompress_audio(audio_processor: &AudioProcessor) -> Result<MediaProcessor> {
  let audio_data = audio_processor.data();
  let processor = AudioProcessor::from_audio_data(audio_data.clone());
  let encoded = processor.encode(AudioFormat::WAV, None)?;
  let decoded = ellastic_audio::decode_audio(&encoded, AudioFormat::WAV)?;
  Ok(MediaProcessor::new(MediaData::Audio(decoded)))
}

fn decompress_video(video_processor: &VideoProcessor) -> Result<MediaProcessor> {
  let video_data = video_processor.data();
  let processor = VideoProcessor::new_with_data(video_data.clone());
  let encoded = processor.encode(VideoFormat::MP4, None)?;
  let decoded = decode_video(&encoded, VideoFormat::MP4)?;
  Ok(MediaProcessor::new(MediaData::Video(decoded)))
}

fn optimize_image_codec(
  image_processor: &ImageProcessor,
  optimization_type: OptimizationType,
) -> Result<MediaProcessor> {
  let image_data = image_processor.data();
  let processor = ImageProcessor::from_image_data(image_data.clone());

  match optimization_type {
    OptimizationType::Quality => {
      let encoded = processor.encode(ImageFormat::PNG, None)?;
      let decoded = ellastic_image::decode_image(&encoded, ImageFormat::PNG)?;
      Ok(MediaProcessor::new(MediaData::Image(decoded)))
    }
    OptimizationType::Size => {
      let encoded = processor.encode(ImageFormat::JPEG, Some(50))?;
      let decoded = ellastic_image::decode_image(&encoded, ImageFormat::JPEG)?;
      Ok(MediaProcessor::new(MediaData::Image(decoded)))
    }
    OptimizationType::Speed => {
      let encoded = processor.encode(ImageFormat::BMP, None)?;
      let decoded = ellastic_image::decode_image(&encoded, ImageFormat::BMP)?;
      Ok(MediaProcessor::new(MediaData::Image(decoded)))
    }
    OptimizationType::Balanced => {
      let encoded = processor.encode(ImageFormat::WEBP, Some(75))?;
      let decoded = ellastic_image::decode_image(&encoded, ImageFormat::WEBP)?;
      Ok(MediaProcessor::new(MediaData::Image(decoded)))
    }
  }
}

fn optimize_audio_codec(
  audio_processor: &AudioProcessor,
  optimization_type: OptimizationType,
) -> Result<MediaProcessor> {
  let audio_data = audio_processor.data();
  let processor = AudioProcessor::from_audio_data(audio_data.clone());

  match optimization_type {
    OptimizationType::Quality => {
      let encoded = processor.encode(AudioFormat::FLAC, None)?;
      let decoded = ellastic_audio::decode_audio(&encoded, AudioFormat::FLAC)?;
      Ok(MediaProcessor::new(MediaData::Audio(decoded)))
    }
    OptimizationType::Size => {
      let encoded = processor.encode(AudioFormat::MP3, Some(96))?;
      let decoded = ellastic_audio::decode_audio(&encoded, AudioFormat::MP3)?;
      Ok(MediaProcessor::new(MediaData::Audio(decoded)))
    }
    OptimizationType::Speed => {
      let encoded = processor.encode(AudioFormat::WAV, None)?;
      let decoded = ellastic_audio::decode_audio(&encoded, AudioFormat::WAV)?;
      Ok(MediaProcessor::new(MediaData::Audio(decoded)))
    }
    OptimizationType::Balanced => {
      let encoded = processor.encode(AudioFormat::OGG, Some(128))?;
      let decoded = ellastic_audio::decode_audio(&encoded, AudioFormat::OGG)?;
      Ok(MediaProcessor::new(MediaData::Audio(decoded)))
    }
  }
}

fn optimize_video_codec(
  video_processor: &VideoProcessor,
  optimization_type: OptimizationType,
) -> Result<MediaProcessor> {
  let video_data = video_processor.data();
  let processor = VideoProcessor::new_with_data(video_data.clone());

  match optimization_type {
    OptimizationType::Quality => {
      let encoded = processor.encode(VideoFormat::MP4, Some(90))?;
      let decoded = decode_video(&encoded, VideoFormat::MP4)?;
      Ok(MediaProcessor::new(MediaData::Video(decoded)))
    }
    OptimizationType::Size => {
      let encoded = processor.encode(VideoFormat::MP4, Some(50))?;
      let decoded = decode_video(&encoded, VideoFormat::MP4)?;
      Ok(MediaProcessor::new(MediaData::Video(decoded)))
    }
    OptimizationType::Speed => {
      let encoded = processor.encode(VideoFormat::AVI, Some(75))?;
      let decoded = decode_video(&encoded, VideoFormat::AVI)?;
      Ok(MediaProcessor::new(MediaData::Video(decoded)))
    }
    OptimizationType::Balanced => {
      let encoded = processor.encode(VideoFormat::WEBM, Some(75))?;
      let decoded = decode_video(&encoded, VideoFormat::WEBM)?;
      Ok(MediaProcessor::new(MediaData::Video(decoded)))
    }
  }
}

fn get_image_codec_capabilities(format: ImageFormat) -> CodecCapabilities {
  match format {
    ImageFormat::PNG => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: false,
      supports_lossless: true,
      supports_lossy: false,
      max_quality: 100,
      max_bitrate: None,
      max_resolution: Some((65536, 65536)),
      max_sample_rate: None,
      max_channels: None,
    },
    ImageFormat::JPEG => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: false,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: None,
      max_resolution: Some((65536, 65536)),
      max_sample_rate: None,
      max_channels: None,
    },
    ImageFormat::BMP => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: false,
      can_decompress: false,
      supports_streaming: false,
      supports_lossless: true,
      supports_lossy: false,
      max_quality: 100,
      max_bitrate: None,
      max_resolution: Some((65536, 65536)),
      max_sample_rate: None,
      max_channels: None,
    },
    ImageFormat::GIF => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: true,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: None,
      max_resolution: Some((65536, 65536)),
      max_sample_rate: None,
      max_channels: None,
    },
    ImageFormat::TIFF => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: false,
      supports_lossless: true,
      supports_lossy: false,
      max_quality: 100,
      max_bitrate: None,
      max_resolution: Some((65536, 65536)),
      max_sample_rate: None,
      max_channels: None,
    },
    ImageFormat::WEBP => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: false,
      supports_lossless: true,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: None,
      max_resolution: Some((16384, 16384)),
      max_sample_rate: None,
      max_channels: None,
    },
  }
}

fn get_audio_codec_capabilities(format: AudioFormat) -> CodecCapabilities {
  match format {
    AudioFormat::WAV => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: false,
      can_decompress: false,
      supports_streaming: true,
      supports_lossless: true,
      supports_lossy: false,
      max_quality: 100,
      max_bitrate: None,
      max_resolution: None,
      max_sample_rate: Some(192000),
      max_channels: Some(8),
    },
    AudioFormat::MP3 => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: true,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: Some(320000),
      max_resolution: None,
      max_sample_rate: Some(48000),
      max_channels: Some(2),
    },
    AudioFormat::FLAC => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: false,
      supports_lossless: true,
      supports_lossy: false,
      max_quality: 100,
      max_bitrate: None,
      max_resolution: None,
      max_sample_rate: Some(192000),
      max_channels: Some(8),
    },
    AudioFormat::OGG => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: true,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: Some(500000),
      max_resolution: None,
      max_sample_rate: Some(96000),
      max_channels: Some(8),
    },
    AudioFormat::AAC => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: true,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: Some(256000),
      max_resolution: None,
      max_sample_rate: Some(96000),
      max_channels: Some(8),
    },
  }
}

fn get_video_codec_capabilities(format: VideoFormat) -> CodecCapabilities {
  match format {
    VideoFormat::MP4 => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: true,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: Some(10000000),
      max_resolution: Some((7680, 4320)),
      max_sample_rate: Some(192000),
      max_channels: Some(8),
    },
    VideoFormat::AVI => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: false,
      can_decompress: false,
      supports_streaming: false,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: Some(5000000),
      max_resolution: Some((4096, 4096)),
      max_sample_rate: Some(48000),
      max_channels: Some(2),
    },
    VideoFormat::MOV => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: true,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: Some(8000000),
      max_resolution: Some((4096, 4096)),
      max_sample_rate: Some(48000),
      max_channels: Some(8),
    },
    VideoFormat::WEBM => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: true,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: Some(5000000),
      max_resolution: Some((4096, 4096)),
      max_sample_rate: Some(96000),
      max_channels: Some(8),
    },
    VideoFormat::MKV => CodecCapabilities {
      is_supported: true,
      can_encode: true,
      can_decode: true,
      can_compress: true,
      can_decompress: true,
      supports_streaming: false,
      supports_lossless: false,
      supports_lossy: true,
      max_quality: 100,
      max_bitrate: Some(10000000),
      max_resolution: Some((7680, 4320)),
      max_sample_rate: Some(192000),
      max_channels: Some(8),
    },
  }
}

fn get_image_codec_parameters(format: ImageFormat) -> Vec<CodecParameter> {
  match format {
    ImageFormat::JPEG => vec![CodecParameter {
      name: "quality".to_string(),
      parameter_type: ParameterType::Integer,
      default_value: CodecParameterValue::Integer(85),
      min_value: Some(CodecParameterValue::Integer(1)),
      max_value: Some(CodecParameterValue::Integer(100)),
      description: "JPEG quality level (1-100)".to_string(),
    }],
    ImageFormat::PNG => vec![CodecParameter {
      name: "compression".to_string(),
      parameter_type: ParameterType::Integer,
      default_value: CodecParameterValue::Integer(6),
      min_value: Some(CodecParameterValue::Integer(0)),
      max_value: Some(CodecParameterValue::Integer(9)),
      description: "PNG compression level (0-9)".to_string(),
    }],
    _ => Vec::new(),
  }
}

fn get_audio_codec_parameters(format: AudioFormat) -> Vec<CodecParameter> {
  match format {
    AudioFormat::MP3 => vec![CodecParameter {
      name: "bitrate".to_string(),
      parameter_type: ParameterType::Integer,
      default_value: CodecParameterValue::Integer(128000),
      min_value: Some(CodecParameterValue::Integer(32000)),
      max_value: Some(CodecParameterValue::Integer(320000)),
      description: "MP3 bitrate in bps".to_string(),
    }],
    AudioFormat::FLAC => vec![CodecParameter {
      name: "compression".to_string(),
      parameter_type: ParameterType::Integer,
      default_value: CodecParameterValue::Integer(5),
      min_value: Some(CodecParameterValue::Integer(0)),
      max_value: Some(CodecParameterValue::Integer(8)),
      description: "FLAC compression level (0-8)".to_string(),
    }],
    _ => Vec::new(),
  }
}

fn get_video_codec_parameters(format: VideoFormat) -> Vec<CodecParameter> {
  match format {
    VideoFormat::MP4 => vec![CodecParameter {
      name: "quality".to_string(),
      parameter_type: ParameterType::Integer,
      default_value: CodecParameterValue::Integer(75),
      min_value: Some(CodecParameterValue::Integer(1)),
      max_value: Some(CodecParameterValue::Integer(100)),
      description: "MP4 quality level (1-100)".to_string(),
    }],
    _ => Vec::new(),
  }
}

fn get_codec_info(codec: &str) -> Option<CodecInfo> {
  let mut codecs = list_all_codecs();
  codecs.retain(|c| c.name == codec);
  codecs.into_iter().next()
}

fn list_codecs_for_media_type(media_type: MediaType) -> Vec<String> {
  list_all_codecs()
    .into_iter()
    .filter(|c| c.media_type == media_type)
    .map(|c| c.name)
    .collect()
}

fn list_all_codecs() -> Vec<CodecInfo> {
  vec![
    CodecInfo {
      name: "PNG".to_string(),
      version: "1.0".to_string(),
      media_type: MediaType::Image,
      capabilities: get_image_codec_capabilities(ImageFormat::PNG),
      parameters: get_image_codec_parameters(ImageFormat::PNG),
      supported_formats: vec!["PNG".to_string()],
    },
    CodecInfo {
      name: "JPEG".to_string(),
      version: "1.0".to_string(),
      media_type: MediaType::Image,
      capabilities: get_image_codec_capabilities(ImageFormat::JPEG),
      parameters: get_image_codec_parameters(ImageFormat::JPEG),
      supported_formats: vec!["JPEG".to_string()],
    },
    CodecInfo {
      name: "MP3".to_string(),
      version: "1.0".to_string(),
      media_type: MediaType::Audio,
      capabilities: get_audio_codec_capabilities(AudioFormat::MP3),
      parameters: get_audio_codec_parameters(AudioFormat::MP3),
      supported_formats: vec!["MP3".to_string()],
    },
    CodecInfo {
      name: "FLAC".to_string(),
      version: "1.0".to_string(),
      media_type: MediaType::Audio,
      capabilities: get_audio_codec_capabilities(AudioFormat::FLAC),
      parameters: get_audio_codec_parameters(AudioFormat::FLAC),
      supported_formats: vec!["FLAC".to_string()],
    },
    CodecInfo {
      name: "H.264".to_string(),
      version: "1.0".to_string(),
      media_type: MediaType::Video,
      capabilities: get_video_codec_capabilities(VideoFormat::MP4),
      parameters: get_video_codec_parameters(VideoFormat::MP4),
      supported_formats: vec!["MP4".to_string()],
    },
  ]
}

fn set_codec_parameter_value(
  media_processor: &mut MediaProcessor,
  codec: &str,
  parameter: &str,
  value: CodecParameterValue,
) -> Result<()> {
  Ok(())
}

fn apply_codec_filter(
  processor: &MediaProcessor,
  filter_type: &str,
  parameters: &[CodecParameterValue],
) -> Result<MediaProcessor> {
  Ok(processor.clone())
}

fn stream_transcode_audio(
  audio_processor: &AudioProcessor,
  target_codec: &str,
  options: &TranscodeOptions,
) -> Result<MediaStream> {
  let transcoded = transcode_audio(audio_processor, target_codec, options)?;
  let audio_data = transcoded.data().clone();

  Ok(MediaStream {
    media_type: MediaType::Audio,
    codec: target_codec.to_string(),
    data: audio_data
      .samples
      .iter()
      .flat_map(|&s| s.to_le_bytes())
      .collect(),
    metadata: std::collections::HashMap::new(),
  })
}

fn stream_transcode_video(
  video_processor: &VideoProcessor,
  target_codec: &str,
  options: &TranscodeOptions,
) -> Result<MediaStream> {
  let transcoded = transcode_video(video_processor, target_codec, options)?;
  let video_data = transcoded.data().clone();

  Ok(MediaStream {
    media_type: MediaType::Video,
    codec: target_codec.to_string(),
    data: video_data
      .frames
      .iter()
      .flat_map(|f| f.data.clone())
      .collect(),
    metadata: std::collections::HashMap::new(),
  })
}

fn detect_video_format_from_data(data: &[u8]) -> Result<VideoFormat> {
  if data.starts_with(b"ftyp") {
    Ok(VideoFormat::MP4)
  } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"AVI " {
    Ok(VideoFormat::AVI)
  } else if data.starts_with(&[0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70]) {
    Ok(VideoFormat::MOV)
  } else {
    Err(EllasticError::UnsupportedFormat(
      "Unknown video format".to_string(),
    ))
  }
}

pub fn create_codec_processor(media_processor: MediaProcessor) -> CodecProcessor {
  CodecProcessor::new(media_processor)
}

pub fn transcode_media(
  media_processor: &MediaProcessor,
  target_codec: &str,
  options: &TranscodeOptions,
) -> Result<MediaProcessor> {
  let mut codec_processor = create_codec_processor(media_processor.clone());
  codec_processor.transcode(target_codec, options)
}

pub fn compress_media(
  media_processor: &MediaProcessor,
  compression_type: CompressionType,
  level: u8,
) -> Result<MediaProcessor> {
  let mut codec_processor = create_codec_processor(media_processor.clone());
  codec_processor.compress(compression_type, level)
}

pub fn create_codec_pipeline(
  media_processor: &MediaProcessor,
  steps: &[CodecPipelineStep],
) -> CodecPipeline {
  CodecPipeline::new(media_processor.clone(), steps.to_vec())
}
