//! Restart command implementation

use anyhow::Result;
use lars_core::{create_runner, ConfigManager};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(config: &ConfigManager, name: &str, ctx: &OutputContext) -> Result<ExitCode> {
    let app_config = config.load()?;
    let service = config.get_service(name)?;
    let runner = create_runner(service.runner_type)?;

    let log_path = config.log_path_for_service(&service.id);

    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let timeout = app_config.settings.restart_timeout_secs;
    runner.restart(&service, &log_path, timeout)?;

    if ctx.json {
        ctx.json(&serde_json::json!({
            "status": "restarted",
            "name": name
        }))?;
    } else {
        ctx.success(&format!("Restarted service '{}'", name));
    }

    Ok(ExitCode::Success)
}
