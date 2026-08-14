# Plano de Implementação: Hooks de Ciclo de Vida de Ambiente

> **Para trabalhadores agênticos:** SUB-SKILL OBRIGATÓRIA: use ultrapowers:smart-plan-execution (recomendado — roteia cada task pela complexidade) para executar este plano, ou ultrapowers:subagent-driven-development / ultrapowers:executing-plans para controle manual. Os passos usam sintaxe de checkbox (`- [ ]`) para acompanhamento.

**Objetivo:** Permitir que o usuário configure scripts globais em `~/.cvm/hooks/` que o `cvm` executa automaticamente ao criar, ativar, desativar e remover qualquer ambiente.

**Arquitetura:** Um novo módulo `src/hooks.rs` concentra a lógica de descoberta e execução de hooks (path por evento, checagem de permissão de execução no Unix, injeção de `CVM_HOOK_EVENT`/`CVM_ENV`/`CVM_ENV_PATH`, semântica de bloqueio para `pre-*` vs. só-aviso para `post-*`). `src/env.rs` chama esse módulo nos 4 pontos do ciclo de vida (`create_env`, `resolve_activate`, `resolve_deactivate`, `remove_env`); `resolve_deactivate` deixa de ser infalível para poder propagar a falha de um `pre-deactivate`, e `main.rs` precisa refletir essa mudança de assinatura.

**Tech Stack:** Rust (edition 2021), `anyhow` para erros, `tempfile` em testes, `std::process::Command` para rodar os hooks.

## Global Constraints

- Diretório de hooks: `~/.cvm/hooks/<evento>` (resolvido via `cvm_home()`, respeita `CVM_HOME`).
- Eventos (exatamente estes 7, sem mais): `post-create`, `pre-activate`, `post-activate`, `pre-deactivate`, `post-deactivate`, `pre-remove`, `post-remove`.
- `pre-*` com exit code != 0 aborta a operação (propaga erro); `post-*` com exit code != 0 só imprime aviso no stderr e a operação em curso continua tendo sucesso.
- Hook ausente é ignorado silenciosamente; hook presente mas sem bit de execução (Unix) é ignorado com aviso, nunca bloqueia.
- Variáveis injetadas no processo do hook: `CVM_HOOK_EVENT`, `CVM_ENV`, `CVM_ENV_PATH` (além de herdar o ambiente do processo `cvm`).
- Nome de arquivo por plataforma: extensão nenhuma no Unix (precisa de shebang + `chmod +x`); `.cmd` no Windows (mesmo padrão dos shims em `bin/claude.cmd`), invocado via `cmd /C`.
- `cvm run`/`cvm open`/`cvm import` **não** disparam hooks — fora de escopo.
- Hooks nunca entram em `cvm export`/`cvm import` — são sempre locais à máquina.
- `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --all -- --check` precisam passar a cada commit (CI já roda isso em `.github/workflows/ci.yml`).

---

## File Structure

- **Create:** `src/hooks.rs` — motor de execução de hooks: resolve o caminho do arquivo por evento/plataforma, checa permissão de execução, roda o processo com as env vars injetadas, e expõe `run_pre_hook` (propaga falha) e `run_post_hook` (só avisa).
- **Modify:** `src/main.rs` — registra `mod hooks;`; `cmd_resolve_deactivate` passa a propagar `Result` (pois `env::resolve_deactivate` deixa de ser infalível).
- **Modify:** `src/env.rs` — chama `hooks::run_pre_hook`/`hooks::run_post_hook` em `create_env`, `remove_env`, `resolve_activate`, `resolve_deactivate`.
- **Modify:** `README.md` — documenta a feature (bullet em "Features" + nova seção "### Lifecycle hooks" em "How It Works").

---

### Task 1: Motor de hooks (`src/hooks.rs`) + evento `post-create`

