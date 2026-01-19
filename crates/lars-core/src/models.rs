//! Data models for LARS (Local App Runner Service)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Current config version for migrations
pub const CURRENT_CONFIG_VERSION: u32 = 1;

/// A managed service configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    /// Unique identifier (UUID)
    pub id: Uuid,
    /// Display name (validated: alphanumeric, underscore, hyphen only)
    pub name: String,
    /// Shell command to execute
    pub command: String,
    /// Working directory (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// Whether the service is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Whether to start when lar starts
    #[serde(default)]
    pub autostart: bool,
    /// Runner type (tmux, screen, or direct)
    #[serde(default)]
    pub runner_type: RunnerType,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

fn default_true() -> bool {
    true
}

impl Service {
    /// Create a new service with the given name and command
    pub fn new(name: String, command: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            command,
            cwd: None,
            env: HashMap::new(),
            enabled: true,
            autostart: false,
            runner_type: RunnerType::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new("default".to_string(), "echo hello".to_string())
    }
}

/// The type of runner to use for a service
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RunnerType {
    /// Use tmux for session management (default)
    #[default]
    Tmux,
    /// Use screen for session management
    Screen,
    /// Direct process spawn (no interactive attach)
    Direct,
}

impl std::fmt::Display for RunnerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunnerType::Tmux => write!(f, "tmux"),
            RunnerType::Screen => write!(f, "screen"),
            RunnerType::Direct => write!(f, "direct"),
        }
    }
}

impl std::str::FromStr for RunnerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tmux" => Ok(RunnerType::Tmux),
            "screen" => Ok(RunnerType::Screen),
            "direct" => Ok(RunnerType::Direct),
            _ => Err(format!("Invalid runner type: {}", s)),
        }
    }
}

/// Behavior when the application shuts down
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownBehavior {
    /// Stop all running services on shutdown
    #[default]
    StopAll,
    /// Leave services running on shutdown
    LeaveRunning,
}

impl std::fmt::Display for ShutdownBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShutdownBehavior::StopAll => write!(f, "stop_all"),
            ShutdownBehavior::LeaveRunning => write!(f, "leave_running"),
        }
    }
}

impl std::str::FromStr for ShutdownBehavior {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "stop_all" | "stopall" => Ok(ShutdownBehavior::StopAll),
            "leave_running" | "leaverunning" => Ok(ShutdownBehavior::LeaveRunning),
            _ => Err(format!("Invalid shutdown behavior: {}", s)),
        }
    }
}

/// Default restart timeout in seconds
const DEFAULT_RESTART_TIMEOUT_SECS: u64 = 10;

fn default_restart_timeout() -> u64 {
    DEFAULT_RESTART_TIMEOUT_SECS
}

/// Application-wide settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// Default runner type for new services
    #[serde(default)]
    pub default_runner: RunnerType,
    /// Behavior when the application shuts down
    #[serde(default)]
    pub shutdown_behavior: ShutdownBehavior,
    /// Timeout in seconds when waiting for a service to stop during restart
    #[serde(default = "default_restart_timeout")]
    pub restart_timeout_secs: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_runner: RunnerType::default(),
            shutdown_behavior: ShutdownBehavior::default(),
            restart_timeout_secs: DEFAULT_RESTART_TIMEOUT_SECS,
        }
    }
}

/// The main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// Config version for migrations
    #[serde(default = "default_config_version")]
    pub config_version: u32,
    /// List of configured services
    #[serde(default)]
    pub services: Vec<Service>,
    /// Application settings
    #[serde(default)]
    pub settings: AppSettings,
}

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_version: CURRENT_CONFIG_VERSION,
            services: Vec::new(),
            settings: AppSettings::default(),
        }
    }
}

impl AppConfig {
    /// Find a service by name
    pub fn find_service_by_name(&self, name: &str) -> Option<&Service> {
        self.services.iter().find(|s| s.name == name)
    }

    /// Find a service by name (mutable)
    pub fn find_service_by_name_mut(&mut self, name: &str) -> Option<&mut Service> {
        self.services.iter_mut().find(|s| s.name == name)
    }

    /// Find a service by ID
    pub fn find_service_by_id(&self, id: Uuid) -> Option<&Service> {
        self.services.iter().find(|s| s.id == id)
    }

    /// Find a service by ID (mutable)
    pub fn find_service_by_id_mut(&mut self, id: Uuid) -> Option<&mut Service> {
        self.services.iter_mut().find(|s| s.id == id)
    }

    /// Add a service to the config
    pub fn add_service(&mut self, service: Service) {
        self.services.push(service);
    }

    /// Remove a service by name, returning it if found
    pub fn remove_service_by_name(&mut self, name: &str) -> Option<Service> {
        if let Some(pos) = self.services.iter().position(|s| s.name == name) {
            Some(self.services.remove(pos))
        } else {
            None
        }
    }

    /// Check if a service name already exists
    pub fn service_name_exists(&self, name: &str) -> bool {
        self.services.iter().any(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_new() {
        let service = Service::new("test".to_string(), "echo hello".to_string());
        assert_eq!(service.name, "test");
        assert_eq!(service.command, "echo hello");
        assert!(service.enabled);
        assert!(!service.autostart);
        assert_eq!(service.runner_type, RunnerType::Tmux);
    }

    #[test]
    fn test_runner_type_display_and_parse() {
        assert_eq!(RunnerType::Tmux.to_string(), "tmux");
        assert_eq!(RunnerType::Screen.to_string(), "screen");
        assert_eq!(RunnerType::Direct.to_string(), "direct");

        assert_eq!("tmux".parse::<RunnerType>().unwrap(), RunnerType::Tmux);
        assert_eq!("TMUX".parse::<RunnerType>().unwrap(), RunnerType::Tmux);
        assert!("invalid".parse::<RunnerType>().is_err());
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.config_version, CURRENT_CONFIG_VERSION);
        assert!(config.services.is_empty());
    }

    #[test]
    fn test_app_config_service_operations() {
        let mut config = AppConfig::default();
        let service = Service::new("test".to_string(), "echo hello".to_string());

        config.add_service(service);
        assert!(config.service_name_exists("test"));
        assert!(config.find_service_by_name("test").is_some());

        let removed = config.remove_service_by_name("test");
        assert!(removed.is_some());
        assert!(!config.service_name_exists("test"));
    }

    #[test]
    fn test_serde_round_trip() {
        let mut config = AppConfig::default();
        config.add_service(Service::new("test".to_string(), "echo hello".to_string()));

        let json = serde_json::to_string_pretty(&config).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, loaded);
    }
}
