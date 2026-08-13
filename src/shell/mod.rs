//! Shell hook generators.
//!
//! `cvm` is a compiled binary and cannot mutate the environment variables of
//! the shell that launched it. Instead, `cvm init <shell>` prints a small
//! wrapper function that shadows the `cvm` command in the current shell. The
//! wrapper intercepts `use`/`activate`/`deactivate`, asks the real binary
//! (via the hidden `__resolve-activate` / `__resolve-deactivate` commands)
//! which variables to set or unset, and applies them itself - every other
//! subcommand is forwarded straight to the real binary unchanged.

mod bash;
mod fish;
mod powershell;
mod zsh;

use crate::cli::Shell;

pub fn generate(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => bash::SCRIPT,
        Shell::Zsh => zsh::SCRIPT,
        Shell::Fish => fish::SCRIPT,
        Shell::Powershell => powershell::SCRIPT,
    }
}
