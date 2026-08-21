use crate::{
  AudioData,
  AudioFormat,
  AudioProcessor,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use hound::{
  SampleFormat,
  WavReader,
  WavSpec,
  WavWriter,
};
use std::io::{
  Cursor,
  Read,
  Write,
};

pub fn load_wav(data: &[u8]) -> Result<AudioData> {
  let cursor = Cursor::new(data);
  let mut reader = WavReader::new(cursor)
    .map_err(|e| EllasticError::IoError(format!("WAV decode error: {}", e)))?;

  let spec = reader.spec();
  let samples: Vec<f32> = reader
    .into_samples()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| EllasticError::IoError(format!("WAV sample conversion error: {}", e)))?;

  AudioData::new(spec.sample_rate, spec.channels as u8, samples)
}

pub fn save_wav(audio_data: &AudioData, _quality: Option<u8>) -> Result<Vec<u8>> {
  let spec = WavSpec {
    channels: audio_data.channels as u16,
    sample_rate: audio_data.sample_rate,
    bits_per_sample: 32,
    sample_format: SampleFormat::Float,
  };

  let mut buffer = Vec::new();
  {
    let cursor = Cursor::new(&mut buffer);
    let mut writer = WavWriter::new(cursor, spec)
      .map_err(|e| EllasticError::IoError(format!("WAV writer error: {}", e)))?;

    for &sample in &audio_data.samples {
      writer
        .write_sample(sample)
        .map_err(|e| EllasticError::IoError(format!("WAV write error: {}", e)))?;
    }

    writer
      .finalize()
      .map_err(|e| EllasticError::IoError(format!("WAV finalize error: {}", e)))?;
  }

  Ok(buffer)
}

pub fn load_mp3(data: &[u8]) -> Result<AudioData> {
  Err(EllasticError::UnsupportedFormat(
    "MP3 decoding not implemented".to_string(),
  ))
}

pub fn save_mp3(audio_data: &AudioData, quality: Option<u8>) -> Result<Vec<u8>> {
  let _ = (audio_data, quality);
  Err(EllasticError::UnsupportedFormat(
    "MP3 encoding not implemented".to_string(),
  ))
}

pub fn load_flac(data: &[u8]) -> Result<AudioData> {
  Err(EllasticError::UnsupportedFormat(
    "FLAC decoding not implemented".to_string(),
  ))
}

pub fn save_flac(audio_data: &AudioData, _quality: Option<u8>) -> Result<Vec<u8>> {
  let _ = audio_data;
  Err(EllasticError::UnsupportedFormat(
    "FLAC encoding not implemented".to_string(),
  ))
}

pub fn load_ogg(data: &[u8]) -> Result<AudioData> {
  Err(EllasticError::UnsupportedFormat(
    "OGG decoding not implemented".to_string(),
  ))
}

pub fn save_ogg(audio_data: &AudioData, quality: Option<u8>) -> Result<Vec<u8>> {
  let _ = (audio_data, quality);
  Err(EllasticError::UnsupportedFormat(
    "OGG encoding not implemented".to_string(),
  ))
}

pub fn load_aac(data: &[u8]) -> Result<AudioData> {
  Err(EllasticError::UnsupportedFormat(
    "AAC decoding not implemented".to_string(),
  ))
}

pub fn save_aac(audio_data: &AudioData, quality: Option<u8>) -> Result<Vec<u8>> {
  let _ = (audio_data, quality);
  Err(EllasticError::UnsupportedFormat(
    "AAC encoding not implemented".to_string(),
  ))
}

