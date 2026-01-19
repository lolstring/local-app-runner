//! List command implementation

use anyhow::Result;
use comfy_table::Cell;
use lars_core::{create_runner, ConfigManager};
use serde::Serialize;

use crate::output::{OutputContext, TableBuilder};
use crate::ExitCode;

#[derive(Serialize)]
struct ServiceInfo {
    name: String,
    status: String,
    enabled: bool,
    runner: String,
    command: String,
}

pub fn run(config: &ConfigManager, all: bool, ctx: &OutputContext) -> Result<ExitCode> {
    let services = config.list_services()?;

    // Filter services
    let services: Vec<_> = if all {
        services
    } else {
        services.into_iter().filter(|s| s.enabled).collect()
    };

    if ctx.json {
        let infos: Vec<ServiceInfo> = services
            .iter()
            .map(|s| {
                let running = create_runner(s.runner_type)
                    .map(|r| r.is_running(s).unwrap_or(false))
                    .unwrap_or(false);

                ServiceInfo {
                    name: s.name.clone(),
                    status: if running {
                        "running".to_string()
                    } else {
                        "stopped".to_string()
                    },
                    enabled: s.enabled,
                    runner: s.runner_type.to_string(),
                    command: s.command.clone(),
                }
            })
            .collect();

        ctx.json(&infos)?;
    } else {
        if services.is_empty() {
            ctx.info("No services configured");
            if !all {
                ctx.info("Use --all to show disabled services");
            }
            return Ok(ExitCode::Success);
        }

        let mut table = TableBuilder::new(vec!["Name", "Status", "Enabled", "Runner", "Command"]);

        for service in &services {
            let running = create_runner(service.runner_type)
                .map(|r| r.is_running(service).unwrap_or(false))
                .unwrap_or(false);

            table.add_row(vec![
                Cell::new(&service.name),
                ctx.status_cell(running),
                ctx.enabled_cell(service.enabled),
                Cell::new(service.runner_type.to_string()),
                Cell::new(&service.command),
            ]);
        }

        println!("{}", table.build());
    }

    Ok(ExitCode::Success)
}
