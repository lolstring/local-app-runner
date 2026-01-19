//! Process runner implementations for LARS
//!
//! This module provides the Runner trait and implementations for
//! managing service processes using tmux, screen, or direct spawning.
//!
//! # Security Note
//!
//! Service commands are executed as-is via a shell (`sh -c`). This is by design,
//! as users need the ability to run arbitrary shell commands including pipelines,
//! redirections, and environment variable expansions. The security boundary is at
//! the service name level (validated via [`crate::validation::validate_service_name`]),
//! not the command level.
//!
//! **Users should only add commands they trust**, as commands run with the same
//! privileges as the LARS process itself.

use crate::error::{LarsError, Result};
use crate::models::{RunnerType, Service};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Trait for process runners that manage service lifecycle
pub trait Runner: Send + Sync {
    /// Start a service
    fn start(&self, service: &Service, log_path: &Path) -> Result<()>;

    /// Stop a service
    fn stop(&self, service: &Service) -> Result<()>;

    /// Restart a service (stop then start)
    ///
    /// The `timeout_secs` parameter controls how long to wait for the service
    /// to stop before returning an error. Use `AppSettings::restart_timeout_secs`
    /// for the configured value.
    fn restart(&self, service: &Service, log_path: &Path, timeout_secs: u64) -> Result<()> {
        if self.is_running(service)? {
            self.stop(service)?;

            let start = Instant::now();
            while self.is_running(service)? {
                if start.elapsed() > Duration::from_secs(timeout_secs) {
                    return Err(LarsError::StopTimeout(service.name.clone()));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }

        self.start(service, log_path)
    }

    /// Check if a service is running
    fn is_running(&self, service: &Service) -> Result<bool>;

    /// Get the PID of a running service (if available)
    fn get_pid(&self, service: &Service) -> Result<Option<u32>>;

    /// Get the command to attach to a service's session
    /// Returns None if the runner doesn't support interactive attach
    fn attach_command(&self, service: &Service) -> Result<Option<Vec<String>>>;

    /// Get the runner type
    fn runner_type(&self) -> RunnerType;
}

/// Tmux-based runner for session management
#[derive(Debug, Default)]
pub struct TmuxRunner;

impl TmuxRunner {
    /// Create a new TmuxRunner
    pub fn new() -> Self {
        Self
    }

    /// Generate the tmux session name for a service
    fn session_name(service: &Service) -> String {
        format!("lar_{}", service.id)
    }

    /// Check if tmux is available
    pub fn is_available() -> bool {
        Command::new("tmux")
            .arg("-V")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get tmux version string
    pub fn version() -> Option<String> {
        Command::new("tmux")
            .arg("-V")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }
}

impl Runner for TmuxRunner {
    fn start(&self, service: &Service, log_path: &Path) -> Result<()> {
        if !Self::is_available() {
            return Err(LarsError::RunnerNotAvailable(
                "tmux is not installed or not in PATH".to_string(),
            ));
        }

        let session_name = Self::session_name(service);

        if self.is_running(service)? {
            return Ok(());
        }

        let log_path_str = log_path.to_str().ok_or(LarsError::InvalidPath)?;
        let escaped_log_path = shell_escape::escape(log_path_str.into());
        let shell_cmd = format!("{} > {} 2>&1", &service.command, escaped_log_path);

        let mut cmd = Command::new("tmux");
        cmd.args(["new-session", "-d", "-s", &session_name]);

        if let Some(cwd) = &service.cwd {
            let cwd_str = cwd.to_str().ok_or(LarsError::InvalidPath)?;
            cmd.args(["-c", cwd_str]);
        }

        cmd.args(["sh", "-c", &shell_cmd]);

        for (key, value) in &service.env {
            cmd.env(key, value);
        }

        let status = cmd.status()?;

        if !status.success() {
            return Err(LarsError::ProcessFailed(format!(
                "tmux new-session failed with status: {}",
                status
            )));
        }

        Ok(())
    }

    fn stop(&self, service: &Service) -> Result<()> {
        let session_name = Self::session_name(service);

        let status = Command::new("tmux")
            .args(["kill-session", "-t", &session_name])
            .status()?;

        // It's okay if the session doesn't exist (might have already exited)
        if !status.success() && self.is_running(service)? {
            return Err(LarsError::ProcessFailed(format!(
                "tmux kill-session failed with status: {}",
                status
            )));
        }

        Ok(())
    }

    fn is_running(&self, service: &Service) -> Result<bool> {
        let session_name = Self::session_name(service);

        let status = Command::new("tmux")
            .args(["has-session", "-t", &session_name])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()?;

        Ok(status.success())
    }

    fn get_pid(&self, service: &Service) -> Result<Option<u32>> {
        let session_name = Self::session_name(service);

        let output = Command::new("tmux")
            .args([
                "list-panes",
                "-t",
                &session_name,
                "-F",
                "#{pane_pid}",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let pid_str = String::from_utf8_lossy(&output.stdout);
        let pid = pid_str.trim().parse::<u32>().ok();

        Ok(pid)
    }

    fn attach_command(&self, service: &Service) -> Result<Option<Vec<String>>> {
        let session_name = Self::session_name(service);

        Ok(Some(vec![
            "tmux".to_string(),
            "attach".to_string(),
            "-t".to_string(),
            session_name,
        ]))
    }

    fn runner_type(&self) -> RunnerType {
        RunnerType::Tmux
    }
}

/// Create a runner for the specified type
pub fn create_runner(runner_type: RunnerType) -> Result<Box<dyn Runner>> {
    match runner_type {
        RunnerType::Tmux => {
            if !TmuxRunner::is_available() {
                return Err(LarsError::RunnerNotAvailable(
                    "tmux is not installed or not in PATH".to_string(),
                ));
            }
            Ok(Box::new(TmuxRunner::new()))
        }
        RunnerType::Screen => Err(LarsError::RunnerNotAvailable(
            "screen runner is not yet implemented".to_string(),
        )),
        RunnerType::Direct => Err(LarsError::RunnerNotAvailable(
            "direct runner is not yet implemented".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_name() {
        let service = Service::new("test".to_string(), "echo hello".to_string());
        let session_name = TmuxRunner::session_name(&service);

        assert!(session_name.starts_with("lar_"));
        assert!(session_name.contains(&service.id.to_string()));
    }

    #[test]
    fn test_tmux_runner_type() {
        let runner = TmuxRunner::new();
        assert_eq!(runner.runner_type(), RunnerType::Tmux);
    }

    // Integration tests that require tmux are marked with #[ignore]
    // Run with: cargo test -- --ignored

    #[test]
    #[ignore]
    fn test_tmux_start_stop() {
        let runner = TmuxRunner::new();
        let mut service = Service::new("test-integration".to_string(), "sleep 60".to_string());
        service.id = uuid::Uuid::new_v4();

        let log_path = std::path::PathBuf::from("/tmp/lar_test.log");

        runner.start(&service, &log_path).unwrap();
        assert!(runner.is_running(&service).unwrap());

        runner.stop(&service).unwrap();

        let start = Instant::now();
        while runner.is_running(&service).unwrap() {
            assert!(start.elapsed() < Duration::from_secs(5));
            std::thread::sleep(Duration::from_millis(100));
        }

        assert!(!runner.is_running(&service).unwrap());
    }

    #[test]
    #[ignore]
    fn test_tmux_attach_command() {
        let runner = TmuxRunner::new();
        let service = Service::new("test".to_string(), "echo hello".to_string());

        let cmd = runner.attach_command(&service).unwrap();
        assert!(cmd.is_some());

        let cmd = cmd.unwrap();
        assert_eq!(cmd[0], "tmux");
        assert_eq!(cmd[1], "attach");
    }
}
