//! Doctor command implementation

use anyhow::Result;
use lars_core::{ConfigManager, TmuxRunner};
use serde::Serialize;
use std::process::Command;

use crate::output::OutputContext;
use crate::ExitCode;

#[derive(Serialize)]
struct Check {
    name: String,
    status: String,
    message: String,
    required: bool,
}

pub fn run(config: &ConfigManager, ctx: &OutputContext) -> Result<ExitCode> {
    let mut checks = Vec::new();
    let mut all_required_passed = true;

    // Check tmux
    let tmux_check = check_tmux();
    if tmux_check.status == "fail" && tmux_check.required {
        all_required_passed = false;
    }
    checks.push(tmux_check);

    // Check screen (optional)
    checks.push(check_screen());

    // Check config directory
    let config_check = check_config_dir(config);
    if config_check.status == "fail" && config_check.required {
        all_required_passed = false;
    }
    checks.push(config_check);

    // Check log directory
    let log_check = check_log_dir(config);
    if log_check.status == "fail" && log_check.required {
        all_required_passed = false;
    }
    checks.push(log_check);

    // Check shell
    let shell_check = check_shell();
    if shell_check.status == "fail" && shell_check.required {
        all_required_passed = false;
    }
    checks.push(shell_check);

    if ctx.json {
        ctx.json(&serde_json::json!({
            "checks": checks,
            "all_passed": all_required_passed
        }))?;
    } else {
        println!("System Diagnostics");
        println!("==================");
        println!();

        for check in &checks {
            let indicator = if check.status == "pass" {
                if ctx.no_color {
                    "✓"
                } else {
                    "\x1b[32m✓\x1b[0m"
                }
            } else if check.required {
                if ctx.no_color {
                    "✗"
                } else {
                    "\x1b[31m✗\x1b[0m"
                }
            } else {
                if ctx.no_color {
                    "-"
                } else {
                    "\x1b[33m-\x1b[0m"
                }
            };

            let required_label = if check.required { "" } else { " (optional)" };
            println!("{} {}{}: {}", indicator, check.name, required_label, check.message);
        }

        println!();
        if all_required_passed {
            ctx.success("All required checks passed");
        } else {
            ctx.error("Some required checks failed");
        }
    }

    if all_required_passed {
        Ok(ExitCode::Success)
    } else {
        Ok(ExitCode::GeneralError)
    }
}

fn check_tmux() -> Check {
    if TmuxRunner::is_available() {
        let version = TmuxRunner::version().unwrap_or_else(|| "unknown".to_string());
        Check {
            name: "tmux".to_string(),
            status: "pass".to_string(),
            message: version,
            required: true,
        }
    } else {
        Check {
            name: "tmux".to_string(),
            status: "fail".to_string(),
            message: "not found in PATH".to_string(),
            required: true,
        }
    }
}

fn check_screen() -> Check {
    let output = Command::new("screen").arg("-v").output();

    match output {
        Ok(o) if o.status.success() => {
            let version = String::from_utf8_lossy(&o.stdout);
            let version = version.lines().next().unwrap_or("unknown").trim();
            Check {
                name: "screen".to_string(),
                status: "pass".to_string(),
                message: version.to_string(),
                required: false,
            }
        }
        _ => Check {
            name: "screen".to_string(),
            status: "fail".to_string(),
            message: "not found".to_string(),
            required: false,
        },
    }
}

fn check_config_dir(config: &ConfigManager) -> Check {
    if config.is_config_dir_writable() {
        Check {
            name: "config_dir".to_string(),
            status: "pass".to_string(),
            message: format!("{} (writable)", config.config_dir().display()),
            required: true,
        }
    } else {
        Check {
            name: "config_dir".to_string(),
            status: "fail".to_string(),
            message: format!("{} (not writable)", config.config_dir().display()),
            required: true,
        }
    }
}

fn check_log_dir(config: &ConfigManager) -> Check {
    if config.is_log_dir_writable() {
        Check {
            name: "log_dir".to_string(),
            status: "pass".to_string(),
            message: format!("{} (writable)", config.log_dir().display()),
            required: true,
        }
    } else {
        Check {
            name: "log_dir".to_string(),
            status: "fail".to_string(),
            message: format!("{} (not writable)", config.log_dir().display()),
            required: true,
        }
    }
}

fn check_shell() -> Check {
    let output = Command::new("sh").arg("-c").arg("echo ok").output();

    match output {
        Ok(o) if o.status.success() => Check {
            name: "shell".to_string(),
            status: "pass".to_string(),
            message: "/bin/sh available".to_string(),
            required: true,
        },
        _ => Check {
            name: "shell".to_string(),
            status: "fail".to_string(),
            message: "sh not available".to_string(),
            required: true,
        },
    }
}
