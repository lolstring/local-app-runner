//! Output formatting utilities for the CLI

use comfy_table::{Cell, Color};
use owo_colors::OwoColorize;
use serde::Serialize;

/// Context for output formatting
#[derive(Debug, Clone)]
pub struct OutputContext {
    pub json: bool,
    pub no_color: bool,
    pub quiet: bool,
}

impl OutputContext {
    pub fn new(json: bool, no_color: bool, quiet: bool) -> Self {
        // Disable colors if NO_COLOR is set or --no-color flag is used
        let no_color = no_color || std::env::var("NO_COLOR").is_ok();

        Self {
            json,
            no_color,
            quiet,
        }
    }

    /// Print a success message
    pub fn success(&self, msg: &str) {
        if self.quiet {
            return;
        }

        if self.no_color {
            println!("✓ {}", msg);
        } else {
            println!("{} {}", "✓".green(), msg);
        }
    }

    /// Print an error message
    pub fn error(&self, msg: &str) {
        if self.no_color {
            eprintln!("✗ {}", msg);
        } else {
            eprintln!("{} {}", "✗".red(), msg);
        }
    }

    /// Print a warning message
    pub fn warn(&self, msg: &str) {
        if self.quiet {
            return;
        }

        if self.no_color {
            eprintln!("! {}", msg);
        } else {
            eprintln!("{} {}", "!".yellow(), msg);
        }
    }

    /// Print an info message
    pub fn info(&self, msg: &str) {
        if self.quiet {
            return;
        }

        if self.no_color {
            println!("  {}", msg);
        } else {
            println!("  {}", msg);
        }
    }

    /// Print JSON output
    pub fn json<T: Serialize>(&self, value: &T) -> anyhow::Result<()> {
        let output = serde_json::to_string_pretty(value)?;
        println!("{}", output);
        Ok(())
    }

    /// Create a status cell for tables (running/stopped)
    pub fn status_cell(&self, running: bool) -> Cell {
        if running {
            let cell = Cell::new("running");
            if self.no_color {
                cell
            } else {
                cell.fg(Color::Green)
            }
        } else {
            let cell = Cell::new("stopped");
            if self.no_color {
                cell
            } else {
                cell.fg(Color::Red)
            }
        }
    }

    /// Create an enabled cell for tables
    pub fn enabled_cell(&self, enabled: bool) -> Cell {
        if enabled {
            let cell = Cell::new("yes");
            if self.no_color {
                cell
            } else {
                cell.fg(Color::Green)
            }
        } else {
            let cell = Cell::new("no");
            if self.no_color {
                cell
            } else {
                cell.fg(Color::DarkGrey)
            }
        }
    }

    /// Format a status indicator string (for non-table output)
    pub fn status_indicator(&self, running: bool) -> String {
        if running {
            if self.no_color {
                "running".to_string()
            } else {
                "running".green().to_string()
            }
        } else {
            if self.no_color {
                "stopped".to_string()
            } else {
                "stopped".red().to_string()
            }
        }
    }

    /// Format an enabled indicator string (for non-table output)
    pub fn enabled_indicator(&self, enabled: bool) -> String {
        if enabled {
            if self.no_color {
                "yes".to_string()
            } else {
                "yes".green().to_string()
            }
        } else {
            if self.no_color {
                "no".to_string()
            } else {
                "no".dimmed().to_string()
            }
        }
    }
}

/// Helper for building tables using comfy-table
pub struct TableBuilder {
    table: comfy_table::Table,
}

impl TableBuilder {
    pub fn new(headers: Vec<&str>) -> Self {
        use comfy_table::{presets::UTF8_FULL, ContentArrangement};

        let mut table = comfy_table::Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(headers);

        Self { table }
    }

    pub fn add_row(&mut self, row: Vec<Cell>) {
        self.table.add_row(row);
    }

    pub fn build(self) -> comfy_table::Table {
        self.table
    }
}
