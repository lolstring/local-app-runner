//! LARS Core Library
//!
//! This crate provides the core functionality for LARS (Local App Runner Service),
//! including configuration management, service models, process runners,
//! and input validation.
//!
//! # Example
//!
//! ```no_run
//! use lars_core::{ConfigManager, Service, TmuxRunner, Runner};
//!
//! // Create a config manager with default paths
//! let config = ConfigManager::with_defaults().unwrap();
//!
//! // Add a service
//! let service = Service::new("my-service".to_string(), "python -m http.server".to_string());
//! config.add_service(service).unwrap();
//!
//! // Start the service
//! let runner = TmuxRunner::new();
//! let service = config.get_service("my-service").unwrap();
//! let log_path = config.log_path_for_service(&service.id);
//! runner.start(&service, &log_path).unwrap();
//! ```

pub mod config;
pub mod error;
pub mod models;
pub mod runner;
pub mod validation;

// Re-export commonly used types
pub use config::ConfigManager;
pub use error::{ConfigError, LarsError, Result, ValidationError};
pub use models::{AppConfig, AppSettings, RunnerType, Service, ShutdownBehavior};
pub use runner::{create_runner, Runner, TmuxRunner};
pub use validation::{
    generate_service_name, sanitize_for_shell, validate_not_empty, validate_service_name,
};
