// Validation service contract for path and input safety checks.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyPath,
    NotAbsolute,
    OutsideBasePath,
    DoesNotExist,
}

pub fn validate_path(path: &str) -> Result<(), ValidationError> {
    if path.trim().is_empty() {
        return Err(ValidationError::EmptyPath);
    }

    if !std::path::Path::new(path).is_absolute() {
        return Err(ValidationError::NotAbsolute);
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
