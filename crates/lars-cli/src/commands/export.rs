//! Export command implementation

use anyhow::Result;
use lars_core::ConfigManager;
use std::fs::File;
use std::io::Write;

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(config: &ConfigManager, output: Option<&str>, ctx: &OutputContext) -> Result<ExitCode> {
    let app_config = config.load()?;
    let json = serde_json::to_string_pretty(&app_config)?;

    match output {
        Some(path) => {
            let mut file = File::create(path)?;
            file.write_all(json.as_bytes())?;

            if !ctx.json {
                ctx.success(&format!("Exported configuration to {}", path));
            }
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(ExitCode::Success)
}
