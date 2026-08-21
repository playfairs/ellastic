use std::io::{
  Read,
  Write,
};
use std::path::Path;
use uuid::Uuid;

use crate::{
  MediaMetadata,
  MediaType,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};

#[derive(Debug, Clone)]
pub enum MediaData {
  Image(ImageData),
  Audio(AudioData),
  Binary(BinaryData),
}

#[derive(Debug, Clone)]
pub struct ImageData {
  pub width: u32,
  pub height: u32,
  pub channels: u8,
  pub data: Vec<u8>,
}

impl ImageData {
  pub fn new(width: u32, height: u32, channels: u8, data: Vec<u8>) -> Result<Self> {
    let expected_size = (width * height * channels as u32) as usize;
    if data.len() != expected_size {
      return Err(EllasticError::InvalidImageData {
        expected: expected_size,
        actual: data.len(),
      });
    }

    Ok(Self {
      width,
      height,
      channels,
      data,
    })
  }

  pub fn pixel_count(&self) -> usize {
    (self.width * self.height) as usize
  }

  pub fn byte_size(&self) -> usize {
    self.data.len()
  }

  pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
    if x >= self.width || y >= self.height {
      return None;
    }

    let offset = ((y * self.width + x) * self.channels as u32) as usize;
    let end = offset + self.channels as usize;
    self.data.get(offset..end)
  }

  pub fn set_pixel(&mut self, x: u32, y: u32, pixel: &[u8]) -> Result<()> {
    if x >= self.width || y >= self.height {
      return Err(EllasticError::CoordinatesOutOfBounds { x, y });
    }

    if pixel.len() != self.channels as usize {
      return Err(EllasticError::InvalidPixelSize {
        expected: self.channels as usize,
        actual: pixel.len(),
      });
    }

    let offset = ((y * self.width + x) * self.channels as u32) as usize;
    let end = offset + self.channels as usize;

    if let Some(target) = self.data.get_mut(offset..end) {
      target.copy_from_slice(pixel);
      Ok(())
    } else {
      Err(EllasticError::InvalidImageData {
        expected: end,
        actual: self.data.len(),
      })
    }
  }
}

#[derive(Debug, Clone)]
pub struct AudioData {
  pub sample_rate: u32,
  pub channels: u16,
  pub bits_per_sample: u16,
  pub data: Vec<u8>,
}

impl AudioData {
  pub fn new(sample_rate: u32, channels: u16, bits_per_sample: u16, data: Vec<u8>) -> Self {
    Self {
      sample_rate,
      channels,
      bits_per_sample,
      data,
    }
  }

  pub fn sample_count(&self) -> usize {
    let bytes_per_sample = (self.bits_per_sample / 8) as usize;
    self.data.len() / (bytes_per_sample * self.channels as usize)
  }

  pub fn duration_seconds(&self) -> f64 {
    self.sample_count() as f64 / self.sample_rate as f64
  }

  pub fn get_sample(&self, index: usize) -> Option<&[u8]> {
    let bytes_per_sample = (self.bits_per_sample / 8) as usize;
    let offset = index * bytes_per_sample * self.channels as usize;
    let end = offset + bytes_per_sample * self.channels as usize;
    self.data.get(offset..end)
  }

  pub fn set_sample(&mut self, index: usize, sample: &[u8]) -> Result<()> {
    let bytes_per_sample = (self.bits_per_sample / 8) as usize;
    let offset = index * bytes_per_sample * self.channels as usize;
    let end = offset + bytes_per_sample * self.channels as usize;

    if sample.len() != bytes_per_sample * self.channels as usize {
      return Err(EllasticError::InvalidSampleSize {
        expected: bytes_per_sample * self.channels as usize,
        actual: sample.len(),
      });
    }

    if let Some(target) = self.data.get_mut(offset..end) {
      target.copy_from_slice(sample);
      Ok(())
    } else {
      Err(EllasticError::InvalidAudioData {
        expected: end,
        actual: self.data.len(),
      })
    }
  }
}

#[derive(Debug, Clone)]
pub struct BinaryData {
  pub data: Vec<u8>,
}

impl BinaryData {
  pub fn new(data: Vec<u8>) -> Self {
    Self { data }
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn read(&self, offset: usize, length: usize) -> Option<&[u8]> {
    let end = offset.checked_add(length)?;
    self.data.get(offset..end)
  }

  pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
    let end = offset
      .checked_add(data.len())
      .ok_or_else(|| EllasticError::OffsetOverflow(offset))?;

    if end > self.data.len() {
      return Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.data.len(),
      });
    }

    if let Some(target) = self.data.get_mut(offset..end) {
      target.copy_from_slice(data);
      Ok(())
    } else {
      Err(EllasticError::InvalidBinaryData {
        expected: end,
        actual: self.data.len(),
      })
    }
  }
}

#[derive(Debug)]
pub struct MediaBuffer {
  pub metadata: MediaMetadata,
  pub data: MediaData,
}

impl MediaBuffer {
  pub fn new(metadata: MediaMetadata, data: MediaData) -> Self {
    Self { metadata, data }
  }

  pub fn id(&self) -> Uuid {
    self.metadata.id
  }

  pub fn media_type(&self) -> MediaType {
    self.metadata.media_type
  }

  pub fn size(&self) -> usize {
    match &self.data {
      MediaData::Image(img) => img.byte_size(),
      MediaData::Audio(audio) => audio.data.len(),
      MediaData::Binary(binary) => binary.len(),
    }
  }

  pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    let data = match &self.data {
      MediaData::Image(img) => img.data.clone(),
      MediaData::Audio(audio) => audio.data.clone(),
      MediaData::Binary(binary) => binary.data.clone(),
    };

    std::fs::write(path, data).map_err(|e| EllasticError::IoError(e.to_string()))?;

    Ok(())
  }

  pub fn load_from_file<P: AsRef<Path>>(path: P, metadata: MediaMetadata) -> Result<Self> {
    let data = std::fs::read(path).map_err(|e| EllasticError::IoError(e.to_string()))?;

    let media_data = match metadata.media_type {
      MediaType::Image => MediaData::Binary(BinaryData::new(data)),
      MediaType::Audio => MediaData::Binary(BinaryData::new(data)),
      MediaType::Video => MediaData::Binary(BinaryData::new(data)),
      MediaType::Binary => MediaData::Binary(BinaryData::new(data)),
    };

    Ok(Self {
      metadata,
      data: media_data,
    })
  }
}
