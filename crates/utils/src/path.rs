use ellastic_errors::{
  EllasticError,
  Result,
};
use std::path::{
  Component,
  Path,
  PathBuf,
};

#[derive(Debug, Clone)]
pub struct PathUtils;

impl PathUtils {
  pub fn normalize<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut result = PathBuf::new();

    for component in path.components() {
      match component {
        Component::RootDir => {
          result.clear();
          result.push(component);
        }
        Component::Normal(name) => {
          if name == "." {
            continue;
          } else if name == ".." {
            if result.parent().is_some() && !result.ends_with("..") {
              result.pop();
            } else {
              result.push(component);
            }
          } else {
            result.push(component);
          }
        }
        Component::CurDir => continue,
        Component::ParentDir => {
          if result.parent().is_some() && !result.ends_with("..") {
            result.pop();
          } else {
            result.push(component);
          }
        }
        Component::Prefix(_) => {
          result.push(component);
        }
      }
    }

    result
  }

  pub fn join_normalized<P: AsRef<Path>, Q: AsRef<Path>>(base: P, path: Q) -> PathBuf {
    let base = base.as_ref();
    let path = path.as_ref();

    if path.is_absolute() {
      Self::normalize(path)
    } else {
      Self::normalize(base.join(path))
    }
  }

  pub fn relative_to<P: AsRef<Path>, Q: AsRef<Path>>(path: P, base: Q) -> Result<PathBuf> {
    let path = Self::normalize(path);
    let base = Self::normalize(base);

    let mut path_components: Vec<_> = path.components().collect();
    let mut base_components: Vec<_> = base.components().collect();

    let mut common_len = 0;
    let min_len = std::cmp::min(path_components.len(), base_components.len());

    for i in 0..min_len {
      if path_components[i] == base_components[i] {
        common_len += 1;
      } else {
        break;
      }
    }

    if common_len == 0 {
      return Err(EllasticError::InvalidParameter(
        "No common path components found".to_string(),
      ));
    }

    let mut result = PathBuf::new();

    for _ in common_len..base_components.len() {
      result.push("..");
    }

    for component in path_components.iter().skip(common_len) {
      result.push(component);
    }

    Ok(result)
  }

  pub fn ensure_extension<P: AsRef<Path>>(path: P, extension: &str) -> PathBuf {
    let path = path.as_ref();

    if let Some(current_ext) = path.extension() {
      if current_ext.to_str() == Some(extension) {
        return path.to_path_buf();
      }
    }

    path.with_extension(extension)
  }

  pub fn replace_extension<P: AsRef<Path>>(path: P, new_extension: &str) -> PathBuf {
    let path = path.as_ref();
    path.with_extension(new_extension)
  }

  pub fn append_to_filename<P: AsRef<Path>>(path: P, suffix: &str) -> PathBuf {
    let path = path.as_ref();

    if let Some(file_stem) = path.file_stem() {
      let new_stem = format!("{}{}", file_stem.to_string_lossy(), suffix);
      path
        .with_file_name(new_stem)
        .with_extension(path.extension().and_then(|s| s.to_str()).unwrap_or(""))
    } else {
      path.to_path_buf()
    }
  }

  pub fn prepend_to_filename<P: AsRef<Path>>(path: P, prefix: &str) -> PathBuf {
    let path = path.as_ref();

    if let Some(file_stem) = path.file_stem() {
      let new_stem = format!("{}{}", prefix, file_stem.to_string_lossy());
      path
        .with_file_name(new_stem)
        .with_extension(path.extension().and_then(|s| s.to_str()).unwrap_or(""))
    } else {
      path.to_path_buf()
    }
  }

  pub fn get_unique_filename<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    if !path.exists() {
      return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let extension = path.extension().and_then(|s| s.to_str());

    let mut counter = 1;
    loop {
      let new_name = if let Some(ext) = extension {
        format!("{}_{}.{}", file_stem, counter, ext)
      } else {
        format!("{}_{}", file_stem, counter)
      };

      let new_path = parent.join(new_name);
      if !new_path.exists() {
        return new_path;
      }

      counter += 1;
      if counter > 9999 {
        break;
      }
    }

    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();

    if let Some(ext) = extension {
      parent.join(format!("{}_{}.{}", file_stem, timestamp, ext))
    } else {
      parent.join(format!("{}_{}", file_stem, timestamp))
    }
  }

  pub fn split_path<P: AsRef<Path>>(path: P) -> (Option<PathBuf>, String, Option<String>) {
    let path = path.as_ref();

    let parent = path.parent().map(|p| p.to_path_buf());
    let filename = path
      .file_name()
      .and_then(|s| s.to_str())
      .unwrap_or("")
      .to_string();
    let extension = path
      .extension()
      .and_then(|s| s.to_str())
      .map(|s| s.to_string());

    (parent, filename, extension)
  }

  pub fn is_subpath<P: AsRef<Path>, Q: AsRef<Path>>(path: P, base: Q) -> bool {
    let path = Self::normalize(path);
    let base = Self::normalize(base);

    path.starts_with(base)
  }

  pub fn common_path<P: AsRef<Path>, Q: AsRef<Path>>(path1: P, path2: Q) -> PathBuf {
    let path1 = Self::normalize(path1);
    let path2 = Self::normalize(path2);

    let mut result = PathBuf::new();
    let mut components1 = path1.components();
    let mut components2 = path2.components();

    loop {
      match (components1.next(), components2.next()) {
        (Some(comp1), Some(comp2)) if comp1 == comp2 => {
          result.push(comp1);
        }
        _ => break,
      }
    }

    result
  }

  pub fn expand_user<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    if path.starts_with("~") {
      if let Some(home_dir) = dirs::home_dir() {
        let path_str = path.to_string_lossy();
        if path_str == "~" {
          return home_dir;
        } else if path_str.starts_with("~/") {
          return home_dir.join(&path_str[2..]);
        }
      }
    }

    path.to_path_buf()
  }

  pub fn shrink_user<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    if let Some(home_dir) = dirs::home_dir() {
      if let Ok(relative) = Self::relative_to(path, &home_dir) {
        return PathBuf::from("~").join(relative);
      }
    }

    path.to_path_buf()
  }

  pub fn make_absolute<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
      Ok(Self::normalize(path))
    } else {
      let current_dir = std::env::current_dir()
        .map_err(|e| EllasticError::IoError(format!("Failed to get current directory: {}", e)))?;
      Ok(Self::normalize(current_dir.join(path)))
    }
  }

  pub fn find_common_base<P: AsRef<Path>>(paths: &[P]) -> Option<PathBuf> {
    if paths.is_empty() {
      return None;
    }

    let mut common_base = Self::normalize(paths[0].as_ref());

    for path in paths.iter().skip(1) {
      common_base = Self::common_path(&common_base, path.as_ref());

      if common_base.as_os_str().is_empty() {
        return None;
      }
    }

    Some(common_base)
  }

  pub fn ensure_directory_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
      std::fs::create_dir_all(path).map_err(|e| {
        EllasticError::IoError(format!(
          "Failed to create directory {}: {}",
          path.display(),
          e
        ))
      })?;
    }

    Ok(())
  }

  pub fn ensure_parent_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
      Self::ensure_directory_exists(parent)?;
    }

    Ok(())
  }

  pub fn is_safe_path<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    if path.is_absolute() {
      return false;
    }

    for component in path.components() {
      match component {
        Component::ParentDir => return false,
        Component::Normal(name) => {
          if let Some(name_str) = name.to_str() {
            if name_str.is_empty() || name_str.contains("..") {
              return false;
            }
          }
        }
        _ => {}
      }
    }

    true
  }

  pub fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];

    let mut result = String::new();
    let mut last_was_space = false;

    for ch in filename.chars() {
      if invalid_chars.contains(&ch) {
        if !last_was_space && !result.is_empty() {
          result.push(' ');
          last_was_space = true;
        }
      } else if ch.is_control() {
        continue;
      } else {
        result.push(ch);
        last_was_space = false;
      }
    }

    result.trim().to_string()
  }

  pub fn get_temp_filename(prefix: &str, extension: Option<&str>) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_millis();

    let random: u32 = rand::random();

    let filename = format!("{}_{}_{}", prefix, timestamp, random);

    match extension {
      Some(ext) => PathBuf::from(format!("{}.{}", filename, ext)),
      None => PathBuf::from(filename),
    }
  }

  pub fn get_backup_filename<P: AsRef<Path>>(original_path: P) -> PathBuf {
    let original_path = original_path.as_ref();
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();

    Self::append_to_filename(original_path, &format!("_backup_{}", timestamp))
  }

  pub fn path_depth<P: AsRef<Path>>(path: P) -> usize {
    let path = path.as_ref();
    path.components().count()
  }

  pub fn is_hidden<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    if let Some(file_name) = path.file_name() {
      if let Some(name_str) = file_name.to_str() {
        return name_str.starts_with('.');
      }
    }

    false
  }

  pub fn has_extension<P: AsRef<Path>>(path: P, extension: &str) -> bool {
    let path = path.as_ref();
    path
      .extension()
      .and_then(|s| s.to_str())
      .map(|s| s.eq_ignore_ascii_case(extension))
      .unwrap_or(false)
  }

  pub fn change_extension<P: AsRef<Path>>(path: P, new_extension: &str) -> PathBuf {
    let path = path.as_ref();
    path.with_extension(new_extension)
  }

  pub fn add_extension<P: AsRef<Path>>(path: P, extension: &str) -> PathBuf {
    let path = path.as_ref();

    if path.extension().is_some() {
      let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
      let new_filename = format!("{}.{}", filename, extension);
      path.with_file_name(new_filename)
    } else {
      path.with_extension(extension)
    }
  }
}

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
  PathUtils::normalize(path)
}