pub fn get_format_info(format: AudioFormat) -> FormatInfo {
  match format {
    AudioFormat::WAV => FormatInfo {
      name: "WAV",
      supports_lossless: true,
      supports_lossy: false,
      supports_metadata: true,
      supports_streaming: false,
      max_sample_rate: 192000,
      max_bit_depth: 32,
      max_channels: 65535,
    },
    AudioFormat::MP3 => FormatInfo {
      name: "MP3",
      supports_lossless: false,
      supports_lossy: true,
      supports_metadata: true,
      supports_streaming: true,
      max_sample_rate: 48000,
      max_bit_depth: 16,
      max_channels: 2,
    },
    AudioFormat::FLAC => FormatInfo {
      name: "FLAC",
      supports_lossless: true,
      supports_lossy: false,
      supports_metadata: true,
      supports_streaming: false,
      max_sample_rate: 655350,
      max_bit_depth: 24,
      max_channels: 8,
    },
    AudioFormat::OGG => FormatInfo {
      name: "OGG Vorbis",
      supports_lossless: false,
      supports_lossy: true,
      supports_metadata: true,
      supports_streaming: true,
      max_sample_rate: 192000,
      max_bit_depth: 24,
      max_channels: 256,
    },
    AudioFormat::AAC => FormatInfo {
      name: "AAC",
      supports_lossless: false,
      supports_lossy: true,
      supports_metadata: true,
      supports_streaming: true,
      max_sample_rate: 96000,
      max_bit_depth: 24,
      max_channels: 48,
    },
  }
}

pub fn get_supported_formats() -> Vec<AudioFormat> {
  vec![
    AudioFormat::WAV,
    AudioFormat::MP3,
    AudioFormat::FLAC,
    AudioFormat::OGG,
    AudioFormat::AAC,
  ]
}

pub fn get_lossless_formats() -> Vec<AudioFormat> {
  get_supported_formats()
    .into_iter()
    .filter(|format| get_format_info(*format).supports_lossless)
    .collect()
}

pub fn get_lossy_formats() -> Vec<AudioFormat> {
  get_supported_formats()
    .into_iter()
    .filter(|format| get_format_info(*format).supports_lossy)
    .collect()
}

pub fn get_streamable_formats() -> Vec<AudioFormat> {
  get_supported_formats()
    .into_iter()
    .filter(|format| get_format_info(*format).supports_streaming)
    .collect()
}

pub fn get_formats_with_metadata() -> Vec<AudioFormat> {
  get_supported_formats()
    .into_iter()
    .filter(|format| get_format_info(*format).supports_metadata)
    .collect()
}

pub fn detect_audio_properties(data: &[u8]) -> AudioProperties {
  let mut properties = AudioProperties::default();

  if data.len() < 12 {
    return properties;
  }

  if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WAVE" {
    properties.format = AudioFormat::WAV;

    if data.len() >= 44 {
      let channels = u16::from_le_bytes([data[22], data[23]]);
      let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
      let byte_rate = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
      let block_align = u16::from_le_bytes([data[32], data[33]]);
      let bits_per_sample = u16::from_le_bytes([data[34], data[35]]);

      properties.channels = channels as u8;
      properties.sample_rate = sample_rate;
      properties.bit_depth = bits_per_sample;
      properties.bitrate = byte_rate * 8 / 1000;
      properties.duration = if byte_rate > 0 {
        (data.len() as f64 - 44.0) / byte_rate as f64
      } else {
        0.0
      };
    }
  } else if data.starts_with(b"ID3") {
    properties.format = AudioFormat::MP3;

    if let Some(mp3_header) = find_mp3_header(data) {
      properties.sample_rate = mp3_header.sample_rate;
      properties.channels = mp3_header.channels;
      properties.bitrate = mp3_header.bitrate;

      if mp3_header.bitrate > 0 {
        properties.duration = (data.len() as f64 * 8.0) / (mp3_header.bitrate as f64 * 1000.0);
      }
    }
  } else if data.starts_with(b"fLaC") {
    properties.format = AudioFormat::FLAC;

    if let Some(flac_header) = parse_flac_header(data) {
      properties.sample_rate = flac_header.sample_rate;
      properties.channels = flac_header.channels;
      properties.bit_depth = flac_header.bit_depth;
    }
  } else if data.starts_with(b"OggS") {
    properties.format = AudioFormat::OGG;

    if let Some(ogg_header) = parse_ogg_header(data) {
      properties.sample_rate = ogg_header.sample_rate;
      properties.channels = ogg_header.channels;
      properties.bitrate = ogg_header.bitrate;
    }
  }

  properties
}

struct MP3Header {
  sample_rate: u32,
  channels: u8,
  bitrate: u32,
}

