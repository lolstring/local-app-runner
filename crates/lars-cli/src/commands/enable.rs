//! Enable/Disable command implementation

use anyhow::Result;
use lars_core::ConfigManager;

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(config: &ConfigManager, name: &str, enable: bool, ctx: &OutputContext) -> Result<ExitCode> {
    config.update_service(name, |service| {
        service.enabled = enable;
    })?;

    let action = if enable { "enabled" } else { "disabled" };

    if ctx.json {
        ctx.json(&serde_json::json!({
            "status": action,
            "name": name
        }))?;
    } else {
        ctx.success(&format!("Service '{}' {}", name, action));
    }

    Ok(ExitCode::Success)
}
