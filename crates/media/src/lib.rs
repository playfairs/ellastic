use std::path::Path;
use ellastic_errors::{Result, EllasticError};
use ellastic_core::{MediaData, MediaType};

pub mod processor;
pub mod converter;
pub mod metadata;
pub mod stream;

pub use processor::*;
pub use converter::*;
pub use metadata::*;
pub use stream::*;

#[derive(Debug, Clone)]
pub struct MediaProcessor {
    data: MediaData,
}

impl MediaProcessor {
    pub fn new(data: MediaData) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &MediaData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut MediaData {
        &mut self.data
    }

    pub fn into_data(self) -> MediaData {
        self.data
    }

    pub fn media_type(&self) -> MediaType {
        match &self.data {
            MediaData::Image(_) => MediaType::Image,
            MediaData::Audio(_) => MediaType::Audio,
            MediaData::Video(_) => MediaType::Video,
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(self.data, MediaData::Image(_))
    }

    pub fn is_audio(&self) -> bool {
        matches!(self.data, MediaData::Audio(_))
    }

    pub fn is_video(&self) -> bool {
        matches!(self.data, MediaData::Video(_))
    }

    pub fn as_image_processor(&self) -> Option<ellastic_image::ImageProcessor> {
        match &self.data {
            MediaData::Image(image_data) => Some(ellastic_image::ImageProcessor::from_image_data(image_data.clone())),
            _ => None,
        }
    }

    pub fn as_audio_processor(&self) -> Option<ellastic_audio::AudioProcessor> {
        match &self.data {
            MediaData::Audio(audio_data) => Some(ellastic_audio::AudioProcessor::from_audio_data(audio_data.clone())),
            _ => None,
        }
    }

    pub fn as_video_processor(&self) -> Option<VideoProcessor> {
        match &self.data {
            MediaData::Video(_) => Some(VideoProcessor::new()),
            _ => None,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path)
            .map_err(|e| EllasticError::IoError(format!("Failed to read media file: {}", e)))?;
        
        let media_type = detect_media_type(&data)?;
        let media_data = load_media_data(&data, media_type)?;
        
        Ok(Self::new(media_data))
    }

    pub fn load_from_buffer(buffer: Vec<u8>, media_type: MediaType) -> Result<Self> {
        let media_data = load_media_data(&buffer, media_type)?;
        Ok(Self::new(media_data))
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        save_media_data(&self.data, path)
    }

    pub fn get_duration(&self) -> Option<f64> {
        match &self.data {
            MediaData::Audio(audio_data) => Some(audio_data.duration_seconds()),
            MediaData::Video(video_data) => Some(video_data.duration_seconds),
            MediaData::Image(_) => None,
        }
    }

    pub fn get_resolution(&self) -> Option<(u32, u32)> {
        match &self.data {
            MediaData::Image(image_data) => Some((image_data.width, image_data.height)),
            MediaData::Video(video_data) => Some((video_data.width, video_data.height)),
            MediaData::Audio(_) => None,
        }
    }

    pub fn get_sample_rate(&self) -> Option<u32> {
        match &self.data {
            MediaData::Audio(audio_data) => Some(audio_data.sample_rate),
            MediaData::Video(video_data) => Some(video_data.sample_rate),
            MediaData::Image(_) => None,
        }
    }

    pub fn get_channels(&self) -> Option<u8> {
        match &self.data {
            MediaData::Image(image_data) => Some(image_data.channels),
            MediaData::Audio(audio_data) => Some(audio_data.channels),
            MediaData::Video(video_data) => Some(video_data.channels),
        }
    }

    pub fn get_bit_depth(&self) -> Option<u8> {
        match &self.data {
            MediaData::Image(image_data) => Some(image_data.bit_depth),
            MediaData::Audio(audio_data) => Some(32),
            MediaData::Video(video_data) => Some(video_data.bit_depth),
        }
    }

    pub fn get_frame_rate(&self) -> Option<f64> {
        match &self.data {
            MediaData::Video(video_data) => Some(video_data.frame_rate),
            MediaData::Image(_) | MediaData::Audio(_) => None,
        }
    }

    pub fn get_frame_count(&self) -> Option<usize> {
        match &self.data {
            MediaData::Video(video_data) => Some(video_data.frame_count),
            MediaData::Image(_) | MediaData::Audio(_) => None,
        }
    }

    pub fn get_metadata(&self) -> MediaMetadata {
        extract_metadata(&self.data)
    }

    pub fn convert_to(&self, target_media_type: MediaType) -> Result<MediaProcessor> {
        let converted_data = convert_media_data(&self.data, target_media_type)?;
        Ok(MediaProcessor::new(converted_data))
    }

    pub fn extract_frame(&self, frame_index: usize) -> Result<MediaProcessor> {
        match &self.data {
            MediaData::Video(video_data) => {
                let frame_data = extract_video_frame(video_data, frame_index)?;
                Ok(MediaProcessor::new(MediaData::Image(frame_data)))
            }
            _ => Err(EllasticError::UnsupportedOperation("Frame extraction only supported for video".to_string())),
        }
    }

    pub fn extract_audio(&self) -> Result<MediaProcessor> {
        match &self.data {
            MediaData::Video(video_data) => {
                let audio_data = extract_audio_from_video(video_data)?;
                Ok(MediaProcessor::new(MediaData::Audio(audio_data)))
            }
            MediaData::Audio(audio_data) => Ok(MediaProcessor::new(MediaData::Audio(audio_data.clone()))),
            _ => Err(EllasticError::UnsupportedOperation("Audio extraction not supported for this media type".to_string())),
        }
    }

    pub fn create_thumbnail(&self, width: u32, height: u32) -> Result<MediaProcessor> {
        match &self.data {
            MediaData::Image(image_data) => {
                let thumbnail_data = create_image_thumbnail(image_data, width, height)?;
                Ok(MediaProcessor::new(MediaData::Image(thumbnail_data)))
            }
            MediaData::Video(video_data) => {
                let thumbnail_data = create_video_thumbnail(video_data, width, height)?;
                Ok(MediaProcessor::new(MediaData::Image(thumbnail_data)))
            }
            _ => Err(EllasticError::UnsupportedOperation("Thumbnail creation not supported for this media type".to_string())),
        }
    }

    pub fn apply_transformation(&mut self, transformation: &MediaTransformation) -> Result<()> {
        apply_media_transformation(&mut self.data, transformation)
    }

    pub fn batch_process(&self, operations: &[MediaOperation]) -> Result<Vec<MediaProcessor>> {
        let mut results = Vec::new();
        
        for operation in operations {
            let mut temp_data = self.data.clone();
            apply_media_operation(&mut temp_data, operation)?;
            results.push(MediaProcessor::new(temp_data));
        }

        Ok(results)
    }

    pub fn stream_processing(&self) -> Result<MediaStream> {
        create_media_stream(&self.data)
    }

    pub fn clone(&self) -> MediaProcessor {
        MediaProcessor::new(self.data.clone())
    }
}

#[derive(Debug, Clone)]
pub struct VideoProcessor {
    data: VideoData,
}

impl VideoProcessor {
    pub fn new() -> Self {
        Self {
            data: VideoData::new(1920, 1080, 30.0, 3, 8, Vec::new()).unwrap(),
        }
    }

