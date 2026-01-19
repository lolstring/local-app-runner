//! Remove command implementation

use anyhow::Result;
use lars_core::{create_runner, ConfigManager};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(
    config: &ConfigManager,
    name: &str,
    _force: bool,
    ctx: &OutputContext,
) -> Result<ExitCode> {
    // Get the service first to check if it exists and get its runner type
    let service = config.get_service(name)?;

    // Stop if running
    if let Ok(runner) = create_runner(service.runner_type) {
        if runner.is_running(&service)? {
            ctx.info(&format!("Stopping service '{}'...", name));
            runner.stop(&service)?;
        }
    }

    // Remove from config
    config.remove_service(name)?;

    if ctx.json {
        ctx.json(&serde_json::json!({
            "status": "removed",
            "name": name
        }))?;
    } else {
        ctx.success(&format!("Removed service '{}'", name));
    }

    Ok(ExitCode::Success)
}