pub fn join_paths<P: AsRef<Path>, Q: AsRef<Path>>(base: P, path: Q) -> PathBuf {
  PathUtils::join_normalized(base, path)
}

pub fn relative_path<P: AsRef<Path>, Q: AsRef<Path>>(path: P, base: Q) -> Result<PathBuf> {
  PathUtils::relative_to(path, base)
}

pub fn ensure_extension<P: AsRef<Path>>(path: P, extension: &str) -> PathBuf {
  PathUtils::ensure_extension(path, extension)
}

pub fn get_unique_path<P: AsRef<Path>>(path: P) -> PathBuf {
  PathUtils::get_unique_filename(path)
}

pub fn expand_user_path<P: AsRef<Path>>(path: P) -> PathBuf {
  PathUtils::expand_user(path)
}

pub fn make_absolute_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
  PathUtils::make_absolute(path)
}

pub fn sanitize_filename_string(filename: &str) -> String {
  PathUtils::sanitize_filename(filename)
}

pub fn is_path_safe<P: AsRef<Path>>(path: P) -> bool {
  PathUtils::is_safe_path(path)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_normalize_path() {
    assert_eq!(normalize_path("foo/bar/../baz"), PathBuf::from("foo/baz"));

    assert_eq!(
      normalize_path("/foo//bar/./baz"),
      PathBuf::from("/foo/bar/baz")
    );

    assert_eq!(normalize_path("../foo/../bar"), PathBuf::from("../bar"));
  }

  #[test]
  fn test_relative_path() {
    let path = PathBuf::from("/foo/bar/baz");
    let base = PathBuf::from("/foo/bar");

    assert_eq!(relative_path(&path, &base).unwrap(), PathBuf::from("baz"));

    let path = PathBuf::from("/foo/bar/baz");
    let base = PathBuf::from("/foo/qux");

    assert!(relative_path(&path, &base).is_err());
  }

  #[test]
  fn test_ensure_extension() {
    let path = PathBuf::from("test.txt");
    assert_eq!(ensure_extension(&path, "txt"), path);

    let path = PathBuf::from("test");
    assert_eq!(ensure_extension(&path, "txt"), PathBuf::from("test.txt"));
  }

  #[test]
  fn test_get_unique_filename() {
    let path = PathBuf::from("nonexistent.txt");
    assert_eq!(get_unique_path(&path), path);
  }

  #[test]
  fn test_sanitize_filename() {
    assert_eq!(
      sanitize_filename_string("test<file>name.txt"),
      "test file name.txt"
    );
    assert_eq!(
      sanitize_filename_string("normal_file.txt"),
      "normal_file.txt"
    );
    assert_eq!(sanitize_filename_string(""), "");
  }

  #[test]
  fn test_is_path_safe() {
    assert!(is_path_safe("safe/path"));
    assert!(!is_path_safe("../unsafe"));
    assert!(!is_path_safe("/absolute/path"));
  }

  #[test]
  fn test_common_path() {
    let path1 = PathBuf::from("/foo/bar/baz");
    let path2 = PathBuf::from("/foo/bar/qux");

    let common = PathUtils::common_path(&path1, &path2);
    assert_eq!(common, PathBuf::from("/foo/bar"));
  }

  #[test]
  fn test_path_depth() {
    assert_eq!(PathUtils::path_depth("foo/bar/baz"), 3);
    assert_eq!(PathUtils::path_depth("/foo/bar"), 3);
    assert_eq!(PathUtils::path_depth("file.txt"), 1);
  }

  #[test]
  fn test_is_hidden() {
    assert!(PathUtils::is_hidden(".hidden"));
    assert!(!PathUtils::is_hidden("visible"));
    assert!(!PathUtils::is_hidden("visible.txt"));
  }

  #[test]
  fn test_has_extension() {
    assert!(PathUtils::has_extension("file.txt", "txt"));
    assert!(PathUtils::has_extension("file.TXT", "txt"));
    assert!(!PathUtils::has_extension("file.txt", "png"));
    assert!(!PathUtils::has_extension("file", "txt"));
  }
}
