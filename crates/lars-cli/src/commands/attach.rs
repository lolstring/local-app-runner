//! Attach command implementation

use anyhow::Result;
use lars_core::{create_runner, ConfigManager, LarsError};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(config: &ConfigManager, name: &str, ctx: &OutputContext) -> Result<ExitCode> {
    let service = config.get_service(name)?;
    let runner = create_runner(service.runner_type)?;

    // Check if running
    if !runner.is_running(&service)? {
        if ctx.json {
            ctx.json(&serde_json::json!({
                "error": "not_running",
                "name": name
            }))?;
        } else {
            ctx.error(&format!("Service '{}' is not running", name));
        }
        return Ok(ExitCode::ServiceNotFound);
    }

    // Get attach command
    let cmd = runner.attach_command(&service)?;

    match cmd {
        Some(args) => {
            if !ctx.quiet && !ctx.json {
                ctx.info("Attaching to session...");
            }

            // Use exec to replace current process
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let mut command = std::process::Command::new(&args[0]);
                command.args(&args[1..]);
                let err = command.exec();
                return Err(anyhow::anyhow!("Failed to exec: {}", err));
            }

            #[cfg(not(unix))]
            {
                let status = std::process::Command::new(&args[0])
                    .args(&args[1..])
                    .status()?;

                if status.success() {
                    return Ok(ExitCode::Success);
                } else {
                    return Err(anyhow::anyhow!("Attach command failed"));
                }
            }
        }
        None => {
            let err = LarsError::OperationNotSupported(format!(
                "Runner '{}' does not support attach",
                service.runner_type
            ));

            if ctx.json {
                ctx.json(&serde_json::json!({
                    "error": "not_supported",
                    "message": err.to_string()
                }))?;
            } else {
                ctx.error(&err.to_string());
            }

            Ok(ExitCode::RunnerUnavailable)
        }
    }
}
