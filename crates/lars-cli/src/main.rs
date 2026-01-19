//! LARS CLI - Local App Runner Service Command Line Interface

use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use lars_core::ConfigManager;

mod commands;
mod output;

use commands::*;

/// Exit codes for the CLI
#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    UsageError = 2,
    ServiceNotFound = 10,
    ServiceAlreadyExists = 11,
    RunnerUnavailable = 20,
    StartFailed = 21,
    StopFailed = 22,
    ConfigError = 30,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

/// LARS (Local App Runner Service) - Manage local services with ease
#[derive(Parser)]
#[command(name = "lars", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Verbose output (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count, global = true)]
    verbose: u8,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new service
    Add {
        /// The command to run
        command: String,

        /// Service name (auto-generated if not provided)
        #[arg(short, long)]
        name: Option<String>,

        /// Working directory
        #[arg(short = 'd', long)]
        workdir: Option<String>,

        /// Environment variables (KEY=VALUE)
        #[arg(short, long = "env", value_name = "KEY=VALUE")]
        env: Vec<String>,

        /// Add in disabled state
        #[arg(long)]
        disabled: bool,

        /// Runner type
        #[arg(short, long, default_value = "tmux")]
        runner: String,
    },

    /// Remove a service
    Remove {
        /// Service name
        name: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Enable a service
    Enable {
        /// Service name
        name: String,
    },

    /// Disable a service
    Disable {
        /// Service name
        name: String,
    },

    /// List all services
    List {
        /// Include disabled services
        #[arg(short, long)]
        all: bool,
    },

    /// Start a service
    Start {
        /// Service name
        name: String,

        /// Attach to session after starting
        #[arg(short, long)]
        attach: bool,
    },

    /// Stop a service
    Stop {
        /// Service name
        name: String,
    },

    /// Restart a service
    Restart {
        /// Service name
        name: String,
    },

    /// Start all enabled services
    StartAll,

    /// Stop all running services
    StopAll,

    /// Rename a service
    Rename {
        /// Current service name
        name: String,

        /// New service name
        new_name: String,
    },

    /// Show detailed service information
    Inspect {
        /// Service name
        name: String,
    },

    /// Attach to a service's session
    Attach {
        /// Service name
        name: String,
    },

    /// View service logs
    Logs {
        /// Service name
        name: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
    },

    /// Show or modify configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Check system requirements
    Doctor,

    /// Export configuration
    Export {
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Import configuration
    Import {
        /// Config file to import
        file: String,

        /// Merge with existing config instead of replacing
        #[arg(short, long)]
        merge: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },
}

fn setup_logging(verbose: u8, quiet: bool) {
    use tracing_subscriber::EnvFilter;

    if quiet {
        return;
    }

    let level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn main() {
    let cli = Cli::parse();

    setup_logging(cli.verbose, cli.quiet);

    // Set up output formatting
    let ctx = output::OutputContext::new(cli.json, cli.no_color, cli.quiet);

    let result = run_command(cli.command, &ctx);

    match result {
        Ok(code) => std::process::exit(code.into()),
        Err(e) => {
            ctx.error(&format!("{:#}", e));
            std::process::exit(ExitCode::GeneralError.into());
        }
    }
}

fn run_command(command: Commands, ctx: &output::OutputContext) -> Result<ExitCode> {
    let config = ConfigManager::with_defaults()?;

    match command {
        Commands::Add {
            command,
            name,
            workdir,
            env,
            disabled,
            runner,
        } => add::run(&config, command, name, workdir, env, disabled, runner, ctx),

        Commands::Remove { name, force } => remove::run(&config, &name, force, ctx),

        Commands::Enable { name } => enable::run(&config, &name, true, ctx),

        Commands::Disable { name } => enable::run(&config, &name, false, ctx),

        Commands::List { all } => list::run(&config, all, ctx),

        Commands::Start { name, attach } => start::run(&config, &name, attach, ctx),

        Commands::Stop { name } => stop::run(&config, &name, ctx),

        Commands::Restart { name } => restart::run(&config, &name, ctx),

        Commands::StartAll => start_all::run(&config, ctx),

        Commands::StopAll => stop_all::run(&config, ctx),

        Commands::Rename { name, new_name } => rename::run(&config, &name, &new_name, ctx),

        Commands::Inspect { name } => inspect::run(&config, &name, ctx),

        Commands::Attach { name } => attach::run(&config, &name, ctx),

        Commands::Logs {
            name,
            follow,
            lines,
        } => logs::run(&config, &name, follow, lines, ctx),

        Commands::Config { action } => match action {
            ConfigAction::Show => config_cmd::show(&config, ctx),
            ConfigAction::Set { key, value } => config_cmd::set(&config, &key, &value, ctx),
        },

        Commands::Doctor => doctor::run(&config, ctx),

        Commands::Export { output } => export::run(&config, output.as_deref(), ctx),

        Commands::Import { file, merge } => import::run(&config, &file, merge, ctx),

        Commands::Completions { shell } => {
            completions::run(shell);
            Ok(ExitCode::Success)
        }
    }
}