fn find_mp3_header(data: &[u8]) -> Option<MP3Header> {
  for i in 0..=data.len().saturating_sub(4) {
    if data[i] == 0xFF && (data[i + 1] & 0xE0) == 0xE0 {
      let header = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

      let version_bits = (header >> 19) & 0x3;
      let layer_bits = (header >> 17) & 0x3;
      let protection_bit = (header >> 16) & 0x1;
      let bitrate_index = (header >> 12) & 0xF;
      let sampling_index = (header >> 10) & 0x3;
      let padding_bit = (header >> 9) & 0x1;
      let channel_mode = (header >> 6) & 0x3;

      if layer_bits == 0 || bitrate_index == 0 || bitrate_index == 15 {
        continue;
      }

      let sample_rates = match version_bits {
        0 => vec![44100, 48000, 32000],
        2 => vec![22050, 24000, 16000],
        3 => vec![11025, 12000, 8000],
        _ => vec![44100, 48000, 32000],
      };

      let bitrates = match (version_bits, layer_bits) {
        (0, 1) | (2, 1) => vec![
          0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448,
        ],
        (0, 2) | (2, 2) => vec![
          0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384,
        ],
        (0, 3) | (2, 3) => vec![
          0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
        ],
        _ => vec![
          0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448,
        ],
      };

      let sample_rate = sample_rates
        .get(sampling_index as usize)
        .copied()
        .unwrap_or(44100);
      let bitrate = bitrates.get(bitrate_index as usize).copied().unwrap_or(128);
      let channels = if channel_mode == 3 { 1 } else { 2 };

      return Some(MP3Header {
        sample_rate,
        channels,
        bitrate,
      });
    }
  }

  None
}

struct FLACHeader {
  sample_rate: u32,
  channels: u8,
  bit_depth: u8,
}

fn parse_flac_header(data: &[u8]) -> Option<FLACHeader> {
  if data.len() < 42 || !data.starts_with(b"fLaC") {
    return None;
  }

  let stream_info = &data[8..42];
  if stream_info[0] != 0x80 {
    return None;
  }

  let sample_rate = u32::from_be_bytes([
    stream_info[10],
    stream_info[11],
    stream_info[12],
    stream_info[13],
  ]) >> 12;

  let channels = ((stream_info[12] >> 9) & 0x7) + 1;
  let bit_depth = ((stream_info[12] >> 4) & 0x1F) + 1;

  Some(FLACHeader {
    sample_rate,
    channels,
    bit_depth,
  })
}

struct OGGHeader {
  sample_rate: u32,
  channels: u8,
  bitrate: u32,
}

fn parse_ogg_header(data: &[u8]) -> Option<OGGHeader> {
  if data.len() < 100 || !data.starts_with(b"OggS") {
    return None;
  }

  for i in 14..=data.len().saturating_sub(20) {
    if data[i..i + 7] == b"\x01vorbis" {
      let channels = data[i + 11];
      let sample_rate =
        u32::from_le_bytes([data[i + 12], data[i + 13], data[i + 14], data[i + 15]]);

      let max_bitrate =
        u32::from_le_bytes([data[i + 16], data[i + 17], data[i + 18], data[i + 19]]);

      return Some(OGGHeader {
        sample_rate,
        channels,
        bitrate: max_bitrate / 1000,
      });
    }
  }

  None
}

#[derive(Debug, Clone, Default)]
pub struct AudioProperties {
  pub format: AudioFormat,
  pub channels: u8,
  pub sample_rate: u32,
  pub bit_depth: u16,
  pub bitrate: u32,
  pub duration: f64,
}

#[derive(Debug, Clone)]
pub struct FormatInfo {
  pub name: &'static str,
  pub supports_lossless: bool,
  pub supports_lossy: bool,
  pub supports_metadata: bool,
  pub supports_streaming: bool,
  pub max_sample_rate: u32,
  pub max_bit_depth: u16,
  pub max_channels: u16,
}