**Files:**
- Create: `src/hooks.rs`
- Modify: `src/main.rs:1-6` (adiciona `mod hooks;`)
- Modify: `src/env.rs:91-116` (função `create_env`) e `src/env.rs:1-9` (imports)
- Test: testes unitários dentro de `src/hooks.rs` (`#[cfg(test)] mod tests`); teste de integração em `src/env.rs`

**Interfaces:**
- Produces (usado pelas Tasks 2 e 3):
  - `pub fn hooks::hooks_dir() -> anyhow::Result<std::path::PathBuf>`
  - `pub fn hooks::run_pre_hook(hooks_dir: &std::path::Path, event: &str, env_name: &str, env_dir: &std::path::Path) -> anyhow::Result<()>`
  - `pub fn hooks::run_post_hook(hooks_dir: &std::path::Path, event: &str, env_name: &str, env_dir: &std::path::Path)` (sem retorno — nunca propaga erro)

- [ ] **Step 1: Escrever `src/hooks.rs` com stubs e os testes**

Crie `src/hooks.rs` com este conteúdo (as funções públicas ainda usam `unimplemented!()`, então os testes devem falhar ao rodar):

```rust
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Directory holding global lifecycle hook scripts (`~/.cvm/hooks`), one
/// file per event, shared by every environment.
pub fn hooks_dir() -> Result<PathBuf> {
    Ok(crate::env::cvm_home()?.join("hooks"))
}

/// Path of `event`'s hook script inside `hooks_dir`: extensionless on Unix
/// (needs `chmod +x` and a shebang), `<event>.cmd` on Windows - the same
/// per-platform convention already used for environment shims in `bin/`.
fn hook_path(hooks_dir: &Path, event: &str) -> PathBuf {
    if cfg!(windows) {
        hooks_dir.join(format!("{event}.cmd"))
    } else {
        hooks_dir.join(event)
    }
}

/// Runs `event`'s hook if present in `hooks_dir`, propagating any failure.
/// For `pre-*` events: a failing hook must abort the operation in progress.
pub fn run_pre_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) -> Result<()> {
    unimplemented!()
}

/// Runs `event`'s hook if present in `hooks_dir`, only warning on failure.
/// For `post-*` events: the operation already completed, so a failing hook
/// must not be treated as a `cvm` failure.
pub fn run_post_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) {
    unimplemented!()
}

fn execute_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) -> Result<()> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_unix_hook(path: &Path, script: &str) {
        fs::write(path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    #[test]
    fn missing_hook_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        execute_hook(dir.path(), "pre-activate", "work", Path::new("/envs/work")).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn hook_receives_expected_env_vars() {
        let dir = tempfile::tempdir().unwrap();
        let out_file = dir.path().join("out.txt");
        let hook = hook_path(dir.path(), "post-create");
        write_unix_hook(
            &hook,
            &format!(
                "#!/bin/sh\necho \"$CVM_HOOK_EVENT|$CVM_ENV|$CVM_ENV_PATH\" > {}\n",
                out_file.display()
            ),
        );

        execute_hook(dir.path(), "post-create", "work", Path::new("/envs/work")).unwrap();

        let contents = fs::read_to_string(&out_file).unwrap();
        assert_eq!(contents.trim(), "post-create|work|/envs/work");
    }

    #[cfg(unix)]
    #[test]
    fn execute_hook_propagates_nonzero_exit() {
        let dir = tempfile::tempdir().unwrap();
        let hook = hook_path(dir.path(), "pre-remove");
        write_unix_hook(&hook, "#!/bin/sh\nexit 1\n");

        let err = execute_hook(dir.path(), "pre-remove", "work", Path::new("/envs/work"))
            .unwrap_err();
        assert!(err.to_string().contains("pre-remove"));
    }

    #[cfg(unix)]
    #[test]
    fn non_executable_hook_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let hook = hook_path(dir.path(), "post-create");
        fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o644)).unwrap();

        execute_hook(dir.path(), "post-create", "work", Path::new("/envs/work")).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn run_pre_hook_propagates_failure() {
        let dir = tempfile::tempdir().unwrap();
        let hook = hook_path(dir.path(), "pre-remove");
        write_unix_hook(&hook, "#!/bin/sh\nexit 1\n");

        assert!(run_pre_hook(dir.path(), "pre-remove", "work", Path::new("/envs/work")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn run_post_hook_swallows_failure() {
        let dir = tempfile::tempdir().unwrap();
        let hook = hook_path(dir.path(), "post-remove");
        write_unix_hook(&hook, "#!/bin/sh\nexit 1\n");

        // Must not panic - failure is only printed to stderr.
        run_post_hook(dir.path(), "post-remove", "work", Path::new("/envs/work"));
    }
}
```

