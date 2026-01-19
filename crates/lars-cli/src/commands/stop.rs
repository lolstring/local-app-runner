//! Stop command implementation

use anyhow::Result;
use lars_core::{create_runner, ConfigManager};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(config: &ConfigManager, name: &str, ctx: &OutputContext) -> Result<ExitCode> {
    let service = config.get_service(name)?;
    let runner = create_runner(service.runner_type)?;

    // Check if running
    if !runner.is_running(&service)? {
        if ctx.json {
            ctx.json(&serde_json::json!({
                "status": "not_running",
                "name": name
            }))?;
        } else {
            ctx.info(&format!("Service '{}' is not running", name));
        }
        return Ok(ExitCode::Success);
    }

    // Stop the service
    runner.stop(&service)?;

    if ctx.json {
        ctx.json(&serde_json::json!({
            "status": "stopped",
            "name": name
        }))?;
    } else {
        ctx.success(&format!("Stopped service '{}'", name));
    }

    Ok(ExitCode::Success)
}
