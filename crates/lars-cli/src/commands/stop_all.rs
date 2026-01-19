//! Stop-all command implementation

use anyhow::Result;
use lars_core::{create_runner, ConfigManager};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(config: &ConfigManager, ctx: &OutputContext) -> Result<ExitCode> {
    let services = config.list_services()?;

    if services.is_empty() {
        if ctx.json {
            ctx.json(&serde_json::json!({
                "stopped": 0,
                "skipped": 0,
                "failed": 0
            }))?;
        } else {
            ctx.info("No services configured");
        }
        return Ok(ExitCode::Success);
    }

    let mut stopped = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for service in &services {
        let runner = match create_runner(service.runner_type) {
            Ok(r) => r,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        // Skip if not running
        match runner.is_running(service) {
            Ok(false) => {
                skipped += 1;
                continue;
            }
            Ok(true) => {}
            Err(_) => {
                skipped += 1;
                continue;
            }
        }

        // Stop the service
        match runner.stop(service) {
            Ok(_) => {
                if !ctx.json {
                    ctx.success(&format!("Stopped '{}'", service.name));
                }
                stopped += 1;
            }
            Err(e) => {
                if !ctx.json {
                    ctx.error(&format!("Failed to stop '{}': {}", service.name, e));
                }
                failed += 1;
            }
        }
    }

    if ctx.json {
        ctx.json(&serde_json::json!({
            "stopped": stopped,
            "skipped": skipped,
            "failed": failed
        }))?;
    } else {
        ctx.info(&format!(
            "Summary: {} stopped, {} skipped, {} failed",
            stopped, skipped, failed
        ));
    }

    if failed > 0 {
        Ok(ExitCode::StopFailed)
    } else {
        Ok(ExitCode::Success)
    }
}
