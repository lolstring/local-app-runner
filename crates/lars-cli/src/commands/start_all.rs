//! Start-all command implementation

use anyhow::Result;
use lars_core::{create_runner, ConfigManager};

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(config: &ConfigManager, ctx: &OutputContext) -> Result<ExitCode> {
    let services = config.list_services()?;
    let enabled_services: Vec<_> = services.into_iter().filter(|s| s.enabled).collect();

    if enabled_services.is_empty() {
        if ctx.json {
            ctx.json(&serde_json::json!({
                "started": 0,
                "skipped": 0,
                "failed": 0
            }))?;
        } else {
            ctx.info("No enabled services to start");
        }
        return Ok(ExitCode::Success);
    }

    let mut started = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for service in &enabled_services {
        let runner = match create_runner(service.runner_type) {
            Ok(r) => r,
            Err(e) => {
                if !ctx.json {
                    ctx.error(&format!("Failed to create runner for '{}': {}", service.name, e));
                }
                failed += 1;
                continue;
            }
        };

        // Skip if already running
        match runner.is_running(service) {
            Ok(true) => {
                if !ctx.json {
                    ctx.info(&format!("Service '{}' is already running", service.name));
                }
                skipped += 1;
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                if !ctx.json {
                    ctx.error(&format!("Failed to check status of '{}': {}", service.name, e));
                }
                failed += 1;
                continue;
            }
        }

        // Get log path
        let log_path = config.log_path_for_service(&service.id);

        // Ensure log directory exists
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Start the service
        match runner.start(service, &log_path) {
            Ok(_) => {
                if !ctx.json {
                    ctx.success(&format!("Started '{}'", service.name));
                }
                started += 1;
            }
            Err(e) => {
                if !ctx.json {
                    ctx.error(&format!("Failed to start '{}': {}", service.name, e));
                }
                failed += 1;
            }
        }
    }

    if ctx.json {
        ctx.json(&serde_json::json!({
            "started": started,
            "skipped": skipped,
            "failed": failed
        }))?;
    } else {
        ctx.info(&format!(
            "Summary: {} started, {} skipped, {} failed",
            started, skipped, failed
        ));
    }

    if failed > 0 {
        Ok(ExitCode::StartFailed)
    } else {
        Ok(ExitCode::Success)
    }
}