Registre o módulo em `src/main.rs`. O topo do arquivo hoje é:

```rust
mod cli;
mod env;
mod manifest;
mod shell;
mod shims;
mod update;
```

Troque por:

```rust
mod cli;
mod env;
mod hooks;
mod manifest;
mod shell;
mod shims;
mod update;
```

- [ ] **Step 2: Rodar os testes e confirmar que falham**

Run: `cargo test --lib hooks::`
Expected: FAIL — panic `not implemented` vindo de `unimplemented!()` (em `execute_hook`, `run_pre_hook` ou `run_post_hook`, dependendo de qual teste rodar primeiro).

- [ ] **Step 3: Implementar de verdade**

Substitua os três corpos `unimplemented!()` por:

```rust
pub fn run_pre_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) -> Result<()> {
    execute_hook(hooks_dir, event, env_name, env_dir)
}
```

```rust
pub fn run_post_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) {
    if let Err(err) = execute_hook(hooks_dir, event, env_name, env_dir) {
        eprintln!("warning: {err:#}");
    }
}
```

```rust
fn execute_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) -> Result<()> {
    let path = hook_path(hooks_dir, event);
    if !path.is_file() {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)
            .with_context(|| format!("failed to stat hook {}", path.display()))?
            .permissions()
            .mode();
        if mode & 0o111 == 0 {
            eprintln!(
                "warning: hook {} exists but is not executable (chmod +x it to enable), skipping",
                path.display()
            );
            return Ok(());
        }
    }

    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(&path);
        c
    } else {
        Command::new(&path)
    };

    cmd.env("CVM_HOOK_EVENT", event)
        .env("CVM_ENV", env_name)
        .env("CVM_ENV_PATH", env_dir);

    let status = cmd
        .status()
        .with_context(|| format!("failed to execute hook {}", path.display()))?;

    if !status.success() {
        bail!("hook '{event}' ({}) exited with {status}", path.display());
    }
    Ok(())
}
```

- [ ] **Step 4: Rodar os testes de novo e confirmar que passam**

Run: `cargo test --lib hooks::`
Expected: PASS (6 testes no Unix; 2 no Windows, já que os demais são `#[cfg(unix)]`).

- [ ] **Step 5: Ligar o evento `post-create` em `env::create_env`**

Em `src/env.rs`, o topo do arquivo hoje é:

```rust
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{bail, Context, Result};

use crate::shims::{env_bin_dir, write_env_shims};
```

Troque a última linha por:

```rust
use crate::hooks;
use crate::shims::{env_bin_dir, write_env_shims};
```

E a função `create_env` hoje termina assim:

```rust
    ensure_env_layout(&dir)?;
    let inherit_stats = if inherit {
        inherit_from_global(&dir)?
    } else {
        InheritStats::default()
    };

    Ok((dir, credentials_copied, inherit_stats))
}
```

Troque por:

```rust
    ensure_env_layout(&dir)?;
    let inherit_stats = if inherit {
        inherit_from_global(&dir)?
    } else {
        InheritStats::default()
    };

    hooks::run_post_hook(&hooks::hooks_dir()?, "post-create", name, &dir);

    Ok((dir, credentials_copied, inherit_stats))
}
```

