//! Shell completions command implementation

use clap::CommandFactory;
use clap_complete::Shell;

pub fn run(shell: Shell) {
    let mut cmd = crate::Cli::command();
    clap_complete::generate(shell, &mut cmd, "lars", &mut std::io::stdout());
}
