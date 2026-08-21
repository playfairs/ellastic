use ellastic_errors::{
  EllasticError,
  Result,
};
use std::fs;
use std::io;
use std::path::{
  Path,
  PathBuf,
};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use walkdir::{
  DirEntry,
  WalkDir,
};

#[derive(Debug, Clone)]
pub struct FileOperations;

impl FileOperations {
  pub fn ensure_directory_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
      fs::create_dir_all(path).map_err(|e| {
        EllasticError::IoError(format!(
          "Failed to create directory {}: {}",
          path.display(),
          e
        ))
      })?;
    }
    Ok(())
  }

  pub fn copy_file_with_progress<S: AsRef<Path>, D: AsRef<Path>>(
    source: S,
    destination: D,
    progress_callback: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
  ) -> Result<u64> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    let source_size = fs::metadata(source)
      .map_err(|e| EllasticError::IoError(format!("Failed to read source metadata: {}", e)))?
      .len();

    Self::ensure_directory_exists(destination.parent().unwrap_or_else(|| Path::new("")))?;

    let mut source_file = fs::File::open(source)
      .map_err(|e| EllasticError::IoError(format!("Failed to open source file: {}", e)))?;

    let mut dest_file = fs::File::create(destination)
      .map_err(|e| EllasticError::IoError(format!("Failed to create destination file: {}", e)))?;

    let mut buffer = vec![0u8; 64 * 1024];
    let mut total_copied = 0u64;

    loop {
      let bytes_read = io::Read::read(&mut source_file, &mut buffer)
        .map_err(|e| EllasticError::IoError(format!("Failed to read from source: {}", e)))?;

      if bytes_read == 0 {
        break;
      }

      io::Write::write_all(&mut dest_file, &buffer[..bytes_read])
        .map_err(|e| EllasticError::IoError(format!("Failed to write to destination: {}", e)))?;

      total_copied += bytes_read as u64;

      if let Some(ref callback) = progress_callback {
        callback(total_copied, source_size);
      }
    }

    Ok(total_copied)
  }

  pub fn move_file<S: AsRef<Path>, D: AsRef<Path>>(source: S, destination: D) -> Result<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    Self::ensure_directory_exists(destination.parent().unwrap_or_else(|| Path::new("")))?;

    fs::rename(source, destination)
      .map_err(|e| EllasticError::IoError(format!("Failed to move file: {}", e)))
  }

  pub fn delete_file_with_metadata<P: AsRef<Path>>(path: P) -> Result<FileMetadata> {
    let path = path.as_ref();
    let metadata = fs::metadata(path)
      .map_err(|e| EllasticError::IoError(format!("Failed to read file metadata: {}", e)))?;

    let file_metadata = FileMetadata::from_path_and_metadata(path, &metadata)?;

    fs::remove_file(path)
      .map_err(|e| EllasticError::IoError(format!("Failed to delete file: {}", e)))?;

    Ok(file_metadata)
  }

  pub fn delete_directory_recursive<P: AsRef<Path>>(path: P) -> Result<usize> {
    let path = path.as_ref();
    let mut deleted_count = 0;

    if path.is_dir() {
      for entry in WalkDir::new(path)
        .min_depth(1)
        .max_depth(usize::MAX)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
      {
        if entry.file_type().is_dir() {
          fs::remove_dir(entry.path())
            .map_err(|e| EllasticError::IoError(format!("Failed to remove directory: {}", e)))?;
        } else {
          fs::remove_file(entry.path())
            .map_err(|e| EllasticError::IoError(format!("Failed to remove file: {}", e)))?;
        }
        deleted_count += 1;
      }

      fs::remove_dir(path)
        .map_err(|e| EllasticError::IoError(format!("Failed to remove root directory: {}", e)))?;
      deleted_count += 1;
    }

    Ok(deleted_count)
  }

  pub fn get_directory_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let path = path.as_ref();
    let mut total_size = 0u64;

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
      if entry.file_type().is_file() {
        if let Ok(metadata) = entry.metadata() {
          total_size += metadata.len();
        }
      }
    }

    Ok(total_size)
  }

  pub fn find_files_by_extension<P: AsRef<Path>>(
    path: P,
    extensions: &[&str],
    recursive: bool,
  ) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();
    let mut found_files = Vec::new();

    let walk_dir = if recursive {
      WalkDir::new(path)
    } else {
      WalkDir::new(path).max_depth(1)
    };

    for entry in walk_dir.into_iter().filter_map(|e| e.ok()) {
      if entry.file_type().is_file() {
        if let Some(extension) = entry.path().extension().and_then(|s| s.to_str()) {
          if extensions.contains(&extension) {
            found_files.push(entry.path().to_path_buf());
          }
        }
      }
    }

    Ok(found_files)
  }

  pub fn find_files_by_pattern<P: AsRef<Path>>(
    path: P,
    pattern: &str,
    recursive: bool,
  ) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();
    let mut found_files = Vec::new();

    let walk_dir = if recursive {
      WalkDir::new(path)
    } else {
      WalkDir::new(path).max_depth(1)
    };

    for entry in walk_dir.into_iter().filter_map(|e| e.ok()) {
      if entry.file_type().is_file() {
        if let Some(file_name) = entry.file_name().to_str() {
          if file_name.contains(pattern) {
            found_files.push(entry.path().to_path_buf());
          }
        }
      }
    }

    Ok(found_files)
  }

  pub fn create_temp_file(prefix: &str, suffix: &str) -> Result<PathBuf> {
    let temp_file = tempfile::Builder::new()
      .prefix(prefix)
      .suffix(suffix)
      .tempfile()
      .map_err(|e| EllasticError::IoError(format!("Failed to create temp file: {}", e)))?;

    let path = temp_file.path().to_path_buf();
    temp_file
      .keep()
      .map_err(|e| EllasticError::IoError(format!("Failed to keep temp file: {}", e)))?;

    Ok(path)
  }

  pub fn create_temp_directory(prefix: &str) -> Result<PathBuf> {
    let temp_dir = tempfile::Builder::new()
      .prefix(prefix)
      .tempdir()
      .map_err(|e| EllasticError::IoError(format!("Failed to create temp directory: {}", e)))?;

    let path = temp_dir.path().to_path_buf();
    let _ = temp_dir.keep();
    Ok(path)
  }

  pub fn is_same_file<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> Result<bool> {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();

    if !path1.exists() || !path2.exists() {
      return Ok(false);
    }

    let metadata1 = fs::metadata(path1).map_err(|e| {
      EllasticError::IoError(format!(
        "Failed to read metadata for {}: {}",
        path1.display(),
        e
      ))
    })?;

    let metadata2 = fs::metadata(path2).map_err(|e| {
      EllasticError::IoError(format!(
        "Failed to read metadata for {}: {}",
        path2.display(),
        e
      ))
    })?;

    Ok(metadata1.ino() == metadata2.ino())
  }

  pub fn get_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    let content = fs::read(path).map_err(|e| {
      EllasticError::IoError(format!("Failed to read file {}: {}", path.display(), e))
    })?;

    use sha2::{
      Digest,
      Sha256,
    };
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
  }

  pub fn backup_file<P: AsRef<Path>>(path: P, backup_suffix: &str) -> Result<PathBuf> {
    let path = path.as_ref();
    let backup_path = path.with_extension(format!(
      "{}.{}",
      path.extension().and_then(|s| s.to_str()).unwrap_or("bak"),
      backup_suffix
    ));

    fs::copy(path, &backup_path)
      .map_err(|e| EllasticError::IoError(format!("Failed to backup file: {}", e)))?;

    Ok(backup_path)
  }

  pub fn restore_from_backup<P: AsRef<Path>>(original_path: P, backup_path: P) -> Result<()> {
    let original_path = original_path.as_ref();
    let backup_path = backup_path.as_ref();

    if !backup_path.exists() {
      return Err(EllasticError::FileNotFound(
        backup_path.to_string_lossy().to_string(),
      ));
    }

    fs::copy(backup_path, original_path)
      .map_err(|e| EllasticError::IoError(format!("Failed to restore from backup: {}", e)))?;

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
  pub path: PathBuf,
  pub size: u64,
  pub modified: std::time::SystemTime,
  pub created: Option<std::time::SystemTime>,
  pub is_directory: bool,
  pub is_readonly: bool,
  pub extension: Option<String>,
}

