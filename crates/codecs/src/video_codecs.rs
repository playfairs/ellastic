use ellastic_core::VideoData;
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_media::{
  VideoFormat,
  VideoProcessor,
};

#[derive(Debug, Clone)]
pub struct H264Codec {
  profile: H264Profile,
  level: H264Level,
  bitrate: u32,
  frame_rate: f64,
  resolution: (u32, u32),
  quality: u8,
  preset: H264Preset,
  tune: H264Tune,
}

impl H264Codec {
  pub fn new(
    profile: H264Profile,
    level: H264Level,
    bitrate: u32,
    frame_rate: f64,
    resolution: (u32, u32),
    quality: u8,
    preset: H264Preset,
    tune: H264Tune,
  ) -> Self {
    Self {
      profile,
      level,
      bitrate,
      frame_rate,
      resolution,
      quality,
      preset,
      tune,
    }
  }

  pub fn profile(&self) -> H264Profile {
    self.profile
  }

  pub fn set_profile(&mut self, profile: H264Profile) {
    self.profile = profile;
  }

  pub fn level(&self) -> H264Level {
    self.level
  }

  pub fn set_level(&mut self, level: H264Level) {
    self.level = level;
  }

  pub fn bitrate(&self) -> u32 {
    self.bitrate
  }

  pub fn set_bitrate(&mut self, bitrate: u32) {
    self.bitrate = bitrate;
  }

  pub fn frame_rate(&self) -> f64 {
    self.frame_rate
  }

  pub fn set_frame_rate(&mut self, frame_rate: f64) {
    self.frame_rate = frame_rate;
  }

  pub fn resolution(&self) -> (u32, u32) {
    self.resolution
  }

  pub fn set_resolution(&mut self, resolution: (u32, u32)) {
    self.resolution = resolution;
  }

