// Validation service contract for path and input safety checks.

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyPath,
    NotAbsolute,
    PathDoesNotExist,
    NotADirectory,
    OutsideBasePath,
    DoesNotExist,
}

/// Validate that a path is non-empty and absolute.
///
/// Existence is intentionally NOT checked here to preserve the original
/// contract used by callers that only need shape validation. Use
/// `validate_directory` when existence is required.
pub fn validate_path(path: &str) -> Result<(), ValidationError> {
    if path.trim().is_empty() {
        return Err(ValidationError::EmptyPath);
    }

    let path_obj = std::path::Path::new(path);
    if !path_obj.is_absolute() {
        return Err(ValidationError::NotAbsolute);
    }

    Ok(())
}

/// Validate that a path is a non-empty absolute directory that exists on disk.
///
/// Returns `PathDoesNotExist` when the path does not exist on disk, and
/// `NotADirectory` when the path exists but is not a directory.
pub fn validate_directory(path: &str) -> Result<(), ValidationError> {
    validate_path(path)?;

    let path_obj = std::path::Path::new(path);
    if !path_obj.exists() {
        return Err(ValidationError::PathDoesNotExist);
    }

    if !path_obj.is_dir() {
        return Err(ValidationError::NotADirectory);
    }

    Ok(())
}

pub fn validate_under_base(path: &str, base_path: &str) -> Result<(), ValidationError> {
    validate_path(path)?;
    validate_path(base_path)?;

    if !std::path::Path::new(path).starts_with(base_path) {
        return Err(ValidationError::OutsideBasePath);
    }

    if !std::path::Path::new(path).exists() {
        return Err(ValidationError::DoesNotExist);
    }

    Ok(())
}

pub fn validate_path_to_app_error(path: &str) -> Result<(), AppError> {
    validate_path(path).map_err(|err| match err {
        ValidationError::EmptyPath => AppError::invalid_input("path must not be empty"),
        ValidationError::NotAbsolute => AppError::invalid_input("path must be absolute"),
        _ => AppError::invalid_input(format!("invalid path: {path}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_validate_path_empty() {
        assert_eq!(validate_path(""), Err(ValidationError::EmptyPath));
        assert_eq!(validate_path("   "), Err(ValidationError::EmptyPath));
    }

    #[test]
    fn test_validate_path_not_absolute() {
        assert_eq!(validate_path("relative/path"), Err(ValidationError::NotAbsolute));
        #[cfg(windows)]
        assert_eq!(validate_path(r"relative\path"), Err(ValidationError::NotAbsolute));
    }

    #[test]
    fn test_validate_path_valid() {
        let temp_dir = std::env::temp_dir();
        let temp_path_str = temp_dir.to_str().unwrap();
        assert_eq!(validate_path(temp_path_str), Ok(()));
    }

    #[test]
    fn test_validate_path_does_not_exist() {
        // validate_path intentionally does NOT check existence (shape only).
        let missing = std::env::temp_dir().join("rec-validation-does-not-exist");
        let missing_str = missing.to_str().unwrap();
        // An absolute but non-existent path is still "valid" for shape validation.
        assert_eq!(validate_path(missing_str), Ok(()));
        // validate_directory DOES check existence.
        assert_eq!(
            validate_directory(missing_str),
            Err(ValidationError::PathDoesNotExist)
        );
    }

    #[test]
    fn test_validate_directory_rejects_file_path() {
        let file = std::env::temp_dir().join("rec-validation-file-check.txt");
        let _ = std::fs::write(&file, b"test");
        let file_str = file.to_str().unwrap();
        assert_eq!(
            validate_directory(file_str),
            Err(ValidationError::NotADirectory)
        );
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn test_validate_directory_accepts_directory() {
        let dir = std::env::temp_dir();
        let dir_str = dir.to_str().unwrap();
        assert_eq!(validate_directory(dir_str), Ok(()));
    }

    #[test]
    fn test_validate_under_base_invalid_path() {
        let temp_dir = std::env::temp_dir();
        let temp_path_str = temp_dir.to_str().unwrap();

        assert_eq!(validate_under_base("", temp_path_str), Err(ValidationError::EmptyPath));
        assert_eq!(validate_under_base("relative/path", temp_path_str), Err(ValidationError::NotAbsolute));
    }

    #[test]
    fn test_validate_under_base_invalid_base() {
        let temp_dir = std::env::temp_dir();
        let temp_path_str = temp_dir.to_str().unwrap();

        assert_eq!(validate_under_base(temp_path_str, ""), Err(ValidationError::EmptyPath));
        assert_eq!(validate_under_base(temp_path_str, "relative/path"), Err(ValidationError::NotAbsolute));
    }

    #[test]
    fn test_validate_under_base_outside() {
        let temp_dir = std::env::temp_dir();
        let base_path = temp_dir.join("embroidery_test_base");
        let other_path = temp_dir.join("embroidery_test_other");

        let base_path_str = base_path.to_str().unwrap();
        let other_path_str = other_path.to_str().unwrap();

        assert_eq!(validate_under_base(other_path_str, base_path_str), Err(ValidationError::OutsideBasePath));
    }

    #[test]
    fn test_validate_under_base_does_not_exist() {
        let temp_dir = std::env::temp_dir();
        let base_path = temp_dir.join("embroidery_test_base_nonexistent");
        let file_path = base_path.join("file_that_does_not_exist.txt");

        let base_path_str = base_path.to_str().unwrap();
        let file_path_str = file_path.to_str().unwrap();

        assert_eq!(validate_under_base(file_path_str, base_path_str), Err(ValidationError::DoesNotExist));
    }

    #[test]
    fn test_validate_under_base_success() {
        let temp_dir = std::env::temp_dir();
        
        let base_path = temp_dir.join("embroidery_test_base_success");
        std::fs::create_dir_all(&base_path).unwrap();

        let file_path = base_path.join("existing_file.txt");
        {
            let _file = File::create(&file_path).unwrap();
        }

        let base_path_str = base_path.to_str().unwrap();
        let file_path_str = file_path.to_str().unwrap();

        assert_eq!(validate_under_base(file_path_str, base_path_str), Ok(()));

        let _ = std::fs::remove_file(file_path);
        let _ = std::fs::remove_dir(base_path);
    }
}

