//! Error types for LARS (Local App Runner Service)

use thiserror::Error;

/// Main error type for lars-core operations
#[derive(Error, Debug)]
pub enum LarsError {
    /// Service not found by name or ID
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// Service already exists with the given name
    #[error("Service already exists: {0}")]
    ServiceAlreadyExists(String),

    /// Runner is not available (e.g., tmux not installed)
    #[error("Runner not available: {0}")]
    RunnerNotAvailable(String),

    /// Runner does not support the requested operation
    #[error("Operation not supported by runner: {0}")]
    OperationNotSupported(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Configuration error
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Service stop timed out
    #[error("Timeout waiting for service to stop: {0}")]
    StopTimeout(String),

    /// Invalid path (e.g., non-UTF8)
    #[error("Invalid path")]
    InvalidPath,

    /// Process execution failed
    #[error("Process execution failed: {0}")]
    ProcessFailed(String),
}

/// Validation errors for input sanitization
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Name length is invalid (must be 1-64 characters)
    #[error("Name must be 1-64 characters, got {0}")]
    InvalidNameLength(usize),

    /// Name contains invalid characters
    #[error("Name can only contain alphanumeric characters, underscores, and hyphens")]
    InvalidNameCharacters,

    /// Input contains null byte
    #[error("Input contains null byte")]
    NullByteInInput,

    /// Empty input where non-empty is required
    #[error("Input cannot be empty")]
    EmptyInput,
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to determine config directory
    #[error("Could not determine config directory")]
    NoConfigDirectory,

    /// Failed to parse config file
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Config directory is not writable
    #[error("Config directory is not writable: {0}")]
    NotWritable(String),

    /// IO error during config operations
    #[error("Config IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for lars-core operations
pub type Result<T> = std::result::Result<T, LarsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = LarsError::ServiceNotFound("test".to_string());
        assert_eq!(err.to_string(), "Service not found: test");

        let err = LarsError::Validation(ValidationError::InvalidNameCharacters);
        assert!(err.to_string().contains("alphanumeric"));
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::InvalidNameLength(100);
        assert!(err.to_string().contains("1-64"));

        let err = ValidationError::NullByteInInput;
        assert!(err.to_string().contains("null"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let lar_err: LarsError = io_err.into();
        assert!(matches!(lar_err, LarsError::Io(_)));
    }
}