  pub fn quality(&self) -> u8 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: u8) {
    self.quality = quality.clamp(0, 51);
  }

  pub fn preset(&self) -> H264Preset {
    self.preset
  }

  pub fn set_preset(&mut self, preset: H264Preset) {
    self.preset = preset;
  }

  pub fn tune(&self) -> H264Tune {
    self.tune
  }

  pub fn set_tune(&mut self, tune: H264Tune) {
    self.tune = tune;
  }

  pub fn encode(&self, video_processor: &VideoProcessor) -> Result<Vec<u8>> {
    video_processor.encode(VideoFormat::MP4, Some(self.quality))
  }

  pub fn decode(&self, data: &[u8]) -> Result<VideoProcessor> {
    let video_data = decode_h264(data)?;
    Ok(VideoProcessor::new_with_data(video_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    true
  }

  pub fn supports_interlaced_encoding(&self) -> bool {
    true
  }

  pub fn supports_variable_bitrate(&self) -> bool {
    true
  }

  pub fn supports_constant_bitrate(&self) -> bool {
    true
  }

  pub fn supports_constant_quality(&self) -> bool {
    true
  }

  pub fn get_max_resolution(&self) -> (u32, u32) {
    match self.level {
      H264Level::Level1 => (352, 288),
      H264Level::Level1b => (352, 288),
      H264Level::Level1_1 => (640, 480),
      H264Level::Level1_2 => (720, 480),
      H264Level::Level1_3 => (720, 480),
      H264Level::Level2 => (720, 480),
      H264Level::Level2_1 => (720, 480),
      H264Level::Level2_2 => (720, 480),
      H264Level::Level3 => (1280, 720),
      H264Level::Level3_1 => (1280, 720),
      H264Level::Level3_2 => (1280, 720),
      H264Level::Level4 => (1920, 1080),
      H264Level::Level4_1 => (1920, 1080),
      H264Level::Level4_2 => (1920, 1080),
      H264Level::Level5 => (3840, 2160),
      H264Level::Level5_1 => (3840, 2160),
      H264Level::Level5_2 => (3840, 2160),
    }
  }

  pub fn get_max_bitrate(&self) -> u32 {
    match self.level {
      H264Level::Level1 => 64000,
      H264Level::Level1b => 128000,
      H264Level::Level1_1 => 192000,
      H264Level::Level1_2 => 384000,
      H264Level::Level1_3 => 768000,
      H264Level::Level2 => 2000000,
      H264Level::Level2_1 => 4000000,
      H264Level::Level2_2 => 4000000,
      H264Level::Level3 => 10000000,
      H264Level::Level3_1 => 14000000,
      H264Level::Level3_2 => 20000000,
      H264Level::Level4 => 20000000,
      H264Level::Level4_1 => 50000000,
      H264Level::Level4_2 => 50000000,
      H264Level::Level5 => 135000000,
      H264Level::Level5_1 => 240000000,
      H264Level::Level5_2 => 240000000,
    }
  }

  pub fn estimate_output_size(&self, video_processor: &VideoProcessor) -> usize {
    let video_data = video_processor.data();
    let duration = video_data.duration_seconds();
    let estimated_size = (self.bitrate as f64 * duration / 8.0) as usize;

    let quality_factor = (51 - self.quality as f32) / 51.0;
    let preset_factor = match self.preset {
      H264Preset::Ultrafast => 1.5,
      H264Preset::Superfast => 1.3,
      H264Preset::Veryfast => 1.2,
      H264Preset::Faster => 1.1,
      H264Preset::Fast => 1.0,
      H264Preset::Medium => 0.9,
      H264Preset::Slow => 0.8,
      H264Preset::Slower => 0.7,
      H264Preset::Veryslow => 0.6,
    };

    (estimated_size as f32 * quality_factor * preset_factor) as usize
  }

  pub fn validate_video_data(&self, video_processor: &VideoProcessor) -> Result<()> {
    let video_data = video_processor.data();
    let max_res = self.get_max_resolution();

    if video_data.width > max_res.0 || video_data.height > max_res.1 {
      return Err(EllasticError::InvalidParameter(format!(
        "Resolution {}x{} exceeds maximum {}x{}",
        video_data.width, video_data.height, max_res.0, max_res.1
      )));
    }

    if video_data.frame_rate > self.frame_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Frame rate {} exceeds maximum {}",
        video_data.frame_rate, self.frame_rate
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> H264Codec {
    Self {
      profile: self.profile,
      level: self.level,
      bitrate: self.bitrate,
      frame_rate: self.frame_rate,
      resolution: self.resolution,
      quality: self.quality,
      preset: self.preset,
      tune: self.tune,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H264Profile {
  Baseline,
  Main,
  High,
  High10,
  High422,
  High444,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H264Level {
  Level1,
  Level1b,
  Level1_1,
  Level1_2,
  Level1_3,
  Level2,
  Level2_1,
  Level2_2,
  Level3,
  Level3_1,
  Level3_2,
  Level4,
  Level4_1,
  Level4_2,
  Level5,
  Level5_1,
  Level5_2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H264Preset {
  Ultrafast,
  Superfast,
  Veryfast,
  Faster,
  Fast,
  Medium,
  Slow,
  Slower,
  Veryslow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H264Tune {
  None,
  Film,
  Animation,
  Grain,
  StillImage,
  FastDecode,
  ZeroLatency,
}

#[derive(Debug, Clone)]
pub struct VP9Codec {
  quality: u8,
  bitrate: u32,
  frame_rate: f64,
  resolution: (u32, u32),
  threads: u8,
  tile_columns: u8,
  row_mt: bool,
  cpu_used: i8,
}

impl VP9Codec {
  pub fn new(
    quality: u8,
    bitrate: u32,
    frame_rate: f64,
    resolution: (u32, u32),
    threads: u8,
    tile_columns: u8,
    row_mt: bool,
    cpu_used: i8,
  ) -> Self {
    Self {
      quality,
      bitrate,
      frame_rate,
      resolution,
      threads,
      tile_columns,
      row_mt,
      cpu_used,
    }
  }

  pub fn quality(&self) -> u8 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: u8) {
    self.quality = quality.clamp(0, 63);
  }

  pub fn bitrate(&self) -> u32 {
    self.bitrate
  }

  pub fn set_bitrate(&mut self, bitrate: u32) {
    self.bitrate = bitrate;
  }

  pub fn frame_rate(&self) -> f64 {
    self.frame_rate
  }

  pub fn set_frame_rate(&mut self, frame_rate: f64) {
    self.frame_rate = frame_rate;
  }

  pub fn resolution(&self) -> (u32, u32) {
    self.resolution
  }

  pub fn set_resolution(&mut self, resolution: (u32, u32)) {
    self.resolution = resolution;
  }

  pub fn threads(&self) -> u8 {
    self.threads
  }

  pub fn set_threads(&mut self, threads: u8) {
    self.threads = threads.clamp(1, 16);
  }

  pub fn tile_columns(&self) -> u8 {
    self.tile_columns
  }

  pub fn set_tile_columns(&mut self, tile_columns: u8) {
    self.tile_columns = tile_columns.clamp(0, 6);
  }

  pub fn row_mt(&self) -> bool {
    self.row_mt
  }

  pub fn set_row_mt(&mut self, row_mt: bool) {
    self.row_mt = row_mt;
  }

  pub fn cpu_used(&self) -> i8 {
    self.cpu_used
  }

  pub fn set_cpu_used(&mut self, cpu_used: i8) {
    self.cpu_used = cpu_used.clamp(-16, 16);
  }

  pub fn encode(&self, video_processor: &VideoProcessor) -> Result<Vec<u8>> {
    video_processor.encode(VideoFormat::WEBM, Some(self.quality))
  }

  pub fn decode(&self, data: &[u8]) -> Result<VideoProcessor> {
    let video_data = decode_vp9(data)?;
    Ok(VideoProcessor::new_with_data(video_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    true
  }

  pub fn supports_interlaced_encoding(&self) -> bool {
    false
  }

  pub fn supports_variable_bitrate(&self) -> bool {
    true
  }

  pub fn supports_constant_bitrate(&self) -> bool {
    true
  }

  pub fn supports_constant_quality(&self) -> bool {
    true
  }

  pub fn get_max_resolution(&self) -> (u32, u32) {
    (16384, 16384)
  }

  pub fn get_max_bitrate(&self) -> u32 {
    100000000
  }

  pub fn estimate_output_size(&self, video_processor: &VideoProcessor) -> usize {
    let video_data = video_processor.data();
    let duration = video_data.duration_seconds();
    let estimated_size = (self.bitrate as f64 * duration / 8.0) as usize;

    let quality_factor = (63 - self.quality as f32) / 63.0;
    let cpu_factor = match self.cpu_used {
      -16..=-12 => 1.8,
      -11..=-8 => 1.6,
      -7..=-4 => 1.4,
      -3..=0 => 1.2,
      1..=4 => 1.0,
      5..=8 => 0.9,
      9..=12 => 0.8,
      13..=16 => 0.7,
      _ => 1.0,
    };

    (estimated_size as f32 * quality_factor * cpu_factor) as usize
  }

  pub fn validate_video_data(&self, video_processor: &VideoProcessor) -> Result<()> {
    let video_data = video_processor.data();
    let max_res = self.get_max_resolution();

    if video_data.width > max_res.0 || video_data.height > max_res.1 {
      return Err(EllasticError::InvalidParameter(format!(
        "Resolution {}x{} exceeds maximum {}x{}",
        video_data.width, video_data.height, max_res.0, max_res.1
      )));
    }

    if video_data.frame_rate > self.frame_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Frame rate {} exceeds maximum {}",
        video_data.frame_rate, self.frame_rate
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> VP9Codec {
    Self {
      quality: self.quality,
      bitrate: self.bitrate,
      frame_rate: self.frame_rate,
      resolution: self.resolution,
      threads: self.threads,
      tile_columns: self.tile_columns,
      row_mt: self.row_mt,
      cpu_used: self.cpu_used,
    }
  }
}

#[derive(Debug, Clone)]
pub struct AV1Codec {
  quality: u8,
  bitrate: u32,
  frame_rate: f64,
  resolution: (u32, u32),
  threads: u8,
  tile_rows: u8,
  tile_cols: u8,
  cpu_used: i8,
}

impl AV1Codec {
  pub fn new(
    quality: u8,
    bitrate: u32,
    frame_rate: f64,
    resolution: (u32, u32),
    threads: u8,
    tile_rows: u8,
    tile_cols: u8,
    cpu_used: i8,
  ) -> Self {
    Self {
      quality,
      bitrate,
      frame_rate,
      resolution,
      threads,
      tile_rows,
      tile_cols,
      cpu_used,
    }
  }

  pub fn quality(&self) -> u8 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: u8) {
    self.quality = quality.clamp(0, 63);
  }

  pub fn bitrate(&self) -> u32 {
    self.bitrate
  }

  pub fn set_bitrate(&mut self, bitrate: u32) {
    self.bitrate = bitrate;
  }

  pub fn frame_rate(&self) -> f64 {
    self.frame_rate
  }

  pub fn set_frame_rate(&mut self, frame_rate: f64) {
    self.frame_rate = frame_rate;
  }

  pub fn resolution(&self) -> (u32, u32) {
    self.resolution
  }

  pub fn set_resolution(&mut self, resolution: (u32, u32)) {
    self.resolution = resolution;
  }

  pub fn threads(&self) -> u8 {
    self.threads
  }

  pub fn set_threads(&mut self, threads: u8) {
    self.threads = threads.clamp(1, 64);
  }

  pub fn tile_rows(&self) -> u8 {
    self.tile_rows
  }

  pub fn set_tile_rows(&mut self, tile_rows: u8) {
    self.tile_rows = tile_rows.clamp(0, 6);
  }

  pub fn tile_cols(&self) -> u8 {
    self.tile_cols
  }

  pub fn set_tile_cols(&mut self, tile_cols: u8) {
    self.tile_cols = tile_cols.clamp(0, 6);
  }

  pub fn cpu_used(&self) -> i8 {
    self.cpu_used
  }

  pub fn set_cpu_used(&mut self, cpu_used: i8) {
    self.cpu_used = cpu_used.clamp(-8, 8);
  }

  pub fn encode(&self, video_processor: &VideoProcessor) -> Result<Vec<u8>> {
    video_processor.encode(VideoFormat::WEBM, Some(self.quality))
  }

  pub fn decode(&self, data: &[u8]) -> Result<VideoProcessor> {
    let video_data = decode_av1(data)?;
    Ok(VideoProcessor::new_with_data(video_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    true
  }

  pub fn supports_interlaced_encoding(&self) -> bool {
    false
  }

  pub fn supports_variable_bitrate(&self) -> bool {
    true
  }

  pub fn supports_constant_bitrate(&self) -> bool {
    true
  }

  pub fn supports_constant_quality(&self) -> bool {
    true
  }

  pub fn get_max_resolution(&self) -> (u32, u32) {
    (32768, 32768)
  }

  pub fn get_max_bitrate(&self) -> u32 {
    1000000000
  }

  pub fn estimate_output_size(&self, video_processor: &VideoProcessor) -> usize {
    let video_data = video_processor.data();
    let duration = video_data.duration_seconds();
    let estimated_size = (self.bitrate as f64 * duration / 8.0) as usize;

    let quality_factor = (63 - self.quality as f32) / 63.0;
    let cpu_factor = match self.cpu_used {
      -8..=-6 => 1.5,
      -5..=-4 => 1.3,
      -3..=-2 => 1.2,
      -1..=0 => 1.1,
      1..=2 => 1.0,
      3..=4 => 0.9,
      5..=6 => 0.8,
      7..=8 => 0.7,
      _ => 1.0,
    };

    (estimated_size as f32 * quality_factor * cpu_factor) as usize
  }

  pub fn validate_video_data(&self, video_processor: &VideoProcessor) -> Result<()> {
    let video_data = video_processor.data();
    let max_res = self.get_max_resolution();

    if video_data.width > max_res.0 || video_data.height > max_res.1 {
      return Err(EllasticError::InvalidParameter(format!(
        "Resolution {}x{} exceeds maximum {}x{}",
        video_data.width, video_data.height, max_res.0, max_res.1
      )));
    }

    if video_data.frame_rate > self.frame_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Frame rate {} exceeds maximum {}",
        video_data.frame_rate, self.frame_rate
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> AV1Codec {
    Self {
      quality: self.quality,
      bitrate: self.bitrate,
      frame_rate: self.frame_rate,
      resolution: self.resolution,
      threads: self.threads,
      tile_rows: self.tile_rows,
      tile_cols: self.tile_cols,
      cpu_used: self.cpu_used,
    }
  }
}

#[derive(Debug, Clone)]
pub struct MJPEGCodec {
  quality: u8,
  bitrate: u32,
  frame_rate: f64,
  resolution: (u32, u32),
}

impl MJPEGCodec {
  pub fn new(quality: u8, bitrate: u32, frame_rate: f64, resolution: (u32, u32)) -> Self {
    Self {
      quality,
      bitrate,
      frame_rate,
      resolution,
    }
  }

  pub fn quality(&self) -> u8 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: u8) {
    self.quality = quality.clamp(1, 100);
  }

  pub fn bitrate(&self) -> u32 {
    self.bitrate
  }

  pub fn set_bitrate(&mut self, bitrate: u32) {
    self.bitrate = bitrate;
  }

  pub fn frame_rate(&self) -> f64 {
    self.frame_rate
  }

  pub fn set_frame_rate(&mut self, frame_rate: f64) {
    self.frame_rate = frame_rate;
  }

  pub fn resolution(&self) -> (u32, u32) {
    self.resolution
  }

  pub fn set_resolution(&mut self, resolution: (u32, u32)) {
    self.resolution = resolution;
  }

  pub fn encode(&self, video_processor: &VideoProcessor) -> Result<Vec<u8>> {
    video_processor.encode(VideoFormat::AVI, Some(self.quality))
  }

  pub fn decode(&self, data: &[u8]) -> Result<VideoProcessor> {
    let video_data = decode_mjpeg(data)?;
    Ok(VideoProcessor::new_with_data(video_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    false
  }

  pub fn supports_interlaced_encoding(&self) -> bool {
    true
  }

  pub fn supports_variable_bitrate(&self) -> bool {
    false
  }

  pub fn supports_constant_bitrate(&self) -> bool {
    true
  }

  pub fn supports_constant_quality(&self) -> bool {
    true
  }

  pub fn get_max_resolution(&self) -> (u32, u32) {
    (4096, 4096)
  }

  pub fn get_max_bitrate(&self) -> u32 {
    50000000
  }

  pub fn estimate_output_size(&self, video_processor: &VideoProcessor) -> usize {
    let video_data = video_processor.data();
    let duration = video_data.duration_seconds();
    let estimated_size = (self.bitrate as f64 * duration / 8.0) as usize;

    let quality_factor = self.quality as f32 / 100.0;
    (estimated_size as f32 * quality_factor) as usize
  }

  pub fn validate_video_data(&self, video_processor: &VideoProcessor) -> Result<()> {
    let video_data = video_processor.data();
    let max_res = self.get_max_resolution();

    if video_data.width > max_res.0 || video_data.height > max_res.1 {
      return Err(EllasticError::InvalidParameter(format!(
        "Resolution {}x{} exceeds maximum {}x{}",
        video_data.width, video_data.height, max_res.0, max_res.1
      )));
    }

    if video_data.frame_rate > self.frame_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Frame rate {} exceeds maximum {}",
        video_data.frame_rate, self.frame_rate
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> MJPEGCodec {
    Self {
      quality: self.quality,
      bitrate: self.bitrate,
      frame_rate: self.frame_rate,
      resolution: self.resolution,
    }
  }
}

#[derive(Debug, Clone)]
pub struct VideoCodecRegistry {
  codecs: std::collections::HashMap<String, Box<dyn VideoCodec>>,
}

impl VideoCodecRegistry {
  pub fn new() -> Self {
    let mut registry = Self {
      codecs: std::collections::HashMap::new(),
    };

    registry.register_codec(
      "h264",
      Box::new(H264Codec::new(
        H264Profile::High,
        H264Level::Level4,
        5000000,
        30.0,
        (1920, 1080),
        23,
        H264Preset::Medium,
        H264Tune::None,
      )),
    );

    registry.register_codec(
      "vp9",
      Box::new(VP9Codec::new(
        31,
        5000000,
        30.0,
        (1920, 1080),
        4,
        2,
        true,
        0,
      )),
    );

    registry.register_codec(
      "av1",
      Box::new(AV1Codec::new(31, 5000000, 30.0, (1920, 1080), 4, 2, 2, 0)),
    );

    registry.register_codec(
      "mjpeg",
      Box::new(MJPEGCodec::new(85, 5000000, 30.0, (1920, 1080))),
    );

    registry
  }

  pub fn register_codec(&mut self, name: &str, codec: Box<dyn VideoCodec>) {
    self.codecs.insert(name.to_lowercase(), codec);
  }

  pub fn get_codec(&self, name: &str) -> Option<&dyn VideoCodec> {
    self
      .codecs
      .get(&name.to_lowercase())
      .map(|codec| codec.as_ref())
  }

  pub fn list_codecs(&self) -> Vec<String> {
    self.codecs.keys().cloned().collect()
  }

  pub fn get_supported_formats(&self) -> Vec<String> {
    vec!["mp4", "webm", "avi", "mov", "mkv"]
  }

  pub fn auto_detect_codec(&self, data: &[u8]) -> Option<String> {
    if data.len() < 12 {
      return None;
    }

    if data.starts_with(b"ftyp") {
      if data.contains(&[0x68, 0x76, 0x63, 0x31]) {
        Some("h264".to_string())
      } else if data.contains(&[0x61, 0x76, 0x30, 0x31]) {
        Some("av1".to_string())
      } else {
        Some("h264".to_string())
      }
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"AVI " {
      Some("mjpeg".to_string())
    } else if data.starts_with(b"OggS") {
      Some("vp9".to_string())
    } else {
      None
    }
  }

  pub fn create_encoder(
    &self,
    name: &str,
    options: &VideoEncodingOptions,
  ) -> Result<Box<dyn VideoCodec>> {
    match name.to_lowercase().as_str() {
      "h264" => Ok(Box::new(H264Codec::new(
        options.h264_profile.unwrap_or(H264Profile::High),
        options.h264_level.unwrap_or(H264Level::Level4),
        options.bitrate.unwrap_or(5000000),
        options.frame_rate.unwrap_or(30.0),
        options.resolution.unwrap_or((1920, 1080)),
        options.quality.unwrap_or(23),
        options.h264_preset.unwrap_or(H264Preset::Medium),
        options.h264_tune.unwrap_or(H264Tune::None),
      ))),
      "vp9" => Ok(Box::new(VP9Codec::new(
        options.quality.unwrap_or(31),
        options.bitrate.unwrap_or(5000000),
        options.frame_rate.unwrap_or(30.0),
        options.resolution.unwrap_or((1920, 1080)),
        options.threads.unwrap_or(4),
        options.vp9_tile_columns.unwrap_or(2),
        options.vp9_row_mt.unwrap_or(true),
        options.cpu_used.unwrap_or(0),
      ))),
      "av1" => Ok(Box::new(AV1Codec::new(
        options.quality.unwrap_or(31),
        options.bitrate.unwrap_or(5000000),
        options.frame_rate.unwrap_or(30.0),
        options.resolution.unwrap_or((1920, 1080)),
        options.threads.unwrap_or(4),
        options.av1_tile_rows.unwrap_or(2),
        options.av1_tile_cols.unwrap_or(2),
        options.cpu_used.unwrap_or(0),
      ))),
      "mjpeg" => Ok(Box::new(MJPEGCodec::new(
        options.quality.unwrap_or(85),
        options.bitrate.unwrap_or(5000000),
        options.frame_rate.unwrap_or(30.0),
        options.resolution.unwrap_or((1920, 1080)),
      ))),
      _ => Err(EllasticError::UnsupportedFormat(format!(
        "Unknown video codec: {}",
        name
      ))),
    }
  }

  pub fn create_decoder(&self, name: &str) -> Result<Box<dyn VideoCodec>> {
    match name.to_lowercase().as_str() {
      "h264" => Ok(Box::new(H264Codec::new(
        H264Profile::High,
        H264Level::Level4,
        5000000,
        30.0,
        (1920, 1080),
        23,
        H264Preset::Medium,
        H264Tune::None,
      ))),
      "vp9" => Ok(Box::new(VP9Codec::new(
        31,
        5000000,
        30.0,
        (1920, 1080),
        4,
        2,
        true,
        0,
      ))),
      "av1" => Ok(Box::new(AV1Codec::new(
        31,
        5000000,
        30.0,
        (1920, 1080),
        4,
        2,
        2,
        0,
      ))),
      "mjpeg" => Ok(Box::new(MJPEGCodec::new(85, 5000000, 30.0, (1920, 1080)))),
      _ => Err(EllasticError::UnsupportedFormat(format!(
        "Unknown video codec: {}",
        name
      ))),
    }
  }

  pub fn batch_encode(
    &self,
    videos: &[VideoProcessor],
    codec_name: &str,
    options: &VideoEncodingOptions,
  ) -> Result<Vec<Vec<u8>>> {
    let codec = self.create_encoder(codec_name, options)?;
    let mut results = Vec::new();

    for video in videos {
      let encoded = codec.encode(video)?;
      results.push(encoded);
    }

    Ok(results)
  }

  pub fn batch_decode(&self, data: &[u8], codec_name: &str) -> Result<Vec<VideoProcessor>> {
    let codec = self.create_decoder(codec_name)?;
    let mut videos = Vec::new();

    let video = codec.decode(data)?;
    videos.push(video);

    Ok(videos)
  }
}

pub trait VideoCodec {
  fn encode(&self, video_processor: &VideoProcessor) -> Result<Vec<u8>>;
  fn decode(&self, data: &[u8]) -> Result<VideoProcessor>;
  fn supports_progressive_encoding(&self) -> bool;
  fn supports_interlaced_encoding(&self) -> bool;
  fn supports_variable_bitrate(&self) -> bool;
  fn supports_constant_bitrate(&self) -> bool;
  fn supports_constant_quality(&self) -> bool;
  fn get_max_resolution(&self) -> (u32, u32);
  fn get_max_bitrate(&self) -> u32;
  fn estimate_output_size(&self, video_processor: &VideoProcessor) -> usize;
  fn validate_video_data(&self, video_processor: &VideoProcessor) -> Result<()>;
  fn clone_codec(&self) -> Box<dyn VideoCodec>;
}

impl dyn VideoCodec {
  fn clone_codec(&self) -> Box<dyn VideoCodec> {
    match self {
      codec => codec.clone_codec(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct VideoEncodingOptions {
  pub quality: Option<u8>,
  pub bitrate: Option<u32>,
  pub frame_rate: Option<f64>,
  pub resolution: Option<(u32, u32)>,
  pub threads: Option<u8>,
  pub h264_profile: Option<H264Profile>,
  pub h264_level: Option<H264Level>,
  pub h264_preset: Option<H264Preset>,
  pub h264_tune: Option<H264Tune>,
  pub vp9_tile_columns: Option<u8>,
  pub vp9_row_mt: Option<bool>,
  pub av1_tile_rows: Option<u8>,
  pub av1_tile_cols: Option<u8>,
  pub cpu_used: Option<i8>,
}

impl Default for VideoEncodingOptions {
  fn default() -> Self {
    Self {
      quality: None,
      bitrate: None,
      frame_rate: None,
      resolution: None,
      threads: None,
      h264_profile: None,
      h264_level: None,
      h264_preset: None,
      h264_tune: None,
      vp9_tile_columns: None,
      vp9_row_mt: None,
      av1_tile_rows: None,
      av1_tile_cols: None,
      cpu_used: None,
    }
  }
}

fn decode_h264(data: &[u8]) -> Result<VideoData> {
  Err(EllasticError::UnsupportedOperation(
    "H264 decoding not implemented".to_string(),
  ))
}

fn decode_vp9(data: &[u8]) -> Result<VideoData> {
  Err(EllasticError::UnsupportedOperation(
    "VP9 decoding not implemented".to_string(),
  ))
}

fn decode_av1(data: &[u8]) -> Result<VideoData> {
  Err(EllasticError::UnsupportedOperation(
    "AV1 decoding not implemented".to_string(),
  ))
}

fn decode_mjpeg(data: &[u8]) -> Result<VideoData> {
  Err(EllasticError::UnsupportedOperation(
    "MJPEG decoding not implemented".to_string(),
  ))
}

pub fn create_h264_codec(
  profile: H264Profile,
  level: H264Level,
  bitrate: u32,
  frame_rate: f64,
  resolution: (u32, u32),
  quality: u8,
  preset: H264Preset,
  tune: H264Tune,
) -> H264Codec {
  H264Codec::new(
    profile, level, bitrate, frame_rate, resolution, quality, preset, tune,
  )
}

pub fn create_vp9_codec(
  quality: u8,
  bitrate: u32,
  frame_rate: f64,
  resolution: (u32, u32),
  threads: u8,
  tile_columns: u8,
  row_mt: bool,
  cpu_used: i8,
) -> VP9Codec {
  VP9Codec::new(
    quality,
    bitrate,
    frame_rate,
    resolution,
    threads,
    tile_columns,
    row_mt,
    cpu_used,
  )
}

pub fn create_av1_codec(
  quality: u8,
  bitrate: u32,
  frame_rate: f64,
  resolution: (u32, u32),
  threads: u8,
  tile_rows: u8,
  tile_cols: u8,
  cpu_used: i8,
) -> AV1Codec {
  AV1Codec::new(
    quality, bitrate, frame_rate, resolution, threads, tile_rows, tile_cols, cpu_used,
  )
}

pub fn create_mjpeg_codec(
  quality: u8,
  bitrate: u32,
  frame_rate: f64,
  resolution: (u32, u32),
) -> MJPEGCodec {
  MJPEGCodec::new(quality, bitrate, frame_rate, resolution)
}

pub fn create_video_codec_registry() -> VideoCodecRegistry {
  VideoCodecRegistry::new()
}
