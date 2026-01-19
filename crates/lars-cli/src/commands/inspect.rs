//! Inspect command implementation

use anyhow::Result;
use lars_core::{create_runner, ConfigManager};
use serde::Serialize;

use crate::output::OutputContext;
use crate::ExitCode;

#[derive(Serialize)]
struct InspectInfo {
    id: String,
    name: String,
    command: String,
    cwd: Option<String>,
    env: std::collections::HashMap<String, String>,
    enabled: bool,
    autostart: bool,
    runner: String,
    status: String,
    pid: Option<u32>,
    log_path: String,
    created_at: String,
    updated_at: String,
}

pub fn run(config: &ConfigManager, name: &str, ctx: &OutputContext) -> Result<ExitCode> {
    let service = config.get_service(name)?;
    let log_path = config.log_path_for_service(&service.id);

    let (running, pid) = if let Ok(runner) = create_runner(service.runner_type) {
        let running = runner.is_running(&service).unwrap_or(false);
        let pid = runner.get_pid(&service).unwrap_or(None);
        (running, pid)
    } else {
        (false, None)
    };

    let info = InspectInfo {
        id: service.id.to_string(),
        name: service.name.clone(),
        command: service.command.clone(),
        cwd: service.cwd.as_ref().map(|p| p.to_string_lossy().to_string()),
        env: service.env.clone(),
        enabled: service.enabled,
        autostart: service.autostart,
        runner: service.runner_type.to_string(),
        status: if running { "running" } else { "stopped" }.to_string(),
        pid,
        log_path: log_path.to_string_lossy().to_string(),
        created_at: service.created_at.to_rfc3339(),
        updated_at: service.updated_at.to_rfc3339(),
    };

    if ctx.json {
        ctx.json(&info)?;
    } else {
        println!("Service: {}", info.name);
        println!("ID:      {}", info.id);
        println!("Command: {}", info.command);
        if let Some(cwd) = &info.cwd {
            println!("Workdir: {}", cwd);
        }
        if !info.env.is_empty() {
            println!("Env:");
            for (key, value) in &info.env {
                println!("  {}={}", key, value);
            }
        }
        println!("Enabled: {}", ctx.enabled_indicator(info.enabled));
        println!("Autostart: {}", info.autostart);
        println!("Runner:  {}", info.runner);
        println!("Status:  {}", ctx.status_indicator(running));
        if let Some(p) = info.pid {
            println!("PID:     {}", p);
        }
        println!("Log:     {}", info.log_path);
        println!("Created: {}", info.created_at);
        println!("Updated: {}", info.updated_at);
    }

    Ok(ExitCode::Success)
}
