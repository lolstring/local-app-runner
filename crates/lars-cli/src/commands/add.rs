//! Add command implementation

use anyhow::Result;
use lars_core::{
    generate_service_name, validate_service_name, ConfigManager, RunnerType, Service,
};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(
    config: &ConfigManager,
    command: String,
    name: Option<String>,
    workdir: Option<String>,
    env: Vec<String>,
    disabled: bool,
    runner: String,
    ctx: &OutputContext,
) -> Result<ExitCode> {
    let name = match name {
        Some(n) => {
            validate_service_name(&n)?;
            n
        }
        None => {
            let generated = generate_service_name(&command);
            let existing = config.list_services()?;
            let mut final_name = generated.clone();
            let mut counter = 1;

            while existing.iter().any(|s| s.name == final_name) {
                final_name = format!("{}-{}", generated, counter);
                counter += 1;
            }

            final_name
        }
    };

    let runner_type: RunnerType = runner
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let mut env_map = HashMap::new();
    for e in env {
        if let Some((key, value)) = e.split_once('=') {
            env_map.insert(key.to_string(), value.to_string());
        } else {
            return Err(anyhow::anyhow!(
                "Invalid environment variable format: {}. Expected KEY=VALUE",
                e
            ));
        }
    }

    let cwd = workdir.map(PathBuf::from);
    if let Some(ref dir) = cwd {
        if !dir.exists() {
            ctx.warn(&format!(
                "Working directory does not exist: {}",
                dir.display()
            ));
        } else if !dir.is_dir() {
            return Err(anyhow::anyhow!(
                "Working directory path is not a directory: {}",
                dir.display()
            ));
        }
    }

    let mut service = Service::new(name.clone(), command);
    service.cwd = cwd;
    service.env = env_map;
    service.enabled = !disabled;
    service.runner_type = runner_type;

    config.add_service(service)?;

    if ctx.json {
        ctx.json(&serde_json::json!({
            "status": "added",
            "name": name
        }))?;
    } else {
        ctx.success(&format!("Added service '{}'", name));
    }

    Ok(ExitCode::Success)
}
