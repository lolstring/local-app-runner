//! Configuration management for LARS
//!
//! Handles loading, saving, and managing the application configuration
//! with support for atomic saves and platform-specific paths.

use crate::error::{ConfigError, LarsError, Result};
use crate::models::{AppConfig, CURRENT_CONFIG_VERSION};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration file name
const CONFIG_FILE_NAME: &str = "config.json";

/// Manages the application configuration
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Directory where config file is stored
    config_dir: PathBuf,
    /// Directory where log files are stored
    log_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager with explicit directories.
    ///
    /// This is the primary constructor, supporting dependency injection
    /// for testing without environment variable manipulation.
    pub fn new(config_dir: PathBuf, log_dir: PathBuf) -> Self {
        Self {
            config_dir,
            log_dir,
        }
    }

    /// Create a ConfigManager using platform-specific default directories.
    ///
    /// - macOS: ~/Library/Application Support/lars/
    /// - Linux: $XDG_CONFIG_HOME/lars/ (config), $XDG_STATE_HOME/lars/logs/ (logs)
    /// - Windows: %APPDATA%\lars\
    pub fn with_defaults() -> std::result::Result<Self, ConfigError> {
        // Check for override environment variable (useful for CLI testing)
        if let Ok(override_path) = std::env::var("LARS_CONFIG_HOME") {
            let base = PathBuf::from(override_path);
            return Ok(Self {
                config_dir: base.clone(),
                log_dir: base.join("logs"),
            });
        }

        let project_dirs =
            ProjectDirs::from("", "", "lars").ok_or(ConfigError::NoConfigDirectory)?;

        let config_dir = project_dirs.config_dir().to_path_buf();

        // On Linux, use state dir for logs; otherwise use config dir
        let log_dir = project_dirs
            .state_dir()
            .map(|p| p.join("logs"))
            .unwrap_or_else(|| config_dir.join("logs"));

        Ok(Self {
            config_dir,
            log_dir,
        })
    }

    /// Get the path to the config file
    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join(CONFIG_FILE_NAME)
    }

    /// Get the config directory
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get the log directory
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    /// Get the log file path for a service
    pub fn log_path_for_service(&self, service_id: &uuid::Uuid) -> PathBuf {
        self.log_dir.join(format!("{}.log", service_id))
    }

    /// Ensure all required directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.log_dir)?;
        Ok(())
    }

    /// Load the configuration from disk.
    ///
    /// If the config file doesn't exist, returns a default configuration.
    pub fn load(&self) -> Result<AppConfig> {
        let config_path = self.config_path();

        if !config_path.exists() {
            return Ok(AppConfig::default());
        }

        let contents = fs::read_to_string(&config_path)?;
        let mut config: AppConfig =
            serde_json::from_str(&contents).map_err(ConfigError::ParseError)?;

        self.migrate(&mut config)?;

        Ok(config)
    }

    /// Save the configuration to disk atomically.
    ///
    /// Uses a write-to-temp-then-rename strategy to prevent corruption
    /// if the process is interrupted during write.
    pub fn save(&self, config: &AppConfig) -> Result<()> {
        self.ensure_directories()?;

        let config_path = self.config_path();
        let temp_path = config_path.with_extension("json.tmp");

        let contents = serde_json::to_string_pretty(config).map_err(ConfigError::ParseError)?;
        fs::write(&temp_path, contents)?;
        fs::rename(&temp_path, &config_path)?;

        Ok(())
    }

    fn migrate(&self, config: &mut AppConfig) -> Result<()> {
        if config.config_version < CURRENT_CONFIG_VERSION {
            config.config_version = CURRENT_CONFIG_VERSION;
        }
        Ok(())
    }

    /// Check if the config directory is writable
    pub fn is_config_dir_writable(&self) -> bool {
        if self.ensure_directories().is_err() {
            return false;
        }

        let test_file = self.config_dir.join(".write_test");
        if fs::write(&test_file, "test").is_ok() {
            let _ = fs::remove_file(&test_file);
            true
        } else {
            false
        }
    }

    /// Check if the log directory is writable
    pub fn is_log_dir_writable(&self) -> bool {
        if self.ensure_directories().is_err() {
            return false;
        }

        let test_file = self.log_dir.join(".write_test");
        if fs::write(&test_file, "test").is_ok() {
            let _ = fs::remove_file(&test_file);
            true
        } else {
            false
        }
    }
}

/// High-level service management operations
impl ConfigManager {
    /// Add a service to the configuration
    pub fn add_service(&self, service: crate::models::Service) -> Result<()> {
        let mut config = self.load()?;

        if config.service_name_exists(&service.name) {
            return Err(LarsError::ServiceAlreadyExists(service.name));
        }

        config.add_service(service);
        self.save(&config)?;

        Ok(())
    }

