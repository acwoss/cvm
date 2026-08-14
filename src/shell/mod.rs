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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_shell_hooks_manage_env_path() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Powershell] {
            let script = generate(shell);
            assert!(script.contains("CVM_OLD_PATH"));
            assert!(script.contains("CLAUDE_CONFIG_DIR"));
            assert!(script.contains("bin"));
        }
    }

    #[test]
    fn all_shell_hooks_auto_activate_from_dot_cvm() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Powershell] {
            let script = generate(shell);
            assert!(script.contains("__cvm_auto_check"));
            assert!(script.contains(".cvm"));
            assert!(!script.contains(".cvm-env"));
            assert!(script.contains("CVM_AUTO"));
            assert!(script.contains("CVM_AUTO_ROOT"));
            assert!(script.contains("CVM_AUTO_LAST_PWD"));
        }
    }

    #[test]
    fn all_shell_hooks_unpin_auto_activation_after_manual_use() {
        assert!(generate(Shell::Bash).contains("unset CVM_AUTO CVM_AUTO_ROOT"));
        assert!(generate(Shell::Zsh).contains("unset CVM_AUTO CVM_AUTO_ROOT"));
        assert!(generate(Shell::Fish).contains("set -e CVM_AUTO_ROOT"));
        assert!(generate(Shell::Powershell)
            .contains("Remove-Item Env:CVM_AUTO_ROOT -ErrorAction SilentlyContinue"));
    }

    #[test]
    fn auto_activation_uses_each_shells_directory_change_hook() {
        assert!(generate(Shell::Bash).contains("PROMPT_COMMAND"));
        assert!(generate(Shell::Zsh).contains("add-zsh-hook chpwd __cvm_auto_check"));
        assert!(generate(Shell::Fish).contains("--on-variable PWD"));
        assert!(generate(Shell::Powershell).contains("__cvm_auto_check"));
    }

    #[test]
    fn bash_appends_prompt_hook_without_a_semicolon_separator() {
        let script = generate(Shell::Bash);
        assert!(script.contains("$'\\n__cvm_auto_check'"));
        assert!(!script.contains("$PROMPT_COMMAND; }__cvm_auto_check"));
    }

    #[test]
    fn powershell_compares_environment_names_and_paths_case_sensitively() {
        let script = generate(Shell::Powershell);
        assert!(script.contains("-ceq $currentPwd"));
        assert!(script.contains("-cne $name"));
    }

    #[test]
    fn powershell_ignores_non_filesystem_providers() {
        assert!(generate(Shell::Powershell).contains("$PWD.Provider.Name -ne 'FileSystem'"));
    }

    #[test]
    fn all_shell_hooks_deactivate_the_current_env_before_switching() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Powershell] {
            let script = generate(shell);
            assert!(
                script.matches("__cvm_deactivate").count() >= 3,
                "{shell:?} must define and reuse one deactivate helper"
            );
        }

        assert!(generate(Shell::Bash).contains(r#"if [ -n "${CVM_ENV-}" ]; then"#));
        assert!(generate(Shell::Zsh).contains(r#"if [ -n "${CVM_ENV-}" ]; then"#));
        assert!(generate(Shell::Fish).contains("__cvm_deactivate; or return $status"));
        assert!(generate(Shell::Powershell).contains("__cvm_deactivate $cvmBin"));
    }
}
