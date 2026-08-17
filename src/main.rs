mod cli;
mod launch;
mod shell;
mod update;

use cvm_core::{env, manifest};

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;

use cli::{Cli, Command};

const MANIFEST_VERSION: &str = "1.0.0";

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {err:#}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { shell } => {
            print!("{}", shell::generate(shell));
        }
        Command::Update => cmd_update()?,
        Command::Create {
            env_name,
            anonymous,
            inherit,
            open,
        } => {
            let (dir, credentials_copied, inherit_stats) =
                env::create_env(&env_name, anonymous, inherit)?;
            println!(
                "{} environment '{}' created at {}",
                "✓".green(),
                env_name.bold(),
                dir.display()
            );
            if credentials_copied {
                println!(
                    "{}",
                    "Reused global Claude Code credentials - no login needed.".dimmed()
                );
            } else if anonymous {
                println!(
                    "{}",
                    "Created without credentials (--anonymous); log in separately.".dimmed()
                );
            } else {
                println!(
                    "{}",
                    "No global Claude Code credentials found to reuse; log in when ready.".dimmed()
                );
            }
            if inherit {
                let settings = if inherit_stats.settings_copied {
                    "settings.json copied"
                } else {
                    "settings.json not found"
                };
                println!(
                    "{}",
                    format!(
                        "Inherited global assets: {} skills linked, {} copied; {settings}.",
                        inherit_stats.skills_linked, inherit_stats.skills_copied
                    )
                    .dimmed()
                );
            }
            if open {
                let code = env::open_env(&env_name)?;
                std::process::exit(code);
            }
        }
        Command::List => cmd_list()?,
        Command::Use { env_name } => cmd_activation_hint(&env_name)?,
        Command::Deactivate => cmd_deactivate_hint()?,
        Command::Current => cmd_current(),
        Command::Remove { env_name, yes } => cmd_remove(&env_name, yes)?,
        Command::Run { env_name, command } => {
            let code = env::run_in_env(&env_name, &command)?;
            std::process::exit(code);
        }
        Command::Edit { env_name } => {
            let code = cmd_edit(env_name)?;
            std::process::exit(code);
        }
        Command::Open { env_name } => {
            let code = env::open_env(&env_name)?;
            std::process::exit(code);
        }
        Command::Export { env_name, output } => cmd_export(env_name, output)?,
        Command::Import { file, name } => cmd_import(file, name)?,
        Command::ResolveActivate { env_name } => cmd_resolve_activate(&env_name)?,
        Command::ResolveDeactivate => cmd_resolve_deactivate()?,
        Command::Launch => cmd_launch()?,
    }

    Ok(())
}

fn cmd_update() -> Result<()> {
    println!(
        "Checking for updates (current: {})...",
        update::current_version()
    );
    match update::run()? {
        update::UpdateOutcome::AlreadyUpToDate { version } => {
            println!("{} already up to date ({version})", "✓".green());
        }
        update::UpdateOutcome::Updated { from, to } => {
            println!("{} updated cvm {from} -> {to}", "✓".green().bold());
            println!(
                "{}",
                "Already-running shells keep using the old binary until you start a new one."
                    .dimmed()
            );
        }
    }
    Ok(())
}

fn cmd_launch() -> Result<()> {
    let path = launch::ensure_installed(update::fetch_latest_tag)?;
    launch::spawn(&path)?;
    println!("{} cvm-ui launched", "✓".green());
    Ok(())
}

fn cmd_list() -> Result<()> {
    let envs = env::list_envs()?;
    if envs.is_empty() {
        println!("No environments yet. Create one with `cvm create <name>`.");
        return Ok(());
    }

    let active = env::active_env();
    for name in envs {
        if active.as_deref() == Some(name.as_str()) {
            println!(
                "{} {} {}",
                "*".green().bold(),
                name.green().bold(),
                "(active)".dimmed()
            );
        } else {
            println!("  {name}");
        }
    }
    Ok(())
}