- [ ] **Step 6: Escrever o teste de integração (falho antes da mudança acima, já passa agora)**

Adicione este teste dentro do `mod tests` já existente no final de `src/env.rs` (perto de `create_env_creates_skills_bin_and_dotenv`):

```rust
    #[test]
    fn create_env_runs_post_create_hook() {
        with_temp_home(|home| {
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let hook = hooks_dir.join("post-create");
            let marker = home.join("hook-ran.txt");
            fs::write(
                &hook,
                format!(
                    "#!/bin/sh\necho \"$CVM_HOOK_EVENT $CVM_ENV\" > {}\n",
                    marker.display()
                ),
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
            }

            create_env("work", true, false).unwrap();

            let contents = fs::read_to_string(&marker).unwrap();
            assert_eq!(contents.trim(), "post-create work");
        });
    }
```

Esse teste depende de `CVM_HOME` já apontar para `home.join(".cvm")` dentro de `with_temp_home` (confira a implementação de `with_temp_home` no topo do `mod tests` de `env.rs` — ela já define `CVM_HOME` como `home.path().join(".cvm")`), então `hooks::hooks_dir()` resolve para `home/.cvm/hooks` durante o teste.

Marque este teste com `#[cfg(unix)]` já que ele escreve um shebang `#!/bin/sh` — no Windows ele não compila/roda da mesma forma (fora de escopo deste teste; o comportamento Windows já está coberto pelos testes de `hooks.rs` que checam só a resolução do nome do arquivo, não a execução real de shell).

- [ ] **Step 7: Rodar toda a suíte e confirmar que passa**

Run: `cargo test`
Expected: PASS, incluindo `hooks::tests::*` e `env::tests::create_env_runs_post_create_hook`.

- [ ] **Step 8: Formatar e lintar**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings`
Expected: sem diffs de formatação pendentes, clippy sem warnings.

- [ ] **Step 9: Commit**

```bash
git add src/hooks.rs src/main.rs src/env.rs
git commit -m "feat(hooks): adiciona motor de hooks globais e evento post-create"
```

---

### Task 2: Eventos `pre-remove`/`post-remove`

**Files:**
- Modify: `src/env.rs` (função `remove_env`)
- Test: teste de integração em `src/env.rs`

**Interfaces:**
- Consumes: `hooks::hooks_dir()`, `hooks::run_pre_hook(...)`, `hooks::run_post_hook(...)` (de `src/hooks.rs`, Task 1)

- [ ] **Step 1: Escrever o teste de integração (deve falhar antes da mudança)**

Adicione ao `mod tests` de `src/env.rs`:

```rust
    #[test]
    fn remove_env_runs_pre_and_post_remove_hooks() {
        with_temp_home(|home| {
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let marker = home.join("hook-log.txt");

            let pre_hook = hooks_dir.join("pre-remove");
            fs::write(
                &pre_hook,
                format!("#!/bin/sh\necho \"pre $CVM_ENV\" >> {}\n", marker.display()),
            )
            .unwrap();
            let post_hook = hooks_dir.join("post-remove");
            fs::write(
                &post_hook,
                format!("#!/bin/sh\necho \"post $CVM_ENV\" >> {}\n", marker.display()),
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&pre_hook, fs::Permissions::from_mode(0o755)).unwrap();
                fs::set_permissions(&post_hook, fs::Permissions::from_mode(0o755)).unwrap();
            }

            create_env("work", true, false).unwrap();
            remove_env("work").unwrap();

            let contents = fs::read_to_string(&marker).unwrap();
            assert_eq!(contents, "pre work\npost work\n");
        });
    }

    #[test]
    fn remove_env_aborts_when_pre_remove_hook_fails() {
        with_temp_home(|home| {
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let pre_hook = hooks_dir.join("pre-remove");
            fs::write(&pre_hook, "#!/bin/sh\nexit 1\n").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&pre_hook, fs::Permissions::from_mode(0o755)).unwrap();
            }

            let (dir, _, _) = create_env("work", true, false).unwrap();

            assert!(remove_env("work").is_err());
            assert!(dir.is_dir(), "environment must not be deleted when pre-remove fails");
        });
    }