    pub fn new_with_data(data: VideoData) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &VideoData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut VideoData {
        &mut self.data
    }

    pub fn into_data(self) -> VideoData {
        self.data
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }

    pub fn height(&self) -> u32 {
        self.data.height
    }

    pub fn frame_rate(&self) -> f64 {
        self.data.frame_rate
    }

    pub fn channels(&self) -> u8 {
        self.data.channels
    }

    pub fn bit_depth(&self) -> u8 {
        self.data.bit_depth
    }

    pub fn frame_count(&self) -> usize {
        self.data.frame_count
    }

    pub fn duration_seconds(&self) -> f64 {
        self.data.duration_seconds()
    }

    pub fn get_frame(&self, frame_index: usize) -> Option<VideoFrame> {
        self.data.get_frame(frame_index)
    }

    pub fn add_frame(&mut self, frame: VideoFrame) -> Result<()> {
        self.data.add_frame(frame)
    }

    pub fn remove_frame(&mut self, frame_index: usize) -> Result<()> {
        self.data.remove_frame(frame_index)
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        self.data.resize(new_width, new_height)
    }

    pub fn change_frame_rate(&mut self, new_frame_rate: f64) -> Result<()> {
        self.data.change_frame_rate(new_frame_rate)
    }

    pub fn encode(&self, format: VideoFormat, quality: Option<u8>) -> Result<Vec<u8>> {
        encode_video(&self.data, format, quality)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, format: VideoFormat, quality: Option<u8>) -> Result<()> {
        let encoded = self.encode(format, quality)?;
        std::fs::write(path, encoded)
            .map_err(|e| EllasticError::IoError(format!("Failed to save video: {}", e)))
    }