/// `cvm use`/`activate` run as the raw binary can't touch the parent shell's
/// environment - only the shell wrapper installed by `cvm init` can. This
/// guides the user toward setting that up instead of silently doing nothing.
fn cmd_activation_hint(env_name: &str) -> Result<()> {
    env::ensure_env_exists(env_name)?;
    eprintln!(
        "{} cvm shell integration is not active in this shell.",
        "warning:".yellow().bold()
    );
    eprintln!("Add one of these to your shell profile, then restart your shell:");
    eprintln!();
    eprintln!("  eval \"$(cvm init bash)\"          # ~/.bashrc");
    eprintln!("  eval \"$(cvm init zsh)\"           # ~/.zshrc");
    eprintln!("  cvm init fish | source            # ~/.config/fish/config.fish");
    eprintln!("  cvm init powershell | Out-String | Invoke-Expression   # $PROFILE");
    eprintln!();
    eprintln!("Once active, run: cvm use {env_name}");
    Ok(())
}

fn cmd_deactivate_hint() -> Result<()> {
    eprintln!(
        "{} cvm shell integration is not active in this shell.",
        "warning:".yellow().bold()
    );
    eprintln!("See `cvm init --help` to set it up, then run: cvm deactivate");
    Ok(())
}

fn cmd_current() {
    match env::active_env() {
        Some(name) => println!("{name}"),
        None => println!("{}", "(no active environment)".dimmed()),
    }
}

fn cmd_remove(env_name: &str, yes: bool) -> Result<()> {
    env::ensure_env_exists(env_name)?;

    if env::active_env().as_deref() == Some(env_name) {
        anyhow::bail!("cannot remove '{env_name}' while it is active; run `cvm deactivate` first");
    }

    if !yes {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!(
                "Delete environment '{env_name}'? This cannot be undone."
            ))
            .default(false)
            .interact()
            .context("failed to read confirmation")?;
        if !confirmed {
            println!("Aborted.");
            return Ok(());
        }
    }

    env::remove_env(env_name)?;
    println!("{} environment '{}' removed", "✓".green(), env_name.bold());
    Ok(())
}

fn cmd_edit(env_name: Option<String>) -> Result<i32> {
    let name = match env_name {
        Some(n) => n,
        None => env::active_env().context(
            "no environment specified and none is active; pass a name or run `cvm use <env>` first",
        )?,
    };
    env::edit_env(&name)
}

fn cmd_export(env_name: Option<String>, output: PathBuf) -> Result<()> {
    let name = match env_name {
        Some(n) => n,
        None => env::active_env().context(
            "no environment specified and none is active; pass a name or run `cvm use <env>` first",
        )?,
    };

    let dir = env::ensure_env_exists(&name)?;
    let manifest = manifest::export_env(&dir, &name, MANIFEST_VERSION, None)?;
    manifest::write_manifest(&manifest, &output)?;

    println!(
        "{} exported environment '{}' to {}",
        "✓".green(),
        name.bold(),
        output.display()
    );
    println!(
        "{}",
        "No credentials, tokens, or session history were included.".dimmed()
    );
    Ok(())
}

fn cmd_import(file: PathBuf, name: Option<String>) -> Result<()> {
    let manifest = manifest::read_manifest(&file)?;
    let env_name = name.unwrap_or_else(|| manifest.name.clone());

    let dir = env::env_dir(&env_name)?;
    if !dir.exists() {
        env::create_env_without_hook(&env_name, false, false)?;
    }
    manifest::apply_manifest(&manifest, &dir)?;

    println!(
        "{} imported {} into environment '{}'",
        "✓".green(),
        file.display(),
        env_name.bold()
    );
    println!("Activate it with: cvm use {env_name}");
    Ok(())
}

fn cmd_resolve_activate(env_name: &str) -> Result<()> {
    let pairs = env::resolve_activate(env_name)?;
    let mut out = String::new();
    for (key, value) in pairs {
        out.push_str(&key);
        out.push('=');
        out.push_str(&value);
        out.push('\n');
    }
    print!("{out}");
    io::stdout().flush().ok();
    Ok(())
}

fn cmd_resolve_deactivate() -> Result<()> {
    for var in env::resolve_deactivate()? {
        println!("{var}");
    }
    Ok(())
}
