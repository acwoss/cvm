# cvm — Claude Virtualenv Manager

`cvm` is a fast, lightweight, cross-platform virtual environment manager for
[Claude Code](https://github.com/anthropics/claude-code). It isolates
configuration, credentials, history, memories, and MCP server registrations
per-project or per-context by managing and overriding the
`CLAUDE_CONFIG_DIR` environment variable — the same mechanism Claude Code
already uses to locate its config directory.

On top of that, `cvm` lets teams **share and reproduce Claude Code setups**
through a plain YAML manifest (`cvm.yaml`) that can live in a Git repository
alongside your code, without ever leaking tokens, auth files, or session
history.

## Features

- **Isolated environments** — each environment is its own directory under
  `~/.cvm/envs/<name>`, completely separate from your global `~/.claude`
  config and from every other environment.
- **No repeated logins** — `cvm create` copies your global Claude Code
  credentials into the new environment by default, so it starts out already
  logged in. Pass `--anonymous` to skip that and get a completely empty
  environment instead.
- **Zero config drift between machines** — export an environment's settings,
  MCP servers, and skills to `cvm.yaml`; teammates import it and get an
  identical setup in one command.
- **Per-environment secrets in `.env`** — each environment can have its own
  `.env` file for things like MCP server credentials, editable with `cvm edit
  <env>`. It's loaded into the process on `use`/`run`/`open`, but `cvm export`
  only ever shares the *names* of those variables, never their values.
  Activation-owned keys such as `PATH`, `CLAUDE_CONFIG_DIR`, and the `CVM_*`
  namespace are ignored so an environment cannot corrupt activation state.
- **Safe by construction** — export/import only ever touch `settings.json`
  (permissions + `mcpServers`), the `skills/` directory, and the *names* of
  `.env` variables. Auth tokens, credentials, and history files are never
  read, matched, or written by those code paths, and `.env` values never
  leave the machine they're on.
- **Ad hoc, no-commitment runs** — `cvm run <env> -- claude` runs a single
  command inside an environment's context without switching your whole
  shell session.
- **Environment-local command shims** — every environment gets `bin/claude`
  and `bin/skills`. Activating an environment or using `cvm run` puts that
  directory first on `PATH`, so both commands automatically use the selected
  Claude config. The skills shim invokes `npx --yes skills`.
- **Project auto-activation** — put an environment name in a project's `.cvm`
  file and the shell hook activates it when you enter that directory tree.
- **Parallel Claude Code instances** — `cvm open <env>` launches Claude Code
  scoped to `<env>`, tagging that single process with `CVM_ENV=<env>` so you
  can run several isolated instances side by side (e.g. one per client or
  project) without them stepping on each other or on your shell.
- **Cross-shell** — bash, zsh, fish, and PowerShell are all first-class.
- **Lifecycle hooks** — drop executable scripts in `~/.cvm/hooks/` (e.g.
  `post-create`, `pre-activate`, `post-deactivate`) and `cvm` runs them
  automatically for every environment. `pre-*` hooks can abort the
  operation by exiting non-zero; `post-*` hooks only ever print a warning.

## Installation

### One-line install script

Linux / macOS:

```sh
curl -fsSL https://getcvm.com/install.sh | bash
```

Windows (PowerShell):

```powershell
powershell -c "irm https://getcvm.com/install.ps1 | iex"
```

This downloads the right prebuilt binary for your OS/architecture, installs
it to `~/.cvm/bin` (`%USERPROFILE%\.cvm\bin` on Windows), and appends the
PATH + shell-hook lines to your shell rc file / PowerShell profile (see
[Shell Integration Setup](#shell-integration-setup) if you'd rather do that
by hand).

### From a release binary

Download the archive for your platform from the
[Releases page](https://github.com/acwoss/cvm/releases), extract it, and
place the `cvm` binary somewhere on your `PATH` (e.g. `~/.cvm/bin`).

### From source with Cargo

```sh
git clone https://github.com/acwoss/cvm.git
cd cvm
cargo install --path . --locked
```

### Updating

```sh
cvm update
```

Checks GitHub Releases for a newer version and, if one exists, downloads
the right asset for your platform and replaces the running binary in
place — no need to re-run the install script. Requires `curl` and `tar` on
`PATH` (the same requirement `install.sh` already has). If you installed
via `cargo install`, re-run that command instead.

## Shell Integration Setup

`cvm` is a compiled binary, so it cannot change the environment variables of
the shell that launched it — a child process can never modify its parent's
environment. `cvm init <shell>` prints a small shell function that wraps the
`cvm` binary; that wrapper is what actually exports/unsets
`CLAUDE_CONFIG_DIR` in your current shell when you run `cvm use`/`cvm
activate`/`cvm deactivate`. Every other subcommand is forwarded to the real
binary unchanged.

Add the appropriate line to your shell's startup file and restart your
shell (or source the file):

| Shell      | File                          | Line to add                                              |
|------------|-------------------------------|-----------------------------------------------------------|
| Bash       | `~/.bashrc`                   | `eval "$(cvm init bash)"`                                 |
| Zsh        | `~/.zshrc`                    | `eval "$(cvm init zsh)"`                                  |
| Fish       | `~/.config/fish/config.fish`  | `cvm init fish \| source`                                 |
| PowerShell | `$PROFILE`                    | `cvm init powershell \| Out-String \| Invoke-Expression`  |

Once installed, calling `cvm` in your shell always goes through the wrapper
function first — `use`, `activate`, and `deactivate` are intercepted, and
everything else is passed straight through to the compiled binary.

The shell hook prefixes the prompt with the active environment name, such as
`(work)`, saves the original `PATH` in `CVM_OLD_PATH`, and prepends the active
environment's `bin/` directory. `cvm deactivate` restores both the previous
prompt and `PATH`. After upgrading `cvm`, re-run `cvm init` for your shell to
install the latest hook.

If you run the raw `cvm use`/`cvm deactivate` binary without this hook
installed, `cvm` prints a warning explaining that shell integration isn't
active instead of silently doing nothing.

### Project auto-activation with `.cvm`

Create a `.cvm` file in a project directory containing the environment name:

```text
# Claude environment for this project
project-backend-api
```

The first non-empty line that is not a `#` comment is the environment name.
On directory changes, the shell hook searches the current directory and its
parents for `.cvm`. It activates a different named environment when found and
automatically deactivates it after you leave that directory tree. A manual
`cvm use <env>` pins the session instead, so later directory changes do not
deactivate that deliberate selection.

After upgrading, re-run `cvm init` in each shell configuration to install the
auto-activation hook.

If you prefer [direnv](https://direnv.net/), an optional `.envrc` equivalent
can consume the activation resolver's `KEY=VALUE` output:

```sh
while IFS='=' read -r key value; do
  export "$key=$value"
done < <(cvm __resolve-activate project-backend-api)
PATH="$CLAUDE_CONFIG_DIR/bin:$PATH"
export PATH
```

## Sharing Environments (`cvm.yaml`)

`cvm export` and `cvm import` let a team commit a reproducible Claude Code
setup to a Git repository, right next to the project it configures.

### Exporting

```sh
cvm use project-backend-api
cvm export -o cvm.yaml
```

This inspects the active environment's `settings.json`, `skills/` directory,
and the *variable names* (never values) in `.env`, and writes a manifest
like:

```yaml
name: project-backend-api
version: "1.0.0"
description: "Standardized Claude environment for backend team"

# Non-sensitive settings & permission rules
settings:
  allowed_tools:
    - "read-file"
    - "run-tests"

# MCP Servers to be registered in the environment
mcp_servers:
  postgres:
    command: "npx"
    args:
      - "-y"
      - "@modelcontextprotocol/server-postgres"
      - "postgresql://localhost:5432/dev_db"
  github:
    command: "npx"
    args:
      - "-y"
      - "@modelcontextprotocol/server-github"

# Custom skills or extensions registered in this env
skills:
  - "git-conventional-commits"
  - "prisma-migration-helper"

# Names of variables expected in this environment's .env file (e.g. MCP
# server credentials). Only names are shared here - never values.
env_vars:
  - "POSTGRES_PASSWORD"
  - "GITHUB_TOKEN"
```

Commit `cvm.yaml` to the repo. Teammates run `cvm import` to reproduce the
same environment locally.

### Importing

```sh
cvm import cvm.yaml -n project-backend-api
cvm use project-backend-api
```

`cvm import` creates the environment if it doesn't exist yet (or updates it
in place if it does), merges the manifest's permissions and `mcpServers`
into `settings.json`, creates a placeholder directory + `SKILL.md` stub for
every skill listed (fill those in, or reinstall the skill from its original
source/marketplace), and ensures `.env` has a blank entry for every variable
name in `env_vars` (existing values, if any, are left untouched). Teammates
fill in the real values themselves:

```sh
cvm import cvm.yaml -n project-backend-api
$EDITOR ~/.cvm/envs/project-backend-api/.env   # fill in POSTGRES_PASSWORD, GITHUB_TOKEN, ...
cvm use project-backend-api
```

### Security guarantee

Export **never** includes:

- `auth.json`, `.credentials.json`, or any OAuth/API tokens
- Session history (`history.json` / `history.jsonl`)
- Memories or other local-only state
- `.env` **values** — only the variable *names* are shared, so a teammate
  knows what to set without ever seeing what you set it to

This isn't a filtering step that could miss a new sensitive filename in the
future — the export/import code paths are architecturally limited to three
things inside an environment directory: `settings.json`, the `skills/`
subdirectory, and `.env` (from which only keys, never values, are ever
copied into the manifest). Nothing else is ever opened.

## Usage Guide & Examples

Create and activate an environment:

```sh
cvm create work
cvm use work
claude   # env-local shim runs against ~/.cvm/envs/work instead of ~/.claude
skills   # env-local shim runs npx --yes skills with the same environment
```

To create an environment and open Claude Code in it right away:

```sh
cvm create work --open
```

To seed a new environment with your global skills and settings:

```sh
cvm create work --inherit
```

`--inherit` links each skill from `~/.claude/skills` into the environment
(falling back to a recursive copy when links are unavailable) and copies
`~/.claude/settings.json` when present. The new environment remains independent
after creation except for skills that were successfully linked.

`cvm create` copies your global Claude Code credentials
(`~/.claude/.credentials.json`) into the new environment if that file
exists, so `claude` above doesn't prompt you to log in again. Everything
else about the environment (settings, MCP servers, `.env`, history) still
starts out empty and isolated. If you want a completely empty environment
instead — no shared credentials at all — pass `--anonymous`:

```sh
cvm create sandbox --anonymous
cvm use sandbox
claude   # prompts for a fresh login
```

> **Note:** this only works on platforms where Claude Code stores its
> credentials as a file in the config directory (Linux, Windows). On macOS,
> Claude Code may store them in the system Keychain instead, in which case
> there is nothing for `cvm create` to copy and you'll still need to log in
> once per environment.

Give it credentials an MCP server needs, without putting them in
`settings.json` (and therefore never in `cvm.yaml` either):

```sh
cvm edit work   # opens ~/.cvm/envs/work/.env in $VISUAL/$EDITOR, creating it if needed
# POSTGRES_PASSWORD=super-secret-value
# GITHUB_TOKEN=ghp_xxx
cvm run work -- claude mcp list   # POSTGRES_PASSWORD/GITHUB_TOKEN are set for this process only
```

`cvm edit` also works without a name if you have an environment active
(`cvm use work` first), and respects `$VISUAL` before falling back to
`$EDITOR` — the same convention `git commit`/`crontab -e` use.

Check what's active:

```sh
cvm current
# work
```

List all environments:

```sh
cvm list
#   personal
# * work (active)
```

Go back to your default global Claude Code setup:

```sh
cvm deactivate
```

Run a single command in an environment without switching your session:

```sh
cvm run client-acme -- claude
cvm run client-acme -- claude mcp list
```

Open Claude Code directly inside an environment — a shorthand for `cvm run
<env> -- claude` that also tags the process with `CVM_ENV=<env>`:

```sh
cvm open client-acme
```

Because each invocation only sets `CVM_ENV`/`CLAUDE_CONFIG_DIR` on that one
child process, you can open several isolated instances in parallel (e.g. in
separate terminal tabs or tmux panes) without any of them interfering with
each other or with your shell's own active environment:

```sh
cvm open client-acme &
cvm open client-globex &
cvm open personal &
```

Remove an environment you no longer need:

```sh
cvm remove work
# Delete environment 'work'? This cannot be undone. [y/N]
```

Share and reproduce a setup across the team:

```sh
cvm export -o cvm.yaml
git add cvm.yaml && git commit -m "Add shared Claude environment"

# teammate:
git pull
cvm import cvm.yaml -n project-backend-api
cvm use project-backend-api
```

## Command Reference

| Command                              | Aliases      | Description                                                                 |
|---------------------------------------|--------------|-------------------------------------------------------------------------------|
| `cvm init <shell>`                    | —            | Prints shell integration hooks for `bash`, `zsh`, `fish`, or `powershell`.    |
| `cvm update`                          | —            | Checks GitHub Releases for a newer version and replaces the running binary in place. |
| `cvm create <env> [--anonymous] [--inherit] [--open]` | — | Creates a new isolated environment at `~/.cvm/envs/<env>`, reusing global credentials unless `--anonymous` is passed. `--inherit` links global skills and copies global settings; `--open` launches Claude Code immediately. |
| `cvm list`                            | `ls`         | Lists all environments, highlighting the active one.                         |
| `cvm use <env>`                       | `activate`   | Activates `<env>` in the current shell session (needs shell integration).    |
| `cvm deactivate`                      | —            | Restores the default global Claude Code setup (needs shell integration).     |
| `cvm current`                         | —            | Prints the name of the environment active in this shell.                     |
| `cvm remove <env>`                    | `rm`         | Deletes an environment directory, with confirmation. `-y`/`--yes` to skip.   |
| `cvm edit [env]`                      | —            | Opens `<env>`'s `.env` in `$VISUAL`/`$EDITOR` (defaults to active), creating it first if missing. |
| `cvm run <env> -- <cmd>`              | —            | Runs `<cmd>` scoped to `<env>` without activating it globally.               |
| `cvm open <env>`                      | —            | Runs `claude` scoped to `<env>`, tagging that process with `CVM_ENV=<env>`. Alias for `cvm run <env> -- claude`; safe to run several in parallel. |
| `cvm export [env] [-o <file>]`        | —            | Exports an environment (defaults to active) to a YAML manifest. `.env` values are never included, only variable names. |
| `cvm import <file> [-n <env>]`        | —            | Creates/updates an environment from a YAML manifest. Recreates `.env` with blank placeholders for any variable names listed. |

## How It Works

### Environment variables

Claude Code reads `CLAUDE_CONFIG_DIR` to decide where its config lives
(defaulting to `~/.claude`). `cvm` never patches Claude Code itself. It sets
these environment variables:

- `CLAUDE_CONFIG_DIR` — points at `~/.cvm/envs/<name>` while `<name>` is
  active.
- `CVM_ENV` — the name of the environment a process is running under, used
  by `cvm current`, `cvm list`, and available to any script (like a
  statusline) that wants to know which environment is active.
- `CVM_OLD_PATH` — shell-hook backup used only while an environment is
  activated, so `cvm deactivate` can restore the original `PATH`.

Because a process can't mutate its parent shell's environment, `use`,
`activate`, and `deactivate` are implemented as a shell function (installed
by `cvm init`) that asks the compiled binary — via the hidden
`cvm __resolve-activate <env>` / `cvm __resolve-deactivate` commands — which
variables to export or unset, then applies them itself. That resolution also
loads the environment's `.env` file, so its variables get exported (and
later unset) right alongside `CLAUDE_CONFIG_DIR`/`CVM_ENV`. `cvm run` and
`cvm open` sidestep the shell function entirely: they spawn the target
command directly as a child process with `.env`'s variables plus
`CLAUDE_CONFIG_DIR`/`CVM_ENV` already set and the environment's `bin/`
prepended to its `PATH`, so they work with or without shell integration
installed. Multiple `cvm open` processes can run in parallel without
interfering with each other or with whatever environment (if any) is active
in the parent shell.

### Directory structure

```
~/.cvm/
├── bin/                  # cvm binary, if installed via install.sh
└── envs/
    ├── work/             # = $CLAUDE_CONFIG_DIR when "work" is active
    │   ├── .env          # starter file for MCP credentials & other local secrets
    │   ├── skills/       # custom skills (created on `cvm create`)
    │   ├── bin/          # claude/skills shims, first on PATH while active
    │   ├── settings.json
    │   └── ...           # anything else Claude Code itself creates here
    └── personal/
        └── ...
```

Each environment directory *is* the `CLAUDE_CONFIG_DIR` Claude Code will use
— `cvm` doesn't copy or mirror files into a separate location.

The generated `claude` shim removes its own `bin/` directory from `PATH`
before resolving the real Claude executable, which prevents recursion. The
`skills` shim does the same environment setup and delegates to `npx --yes
skills`. Existing environments receive missing or stale shims lazily the next
time they are activated or used with `cvm run`/`cvm open`.

### Manifest handling

`cvm.yaml` is a deliberately small, human-reviewable surface:

- `settings.allowed_tools` maps to `permissions.allow` in `settings.json`.
- `mcp_servers` maps directly to the `mcpServers` block in `settings.json`.
- `skills` is a list of skill names; import creates a stub directory per
  skill under `skills/<name>/SKILL.md` for you to fill in or reinstall.
- `env_vars` is a list of variable *names* found in `.env`; import ensures
  each one exists in `.env` (blank if new, untouched if you'd already set
  it), and you fill in the real values by hand.

Any other keys already present in an environment's `settings.json` (or
values already set in `.env`) are left untouched on import — `cvm` only
ever merges the fields it manages.

### Lifecycle hooks

Drop an executable script per event into `~/.cvm/hooks/` and `cvm` runs it
automatically, for every environment — no configuration file needed:

| Event             | Runs                                                                                   | A failing hook (non-zero exit)... |
|-------------------|-----------------------------------------------------------------------------------------|--------------------------------------|
| `post-create`     | after `cvm create` finishes setting up the environment                                  | only prints a warning                |
| `pre-activate`    | before `cvm` resolves the variables to export for `cvm use`/`activate`                  | aborts the activation                |
| `post-activate`   | after `cvm` has resolved the variables to export (the shell applies them immediately afterwards) | only prints a warning       |
| `pre-deactivate`  | before `cvm` resolves the variables to unset for `cvm deactivate`                        | aborts the deactivation              |
| `post-deactivate` | after `cvm` has resolved the variables to unset (the shell unsets them immediately afterwards) | only prints a warning          |
| `pre-remove`      | before `cvm remove` deletes the environment directory                                    | aborts the removal                   |
| `post-remove`     | after `cvm remove` deletes the environment directory                                     | only prints a warning                |

`pre-activate`/`post-activate` and `pre-deactivate`/`post-deactivate` all run
*inside* `cvm`'s hidden `__resolve-activate`/`__resolve-deactivate` commands,
which is before the shell wrapper installed by `cvm init` has actually
exported or unset anything in your shell (see [Shell Integration
Setup](#shell-integration-setup) — the wrapper only applies the returned
variables after `cvm` returns). This means a hook can't rely on inherited
variables like `CLAUDE_CONFIG_DIR` to reflect the new/old state — a
`post-activate` hook reading it from its own inherited environment still
sees the *previous* value (or none), and a `pre-deactivate` hook still sees
the *currently active* one either way. Rely on the explicitly injected
`CVM_HOOK_EVENT`/`CVM_ENV`/`CVM_ENV_PATH` instead (see below).

Switching directly from one active environment to another (`cvm use B` while
`A` is active) runs `B`'s `pre-activate`/`post-activate` *before* `A`'s
`pre-deactivate`/`post-deactivate` — the shell wrapper resolves and applies
the new environment first, only deactivating the old one afterwards. If
`A`'s `pre-deactivate` hook then fails, the command aborts after `B`'s
`post-activate` side effects have already run, with neither environment's
variables actually swapped in the shell. Hook authors relying on stateful
pairs (e.g. a lock file written by `pre-activate` and removed by
`post-deactivate`) should account for this ordering.

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

A hook's stdout is always discarded, for every event — a `post-create` or
`pre-remove` hook that `echo`s progress won't print anything the user can
see. Hooks that want to print something for the user should write to
stderr instead.

Hooks are global and local to your machine — they are never included in
`cvm export`/`cvm import` (importing an environment that doesn't exist yet
does not fire `post-create` either), so importing a teammate's `cvm.yaml`
never runs code you didn't write yourself.

## Showing the Active Environment in Your Statusline

When you run several `cvm open`/`cvm run` instances in parallel (or just
switch between activated environments a lot), it's easy to lose track of
which terminal is pointed at which `CLAUDE_CONFIG_DIR`. Claude Code's
statusline can surface that for you, since `CVM_ENV` is already set in the
environment of any process started via `cvm use`/`activate`, `cvm run`, or
`cvm open`.

Paste the prompt below into Claude Code (in the project or globally, your
choice) to have it wire this up for you:

> Please update my Claude Code statusline so it shows the active cvm
> environment when there is one. Find my current `statusLine` configuration
> (the `statusLine` block in `settings.json` — check the project's
> `.claude/settings.json`, then `~/.claude/settings.json`) and adjust its
> command/script in place, without removing anything it already displays.
>
> Requirements:
> - If the `CVM_ENV` environment variable is set and non-empty when the
>   statusline script runs, prepend a short badge with its value (e.g.
>   `[env-name]`) before the rest of the existing statusline output.
> - If `CVM_ENV` is not set (or empty), the statusline must render exactly
>   as it does today — no empty brackets, no extra spaces.
> - Keep whatever language/tooling the existing statusline script already
>   uses (shell, Node, Python, etc.) instead of rewriting it in something
>   else.
> - If there is no statusline configured yet, create a minimal one that
>   shows the model name plus the `CVM_ENV` badge when present.
>
> Show me the diff before writing it, and explain where the file lives.

This keeps the change scoped to your own statusline script and lets Claude
Code adapt it to however your statusline is already implemented, rather than
prescribing one exact shell snippet here.

## Development

```sh
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to set up a pull request.

## License

[MIT](LICENSE)