impl FileMetadata {
  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
    let path = path.as_ref();
    let metadata = fs::metadata(path)
      .map_err(|e| EllasticError::IoError(format!("Failed to read metadata: {}", e)))?;

    Self::from_path_and_metadata(path, &metadata)
  }

  pub fn from_path_and_metadata<P: AsRef<Path>>(path: P, metadata: &fs::Metadata) -> Result<Self> {
    let path = path.as_ref();

    Ok(Self {
      path: path.to_path_buf(),
      size: metadata.len(),
      modified: metadata
        .modified()
        .map_err(|e| EllasticError::IoError(format!("Failed to get modified time: {}", e)))?,
      created: metadata.created().ok(),
      is_directory: metadata.is_dir(),
      is_readonly: metadata.permissions().readonly(),
      extension: path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string()),
    })
  }

  pub fn size_mb(&self) -> f64 {
    self.size as f64 / (1024.0 * 1024.0)
  }

  pub fn size_gb(&self) -> f64 {
    self.size as f64 / (1024.0 * 1024.0 * 1024.0)
  }

  pub fn is_image(&self) -> bool {
    match self.extension.as_deref() {
      Some("png") | Some("jpg") | Some("jpeg") | Some("bmp") | Some("gif") | Some("tiff") => true,
      _ => false,
    }
  }

  pub fn is_audio(&self) -> bool {
    match self.extension.as_deref() {
      Some("wav") | Some("mp3") | Some("flac") | Some("ogg") => true,
      _ => false,
    }
  }

  pub fn is_video(&self) -> bool {
    match self.extension.as_deref() {
      Some("mp4") | Some("avi") | Some("mkv") | Some("mov") | Some("wmv") => true,
      _ => false,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DirectoryScanner {
  max_depth: Option<usize>,
  follow_links: bool,
  include_hidden: bool,
}

impl DirectoryScanner {
  pub fn new() -> Self {
    Self {
      max_depth: None,
      follow_links: false,
      include_hidden: false,
    }
  }

  pub fn max_depth(mut self, depth: usize) -> Self {
    self.max_depth = Some(depth);
    self
  }

  pub fn follow_links(mut self, follow: bool) -> Self {
    self.follow_links = follow;
    self
  }

  pub fn include_hidden(mut self, include: bool) -> Self {
    self.include_hidden = include;
    self
  }

  pub fn scan<P: AsRef<Path>>(&self, path: P) -> Result<Vec<FileMetadata>> {
    let path = path.as_ref();
    let mut results = Vec::new();

    let mut walk_dir = WalkDir::new(path);

    if let Some(max_depth) = self.max_depth {
      walk_dir = walk_dir.max_depth(max_depth);
    }

    if !self.follow_links {
      walk_dir = walk_dir.follow_links(false);
    }

    for entry in walk_dir.into_iter().filter_map(|e| e.ok()) {
      if !self.include_hidden {
        if let Some(file_name) = entry.file_name().to_str() {
          if file_name.starts_with('.') {
            continue;
          }
        }
      }

      let metadata = FileMetadata::from_path_and_metadata(
        entry.path(),
        &entry.metadata().map_err(|e| EllasticError::IoError(e.to_string()))?,
      )?;
      results.push(metadata);
    }

    Ok(results)
  }

  pub fn scan_files_only<P: AsRef<Path>>(&self, path: P) -> Result<Vec<FileMetadata>> {
    let files = self.scan(path)?;
    Ok(files.into_iter().filter(|m| !m.is_directory).collect())
  }

  pub fn scan_directories_only<P: AsRef<Path>>(&self, path: P) -> Result<Vec<FileMetadata>> {
    let files = self.scan(path)?;
    Ok(files.into_iter().filter(|m| m.is_directory).collect())
  }
}

impl Default for DirectoryScanner {
  fn default() -> Self {
    Self::new()
  }
}

pub fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<()> {
  FileOperations::ensure_directory_exists(path)
}

pub fn copy_file<S: AsRef<Path>, D: AsRef<Path>>(source: S, destination: D) -> Result<u64> {
  FileOperations::copy_file_with_progress(source, destination, None)
}

pub fn move_file<S: AsRef<Path>, D: AsRef<Path>>(source: S, destination: D) -> Result<()> {
  FileOperations::move_file(source, destination)
}

pub fn delete_file<P: AsRef<Path>>(path: P) -> Result<FileMetadata> {
  FileOperations::delete_file_with_metadata(path)
}

pub fn get_file_metadata<P: AsRef<Path>>(path: P) -> Result<FileMetadata> {
  FileMetadata::from_path(path)
}

pub fn scan_directory<P: AsRef<Path>>(path: P) -> Result<Vec<FileMetadata>> {
  DirectoryScanner::new().scan(path)
}