```

Marque as duas com `#[cfg(unix)]` (mesmo motivo do Step 6 da Task 1: usam shebang `#!/bin/sh`).

- [ ] **Step 2: Rodar e confirmar que falham**

Run: `cargo test --lib env::tests::remove_env`
Expected: FAIL — `remove_env` ainda não roda nenhum hook, então o marker file não é criado (`remove_env_runs_pre_and_post_remove_hooks` falha no `read_to_string`) e `remove_env("work")` não retorna erro quando o `pre-remove` falha (`remove_env_aborts_when_pre_remove_hook_fails` falha no `assert!(...is_err())`).

- [ ] **Step 3: Implementar**

A função `remove_env` hoje é:

```rust
pub fn remove_env(name: &str) -> Result<()> {
    let dir = ensure_env_exists(name)?;
    fs::remove_dir_all(&dir).with_context(|| format!("failed to remove {}", dir.display()))?;
    Ok(())
}
```

Troque por:

```rust
pub fn remove_env(name: &str) -> Result<()> {
    let dir = ensure_env_exists(name)?;
    let hooks_dir = hooks::hooks_dir()?;
    hooks::run_pre_hook(&hooks_dir, "pre-remove", name, &dir)?;
    fs::remove_dir_all(&dir).with_context(|| format!("failed to remove {}", dir.display()))?;
    hooks::run_post_hook(&hooks_dir, "post-remove", name, &dir);
    Ok(())
}
```

- [ ] **Step 4: Rodar de novo e confirmar que passam**

Run: `cargo test --lib env::tests::remove_env`
Expected: PASS.

- [ ] **Step 5: Rodar toda a suíte, formatar e lintar**

Run: `cargo test && cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings`
Expected: tudo passa, sem warnings.

- [ ] **Step 6: Commit**

```bash
git add src/env.rs
git commit -m "feat(hooks): dispara pre-remove/post-remove em cvm remove"
```

---

### Task 3: Eventos `pre-activate`/`post-activate`/`pre-deactivate`/`post-deactivate`

**Files:**
- Modify: `src/env.rs` (funções `resolve_activate` e `resolve_deactivate`, e o teste `deactivate_unsets_both_vars`)
- Modify: `src/main.rs` (`cmd_resolve_deactivate` e o `match` em `run()`)
- Test: testes de integração em `src/env.rs`

**Interfaces:**
- Consumes: `hooks::hooks_dir()`, `hooks::run_pre_hook(...)`, `hooks::run_post_hook(...)` (de `src/hooks.rs`, Task 1)
- Produces: `pub fn env::resolve_deactivate() -> anyhow::Result<Vec<String>>` (assinatura muda de `Vec<String>` para `Result<Vec<String>>` — quem chama precisa propagar/tratar o erro)

- [ ] **Step 1: Escrever os testes de integração (devem falhar antes da mudança)**

Adicione ao `mod tests` de `src/env.rs`:

```rust
    #[test]
    fn resolve_activate_runs_pre_and_post_activate_hooks() {
        with_temp_home(|home| {
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let marker = home.join("hook-log.txt");

            for event in ["pre-activate", "post-activate"] {
                let hook = hooks_dir.join(event);
                fs::write(
                    &hook,
                    format!("#!/bin/sh\necho \"{event} $CVM_ENV\" >> {}\n", marker.display()),
                )
                .unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
                }
            }

            create_env("work", true, false).unwrap();
            resolve_activate("work").unwrap();

            let contents = fs::read_to_string(&marker).unwrap();
            assert_eq!(contents, "pre-activate work\npost-activate work\n");
        });
    }

    #[test]
    fn resolve_activate_aborts_when_pre_activate_hook_fails() {
        with_temp_home(|home| {
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let hook = hooks_dir.join("pre-activate");
            fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
            }

            create_env("work", true, false).unwrap();

            assert!(resolve_activate("work").is_err());
        });
    }

    #[test]
    fn resolve_deactivate_runs_pre_and_post_deactivate_hooks_when_env_active() {
        with_temp_home(|home| {
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let marker = home.join("hook-log.txt");

            for event in ["pre-deactivate", "post-deactivate"] {
                let hook = hooks_dir.join(event);
                fs::write(
                    &hook,
                    format!("#!/bin/sh\necho \"{event} $CVM_ENV\" >> {}\n", marker.display()),
                )
                .unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
                }
            }

            create_env("work", true, false).unwrap();
            // SAFETY: guarded by HOME_LOCK (held for the whole with_temp_home closure).
            unsafe {
                env::set_var(ACTIVE_ENV_VAR, "work");
            }

            resolve_deactivate().unwrap();

            let contents = fs::read_to_string(&marker).unwrap();
            assert_eq!(contents, "pre-deactivate work\npost-deactivate work\n");

            unsafe {
                env::remove_var(ACTIVE_ENV_VAR);
            }
        });
    }

    #[test]
    fn resolve_deactivate_skips_hooks_when_no_env_active() {
        with_temp_home(|_home| {
            // No CVM_ENV set - resolve_deactivate must not error and must not
            // try to resolve any hook.
            let vars = resolve_deactivate().unwrap();
            assert!(vars.contains(&CONFIG_DIR_VAR.to_string()));
        });
    }
```

Marque `resolve_activate_runs_pre_and_post_activate_hooks`, `resolve_activate_aborts_when_pre_activate_hook_fails` e `resolve_deactivate_runs_pre_and_post_deactivate_hooks_when_env_active` com `#[cfg(unix)]` (usam shebang). `resolve_deactivate_skips_hooks_when_no_env_active` roda em qualquer plataforma.

O teste já existente `deactivate_unsets_both_vars` hoje é:

```rust
    #[test]
    fn deactivate_unsets_both_vars() {
        let vars = resolve_deactivate();
        assert!(vars.contains(&CONFIG_DIR_VAR.to_string()));
        assert!(vars.contains(&ACTIVE_ENV_VAR.to_string()));
    }
```

Troque por (adiciona `.unwrap()`, já que `resolve_deactivate` passa a retornar `Result`):

```rust
    #[test]
    fn deactivate_unsets_both_vars() {
        let vars = resolve_deactivate().unwrap();
        assert!(vars.contains(&CONFIG_DIR_VAR.to_string()));
        assert!(vars.contains(&ACTIVE_ENV_VAR.to_string()));
    }
```

- [ ] **Step 2: Rodar e confirmar que falham**

Run: `cargo test --lib env::tests::resolve_activate_runs_pre_and_post_activate_hooks env::tests::resolve_activate_aborts_when_pre_activate_hook_fails env::tests::resolve_deactivate_runs_pre_and_post_deactivate_hooks_when_env_active env::tests::deactivate_unsets_both_vars`
Expected: FAIL — não compila ainda (`deactivate_unsets_both_vars` chama `.unwrap()` num `Vec<String>`, que não tem esse método com essa semântica de erro) ou, uma vez ajustada a assinatura, falha porque nenhum hook roda de fato.

- [ ] **Step 3: Implementar**

`resolve_activate` hoje é:

```rust
pub fn resolve_activate(name: &str) -> Result<Vec<(String, String)>> {
    let dir = ensure_env_exists(name)?;
    ensure_env_bin(&dir)?;
    let dir_str = dir
        .to_str()
        .with_context(|| format!("environment path is not valid UTF-8: {}", dir.display()))?
        .to_string();
    let mut pairs = load_env_file(&dir)?;
    pairs.push((CONFIG_DIR_VAR.to_string(), dir_str));
    pairs.push((ACTIVE_ENV_VAR.to_string(), name.to_string()));
    Ok(pairs)
}
```

