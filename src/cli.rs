use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "cvm",
    version,
    about = "Claude Virtualenv Manager - isolate and share Claude Code configuration environments"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Print shell integration hooks for the given shell (eval this in your shell profile)
    Init {
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Create a new isolated environment
    Create {
        /// Name of the environment to create
        env_name: String,
    },

    /// List all available environments, highlighting the active one
    #[command(alias = "ls")]
    List,

    /// Switch the current shell session to the target environment
    ///
    /// Requires shell integration (see `cvm init`); calling the raw binary
    /// cannot change the parent shell's environment variables.
    #[command(alias = "activate")]
    Use {
        /// Name of the environment to activate
        env_name: String,
    },

    /// Unset environment overrides and restore the default global setup
    ///
    /// Requires shell integration (see `cvm init`).
    Deactivate,

    /// Print the name of the environment active in this shell session
    Current,

    /// Delete the specified environment directory
    #[command(alias = "rm")]
    Remove {
        /// Name of the environment to delete
        env_name: String,

        /// Skip the confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },

    /// Run a single command inside the context of an environment, without activating it
    ///
    /// Example: cvm run work -- claude
    Run {
        /// Name of the environment to run the command in
        env_name: String,

        /// Command (and its arguments) to execute, after `--`
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },

    /// Open Claude Code inside an environment's context (alias for `run <env> -- claude`)
    ///
    /// Sets `CVM_ENV=<env>` for that process only, so multiple isolated
    /// Claude Code instances can be opened side by side in parallel.
    Open {
        /// Name of the environment to open Claude Code in
        env_name: String,
    },

    /// Export an environment's configuration to a YAML manifest (defaults to the active environment)
    ///
    /// Never includes credentials, auth tokens, session history, or memories.
    Export {
        /// Environment to export (defaults to the currently active one)
        env_name: Option<String>,

        /// Output file path
        #[arg(short = 'o', long = "output", default_value = "cvm.yaml")]
        output: PathBuf,
    },

    /// Import a YAML manifest and create/configure a local environment from it
    Import {
        /// Path to the cvm.yaml manifest to import
        file: PathBuf,

        /// Name of the environment to create/update (defaults to the manifest's `name` field)
        #[arg(short = 'n', long = "name")]
        name: Option<String>,
    },

    /// Resolve the env vars to export in order to activate an environment (used by shell hooks)
    #[command(name = "__resolve-activate", hide = true)]
    ResolveActivate { env_name: String },

    /// Resolve the env vars to unset in order to deactivate (used by shell hooks)
    #[command(name = "__resolve-deactivate", hide = true)]
    ResolveDeactivate,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}
