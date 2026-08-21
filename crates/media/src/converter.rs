use crate::{
  MediaData,
  MediaProcessor,
  MediaType,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};
use ellastic_core::{
  AudioData,
  ImageData,
  VideoData,
};

#[derive(Debug, Clone)]
pub struct MediaConverter {
  source: MediaProcessor,
}

impl MediaConverter {
  pub fn new(source: MediaProcessor) -> Self {
    Self { source }
  }

  pub fn source(&self) -> &MediaProcessor {
    &self.source
  }

  pub fn source_mut(&mut self) -> &mut MediaProcessor {
    &mut self.source
  }

  pub fn into_source(self) -> MediaProcessor {
    self.source
  }

  pub fn convert_to(&mut self, target_format: MediaType) -> Result<MediaProcessor> {
    let source_format = self.source.media_type();

    match (source_format, target_format) {
      (MediaType::Image, MediaType::Image) => Ok(self.source.clone()),
      (MediaType::Image, MediaType::Audio) => self.convert_image_to_audio(),
      (MediaType::Image, MediaType::Video) => self.convert_image_to_video(),
      (MediaType::Audio, MediaType::Image) => self.convert_audio_to_image(),
      (MediaType::Audio, MediaType::Audio) => Ok(self.source.clone()),
      (MediaType::Audio, MediaType::Video) => self.convert_audio_to_video(),
      (MediaType::Video, MediaType::Image) => self.convert_video_to_image(),
      (MediaType::Video, MediaType::Audio) => self.convert_video_to_audio(),
      (MediaType::Video, MediaType::Video) => Ok(self.source.clone()),
    }
  }

