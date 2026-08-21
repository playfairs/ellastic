use ellastic_audio::{
  AudioFormat,
  AudioProcessor,
};
use ellastic_core::AudioData;
use ellastic_errors::{
  EllasticError,
  Result,
};

#[derive(Debug, Clone)]
pub struct WAVCodec {
  bit_depth: u16,
  sample_rate: u32,
  channels: u16,
}

impl WAVCodec {
  pub fn new(bit_depth: u16, sample_rate: u32, channels: u16) -> Self {
    Self {
      bit_depth,
      sample_rate,
      channels,
    }
  }

  pub fn bit_depth(&self) -> u16 {
    self.bit_depth
  }

  pub fn set_bit_depth(&mut self, bit_depth: u16) {
    self.bit_depth = bit_depth;
  }

  pub fn sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub fn set_sample_rate(&mut self, sample_rate: u32) {
    self.sample_rate = sample_rate;
  }

  pub fn channels(&self) -> u16 {
    self.channels
  }

  pub fn set_channels(&mut self, channels: u16) {
    self.channels = channels;
  }

  pub fn encode(&self, audio_processor: &AudioProcessor) -> Result<Vec<u8>> {
    audio_processor.encode(AudioFormat::WAV, None)
  }

  pub fn decode(&self, data: &[u8]) -> Result<AudioProcessor> {
    let audio_data = ellastic_audio::decode_audio(data, AudioFormat::WAV)?;
    Ok(AudioProcessor::from_audio_data(audio_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    false
  }

  pub fn supports_metadata(&self) -> bool {
    true
  }

  pub fn get_format_info(&self) -> AudioFormatInfo {
    AudioFormatInfo {
      name: "WAV".to_string(),
      version: "1.0".to_string(),
      description: "Waveform Audio File Format".to_string(),
      mime_type: "audio/wav".to_string(),
      extensions: vec!["wav".to_string()],
      supports_lossless: true,
      supports_lossy: false,
      max_bit_depth: Some(32),
      max_sample_rate: Some(192000),
      max_channels: Some(65535),
      supports_metadata: true,
      supports_streaming: false,
      supports_gapless: true,
    }
  }

  pub fn estimate_output_size(&self, audio_processor: &AudioProcessor) -> usize {
    let audio_data = audio_processor.data();
    let samples = audio_data.samples.len();
    let bytes_per_sample = (self.bit_depth / 8) as usize * self.channels as usize;
    let header_size = 44;
    let data_size = samples * bytes_per_sample;
    let total_size = header_size + data_size;

    if total_size % 2 != 0 {
      total_size + 1
    } else {
      total_size
    }
  }

  pub fn validate_audio_data(&self, audio_processor: &AudioProcessor) -> Result<()> {
    let audio_data = audio_processor.data();

    if audio_data.sample_rate > self.sample_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Sample rate {} exceeds maximum {}",
        audio_data.sample_rate, self.sample_rate
      )));
    }

    if audio_data.channels > self.channels as u8 {
      return Err(EllasticError::InvalidParameter(format!(
        "Channel count {} exceeds maximum {}",
        audio_data.channels, self.channels
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> WAVCodec {
    Self {
      bit_depth: self.bit_depth,
      sample_rate: self.sample_rate,
      channels: self.channels,
    }
  }
}

#[derive(Debug, Clone)]
pub struct MP3Codec {
  bit_rate: u32,
  sample_rate: u32,
  channels: u8,
  quality: u8,
  vbr: bool,
}

impl MP3Codec {
  pub fn new(bit_rate: u32, sample_rate: u32, channels: u8, quality: u8, vbr: bool) -> Self {
    Self {
      bit_rate,
      sample_rate,
      channels,
      quality,
      vbr,
    }
  }

  pub fn bit_rate(&self) -> u32 {
    self.bit_rate
  }

  pub fn set_bit_rate(&mut self, bit_rate: u32) {
    self.bit_rate = bit_rate;
  }

  pub fn sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub fn set_sample_rate(&mut self, sample_rate: u32) {
    self.sample_rate = sample_rate;
  }

  pub fn channels(&self) -> u8 {
    self.channels
  }

  pub fn set_channels(&mut self, channels: u8) {
    self.channels = channels;
  }

  pub fn quality(&self) -> u8 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: u8) {
    self.quality = quality.clamp(0, 9);
  }

  pub fn vbr(&self) -> bool {
    self.vbr
  }

  pub fn set_vbr(&mut self, vbr: bool) {
    self.vbr = vbr;
  }

  pub fn encode(&self, audio_processor: &AudioProcessor) -> Result<Vec<u8>> {
    audio_processor.encode(AudioFormat::MP3, Some(self.quality))
  }

  pub fn decode(&self, data: &[u8]) -> Result<AudioProcessor> {
    let audio_data = ellastic_audio::decode_audio(data, AudioFormat::MP3)?;
    Ok(AudioProcessor::from_audio_data(audio_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    true
  }

  pub fn supports_metadata(&self) -> bool {
    true
  }

  pub fn get_format_info(&self) -> AudioFormatInfo {
    AudioFormatInfo {
      name: "MP3".to_string(),
      version: "3.0".to_string(),
      description: "MPEG Audio Layer 3".to_string(),
      mime_type: "audio/mpeg".to_string(),
      extensions: vec!["mp3".to_string()],
      supports_lossless: false,
      supports_lossy: true,
      max_bit_depth: Some(16),
      max_sample_rate: Some(48000),
      max_channels: Some(2),
      supports_metadata: true,
      supports_streaming: true,
      supports_gapless: false,
    }
  }

  pub fn estimate_output_size(&self, audio_processor: &AudioProcessor) -> usize {
    let audio_data = audio_processor.data();
    let duration = audio_data.duration_seconds();
    let estimated_size = (self.bit_rate as f64 * duration / 8.0) as usize;

    estimated_size + 1024
  }

  pub fn validate_audio_data(&self, audio_processor: &AudioProcessor) -> Result<()> {
    let audio_data = audio_processor.data();

    if audio_data.sample_rate > self.sample_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Sample rate {} exceeds maximum {}",
        audio_data.sample_rate, self.sample_rate
      )));
    }

    if audio_data.channels > self.channels {
      return Err(EllasticError::InvalidParameter(format!(
        "Channel count {} exceeds maximum {}",
        audio_data.channels, self.channels
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> MP3Codec {
    Self {
      bit_rate: self.bit_rate,
      sample_rate: self.sample_rate,
      channels: self.channels,
      quality: self.quality,
      vbr: self.vbr,
    }
  }
}

#[derive(Debug, Clone)]
pub struct FLACCodec {
  compression_level: u8,
  sample_rate: u32,
  channels: u8,
  bits_per_sample: u8,
}

impl FLACCodec {
  pub fn new(compression_level: u8, sample_rate: u32, channels: u8, bits_per_sample: u8) -> Self {
    Self {
      compression_level,
      sample_rate,
      channels,
      bits_per_sample,
    }
  }

  pub fn compression_level(&self) -> u8 {
    self.compression_level
  }

  pub fn set_compression_level(&mut self, level: u8) {
    self.compression_level = level.clamp(0, 8);
  }

  pub fn sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub fn set_sample_rate(&mut self, sample_rate: u32) {
    self.sample_rate = sample_rate;
  }

  pub fn channels(&self) -> u8 {
    self.channels
  }

  pub fn set_channels(&mut self, channels: u8) {
    self.channels = channels;
  }

  pub fn bits_per_sample(&self) -> u8 {
    self.bits_per_sample
  }

  pub fn set_bits_per_sample(&mut self, bits_per_sample: u8) {
    self.bits_per_sample = bits_per_sample.clamp(8, 24);
  }

  pub fn encode(&self, audio_processor: &AudioProcessor) -> Result<Vec<u8>> {
    audio_processor.encode(AudioFormat::FLAC, Some(self.compression_level))
  }

  pub fn decode(&self, data: &[u8]) -> Result<AudioProcessor> {
    let audio_data = ellastic_audio::decode_audio(data, AudioFormat::FLAC)?;
    Ok(AudioProcessor::from_audio_data(audio_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    false
  }

  pub fn supports_metadata(&self) -> bool {
    true
  }

  pub fn get_format_info(&self) -> AudioFormatInfo {
    AudioFormatInfo {
      name: "FLAC".to_string(),
      version: "1.3.0".to_string(),
      description: "Free Lossless Audio Codec".to_string(),
      mime_type: "audio/flac".to_string(),
      extensions: vec!["flac".to_string()],
      supports_lossless: true,
      supports_lossy: false,
      max_bit_depth: Some(24),
      max_sample_rate: Some(655350),
      max_channels: Some(8),
      supports_metadata: true,
      supports_streaming: false,
      supports_gapless: true,
    }
  }

  pub fn estimate_output_size(&self, audio_processor: &AudioProcessor) -> usize {
    let audio_data = audio_processor.data();
    let samples = audio_data.samples.len();
    let bytes_per_sample = (self.bits_per_sample / 8) as usize * self.channels as usize;
    let estimated_size = samples * bytes_per_sample;

    let compression_ratio = match self.compression_level {
      0 => 1.0,
      1 => 0.8,
      2 => 0.7,
      3 => 0.6,
      4 => 0.5,
      5 => 0.4,
      6 => 0.35,
      7 => 0.3,
      8 => 0.25,
      _ => 0.5,
    };

    (estimated_size as f64 * compression_ratio) as usize
  }

  pub fn validate_audio_data(&self, audio_processor: &AudioProcessor) -> Result<()> {
    let audio_data = audio_processor.data();

    if audio_data.sample_rate > self.sample_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Sample rate {} exceeds maximum {}",
        audio_data.sample_rate, self.sample_rate
      )));
    }

    if audio_data.channels > self.channels {
      return Err(EllasticError::InvalidParameter(format!(
        "Channel count {} exceeds maximum {}",
        audio_data.channels, self.channels
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> FLACCodec {
    Self {
      compression_level: self.compression_level,
      sample_rate: self.sample_rate,
      channels: self.channels,
      bits_per_sample: self.bits_per_sample,
    }
  }
}

#[derive(Debug, Clone)]
pub struct OGGCodec {
  quality: f32,
  sample_rate: u32,
  channels: u8,
}

impl OGGCodec {
  pub fn new(quality: f32, sample_rate: u32, channels: u8) -> Self {
    Self {
      quality,
      sample_rate,
      channels,
    }
  }

  pub fn quality(&self) -> f32 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: f32) {
    self.quality = quality.clamp(-1.0, 10.0);
  }

  pub fn sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub fn set_sample_rate(&mut self, sample_rate: u32) {
    self.sample_rate = sample_rate;
  }

  pub fn channels(&self) -> u8 {
    self.channels
  }

  pub fn set_channels(&mut self, channels: u8) {
    self.channels = channels;
  }

  pub fn encode(&self, audio_processor: &AudioProcessor) -> Result<Vec<u8>> {
    let quality_level = ((self.quality + 1.0) * 10.0) as u8;
    audio_processor.encode(AudioFormat::OGG, Some(quality_level))
  }

  pub fn decode(&self, data: &[u8]) -> Result<AudioProcessor> {
    let audio_data = ellastic_audio::decode_audio(data, AudioFormat::OGG)?;
    Ok(AudioProcessor::from_audio_data(audio_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    true
  }

  pub fn supports_metadata(&self) -> bool {
    true
  }

  pub fn get_format_info(&self) -> AudioFormatInfo {
    AudioFormatInfo {
      name: "OGG Vorbis".to_string(),
      version: "1.0".to_string(),
      description: "Ogg Vorbis Audio Codec".to_string(),
      mime_type: "audio/ogg".to_string(),
      extensions: vec!["ogg".to_string()],
      supports_lossless: false,
      supports_lossy: true,
      max_bit_depth: Some(24),
      max_sample_rate: Some(192000),
      max_channels: Some(255),
      supports_metadata: true,
      supports_streaming: true,
      supports_gapless: false,
    }
  }

  pub fn estimate_output_size(&self, audio_processor: &AudioProcessor) -> usize {
    let audio_data = audio_processor.data();
    let duration = audio_data.duration_seconds();
    let bitrate = match self.quality {
      q if q <= -0.5 => 192000,
      q if q <= 0.0 => 160000,
      q if q <= 0.5 => 128000,
      q if q <= 1.0 => 96000,
      q if q <= 2.0 => 80000,
      q if q <= 3.0 => 64000,
      q if q <= 4.0 => 48000,
      q if q <= 5.0 => 32000,
      q if q <= 6.0 => 24000,
      _ => 16000,
    };

    (bitrate as f64 * duration / 8.0) as usize
  }

  pub fn validate_audio_data(&self, audio_processor: &AudioProcessor) -> Result<()> {
    let audio_data = audio_processor.data();

    if audio_data.sample_rate > self.sample_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Sample rate {} exceeds maximum {}",
        audio_data.sample_rate, self.sample_rate
      )));
    }

    if audio_data.channels > self.channels {
      return Err(EllasticError::InvalidParameter(format!(
        "Channel count {} exceeds maximum {}",
        audio_data.channels, self.channels
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> OGGCodec {
    Self {
      quality: self.quality,
      sample_rate: self.sample_rate,
      channels: self.channels,
    }
  }
}

#[derive(Debug, Clone)]
pub struct AACCodec {
  bit_rate: u32,
  sample_rate: u32,
  channels: u8,
  profile: AACProfile,
  quality: u8,
}

impl AACCodec {
  pub fn new(
    bit_rate: u32,
    sample_rate: u32,
    channels: u8,
    profile: AACProfile,
    quality: u8,
  ) -> Self {
    Self {
      bit_rate,
      sample_rate,
      channels,
      profile,
      quality,
    }
  }

  pub fn bit_rate(&self) -> u32 {
    self.bit_rate
  }

  pub fn set_bit_rate(&mut self, bit_rate: u32) {
    self.bit_rate = bit_rate;
  }

  pub fn sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub fn set_sample_rate(&mut self, sample_rate: u32) {
    self.sample_rate = sample_rate;
  }

  pub fn channels(&self) -> u8 {
    self.channels
  }

  pub fn set_channels(&mut self, channels: u8) {
    self.channels = channels;
  }

  pub fn profile(&self) -> AACProfile {
    self.profile
  }

  pub fn set_profile(&mut self, profile: AACProfile) {
    self.profile = profile;
  }

  pub fn quality(&self) -> u8 {
    self.quality
  }

  pub fn set_quality(&mut self, quality: u8) {
    self.quality = quality.clamp(0, 9);
  }

  pub fn encode(&self, audio_processor: &AudioProcessor) -> Result<Vec<u8>> {
    audio_processor.encode(AudioFormat::AAC, Some(self.quality))
  }

  pub fn decode(&self, data: &[u8]) -> Result<AudioProcessor> {
    let audio_data = ellastic_audio::decode_audio(data, AudioFormat::AAC)?;
    Ok(AudioProcessor::from_audio_data(audio_data))
  }

  pub fn supports_progressive_encoding(&self) -> bool {
    true
  }

  pub fn supports_metadata(&self) -> bool {
    true
  }

  pub fn get_format_info(&self) -> AudioFormatInfo {
    AudioFormatInfo {
      name: "AAC".to_string(),
      version: "2.0".to_string(),
      description: "Advanced Audio Coding".to_string(),
      mime_type: "audio/aac".to_string(),
      extensions: vec!["aac".to_string(), "m4a".to_string()],
      supports_lossless: false,
      supports_lossy: true,
      max_bit_depth: Some(24),
      max_sample_rate: Some(96000),
      max_channels: Some(48),
      supports_metadata: true,
      supports_streaming: true,
      supports_gapless: false,
    }
  }

  pub fn estimate_output_size(&self, audio_processor: &AudioProcessor) -> usize {
    let audio_data = audio_processor.data();
    let duration = audio_data.duration_seconds();

    (self.bit_rate as f64 * duration / 8.0) as usize
  }

  pub fn validate_audio_data(&self, audio_processor: &AudioProcessor) -> Result<()> {
    let audio_data = audio_processor.data();

    if audio_data.sample_rate > self.sample_rate {
      return Err(EllasticError::InvalidParameter(format!(
        "Sample rate {} exceeds maximum {}",
        audio_data.sample_rate, self.sample_rate
      )));
    }

    if audio_data.channels > self.channels {
      return Err(EllasticError::InvalidParameter(format!(
        "Channel count {} exceeds maximum {}",
        audio_data.channels, self.channels
      )));
    }

    Ok(())
  }

  pub fn clone(&self) -> AACCodec {
    Self {
      bit_rate: self.bit_rate,
      sample_rate: self.sample_rate,
      channels: self.channels,
      profile: self.profile,
      quality: self.quality,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AACProfile {
  Main,
  LowComplexity,
  ScalableSamplingRate,
  HighEfficiency,
}

#[derive(Debug, Clone)]
pub struct AudioFormatInfo {
  pub name: String,
  pub version: String,
  pub description: String,
  pub mime_type: String,
  pub extensions: Vec<String>,
  pub supports_lossless: bool,
  pub supports_lossy: bool,
  pub max_bit_depth: Option<u8>,
  pub max_sample_rate: Option<u32>,
  pub max_channels: Option<u8>,
  pub supports_metadata: bool,
  pub supports_streaming: bool,
  pub supports_gapless: bool,
}

#[derive(Debug, Clone)]
pub struct AudioCodecRegistry {
  codecs: std::collections::HashMap<String, Box<dyn AudioCodec>>,
}

impl AudioCodecRegistry {
  pub fn new() -> Self {
    let mut registry = Self {
      codecs: std::collections::HashMap::new(),
    };

    registry.register_codec("wav", Box::new(WAVCodec::new(16, 44100, 2)));
    registry.register_codec("mp3", Box::new(MP3Codec::new(128000, 44100, 2, 5, false)));
    registry.register_codec("flac", Box::new(FLACCodec::new(5, 44100, 2, 16)));
    registry.register_codec("ogg", Box::new(OGGCodec::new(0.0, 44100, 2)));
    registry.register_codec(
      "aac",
      Box::new(AACCodec::new(128000, 44100, 2, AACProfile::Main, 5)),
    );

    registry
  }

  pub fn register_codec(&mut self, name: &str, codec: Box<dyn AudioCodec>) {
    self.codecs.insert(name.to_lowercase(), codec);
  }

  pub fn get_codec(&self, name: &str) -> Option<&dyn AudioCodec> {
    self
      .codecs
      .get(&name.to_lowercase())
      .map(|codec| codec.as_ref())
  }

  pub fn list_codecs(&self) -> Vec<String> {
    self.codecs.keys().cloned().collect()
  }

  pub fn get_supported_formats(&self) -> Vec<String> {
    self
      .codecs
      .values()
      .map(|codec| codec.get_format_info().name)
      .collect()
  }

  pub fn auto_detect_codec(&self, data: &[u8]) -> Option<String> {
    if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WAVE" {
      Some("wav".to_string())
    } else if data.starts_with(b"ID3") || (data[0] == 0xFF && data[1] == 0xFB) {
      Some("mp3".to_string())
    } else if data.starts_with(b"fLaC") {
      Some("flac".to_string())
    } else if data.starts_with(b"OggS") {
      Some("ogg".to_string())
    } else if data.starts_with(&[0xFF, 0xF1, 0x50, 0x80]) {
      Some("aac".to_string())
    } else {
      None
    }
  }

  pub fn batch_encode(
    &self,
    audio_processors: &[AudioProcessor],
    codec_name: &str,
  ) -> Result<Vec<Vec<u8>>> {
    if let Some(codec) = self.get_codec(codec_name) {
      let mut results = Vec::new();

      for audio_processor in audio_processors {
        let encoded = codec.encode(audio_processor)?;
        results.push(encoded);
      }

      Ok(results)
    } else {
      Err(EllasticError::UnsupportedFormat(format!(
        "Unknown codec: {}",
        codec_name
      )))
    }
  }

  pub fn batch_decode(&self, data: &[u8], codec_name: &str) -> Result<Vec<AudioProcessor>> {
    if let Some(codec) = self.get_codec(codec_name) {
      let mut results = Vec::new();

      if codec_name == "gif" {
        let mut offset = 0;
        while offset < data.len() {
          if let Ok(audio_processor) = codec.decode(&data[offset..]) {
            results.push(audio_processor);
            offset += audio_processor.data().byte_size();
          } else {
            break;
          }
        }
      } else {
        let audio_processor = codec.decode(data)?;
        results.push(audio_processor);
      }

      Ok(results)
    } else {
      Err(EllasticError::UnsupportedFormat(format!(
        "Unknown codec: {}",
        codec_name
      )))
    }
  }

  pub fn create_transcoder(&self, from_codec: &str, to_codec: &str) -> Result<AudioTranscoder> {
    if let (Some(from), Some(to)) = (self.get_codec(from_codec), self.get_codec(to_codec)) {
      Ok(AudioTranscoder::new(from, to))
    } else {
      Err(EllasticError::UnsupportedFormat(
        "Codec not found".to_string(),
      ))
    }
  }
}

pub trait AudioCodec {
  fn encode(&self, audio_processor: &AudioProcessor) -> Result<Vec<u8>>;
  fn decode(&self, data: &[u8]) -> Result<AudioProcessor>;
  fn supports_progressive_encoding(&self) -> bool;
  fn supports_metadata(&self) -> bool;
  fn get_format_info(&self) -> AudioFormatInfo;
  fn clone_codec(&self) -> Box<dyn AudioCodec>;
}

impl dyn AudioCodec {
  fn clone_codec(&self) -> Box<dyn AudioCodec> {
    match self {
      codec => codec.clone_codec(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct AudioTranscoder {
  from_codec: Box<dyn AudioCodec>,
  to_codec: Box<dyn AudioCodec>,
}

impl AudioTranscoder {
  pub fn new(from_codec: Box<dyn AudioCodec>, to_codec: Box<dyn AudioCodec>) -> Self {
    Self {
      from_codec,
      to_codec,
    }
  }

  pub fn transcode(&self, audio_processor: &AudioProcessor) -> Result<Vec<u8>> {
    let decoded = self
      .from_codec
      .decode(&self.from_codec.encode(audio_processor)?)?;
    self.to_codec.encode(decoded)
  }

  pub fn transcode_with_options(
    &self,
    audio_processor: &AudioProcessor,
    options: &TranscodeOptions,
  ) -> Result<Vec<u8>> {
    let decoded = self
      .from_codec
      .decode(&self.from_codec.encode(audio_processor)?)?;
    let temp_processor = AudioProcessor::from_audio_data(decoded);

    if let Some(quality) = options.quality {
      match self.to_codec.get_format_info().name.as_str() {
        "mp3" => {
          if let Some(mp3_codec) = self.to_codec.as_any().downcast_ref::<MP3Codec>() {
            mp3_codec.set_quality(quality);
          }
        }
        "flac" => {
          if let Some(flac_codec) = self.to_codec.as_any().downcast_ref::<FLACCodec>() {
            flac_codec.set_compression_level(quality);
          }
        }
        "ogg" => {
          if let Some(ogg_codec) = self.to_codec.as_any().downcast_ref::<OGGCodec>() {
            let quality_level = (quality as f32 / 10.0) - 1.0;
            ogg_codec.set_quality(quality_level);
          }
        }
        "aac" => {
          if let Some(aac_codec) = self.to_codec.as_any().downcast_ref::<AACCodec>() {
            aac_codec.set_quality(quality);
          }
        }
        _ => {}
      }
    }

    self.to_codec.encode(temp_processor)
  }

  pub fn get_from_codec(&self) -> &dyn AudioCodec {
    self.from_codec.as_ref()
  }

  pub fn get_to_codec(&self) -> &dyn AudioCodec {
    self.to_codec.as_ref()
  }

  pub fn clone(&self) -> AudioTranscoder {
    AudioTranscoder::new(self.from_codec.clone_codec(), self.to_codec.clone_codec())
  }
}

#[derive(Debug, Clone)]
pub struct TranscodeOptions {
  pub quality: Option<u8>,
  pub bit_rate: Option<u32>,
  pub sample_rate: Option<u32>,
  pub channels: Option<u8>,
}

impl Default for TranscodeOptions {
  fn default() -> Self {
    Self {
      quality: None,
      bit_rate: None,
      sample_rate: None,
      channels: None,
    }
  }
}

pub fn create_wav_codec(bit_depth: u16, sample_rate: u32, channels: u16) -> WAVCodec {
  WAVCodec::new(bit_depth, sample_rate, channels)
}

pub fn create_mp3_codec(
  bit_rate: u32,
  sample_rate: u32,
  channels: u8,
  quality: u8,
  vbr: bool,
) -> MP3Codec {
  MP3Codec::new(bit_rate, sample_rate, channels, quality, vbr)
}

pub fn create_flac_codec(
  compression_level: u8,
  sample_rate: u32,
  channels: u8,
  bits_per_sample: u8,
) -> FLACCodec {
  FLACCodec::new(compression_level, sample_rate, channels, bits_per_sample)
}

pub fn create_ogg_codec(quality: f32, sample_rate: u32, channels: u8) -> OGGCodec {
  OGGCodec::new(quality, sample_rate, channels)
}

pub fn create_aac_codec(
  bit_rate: u32,
  sample_rate: u32,
  channels: u8,
  profile: AACProfile,
  quality: u8,
) -> AACCodec {
  AACCodec::new(bit_rate, sample_rate, channels, profile, quality)
}

pub fn create_audio_codec_registry() -> AudioCodecRegistry {
  AudioCodecRegistry::new()
}

pub fn create_audio_transcoder(from_codec: &str, to_codec: &str) -> Result<AudioTranscoder> {
  let registry = create_audio_codec_registry();
  registry.create_transcoder(from_codec, to_codec)
}
