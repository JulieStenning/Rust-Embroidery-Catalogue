// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    InvalidInput {
        message: String,
    },
    NotFound {
        resource: &'static str,
        id: Option<String>,
    },
    Database {
        message: String,
    },
    Io {
        message: String,
    },
    Parse {
        message: String,
    },
    Unsupported {
        message: String,
    },
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
        let msg = message.into();
        let enriched = crate::database::error_diagnostics::enrich_db_error_message(&msg, None);
        Self::Database { message: enriched }
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

/// Structured, serializable error enum exchanged over the Tauri IPC boundary.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, thiserror::Error)]
#[serde(tag = "code", content = "message", rename_all = "snake_case")]
pub enum IpcError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Unsupported: {0}")]
    Unsupported(String),

    #[error("Operation cancelled: {0}")]
    Cancelled(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<AppError> for IpcError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::InvalidInput { message } => IpcError::InvalidInput(message),
            AppError::NotFound { resource, id } => match id {
                Some(id) => IpcError::NotFound(format!("{resource}: {id}")),
                None => IpcError::NotFound(resource.to_string()),
            },
            AppError::Database { message } => IpcError::Database(message),
            AppError::Io { message } => IpcError::Io(message),
            AppError::Parse { message } => IpcError::Parse(message),
            AppError::Unsupported { message } => IpcError::Unsupported(message),
        }
    }
}

impl From<sqlx::Error> for IpcError {
    fn from(err: sqlx::Error) -> Self {
        let msg = err.to_string();
        let enriched = crate::database::error_diagnostics::enrich_db_error_message(&msg, None);
        IpcError::Database(enriched)
    }
}

impl From<std::io::Error> for IpcError {
    fn from(err: std::io::Error) -> Self {
        IpcError::Io(err.to_string())
    }
}

impl From<String> for IpcError {
    fn from(msg: String) -> Self {
        IpcError::Internal(msg)
    }
}

impl From<&str> for IpcError {
    fn from(msg: &str) -> Self {
        IpcError::Internal(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{AppError, IpcError};

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
    fn app_error_database_sqlite_full_enriched() {
        let err = AppError::database("error: (code: 13) database or disk is full");
        let display = err.to_string();
        assert!(display.contains("database or disk is full"));
        assert!(display.contains("Your catalogue data root maybe on a FAT32-formatted drive"));
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

    #[test]
    fn ipc_error_from_app_error_converts_variants() {
        let app_err = AppError::invalid_input("invalid field");
        let ipc_err: IpcError = app_err.into();
        assert_eq!(ipc_err, IpcError::InvalidInput("invalid field".to_string()));

        let app_not_found = AppError::not_found("design", Some("10".to_string()));
        let ipc_not_found: IpcError = app_not_found.into();
        assert_eq!(ipc_not_found, IpcError::NotFound("design: 10".to_string()));
    }

    #[test]
    fn ipc_error_serializes_with_code_and_message() {
        let err = IpcError::Database("disk full".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#"{"code":"database","message":"disk full"}"#);
    }
}