  fn convert_image_to_audio(&mut self) -> Result<MediaProcessor> {
    if let Some(image_processor) = self.source.as_image_processor() {
      let image_data = image_processor.data();
      let sample_rate = 44100;
      let channels = 2;
      let duration = 5.0;
      let samples_needed = (sample_rate as f64 * duration) as usize;

      let mut audio_samples = Vec::with_capacity(samples_needed * channels as usize);

      for y in 0..image_data.height {
        for x in 0..image_data.width {
          if let Some(pixel) = image_processor.get_pixel(x, y) {
            let left_sample = if pixel.len() >= 1 {
              pixel[0] as f32 / 255.0 - 0.5
            } else {
              0.0
            };
            let right_sample = if pixel.len() >= 2 {
              pixel[1] as f32 / 255.0 - 0.5
            } else {
              0.0
            };

            for _ in 0..(samples_needed / (image_data.width * image_data.height) as usize) {
              audio_samples.push(left_sample);
              audio_samples.push(right_sample);
            }
          }
        }
      }

      audio_samples.truncate(samples_needed * channels as usize);

      let audio_data = AudioData::new(sample_rate, channels, audio_samples)?;
      Ok(MediaProcessor::new(MediaData::Audio(audio_data)))
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not an image".to_string(),
      ))
    }
  }

  fn convert_image_to_video(&mut self) -> Result<MediaProcessor> {
    if let Some(image_processor) = self.source.as_image_processor() {
      let image_data = image_processor.data();
      let frame_rate = 30.0;
      let duration = 5.0;
      let frames_needed = (frame_rate * duration) as usize;

      let mut video_frames = Vec::with_capacity(frames_needed);

      for _ in 0..frames_needed {
        let frame_data = image_data.clone();
        video_frames.push(crate::VideoFrame::new(
          image_data.width,
          image_data.height,
          image_data.channels,
          image_data.bit_depth,
          frame_data.data.clone(),
        )?);
      }

      let video_data = VideoData::new(
        image_data.width,
        image_data.height,
        frame_rate,
        image_data.channels,
        image_data.bit_depth,
        video_frames,
      )?;

      Ok(MediaProcessor::new(MediaData::Video(video_data)))
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not an image".to_string(),
      ))
    }
  }

  fn convert_audio_to_image(&mut self) -> Result<MediaProcessor> {
    if let Some(audio_processor) = self.source.as_audio_processor() {
      let audio_data = audio_processor.data();
      let sample_rate = audio_data.sample_rate;
      let width = 1024;
      let height = 768;
      let channels = 3;

      let mut image_data = vec![0u8; (width * height * channels) as usize];

      for (i, &sample) in audio_data.samples.iter().enumerate() {
        let x = (i as u32 % width) as usize;
        let y = (i as u32 / width) as usize;
        let pixel_offset = (y * width + x) * channels;

        if pixel_offset + channels <= image_data.len() {
          let value = (sample * 255.0).clamp(0.0, 255.0) as u8;
          for c in 0..channels {
            if pixel_offset + c < image_data.len() {
              image_data[pixel_offset + c] = value;
            }
          }
        }
      }

      let image_data = ImageData::new(width, height, channels, image_data)?;
      Ok(MediaProcessor::new(MediaData::Image(image_data)))
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not audio".to_string(),
      ))
    }
  }

  fn convert_audio_to_video(&mut self) -> Result<MediaProcessor> {
    if let Some(audio_processor) = self.source.as_audio_processor() {
      let audio_data = audio_processor.data();
      let frame_rate = 30.0;
      let duration = audio_data.duration_seconds();
      let frames_needed = (frame_rate * duration) as usize;
      let width = 1024;
      let height = 768;
      let channels = 3;
      let bit_depth = 8;

      let mut video_frames = Vec::with_capacity(frames_needed);

      for frame_index in 0..frames_needed {
        let start_sample = ((frame_index as f64 / frame_rate * audio_data.sample_rate as f64)
          as usize)
          * audio_data.channels as usize;
        let end_sample = (((frame_index + 1) as f64 / frame_rate * audio_data.sample_rate as f64)
          as usize)
          .min(audio_data.samples.len() / audio_data.channels as usize)
          * audio_data.channels as usize;

        let mut frame_data = vec![0u8; (width * height * channels) as usize];

        for y in 0..height {
          for x in 0..width {
            let sample_index = start_sample
              + ((y * width + x) * audio_data.channels as usize).min(end_sample - start_sample - 1);

            if sample_index < audio_data.samples.len() {
              let sample = audio_data.samples[sample_index];
              let value = (sample * 255.0).clamp(0.0, 255.0) as u8;

              for c in 0..channels {
                let pixel_offset = (y * width + x) * channels + c;
                if pixel_offset < frame_data.len() {
                  frame_data[pixel_offset] = value;
                }
              }
            }
          }
        }

        video_frames.push(crate::VideoFrame::new(
          width, height, channels, bit_depth, frame_data,
        )?);
      }

      let video_data =
        VideoData::new(width, height, frame_rate, channels, bit_depth, video_frames)?;

      Ok(MediaProcessor::new(MediaData::Video(video_data)))
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not audio".to_string(),
      ))
    }
  }

  fn convert_video_to_image(&mut self) -> Result<MediaProcessor> {
    if let Some(video_processor) = self.source.as_video_processor() {
      let video_data = video_processor.data();

      if let Some(first_frame) = video_data.frames.first() {
        let image_data = ImageData::new(
          first_frame.width,
          first_frame.height,
          first_frame.channels,
          first_frame.data.clone(),
        )?;

        Ok(MediaProcessor::new(MediaData::Image(image_data)))
      } else {
        Err(EllasticError::InvalidParameter(
          "Video has no frames".to_string(),
        ))
      }
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not video".to_string(),
      ))
    }
  }

  fn convert_video_to_audio(&mut self) -> Result<MediaProcessor> {
    if let Some(video_processor) = self.source.as_video_processor() {
      let video_data = video_processor.data();
      let sample_rate = 44100;
      let channels = 2;

      let mut audio_samples = Vec::new();

      for frame in &video_data.frames {
        for y in 0..frame.height {
          for x in 0..frame.width {
            if let Some(pixel_index) = (y * frame.width + x) * (frame.bit_depth as usize / 8) {
              if pixel_index < frame.data.len() {
                let byte_value = frame.data[pixel_index];
                let sample_value = (byte_value as f32 / 255.0 - 0.5) * 2.0;

                audio_samples.push(sample_value);
                if channels == 2 {
                  audio_samples.push(sample_value);
                }
              }
            }
          }
        }
      }

      let audio_data = AudioData::new(sample_rate, channels, audio_samples)?;
      Ok(MediaProcessor::new(MediaData::Audio(audio_data)))
    } else {
      Err(EllasticError::InvalidParameter(
        "Source is not video".to_string(),
      ))
    }
  }

  pub fn resize(
    &mut self,
    width: u32,
    height: u32,
    method: ResizeMethod,
  ) -> Result<MediaProcessor> {
    match self.source.media_type() {
      MediaType::Image => {
        if let Some(image_processor) = self.source.as_image_processor() {
          let resized = image_processor.resize(
            width,
            height,
            match method {
              ResizeMethod::Nearest => ellastic_image::ResampleMethod::Nearest,
              ResizeMethod::Linear => ellastic_image::ResampleMethod::Linear,
              ResizeMethod::Cubic => ellastic_image::ResampleMethod::Cubic,
              ResizeMethod::Lanczos => ellastic_image::ResampleMethod::Lanczos,
            },
          )?;
          Ok(MediaProcessor::new(MediaData::Image(resized.into_data())))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not an image".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.source.as_audio_processor() {
          let resampled = audio_processor.resample(
            width as u32 * height as u32,
            match method {
              ResizeMethod::Nearest => ellastic_audio::ResampleMethod::Nearest,
              ResizeMethod::Linear => ellastic_audio::ResampleMethod::Linear,
              ResizeMethod::Cubic => ellastic_audio::ResampleMethod::Cubic,
              ResizeMethod::Lanczos => ellastic_audio::ResampleMethod::Sinc,
            },
          )?;
          Ok(MediaProcessor::new(MediaData::Audio(resampled.into_data())))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not audio".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let mut resized_video = video_processor.clone();
          resized_video.resize(width, height)?;
          Ok(MediaProcessor::new(MediaData::Video(
            resized_video.into_data(),
          )))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
    }
  }

  pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<MediaProcessor> {
    match self.source.media_type() {
      MediaType::Image => {
        if let Some(image_processor) = self.source.as_image_processor() {
          let cropped = image_processor.crop(x, y, width, height)?;
          Ok(MediaProcessor::new(MediaData::Image(cropped.into_data())))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not an image".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.source.as_audio_processor() {
          let sample_rate = audio_processor.sample_rate();
          let start_sample =
            ((y as f64 * sample_rate as f64) as usize) * audio_processor.channels() as usize;
          let end_sample = (((y + height) as f64 * sample_rate as f64) as usize)
            .min(audio_processor.sample_count())
            * audio_processor.channels() as usize;

          let cropped_samples = audio_processor.data().samples[start_sample..end_sample].to_vec();
          let cropped_data =
            AudioData::new(sample_rate, audio_processor.channels(), cropped_samples)?;
          Ok(MediaProcessor::new(MediaData::Audio(cropped_data)))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not audio".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let start_frame = (y as f64 * video_processor.frame_rate()) as usize;
          let end_frame = ((y + height) as f64 * video_processor.frame_rate()) as usize;

          let mut frames = Vec::new();
          for frame_index in start_frame..=end_frame.min(video_processor.frame_count()) {
            if let Some(frame) = video_processor.get_frame(frame_index) {
              frames.push(frame.clone());
            }
          }

          let cropped_video = VideoData::new(
            video_processor.width(),
            video_processor.height(),
            video_processor.frame_rate(),
            video_processor.channels(),
            video_processor.bit_depth(),
            frames,
          )?;

          Ok(MediaProcessor::new(MediaData::Video(cropped_video)))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
    }
  }

  pub fn change_format(
    &mut self,
    target_format: MediaType,
    codec: Option<String>,
  ) -> Result<MediaProcessor> {
    let converted_data = self.source.convert_to(target_format)?;

    match codec {
      Some(codec) => self.apply_codec(&converted_data, codec),
      None => Ok(converted_data),
    }
  }

  fn apply_codec(&self, data: &MediaData, codec: &str) -> Result<MediaProcessor> {
    match data {
      MediaData::Image(image_data) => match codec {
        "jpeg" | "jpg" => {
          let processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
          let encoded = processor.encode(ellastic_image::ImageFormat::JPEG, Some(85))?;
          let redecoded =
            ellastic_image::decode_image(&encoded, ellastic_image::ImageFormat::JPEG)?;
          Ok(MediaProcessor::new(MediaData::Image(redecoded)))
        }
        "png" => {
          let processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
          let encoded = processor.encode(ellastic_image::ImageFormat::PNG, None)?;
          let redecoded = ellastic_image::decode_image(&encoded, ellastic_image::ImageFormat::PNG)?;
          Ok(MediaProcessor::new(MediaData::Image(redecoded)))
        }
        "webp" => {
          let processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
          let encoded = processor.encode(ellastic_image::ImageFormat::WEBP, Some(80))?;
          let redecoded =
            ellastic_image::decode_image(&encoded, ellastic_image::ImageFormat::WEBP)?;
          Ok(MediaProcessor::new(MediaData::Image(redecoded)))
        }
        _ => Err(EllasticError::UnsupportedFormat(format!(
          "Unsupported image codec: {}",
          codec
        ))),
      },
      MediaData::Audio(audio_data) => match codec {
        "mp3" => {
          let processor = ellastic_audio::AudioProcessor::from_audio_data(audio_data.clone());
          let encoded = processor.encode(ellastic_audio::AudioFormat::MP3, Some(128))?;
          let redecoded = ellastic_audio::decode_audio(&encoded, ellastic_audio::AudioFormat::MP3)?;
          Ok(MediaProcessor::new(MediaData::Audio(redecoded)))
        }
        "wav" => {
          let processor = ellastic_audio::AudioProcessor::from_audio_data(audio_data.clone());
          let encoded = processor.encode(ellastic_audio::AudioFormat::WAV, None)?;
          let redecoded = ellastic_audio::decode_audio(&encoded, ellastic_audio::AudioFormat::WAV)?;
          Ok(MediaProcessor::new(MediaData::Audio(redecoded)))
        }
        "flac" => {
          let processor = ellastic_audio::AudioProcessor::from_audio_data(audio_data.clone());
          let encoded = processor.encode(ellastic_audio::AudioFormat::FLAC, None)?;
          let redecoded =
            ellastic_audio::decode_audio(&encoded, ellastic_audio::AudioFormat::FLAC)?;
          Ok(MediaProcessor::new(MediaData::Audio(redecoded)))
        }
        "ogg" => {
          let processor = ellastic_audio::AudioProcessor::from_audio_data(audio_data.clone());
          let encoded = processor.encode(ellastic_audio::AudioFormat::OGG, Some(6))?;
          let redecoded = ellastic_audio::decode_audio(&encoded, ellastic_audio::AudioFormat::OGG)?;
          Ok(MediaProcessor::new(MediaData::Audio(redecoded)))
        }
        _ => Err(EllasticError::UnsupportedFormat(format!(
          "Unsupported audio codec: {}",
          codec
        ))),
      },
      MediaData::Video(video_data) => match codec {
        "mp4" => {
          let processor = crate::VideoProcessor::new_with_data(video_data.clone());
          let encoded = processor.encode(crate::VideoFormat::MP4, Some(75))?;
          let redecoded = crate::decode_video(&encoded, crate::VideoFormat::MP4)?;
          Ok(MediaProcessor::new(MediaData::Video(redecoded)))
        }
        "avi" => {
          let processor = crate::VideoProcessor::new_with_data(video_data.clone());
          let encoded = processor.encode(crate::VideoFormat::AVI, Some(75))?;
          let redecoded = crate::decode_video(&encoded, crate::VideoFormat::AVI)?;
          Ok(MediaProcessor::new(MediaData::Video(redecoded)))
        }
        "mov" => {
          let processor = crate::VideoProcessor::new_with_data(video_data.clone());
          let encoded = processor.encode(crate::VideoFormat::MOV, Some(75))?;
          let redecoded = crate::decode_video(&encoded, crate::VideoFormat::MOV)?;
          Ok(MediaProcessor::new(MediaData::Video(redecoded)))
        }
        "webm" => {
          let processor = crate::VideoProcessor::new_with_data(video_data.clone());
          let encoded = processor.encode(crate::VideoFormat::WEBM, Some(50))?;
          let redecoded = crate::decode_video(&encoded, crate::VideoFormat::WEBM)?;
          Ok(MediaProcessor::new(MediaData::Video(redecoded)))
        }
        _ => Err(EllasticError::UnsupportedFormat(format!(
          "Unsupported video codec: {}",
          codec
        ))),
      },
    }
  }

  pub fn optimize_for_web(&mut self) -> Result<MediaProcessor> {
    match self.source.media_type() {
      MediaType::Image => {
        if let Some(image_processor) = self.source.as_image_processor() {
          let optimized_data = self.optimize_image_for_web(image_processor)?;
          Ok(MediaProcessor::new(MediaData::Image(optimized_data)))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not an image".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.source.as_audio_processor() {
          let optimized_data = self.optimize_audio_for_web(audio_processor)?;
          Ok(MediaProcessor::new(MediaData::Audio(optimized_data)))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not audio".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let optimized_data = self.optimize_video_for_web(video_processor)?;
          Ok(MediaProcessor::new(MediaData::Video(optimized_data)))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
    }
  }

  fn optimize_image_for_web(
    &self,
    image_processor: &ellastic_image::ImageProcessor,
  ) -> Result<ImageData> {
    let image_data = image_processor.data();
    let target_width = image_data.width;
    let target_height = image_data.height;

    if target_width > 2048 || target_height > 2048 {
      let scale = (2048.0 / target_width.max(target_height) as f32).min(1.0);
      let new_width = (target_width as f32 * scale) as u32;
      let new_height = (target_height as f32 * scale) as u32;

      let resized =
        image_processor.resize(new_width, new_height, ellastic_image::ResampleMethod::Cubic)?;
      Ok(resized.into_data())
    } else {
      Ok(image_data.clone())
    }
  }

  fn optimize_audio_for_web(
    &self,
    audio_processor: &ellastic_audio::AudioProcessor,
  ) -> Result<AudioData> {
    let audio_data = audio_processor.data();
    let target_sample_rate = audio_data.sample_rate;

    let optimized_sample_rate = if target_sample_rate > 48000 {
      48000
    } else {
      target_sample_rate
    };

    if optimized_sample_rate != target_sample_rate {
      audio_processor
        .resample(optimized_sample_rate, ellastic_audio::ResampleMethod::Cubic)
        .map(|processor| processor.into_data())
    } else {
      Ok(audio_data.clone())
    }
  }

  fn optimize_video_for_web(&self, video_processor: &crate::VideoProcessor) -> Result<VideoData> {
    let video_data = video_processor.data();
    let target_frame_rate = video_data.frame_rate;

    let optimized_frame_rate = if target_frame_rate > 30.0 {
      30.0
    } else {
      target_frame_rate
    };

    if optimized_frame_rate != target_frame_rate {
      let mut optimized_video = video_processor.clone();
      optimized_video.change_frame_rate(optimized_frame_rate)?;
      Ok(optimized_video.into_data())
    } else {
      Ok(video_data.clone())
    }
  }

  pub fn extract_thumbnail(&mut self, width: u32, height: u32) -> Result<MediaProcessor> {
    match self.source.media_type() {
      MediaType::Image => {
        if let Some(image_processor) = self.source.as_image_processor() {
          let thumbnail = image_processor.create_thumbnail(width, height)?;
          Ok(MediaProcessor::new(MediaData::Image(thumbnail.into_data())))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not an image".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let thumbnail = video_processor.create_thumbnail(width, height)?;
          Ok(MediaProcessor::new(MediaData::Image(thumbnail.into_data())))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
      MediaType::Audio => Err(EllasticError::UnsupportedOperation(
        "Cannot create thumbnail from audio".to_string(),
      )),
    }
  }

  pub fn extract_audio_from_video(&mut self) -> Result<MediaProcessor> {
    match self.source.media_type() {
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let audio_data = crate::extract_audio_from_video(video_processor.data())?;
          Ok(MediaProcessor::new(MediaData::Audio(audio_data)))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
      _ => Err(EllasticError::UnsupportedOperation(
        "Cannot extract audio from non-video media".to_string(),
      )),
    }
  }

  pub fn extract_frames(&mut self, frame_indices: &[usize]) -> Result<Vec<MediaProcessor>> {
    match self.source.media_type() {
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let mut frames = Vec::new();
          for &frame_index in frame_indices {
            if let Some(frame) = video_processor.get_frame(*frame_index) {
              let frame_data = ellastic_core::ImageData::new(
                frame.width,
                frame.height,
                frame.channels,
                frame.data.clone(),
              )?;
              frames.push(MediaProcessor::new(MediaData::Image(frame_data)));
            }
          }
          Ok(frames)
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
      _ => Err(EllasticError::UnsupportedOperation(
        "Cannot extract frames from non-video media".to_string(),
      )),
    }
  }

  pub fn create_slideshow(&mut self, frame_duration: f64) -> Result<MediaProcessor> {
    match self.source.media_type() {
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let video_data = video_processor.data();
          let frame_count = video_data.frames.len();
          let slideshow_duration = frame_count as f64 * frame_duration;
          let slideshow_frame_rate = 1.0 / frame_duration;

          let mut slideshow_frames = Vec::new();
          for frame in &video_data.frames {
            let mut new_frame = frame.clone();
            new_frame.timestamp = Some((slideshow_frames.len() as f64 * frame_duration));
            slideshow_frames.push(new_frame);
          }

          let slideshow_video = VideoData::new(
            video_data.width,
            video_data.height,
            slideshow_frame_rate,
            video_data.channels,
            video_data.bit_depth,
            slideshow_frames,
          )?;

          Ok(MediaProcessor::new(MediaData::Video(slideshow_video)))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
      _ => Err(EllasticError::UnsupportedOperation(
        "Cannot create slideshow from non-video media".to_string(),
      )),
    }
  }

  pub fn apply_fade(&mut self, fade_type: FadeType, duration: f64) -> Result<MediaProcessor> {
    match self.source.media_type() {
      MediaType::Audio => {
        if let Some(audio_processor) = self.source.as_audio_processor() {
          match fade_type {
            FadeType::In => {
              audio_processor.fade_in(duration)?;
            }
            FadeType::Out => {
              audio_processor.fade_out(duration)?;
            }
            FadeType::Crossfade {
              other,
              crossfade_duration,
            } => {
              if let Some(other_audio) = other.as_audio_processor() {
                audio_processor.crossfade(&other_audio, crossfade_duration)?;
              }
            }
          }
          Ok(audio_processor.into_processor())
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not audio".to_string(),
          ))
        }
      }
      _ => Err(EllasticError::UnsupportedOperation(
        "Fade not supported for this media type".to_string(),
      )),
    }
  }

  pub fn create_gif(&mut self, frame_delay: f64) -> Result<MediaProcessor> {
    match self.source.media_type() {
      MediaType::Image => Err(EllasticError::UnsupportedOperation(
        "GIF creation from single image not implemented".to_string(),
      )),
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let video_data = video_processor.data();
          let frame_count = video_data.frames.len();
          let gif_delay_ms = (frame_delay * 1000.0) as u32;

          let mut gif_frames = Vec::new();
          for (i, frame) in video_data.frames.iter().enumerate() {
            let mut gif_frame = frame.clone();
            gif_frame.timestamp = Some(i as f64 * frame_delay);
            gif_frames.push(gif_frame);
          }

          let gif_video = VideoData::new(
            video_data.width,
            video_data.height,
            1.0 / frame_delay,
            video_data.channels,
            video_data.bit_depth,
            gif_frames,
          )?;

          Ok(MediaProcessor::new(MediaData::Video(gif_video)))
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
      _ => Err(EllasticError::UnsupportedOperation(
        "GIF creation not supported for this media type".to_string(),
      )),
    }
  }

  pub fn extract_metadata(&self) -> crate::MediaMetadata {
    self.source.get_metadata()
  }

  pub fn validate_format(&self) -> Result<ValidationResult> {
    let media_type = self.source.media_type();

    match media_type {
      MediaType::Image => {
        if let Some(image_processor) = self.source.as_image_processor() {
          let image_data = image_processor.data();
          Ok(ValidationResult {
            is_valid: true,
            issues: Vec::new(),
            format: "Image",
            size: (image_data.width * image_data.height) as usize,
            duration: None,
            bitrate: None,
          })
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not an image".to_string(),
          ))
        }
      }
      MediaType::Audio => {
        if let Some(audio_processor) = self.source.as_audio_processor() {
          let audio_data = audio_processor.data();
          Ok(ValidationResult {
            is_valid: true,
            issues: Vec::new(),
            format: "Audio",
            size: audio_data.samples.len(),
            duration: Some(audio_data.duration_seconds()),
            bitrate: None,
          })
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not audio".to_string(),
          ))
        }
      }
      MediaType::Video => {
        if let Some(video_processor) = self.source.as_video_processor() {
          let video_data = video_processor.data();
          Ok(ValidationResult {
            is_valid: true,
            issues: Vec::new(),
            format: "Video",
            size: video_data.frames.len() * video_data.frame_size(),
            duration: Some(video_data.duration_seconds()),
            bitrate: None,
          })
        } else {
          Err(EllasticError::InvalidParameter(
            "Source is not video".to_string(),
          ))
        }
      }
    }
  }

  pub fn get_supported_formats(&self) -> Vec<MediaType> {
    vec![MediaType::Image, MediaType::Audio, MediaType::Video]
  }

  pub fn get_supported_codecs(&self, media_type: MediaType) -> Vec<String> {
    match media_type {
      MediaType::Image => vec!["jpeg", "png", "webp", "gif", "bmp", "tiff"],
      MediaType::Audio => vec!["mp3", "wav", "flac", "ogg", "aac"],
      MediaType::Video => vec!["mp4", "avi", "mov", "webm", "mkv"],
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeMethod {
  Nearest,
  Linear,
  Cubic,
  Lanczos,
}

#[derive(Debug, Clone)]
pub enum FadeType {
  In,
  Out,
  Crossfade {
    other: Box<MediaProcessor>,
    crossfade_duration: f64,
  },
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
  pub is_valid: bool,
  pub issues: Vec<String>,
  pub format: String,
  pub size: usize,
  pub duration: Option<f64>,
  pub bitrate: Option<u32>,
}

pub fn create_media_converter(source: MediaProcessor) -> MediaConverter {
  MediaConverter::new(source)
}

pub fn convert_media(source: MediaProcessor, target_format: MediaType) -> Result<MediaProcessor> {
  let mut converter = create_media_converter(source);
  converter.convert_to(target_format)
}

pub fn convert_media_with_codec(
  source: MediaProcessor,
  target_format: MediaType,
  codec: &str,
) -> Result<MediaProcessor> {
  let mut converter = create_media_converter(source);
  converter.change_format(target_format, Some(codec.to_string()))
}

pub fn create_thumbnail(
  media_processor: &MediaProcessor,
  width: u32,
  height: u32,
) -> Result<MediaProcessor> {
  let mut converter = create_media_converter(media_processor.clone());
  converter.extract_thumbnail(width, height)
}

pub fn optimize_for_web(media_processor: MediaProcessor) -> Result<MediaProcessor> {
  let mut converter = create_media_converter(media_processor);
  converter.optimize_for_web()
}

pub fn extract_audio_from_video(video_processor: &crate::VideoProcessor) -> Result<MediaProcessor> {
  let mut converter = create_media_converter(MediaProcessor::new(video_processor.into_data()));
  converter.extract_audio_from_video()
}