Troque por:

```rust
pub fn resolve_activate(name: &str) -> Result<Vec<(String, String)>> {
    let dir = ensure_env_exists(name)?;
    let hooks_dir = hooks::hooks_dir()?;
    hooks::run_pre_hook(&hooks_dir, "pre-activate", name, &dir)?;
    ensure_env_bin(&dir)?;
    let dir_str = dir
        .to_str()
        .with_context(|| format!("environment path is not valid UTF-8: {}", dir.display()))?
        .to_string();
    let mut pairs = load_env_file(&dir)?;
    pairs.push((CONFIG_DIR_VAR.to_string(), dir_str));
    pairs.push((ACTIVE_ENV_VAR.to_string(), name.to_string()));
    hooks::run_post_hook(&hooks_dir, "post-activate", name, &dir);
    Ok(pairs)
}
```

`resolve_deactivate` hoje é:

```rust
pub fn resolve_deactivate() -> Vec<String> {
    let mut vars = vec![CONFIG_DIR_VAR.to_string(), ACTIVE_ENV_VAR.to_string()];
    if let Some(name) = active_env() {
        if let Ok(dir) = env_dir(&name) {
            if let Ok(env_vars) = load_env_file(&dir) {
                vars.extend(env_vars.into_iter().map(|(key, _)| key));
            }
        }
    }
    vars
}
```

Troque por:

```rust
pub fn resolve_deactivate() -> Result<Vec<String>> {
    let mut vars = vec![CONFIG_DIR_VAR.to_string(), ACTIVE_ENV_VAR.to_string()];
    if let Some(name) = active_env() {
        if let Ok(dir) = env_dir(&name) {
            let hooks_dir = hooks::hooks_dir()?;
            hooks::run_pre_hook(&hooks_dir, "pre-deactivate", &name, &dir)?;
            if let Ok(env_vars) = load_env_file(&dir) {
                vars.extend(env_vars.into_iter().map(|(key, _)| key));
            }
            hooks::run_post_hook(&hooks_dir, "post-deactivate", &name, &dir);
        }
    }
    Ok(vars)
}
```

Agora atualize `src/main.rs`. A função `cmd_resolve_deactivate` hoje é:

```rust
fn cmd_resolve_deactivate() {
    for var in env::resolve_deactivate() {
        println!("{var}");
    }
}
```

Troque por:

```rust
fn cmd_resolve_deactivate() -> Result<()> {
    for var in env::resolve_deactivate()? {
        println!("{var}");
    }
    Ok(())
}
```

E, no `match cli.command { ... }` dentro de `fn run() -> Result<()>`, a linha:

```rust
        Command::ResolveDeactivate => cmd_resolve_deactivate(),
```

vira:

```rust
        Command::ResolveDeactivate => cmd_resolve_deactivate()?,
```

- [ ] **Step 4: Rodar de novo e confirmar que passam**

Run: `cargo test`
Expected: PASS — toda a suíte, incluindo os novos testes e `deactivate_unsets_both_vars` ajustado.

- [ ] **Step 5: Formatar e lintar**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings`
Expected: sem diffs pendentes, sem warnings.

- [ ] **Step 6: Commit**

```bash
git add src/env.rs src/main.rs
git commit -m "feat(hooks): dispara pre/post-activate e pre/post-deactivate em cvm use/deactivate"
```

---

### Task 4: Documentação no README

**Files:**
- Modify: `README.md`

**Interfaces:**
- Nenhuma (não há dependência de código; documenta o comportamento já implementado nas Tasks 1–3).

- [ ] **Step 1: Adicionar o bullet em "Features"**

Em `README.md`, a lista de features termina hoje com:

```markdown
- **Cross-shell** — bash, zsh, fish, and PowerShell are all first-class.
```

Adicione logo depois, ainda dentro da mesma lista:

```markdown
- **Lifecycle hooks** — drop executable scripts in `~/.cvm/hooks/` (e.g.
  `post-create`, `pre-activate`, `post-deactivate`) and `cvm` runs them
  automatically for every environment. `pre-*` hooks can abort the
  operation by exiting non-zero; `post-*` hooks only ever print a warning.