pub fn convert_format(
  audio_data: &AudioData,
  from_format: AudioFormat,
  to_format: AudioFormat,
) -> Result<Vec<u8>> {
  if from_format == to_format {
    return match from_format {
      AudioFormat::WAV => save_wav(audio_data, None),
      AudioFormat::MP3 => save_mp3(audio_data, None),
      AudioFormat::FLAC => save_flac(audio_data, None),
      AudioFormat::OGG => save_ogg(audio_data, None),
      AudioFormat::AAC => save_aac(audio_data, None),
    };
  }

  let decoded = match from_format {
    AudioFormat::WAV => load_wav(&std::fs::read("temp.wav")?),
    AudioFormat::MP3 => load_mp3(&std::fs::read("temp.mp3")?),
    AudioFormat::FLAC => load_flac(&std::fs::read("temp.flac")?),
    AudioFormat::OGG => load_ogg(&std::fs::read("temp.ogg")?),
    AudioFormat::AAC => load_aac(&std::fs::read("temp.aac")?),
  }?;

  match to_format {
    AudioFormat::WAV => save_wav(&decoded, None),
    AudioFormat::MP3 => save_mp3(&decoded, None),
    AudioFormat::FLAC => save_flac(&decoded, None),
    AudioFormat::OGG => save_ogg(&decoded, None),
    AudioFormat::AAC => save_aac(&decoded, None),
  }
}

pub fn get_audio_metadata(data: &[u8]) -> AudioMetadata {
  let mut metadata = AudioMetadata::default();

  let properties = detect_audio_properties(data);
  metadata.format = properties.format;
  metadata.duration = properties.duration;
  metadata.sample_rate = properties.sample_rate;
  metadata.channels = properties.channels;
  metadata.bitrate = properties.bitrate;

  if data.starts_with(b"ID3") {
    metadata = parse_id3_metadata(data, metadata);
  }

  metadata
}

#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
  pub format: AudioFormat,
  pub title: Option<String>,
  pub artist: Option<String>,
  pub album: Option<String>,
  pub year: Option<u32>,
  pub genre: Option<String>,
  pub track: Option<u32>,
  pub duration: f64,
  pub sample_rate: u32,
  pub channels: u8,
  pub bitrate: u32,
}

fn parse_id3_metadata(data: &[u8], mut metadata: AudioMetadata) -> AudioMetadata {
  if data.len() < 10 || !data.starts_with(b"ID3") {
    return metadata;
  }

  let version_major = data[3];
  let version_minor = data[4];
  let flags = data[5];
  let size = ((data[6] as u32) << 21)
    | ((data[7] as u32) << 14)
    | ((data[8] as u32) << 7)
    | (data[9] as u32);

  if size == 0 || data.len() < 10 + size as usize {
    return metadata;
  }

  let header_size = 10;
  if flags & 0x40 != 0 {
    header_size += 10;
  }

  let mut offset = header_size;
  let end = std::cmp::min(offset + size as usize, data.len());

  while offset < end - 10 {
    let frame_id = std::str::from_utf8(&data[offset..offset + 4]).unwrap_or("");
    let frame_size = ((data[offset + 4] as u32) << 24)
      | ((data[offset + 5] as u32) << 16)
      | ((data[offset + 6] as u32) << 8)
      | (data[offset + 7] as u32);
    let flags = data[offset + 8];

    if frame_size == 0 {
      break;
    }

    let data_start = offset + 10;
    let data_end = std::cmp::min(data_start + frame_size as usize, end);

    if data_start < data_end {
      let frame_data = &data[data_start..data_end];

      if frame_data.len() > 0 {
        let text_start = if frame_data[0] == 0 { 1 } else { 0 };
        let text = std::str::from_utf8(&frame_data[text_start..])
          .unwrap_or("")
          .trim_end_matches('\0');

        match frame_id {
          "TIT2" => metadata.title = Some(text.to_string()),
          "TPE1" => metadata.artist = Some(text.to_string()),
          "TALB" => metadata.album = Some(text.to_string()),
          "TDRC" => {
            if let Ok(year) = text.parse::<u32>() {
              metadata.year = Some(year);
            }
          }
          "TCON" => metadata.genre = Some(text.to_string()),
          "TRCK" => {
            if let Ok(track) = text.split('/').next().unwrap_or("0").parse::<u32>() {
              metadata.track = Some(track);
            }
          }
          _ => {}
        }
      }
    }

    offset = data_end;
    if version_major >= 4 && flags & 0x10 != 0 {
      offset += frame_size as usize;
    }
  }

  metadata
}