    /// Remove a service from the configuration
    pub fn remove_service(&self, name: &str) -> Result<crate::models::Service> {
        let mut config = self.load()?;

        let service = config
            .remove_service_by_name(name)
            .ok_or_else(|| LarsError::ServiceNotFound(name.to_string()))?;

        self.save(&config)?;

        Ok(service)
    }

    /// Get a service by name
    pub fn get_service(&self, name: &str) -> Result<crate::models::Service> {
        let config = self.load()?;

        config
            .find_service_by_name(name)
            .cloned()
            .ok_or_else(|| LarsError::ServiceNotFound(name.to_string()))
    }

    /// Update a service in the configuration
    pub fn update_service<F>(&self, name: &str, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut crate::models::Service),
    {
        let mut config = self.load()?;

        let service = config
            .find_service_by_name_mut(name)
            .ok_or_else(|| LarsError::ServiceNotFound(name.to_string()))?;

        update_fn(service);
        service.touch();

        self.save(&config)?;

        Ok(())
    }

    /// List all services
    pub fn list_services(&self) -> Result<Vec<crate::models::Service>> {
        let config = self.load()?;
        Ok(config.services)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Service;
    use tempfile::TempDir;

    fn test_config_manager() -> (ConfigManager, TempDir) {
        let temp = TempDir::new().unwrap();
        let config_dir = temp.path().join("config");
        let log_dir = temp.path().join("logs");

        let manager = ConfigManager::new(config_dir, log_dir);
        (manager, temp)
    }

    #[test]
    fn test_config_round_trip() {
        let (manager, _temp) = test_config_manager();

        let mut config = AppConfig::default();
        config.add_service(Service::new("test".to_string(), "echo hello".to_string()));

        manager.save(&config).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(config.services.len(), loaded.services.len());
        assert_eq!(config.services[0].name, loaded.services[0].name);
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let (manager, _temp) = test_config_manager();

        let config = manager.load().unwrap();
        assert!(config.services.is_empty());
        assert_eq!(config.config_version, CURRENT_CONFIG_VERSION);
    }

    #[test]
    fn test_atomic_save() {
        let (manager, _temp) = test_config_manager();

        let config = AppConfig::default();
        manager.save(&config).unwrap();

        let temp_path = manager.config_path().with_extension("json.tmp");
        assert!(!temp_path.exists());
        assert!(manager.config_path().exists());
    }

    #[test]
    fn test_ensure_directories() {
        let (manager, _temp) = test_config_manager();

        manager.ensure_directories().unwrap();

        assert!(manager.config_dir().exists());
        assert!(manager.log_dir().exists());
    }

    #[test]
    fn test_add_service() {
        let (manager, _temp) = test_config_manager();

        let service = Service::new("test".to_string(), "echo hello".to_string());
        manager.add_service(service).unwrap();

        let services = manager.list_services().unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test");
    }

    #[test]
    fn test_add_duplicate_service_fails() {
        let (manager, _temp) = test_config_manager();

        let service1 = Service::new("test".to_string(), "echo hello".to_string());
        let service2 = Service::new("test".to_string(), "echo world".to_string());

        manager.add_service(service1).unwrap();
        let result = manager.add_service(service2);

        assert!(matches!(result, Err(LarsError::ServiceAlreadyExists(_))));
    }

    #[test]
    fn test_remove_service() {
        let (manager, _temp) = test_config_manager();

        let service = Service::new("test".to_string(), "echo hello".to_string());
        manager.add_service(service).unwrap();

        let removed = manager.remove_service("test").unwrap();
        assert_eq!(removed.name, "test");

        let services = manager.list_services().unwrap();
        assert!(services.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_service_fails() {
        let (manager, _temp) = test_config_manager();

        let result = manager.remove_service("nonexistent");
        assert!(matches!(result, Err(LarsError::ServiceNotFound(_))));
    }

    #[test]
    fn test_update_service() {
        let (manager, _temp) = test_config_manager();

        let service = Service::new("test".to_string(), "echo hello".to_string());
        manager.add_service(service).unwrap();

        manager
            .update_service("test", |s| {
                s.enabled = false;
            })
            .unwrap();

        let service = manager.get_service("test").unwrap();
        assert!(!service.enabled);
    }

    #[test]
    fn test_log_path_for_service() {
        let (manager, _temp) = test_config_manager();

        let id = uuid::Uuid::new_v4();
        let log_path = manager.log_path_for_service(&id);

        assert!(log_path.to_string_lossy().contains(&id.to_string()));
        assert!(log_path.to_string_lossy().ends_with(".log"));
    }
}
