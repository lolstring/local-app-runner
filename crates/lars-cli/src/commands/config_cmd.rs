//! Config command implementation

use anyhow::Result;
use lars_core::{ConfigManager, RunnerType, ShutdownBehavior};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn show(config: &ConfigManager, ctx: &OutputContext) -> Result<ExitCode> {
    let app_config = config.load()?;

    if ctx.json {
        ctx.json(&app_config)?;
    } else {
        println!("Configuration:");
        println!("  Config file: {}", config.config_path().display());
        println!("  Log directory: {}", config.log_dir().display());
        println!();
        println!("Settings:");
        println!("  default_runner: {}", app_config.settings.default_runner);
        println!("  shutdown_behavior: {}", app_config.settings.shutdown_behavior);
        println!(
            "  restart_timeout_secs: {}",
            app_config.settings.restart_timeout_secs
        );
        println!();
        println!("Services: {}", app_config.services.len());
    }

    Ok(ExitCode::Success)
}

pub fn set(config: &ConfigManager, key: &str, value: &str, ctx: &OutputContext) -> Result<ExitCode> {
    let mut app_config = config.load()?;

    match key {
        "default_runner" => {
            let runner: RunnerType = value
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            app_config.settings.default_runner = runner;
        }
        "shutdown_behavior" => {
            let behavior: ShutdownBehavior = value
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            app_config.settings.shutdown_behavior = behavior;
        }
        "restart_timeout_secs" => {
            let timeout: u64 = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid timeout value: must be a positive integer"))?;
            if timeout == 0 {
                return Err(anyhow::anyhow!("Restart timeout must be greater than 0"));
            }
            app_config.settings.restart_timeout_secs = timeout;
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown config key: {}. Valid keys: default_runner, shutdown_behavior, restart_timeout_secs",
                key
            ));
        }
    }

    config.save(&app_config)?;

    if ctx.json {
        ctx.json(&serde_json::json!({
            "status": "updated",
            "key": key,
            "value": value
        }))?;
    } else {
        ctx.success(&format!("Set {} = {}", key, value));
    }

    Ok(ExitCode::Success)
}
