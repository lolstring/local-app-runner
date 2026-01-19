//! Start command implementation

use anyhow::Result;
use lars_core::{create_runner, ConfigManager, Runner};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(
    config: &ConfigManager,
    name: &str,
    attach: bool,
    ctx: &OutputContext,
) -> Result<ExitCode> {
    let service = config.get_service(name)?;
    let runner = create_runner(service.runner_type)?;

    if runner.is_running(&service)? {
        if ctx.json {
            ctx.json(&serde_json::json!({
                "status": "already_running",
                "name": name
            }))?;
        } else {
            ctx.info(&format!("Service '{}' is already running", name));
        }

        if attach {
            return do_attach(&*runner, &service, ctx);
        }

        return Ok(ExitCode::Success);
    }

    let log_path = config.log_path_for_service(&service.id);

    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    runner.start(&service, &log_path)?;

    if ctx.json {
        ctx.json(&serde_json::json!({
            "status": "started",
            "name": name,
            "log_path": log_path.to_string_lossy()
        }))?;
    } else {
        ctx.success(&format!("Started service '{}'", name));
    }

    if attach {
        return do_attach(&*runner, &service, ctx);
    }

    Ok(ExitCode::Success)
}

fn do_attach(
    runner: &dyn Runner,
    service: &lars_core::Service,
    ctx: &OutputContext,
) -> Result<ExitCode> {
    if let Some(cmd) = runner.attach_command(service)? {
        if !ctx.quiet {
            ctx.info("Attaching to session...");
        }

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let mut command = std::process::Command::new(&cmd[0]);
            command.args(&cmd[1..]);
            let err = command.exec();
            return Err(anyhow::anyhow!("Failed to exec: {}", err));
        }

        #[cfg(not(unix))]
        {
            let status = std::process::Command::new(&cmd[0])
                .args(&cmd[1..])
                .status()?;

            if status.success() {
                return Ok(ExitCode::Success);
            } else {
                return Err(anyhow::anyhow!("Attach command failed"));
            }
        }
    } else {
        ctx.warn("Runner does not support attach");
        Ok(ExitCode::Success)
    }
}
