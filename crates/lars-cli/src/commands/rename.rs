//! Rename command implementation

use anyhow::Result;
use lars_core::{validate_service_name, ConfigManager};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(
    config: &ConfigManager,
    name: &str,
    new_name: &str,
    ctx: &OutputContext,
) -> Result<ExitCode> {
    // Validate the new name
    validate_service_name(new_name)?;

    // Get the existing service
    let mut service = config.get_service(name)?;

    // Check if new name already exists
    if config.get_service(new_name).is_ok() {
        return Err(anyhow::anyhow!(
            "Service '{}' already exists",
            new_name
        ));
    }

    let old_name = service.name.clone();
    service.name = new_name.to_string();

    // Remove old and add with new name
    config.remove_service(&old_name)?;
    config.add_service(service)?;

    if ctx.json {
        ctx.json(&serde_json::json!({
            "status": "renamed",
            "old_name": old_name,
            "new_name": new_name
        }))?;
    } else {
        ctx.success(&format!("Renamed service '{}' to '{}'", old_name, new_name));
    }

    Ok(ExitCode::Success)
}