    pub fn clone(&self) -> VideoProcessor {
        VideoProcessor::new_with_data(self.data.clone())
    }
}

#[derive(Debug, Clone)]
pub struct VideoData {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub channels: u8,
    pub bit_depth: u8,
    pub frames: Vec<VideoFrame>,
}

impl VideoData {
    pub fn new(width: u32, height: u32, frame_rate: f64, channels: u8, bit_depth: u8, frames: Vec<VideoFrame>) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(EllasticError::InvalidParameter("Invalid video dimensions".to_string()));
        }
        if frame_rate <= 0.0 {
            return Err(EllasticError::InvalidParameter("Invalid frame rate".to_string()));
        }
        if channels == 0 || channels > 4 {
            return Err(EllasticError::InvalidParameter("Invalid channel count".to_string()));
        }
        if bit_depth == 0 || bit_depth > 32 {
            return Err(EllasticError::InvalidParameter("Invalid bit depth".to_string()));
        }

        Ok(Self {
            width,
            height,
            frame_rate,
            channels,
            bit_depth,
            frames,
        })
    }

    pub fn frame_size(&self) -> usize {
        (self.width as usize * self.height as usize * self.channels as usize * (self.bit_depth as usize / 8))
    }

    pub fn duration_seconds(&self) -> f64 {
        self.frames.len() as f64 / self.frame_rate
    }

    pub fn get_frame(&self, frame_index: usize) -> Option<VideoFrame> {
        self.frames.get(frame_index).cloned()
    }

    pub fn add_frame(&mut self, frame: VideoFrame) -> Result<()> {
        if frame.width != self.width || frame.height != self.height {
            return Err(EllasticError::InvalidParameter("Frame dimensions don't match video".to_string()));
        }
        if frame.channels != self.channels {
            return Err(EllasticError::InvalidParameter("Frame channel count doesn't match video".to_string()));
        }
        if frame.bit_depth != self.bit_depth {
            return Err(EllasticError::InvalidParameter("Frame bit depth doesn't match video".to_string()));
        }

        self.frames.push(frame);
        Ok(())
    }

    pub fn remove_frame(&mut self, frame_index: usize) -> Result<()> {
        if frame_index >= self.frames.len() {
            return Err(EllasticError::InvalidParameter("Frame index out of range".to_string()));
        }

        self.frames.remove(frame_index);
        Ok(())
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        if new_width == 0 || new_height == 0 {
            return Err(EllasticError::InvalidParameter("Invalid new dimensions".to_string()));
        }

        for frame in &mut self.frames {
            frame.resize(new_width, new_height)?;
        }

        self.width = new_width;
        self.height = new_height;
        Ok(())
    }

    pub fn change_frame_rate(&mut self, new_frame_rate: f64) -> Result<()> {
        if new_frame_rate <= 0.0 {
            return Err(EllasticError::InvalidParameter("Invalid frame rate".to_string()));
        }

        self.frame_rate = new_frame_rate;
        Ok(())
    }

    pub fn clone(&self) -> VideoData {
        Self {
            width: self.width,
            height: self.height,
            frame_rate: self.frame_rate,
            channels: self.channels,
            bit_depth: self.bit_depth,
            frames: self.frames.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub bit_depth: u8,
    pub data: Vec<u8>,
    pub timestamp: Option<f64>,
}

impl VideoFrame {
    pub fn new(width: u32, height: u32, channels: u8, bit_depth: u8, data: Vec<u8>) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(EllasticError::InvalidParameter("Invalid frame dimensions".to_string()));
        }
        if channels == 0 || channels > 4 {
            return Err(EllasticError::InvalidParameter("Invalid channel count".to_string()));
        }
        if bit_depth == 0 || bit_depth > 32 {
            return Err(EllasticError::InvalidParameter("Invalid bit depth".to_string()));
        }
        let expected_size = (width as usize * height as usize * channels as usize * (bit_depth as usize / 8));
        if data.len() != expected_size {
            return Err(EllasticError::InvalidParameter("Data size doesn't match frame dimensions".to_string()));
        }

        Ok(Self {
            width,
            height,
            channels,
            bit_depth,
            data,
            timestamp: None,
        }
    }

    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        if new_width == 0 || new_height == 0 {
            return Err(EllasticError::InvalidParameter("Invalid new dimensions".to_string()));
        }

        let old_width = self.width;
        let old_height = self.height;
        let channels = self.channels;
        let bit_depth = self.bit_depth;

        let mut new_data = vec![0u8; new_width as usize * new_height as usize * channels as usize * (bit_depth as usize / 8)];

        for y in 0..new_height {
            for x in 0..new_width {
                let old_x = (x as f32 * old_width as f32 / new_width as f32) as u32;
                let old_y = (y as f32 * old_height as f32 / new_height as f32) as u32;

                if old_x < old_width && old_y < old_height {
                    let old_index = (old_y as usize * old_width as usize * channels as usize + old_x as usize * channels as usize) * (bit_depth as usize / 8);
                    let new_index = (y as usize * new_width as usize * channels as usize + x as usize * channels as usize) * (bit_depth as usize / 8);

                    let bytes_per_pixel = (channels * bit_depth / 8) as usize;
                    for byte_offset in 0..bytes_per_pixel {
                        if old_index + byte_offset < self.data.len() && new_index + byte_offset < new_data.len() {
                            new_data[new_index + byte_offset] = self.data[old_index + byte_offset];
                        }
                    }
                }
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.data = new_data;
        Ok(())
    }

    pub fn clone(&self) -> VideoFrame {
        Self {
            width: self.width,
            height: self.height,
            channels: self.channels,
            bit_depth: self.bit_depth,
            data: self.data.clone(),
            timestamp: self.timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    MP4,
    AVI,
    MOV,
    MKV,
    WEBM,
    GIF,
}

#[derive(Debug, Clone)]
pub struct MediaTransformation {
    pub transformation_type: TransformationType,
    pub parameters: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum TransformationType {
    Resize { width: u32, height: u32 },
    Crop { x: u32, y: u32, width: u32, height: u32 },
    Rotate { degrees: f32 },
    Flip { direction: FlipDirection },
    ColorAdjust { brightness: f32, contrast: f32, saturation: f32 },
    Filter { filter_type: FilterType },
    Effect { effect_type: EffectType },
    FormatConversion { target_format: MediaType },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipDirection {
    Horizontal,
    Vertical,
    Both,
}

#[derive(Debug, Clone)]
pub struct MediaOperation {
    pub operation_type: OperationType,
    pub parameters: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum OperationType {
    ExtractFrame { frame_index: usize },
    ExtractAudio,
    CreateThumbnail { width: u32, height: u32 },
    ApplyEffect { effect: MediaTransformation },
    BatchProcess { operations: Vec<MediaOperation> },
    StreamProcess,
}

#[derive(Debug, Clone)]
pub struct MediaStream {
    pub media_type: MediaType,
    pub stream_type: StreamType,
    pub codec: String,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
    Video,
    Audio,
    Subtitle,
    Data,
}

#[derive(Debug, Clone)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub duration: Option<f64>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f64>,
    pub codec: Option<String>,
    pub format: Option<String>,
    pub tags: std::collections::HashMap<String, String>,
}

pub fn detect_media_type(data: &[u8]) -> Result<MediaType> {
    if data.len() < 12 {
        return Err(EllasticError::UnsupportedFormat("Insufficient data".to_string()));
    }

    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Ok(MediaType::Image)
    } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Ok(MediaType::Image)
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WAVE" {
        Ok(MediaType::Audio)
    } else if data.starts_with(b"ID3") || (data[0] == 0xFF && data[1] == 0xFB) {
        Ok(MediaType::Audio)
    } else if data.starts_with(b"fLaC") {
        Ok(MediaType::Audio)
    } else if data.starts_with(b"OggS") {
        Ok(MediaType::Audio)
    } else if data.starts_with(b"ftyp") {
        Ok(MediaType::Video)
    } else if data.starts_with(&[0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70]) {
        Ok(MediaType::Video)
    } else {
        Err(EllasticError::UnsupportedFormat("Unknown media format".to_string()))
    }
}

pub fn load_media_data(data: &[u8], media_type: MediaType) -> Result<MediaData> {
    match media_type {
        MediaType::Image => {
            let image_format = ellastic_image::detect_format(data)?;
            let image_data = ellastic_image::decode_image(data, image_format)?;
            Ok(MediaData::Image(image_data))
        }
        MediaType::Audio => {
            let audio_format = ellastic_audio::detect_format(data)?;
            let audio_data = ellastic_audio::decode_audio(data, audio_format)?;
            Ok(MediaData::Audio(audio_data))
        }
        MediaType::Video => {
            let video_format = detect_video_format(data)?;
            let video_data = decode_video(data, video_format)?;
            Ok(MediaData::Video(video_data))
        }
    }
}

pub fn save_media_data(data: &MediaData, path: &Path) -> Result<()> {
    match data {
        MediaData::Image(image_data) => {
            let format = ellastic_image::detect_format(&std::fs::read(path)?)?;
            let encoded = ellastic_image::encode_image(image_data, format, None)?;
            std::fs::write(path, encoded)
                .map_err(|e| EllasticError::IoError(format!("Failed to save image: {}", e)))
        }
        MediaData::Audio(audio_data) => {
            let format = ellastic_audio::detect_format(&std::fs::read(path)?)?;
            let encoded = ellastic_audio::encode_audio(audio_data, format, None)?;
            std::fs::write(path, encoded)
                .map_err(|e| EllasticError::IoError(format!("Failed to save audio: {}", e)))
        }
        MediaData::Video(video_data) => {
            let format = detect_video_format(&std::fs::read(path)?)?;
            let encoded = encode_video(video_data, format, None)?;
            std::fs::write(path, encoded)
                .map_err(|e| EllasticError::IoError(format!("Failed to save video: {}", e)))
        }
    }
}

pub fn extract_metadata(data: &MediaData) -> MediaMetadata {
    let mut metadata = MediaMetadata {
        title: None,
        artist: None,
        album: None,
        year: None,
        genre: None,
        duration: None,
        bitrate: None,
        sample_rate: None,
        channels: None,
        width: None,
        height: None,
        frame_rate: None,
        codec: None,
        format: None,
        tags: std::collections::HashMap::new(),
    };

    match data {
        MediaData::Image(image_data) => {
            metadata.width = Some(image_data.width);
            metadata.height = Some(image_data.height);
            metadata.channels = Some(image_data.channels);
        }
        MediaData::Audio(audio_data) => {
            metadata.duration = Some(audio_data.duration_seconds());
            metadata.sample_rate = Some(audio_data.sample_rate);
            metadata.channels = Some(audio_data.channels);
        }
        MediaData::Video(video_data) => {
            metadata.duration = Some(video_data.duration_seconds());
            metadata.width = Some(video_data.width);
            metadata.height = Some(video_data.height);
            metadata.frame_rate = Some(video_data.frame_rate);
            metadata.channels = Some(video_data.channels);
        }
    }

    metadata
}

pub fn convert_media_data(data: &MediaData, target_type: MediaType) -> Result<MediaData> {
    match (data, target_type) {
        (MediaData::Image(image_data), MediaType::Audio) => {
            Err(EllasticError::UnsupportedOperation("Cannot convert image to audio".to_string()))
        }
        (MediaData::Audio(audio_data), MediaType::Image) => {
            Err(EllasticError::UnsupportedOperation("Cannot convert audio to image".to_string()))
        }
        (MediaData::Video(video_data), MediaType::Image) => {
            let frame_data = extract_video_frame(video_data, 0)?;
            Ok(MediaData::Image(frame_data))
        }
        (MediaData::Image(_), MediaType::Video) => {
            Err(EllasticError::UnsupportedOperation("Cannot convert image to video".to_string()))
        }
        _ => Ok(data.clone()),
    }
}

pub fn extract_video_frame(video_data: &VideoData, frame_index: usize) -> Result<ellastic_core::ImageData> {
    if frame_index >= video_data.frames.len() {
        return Err(EllasticError::InvalidParameter("Frame index out of range".to_string()));
    }

    let frame = &video_data.frames[frame_index];
    ellastic_core::ImageData::new(
        frame.width,
        frame.height,
        frame.channels,
        frame.data.clone(),
    )
}

pub fn extract_audio_from_video(video_data: &VideoData) -> Result<ellastic_core::AudioData> {
    Err(EllasticError::UnsupportedOperation("Audio extraction from video not implemented".to_string()))
}

pub fn create_image_thumbnail(image_data: &ellastic_core::ImageData, width: u32, height: u32) -> Result<ellastic_core::ImageData> {
    let processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
    let resized = processor.resize(width, height, ellastic_image::ResampleMethod::Cubic)?;
    Ok(resized.into_data())
}

pub fn create_video_thumbnail(video_data: &VideoData, width: u32, height: u32) -> Result<ellastic_core::ImageData> {
    if let Some(first_frame) = video_data.frames.first() {
        let image_data = ellastic_core::ImageData::new(
            first_frame.width,
            first_frame.height,
            first_frame.channels,
            first_frame.data.clone(),
        )?;
        create_image_thumbnail(&image_data, width, height)
    } else {
        Err(EllasticError::InvalidParameter("Video has no frames".to_string()))
    }
}

pub fn apply_media_transformation(data: &mut MediaData, transformation: &MediaTransformation) -> Result<()> {
    match transformation.transformation_type {
        TransformationType::Resize { width, height } => {
            match data {
                MediaData::Image(image_data) => {
                    let processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
                    let resized = processor.resize(width, height, ellastic_image::ResampleMethod::Cubic)?;
                    *data = MediaData::Image(resized.into_data());
                }
                MediaData::Video(video_data) => {
                    video_data.resize(width, height)?;
                }
                MediaData::Audio(_) => {
                    return Err(EllasticError::UnsupportedOperation("Resize not supported for audio".to_string()));
                }
            }
        }
        TransformationType::Crop { x, y, width, height } => {
            match data {
                MediaData::Image(image_data) => {
                    let processor = ellastic_image::ImageProcessor::from_image_data(image_data.clone());
                    let cropped = processor.crop(x, y, width, height)?;
                    *data = MediaData::Image(cropped.into_data());
                }
                MediaData::Video(_) => {
                    return Err(EllasticError::UnsupportedOperation("Crop not supported for video".to_string()));
                }
                MediaData::Audio(_) => {
                    return Err(EllasticError::UnsupportedOperation("Crop not supported for audio".to_string()));
                }
            }
        }
        _ => {
            return Err(EllasticError::UnsupportedOperation("Transformation not implemented".to_string()));
        }
    }

    Ok(())
}

pub fn apply_media_operation(data: &mut MediaData, operation: &MediaOperation) -> Result<()> {
    match operation.operation_type {
        OperationType::ExtractFrame { frame_index } => {
            if let MediaData::Video(video_data) = data {
                let frame_data = extract_video_frame(video_data, frame_index)?;
                *data = MediaData::Image(frame_data);
            } else {
                return Err(EllasticError::UnsupportedOperation("Frame extraction only supported for video".to_string()));
            }
        }
        OperationType::ExtractAudio => {
            if let MediaData::Video(video_data) = data {
                let audio_data = extract_audio_from_video(video_data)?;
                *data = MediaData::Audio(audio_data);
            } else {
                return Err(EllasticError::UnsupportedOperation("Audio extraction only supported for video".to_string()));
            }
        }
        _ => {
            return Err(EllasticError::UnsupportedOperation("Operation not implemented".to_string()));
        }
    }

    Ok(())
}

pub fn create_media_stream(data: &MediaData) -> Result<MediaStream> {
    match data {
        MediaData::Audio(audio_data) => {
            Ok(MediaStream {
                media_type: MediaType::Audio,
                stream_type: StreamType::Audio,
                codec: "PCM".to_string(),
                bitrate: None,
                sample_rate: Some(audio_data.sample_rate),
                channels: Some(audio_data.channels),
                width: None,
                height: None,
                frame_rate: None,
            })
        }
        MediaData::Video(video_data) => {
            Ok(MediaStream {
                media_type: MediaType::Video,
                stream_type: StreamType::Video,
                codec: "RAW".to_string(),
                bitrate: None,
                sample_rate: Some(video_data.sample_rate),
                channels: Some(video_data.channels),
                width: Some(video_data.width),
                height: Some(video_data.height),
                frame_rate: Some(video_data.frame_rate),
            })
        }
        MediaData::Image(image_data) => {
            Ok(MediaStream {
                media_type: MediaType::Image,
                stream_type: StreamType::Video,
                codec: "RAW".to_string(),
                bitrate: None,
                sample_rate: None,
                channels: Some(image_data.channels),
                width: Some(image_data.width),
                height: Some(image_data.height),
                frame_rate: Some(1.0),
            })
        }
    }
}

fn detect_video_format(data: &[u8]) -> Result<VideoFormat> {
    if data.starts_with(b"ftyp") {
        Ok(VideoFormat::MP4)
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"AVI " {
        Ok(VideoFormat::AVI)
    } else if data.starts_with(&[0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70]) {
        Ok(VideoFormat::MOV)
    } else {
        Err(EllasticError::UnsupportedFormat("Unknown video format".to_string()))
    }
}

fn decode_video(data: &[u8], format: VideoFormat) -> Result<VideoData> {
    match format {
        VideoFormat::MP4 => Err(EllasticError::UnsupportedOperation("MP4 decoding not implemented".to_string())),
        VideoFormat::AVI => Err(EllasticError::UnsupportedOperation("AVI decoding not implemented".to_string())),
        VideoFormat::MOV => Err(EllasticError::UnsupportedOperation("MOV decoding not implemented".to_string())),
        VideoFormat::MKV => Err(EllasticError::UnsupportedOperation("MKV decoding not implemented".to_string())),
        VideoFormat::WEBM => Err(EllasticError::UnsupportedOperation("WEBM decoding not implemented".to_string())),
        VideoFormat::GIF => Err(EllasticError::UnsupportedOperation("GIF decoding not implemented".to_string())),
    }
}

fn encode_video(video_data: &VideoData, format: VideoFormat, quality: Option<u8>) -> Result<Vec<u8>> {
    match format {
        VideoFormat::MP4 => Err(EllasticError::UnsupportedOperation("MP4 encoding not implemented".to_string())),
        VideoFormat::AVI => Err(EllasticError::UnsupportedOperation("AVI encoding not implemented".to_string())),
        VideoFormat::MOV => Err(EllasticError::UnsupportedOperation("MOV encoding not implemented".to_string())),
        VideoFormat::MKV => Err(EllasticError::UnsupportedOperation("MKV encoding not implemented".to_string())),
        VideoFormat::WEBM => Err(EllasticError::UnsupportedOperation("WEBM encoding not implemented".to_string())),
        VideoFormat::GIF => Err(EllasticError::UnsupportedOperation("GIF encoding not implemented".to_string())),
    }
}

pub fn create_media_processor(data: MediaData) -> MediaProcessor {
    MediaProcessor::new(data)
}

pub fn load_media<P: AsRef<Path>>(path: P) -> Result<MediaProcessor> {
    MediaProcessor::load_from_file(path)
}

pub fn create_video_processor() -> VideoProcessor {
    VideoProcessor::new()
}

pub fn create_video_processor_with_data(data: VideoData) -> VideoProcessor {
    VideoProcessor::new_with_data(data)
}
