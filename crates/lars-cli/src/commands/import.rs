//! Import command implementation

use anyhow::Result;
use lars_core::{AppConfig, ConfigManager};
use std::fs;
use uuid::Uuid;

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(config: &ConfigManager, file: &str, merge: bool, ctx: &OutputContext) -> Result<ExitCode> {
    let contents = fs::read_to_string(file)?;
    let mut imported: AppConfig = serde_json::from_str(&contents)?;

    if merge {
        let mut existing = config.load()?;

        for mut service in imported.services {
            if existing.service_name_exists(&service.name) {
                if !ctx.json {
                    ctx.warn(&format!(
                        "Skipping '{}' - service with same name already exists",
                        service.name
                    ));
                }
                continue;
            }

            service.id = Uuid::new_v4();
            existing.add_service(service);
        }

        config.save(&existing)?;

        if ctx.json {
            ctx.json(&serde_json::json!({
                "status": "merged",
                "services": existing.services.len()
            }))?;
        } else {
            ctx.success(&format!(
                "Merged configuration. Total services: {}",
                existing.services.len()
            ));
        }
    } else {
        let existing = config.load()?;
        if !existing.services.is_empty() && !ctx.json {
            ctx.warn(&format!(
                "This will replace {} existing service(s)",
                existing.services.len()
            ));
        }

        for service in &mut imported.services {
            service.id = Uuid::new_v4();
        }

        config.save(&imported)?;

        if ctx.json {
            ctx.json(&serde_json::json!({
                "status": "imported",
                "services": imported.services.len()
            }))?;
        } else {
            ctx.success(&format!(
                "Imported configuration with {} services",
                imported.services.len()
            ));
        }
    }

    Ok(ExitCode::Success)
}
