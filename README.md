# LARS - Local App Runner Service

A CLI tool for managing local development services using tmux. Start, stop, and monitor multiple services with simple commands.

## Features

- **Service Management** - Add, remove, enable, disable, and rename services
- **Process Control** - Start, stop, and restart services individually or all at once
- **tmux Integration** - Services run in tmux sessions for persistence and easy access
- **Live Logs** - View and follow service logs in real-time
- **Configuration Export/Import** - Share service configurations across machines
- **Shell Completions** - Tab completion for bash, zsh, fish, and PowerShell

## Installation

### Shell Script (Linux/macOS)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/lolstring/local-app-runner/releases/latest/download/lars-cli-installer.sh | sh
```

### Homebrew (macOS/Linux)

```bash
brew tap lolstring/tap
brew install lars
```

### PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/lolstring/local-app-runner/releases/latest/download/lars-cli-installer.ps1 | iex"
```

### From Source

```bash
cargo install --git https://github.com/lolstring/local-app-runner lars-cli
```

## Quick Start

```bash
# Add a service
lars add "npm run dev" --name frontend -d ~/projects/my-app

# Add another service
lars add "cargo run" --name backend -d ~/projects/api

# Add another local service
lars add 'OLLAMA_ORIGINS=* OLLAMA_DEBUG="1" ollama serve'

# Start all services
lars start-all

# Check status
lars list

# View logs
lars logs frontend -f

# Attach to a service's tmux session
lars attach ollama

# Stop everything
lars stop-all
```

## Commands

| Command | Description |
|---------|-------------|
| `add <command>` | Add a new service |
| `remove <name>` | Remove a service |
| `rename <name> <new_name>` | Rename a service |
| `enable <name>` | Enable a disabled service |
| `disable <name>` | Disable a service |
| `list` | List all services |
| `start <name>` | Start a service |
| `stop <name>` | Stop a service |
| `restart <name>` | Restart a service |
| `start-all` | Start all enabled services |
| `stop-all` | Stop all running services |
| `inspect <name>` | Show detailed service info |
| `logs <name>` | View service logs |
| `attach <name>` | Attach to service's tmux session |
| `config show` | Show current configuration |
| `config set <key> <value>` | Update configuration |
| `export` | Export services to JSON |
| `import <file>` | Import services from JSON |
| `doctor` | Run system diagnostics |
| `completions <shell>` | Generate shell completions |

## Adding Services

```bash
# Basic usage
lars add "python -m http.server 8000"

# With a custom name
lars add "npm start" --name my-app

# With working directory
lars add "cargo run" --name api -d ~/projects/api

# With environment variables
lars add "node server.js" --name server -e PORT=3000 -e NODE_ENV=development

# Disabled by default (won't start with start-all)
lars add "redis-server" --name redis --disabled
```

## Configuration

Configuration is stored in `~/.config/lars/` (Linux/macOS) or `%APPDATA%\lars\` (Windows).

### Settings

```bash
# View all settings
lars config show

# Set default runner (tmux or process)
lars config set default_runner tmux

# Set shutdown behavior (stop_all or leave_running)
lars config set shutdown_behavior stop_all
```

### Export/Import

```bash
# Export to file
lars export -o my-services.json

# Import from file
lars import my-services.json
```

## Shell Completions

```bash
# Bash
lars completions bash > ~/.local/share/bash-completion/completions/lars

# Zsh
lars completions zsh > ~/.zfunc/_lars

# Fish
lars completions fish > ~/.config/fish/completions/lars.fish

# PowerShell
lars completions powershell >> $PROFILE
```

## Requirements

- **tmux** - Required for service management
- Unix-like OS (Linux, macOS) or Windows with WSL

Check your setup:

```bash
lars doctor
```

## Output Formats

Most commands support JSON output for scripting:

```bash
lars list --json
lars inspect myservice --json
```

Control verbosity and colors:

```bash
lars -v list       # Verbose output
lars -q start-all  # Quiet mode
lars --no-color list # Disable colors
```

## License

MIT