```

- [ ] **Step 2: Adicionar a seção "### Lifecycle hooks"**

Em `README.md`, a seção `### Manifest handling` termina hoje com:

```markdown
Any other keys already present in an environment's `settings.json` (or
values already set in `.env`) are left untouched on import — `cvm` only
ever merges the fields it manages.

## Showing the Active Environment in Your Statusline
```

Insira uma nova subseção entre as duas, resultando em:

```markdown
Any other keys already present in an environment's `settings.json` (or
values already set in `.env`) are left untouched on import — `cvm` only
ever merges the fields it manages.

### Lifecycle hooks

Drop an executable script per event into `~/.cvm/hooks/` and `cvm` runs it
automatically, for every environment — no configuration file needed:

| Event             | Runs                                                   | A failing hook (non-zero exit)... |
|-------------------|---------------------------------------------------------|--------------------------------------|
| `post-create`     | after `cvm create` finishes setting up the environment  | only prints a warning                |
| `pre-activate`    | before `cvm use`/`activate` exports its variables        | aborts the activation                |
| `post-activate`   | after `cvm use`/`activate` exports its variables          | only prints a warning                |
| `pre-deactivate`  | before `cvm deactivate` unsets its variables              | aborts the deactivation              |
| `post-deactivate` | after `cvm deactivate` unsets its variables                | only prints a warning                |
| `pre-remove`      | before `cvm remove` deletes the environment directory      | aborts the removal                   |
| `post-remove`     | after `cvm remove` deletes the environment directory        | only prints a warning                |

Each hook receives which event fired, plus the environment's name and
directory, as environment variables:

- `CVM_HOOK_EVENT` — the event name (e.g. `post-create`)
- `CVM_ENV` — the environment's name
- `CVM_ENV_PATH` — the environment's absolute directory (may no longer exist
  by the time `post-remove` runs)

On Unix, a hook file needs to be executable (`chmod +x`) and can use any
shebang; a present-but-non-executable hook is skipped with a warning rather
than blocking anything. On Windows, hooks use a `.cmd` extension (e.g.
`post-create.cmd`), matching the convention already used for the `bin/`
shims.

Hooks are global and local to your machine — they are never included in
`cvm export`/`cvm import`, so importing a teammate's `cvm.yaml` never runs
code you didn't write yourself.

## Showing the Active Environment in Your Statusline
```

- [ ] **Step 3: Revisar visualmente**

Run: `grep -n "Lifecycle hooks" README.md`
Expected: duas ocorrências — o bullet em "Features" e o título da nova seção.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs(readme): documenta hooks de ciclo de vida"
```

---

## Suggested Stack

Tasks 1 → 2 → 3 constroem em cima uma da outra no mesmo arquivo (`src/env.rs`) e cada uma entrega um pedaço de comportamento testável de forma independente (post-create funcionando; depois pre/post-remove; depois pre/post-activate/deactivate) — candidatas a uma stack de 3 PRs com `gh-stack` (extensão já instalada neste ambiente).

## Suggested Parallel Batch

Task 4 só toca `README.md`, sem sobreposição de arquivos nem dependência de interface com as Tasks 1–3 (é só documentação) — candidata a rodar em paralelo com qualquer uma delas. **Nota:** esta é uma sugestão em tempo de plano; o controlador de execução deve reconfirmar a ausência de sobreposição antes de despachar em paralelo, já que o plano pode evoluir entre a escrita e a execução.
