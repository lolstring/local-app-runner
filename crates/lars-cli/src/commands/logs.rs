//! Logs command implementation

use anyhow::Result;
use lars_core::ConfigManager;
use std::fs::File;
use std::io::{BufRead, BufReader};
#[cfg(not(unix))]
use std::io::{Seek, SeekFrom};
use std::process::Command;

use crate::output::OutputContext;
use crate::ExitCode;

pub fn run(
    config: &ConfigManager,
    name: &str,
    follow: bool,
    lines: usize,
    ctx: &OutputContext,
) -> Result<ExitCode> {
    let service = config.get_service(name)?;
    let log_path = config.log_path_for_service(&service.id);

    if !log_path.exists() {
        if ctx.json {
            ctx.json(&serde_json::json!({
                "error": "no_logs",
                "message": "No log file found for this service"
            }))?;
        } else {
            ctx.warn("No log file found for this service");
            ctx.info(&format!("Expected at: {}", log_path.display()));
        }
        return Ok(ExitCode::Success);
    }

    if follow {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let log_path_str = log_path.to_string_lossy();
            let mut cmd = Command::new("tail");
            cmd.args(["-f", "-n", &lines.to_string(), &log_path_str]);

            if !ctx.quiet && !ctx.json {
                ctx.info(&format!("Following logs for '{}' (Ctrl+C to stop)...", name));
            }

            let err = cmd.exec();
            return Err(anyhow::anyhow!("Failed to exec tail: {}", err));
        }

        #[cfg(not(unix))]
        {
            let mut file = File::open(&log_path)?;
            file.seek(SeekFrom::End(0))?;
            let mut reader = BufReader::new(file);

            if !ctx.quiet && !ctx.json {
                ctx.info(&format!("Following logs for '{}' (Ctrl+C to stop)...", name));
            }

            loop {
                let mut line = String::new();
                let bytes = reader.read_line(&mut line)?;
                if bytes > 0 {
                    print!("{}", line);
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    } else {
        let file = File::open(&log_path)?;
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

        let start = if all_lines.len() > lines {
            all_lines.len() - lines
        } else {
            0
        };

        if ctx.json {
            ctx.json(&serde_json::json!({
                "log_path": log_path.to_string_lossy(),
                "lines": &all_lines[start..]
            }))?;
        } else {
            for line in &all_lines[start..] {
                println!("{}", line);
            }
        }

        Ok(ExitCode::Success)
    }
}
