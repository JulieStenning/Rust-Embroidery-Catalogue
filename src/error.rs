use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    InvalidInput { message: String },
    NotFound { resource: &'static str, id: Option<String> },
    Database { message: String },
    Io { message: String },
    Parse { message: String },
    Unsupported { message: String },
}

impl AppError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    pub fn not_found(resource: &'static str, id: impl Into<Option<String>>) -> Self {
        Self::NotFound {
            resource,
            id: id.into(),
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
        }
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported {
            message: message.into(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { message } => write!(f, "invalid input: {message}"),
            Self::NotFound { resource, id } => match id {
                Some(id) => write!(f, "{resource} not found: {id}"),
                None => write!(f, "{resource} not found"),
            },
            Self::Database { message } => write!(f, "database error: {message}"),
            Self::Io { message } => write!(f, "i/o error: {message}"),
            Self::Parse { message } => write!(f, "parse error: {message}"),
            Self::Unsupported { message } => write!(f, "unsupported: {message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::io(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn app_error_display_is_readable() {
        let err = AppError::invalid_input("bad value");
        assert_eq!(err.to_string(), "invalid input: bad value");
    }

    #[test]
    fn app_error_not_found_formats_id() {
        let err = AppError::not_found("design", Some("42".to_string()));
        assert_eq!(err.to_string(), "design not found: 42");
    }

    #[test]
    fn app_error_not_found_formats_without_id() {
        let err = AppError::not_found("design", None::<String>);
        assert_eq!(err.to_string(), "design not found");
    }

    #[test]
    fn app_error_database_display() {
        let err = AppError::database("connection refused");
        assert_eq!(err.to_string(), "database error: connection refused");
    }

    #[test]
    fn app_error_io_display() {
        let err = AppError::io("permission denied");
        assert_eq!(err.to_string(), "i/o error: permission denied");
    }

    #[test]
    fn app_error_parse_display() {
        let err = AppError::parse("invalid header");
        assert_eq!(err.to_string(), "parse error: invalid header");
    }

    #[test]
    fn app_error_unsupported_display() {
        let err = AppError::unsupported("format xyz");
        assert_eq!(err.to_string(), "unsupported: format xyz");
    }

    #[test]
    fn app_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: AppError = io_err.into();
        assert_eq!(err, AppError::io("access denied"));
    }
}
