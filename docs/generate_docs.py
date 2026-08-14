#!/usr/bin/env python3
"""Generate cvm documentation HTML pages."""
from pathlib import Path

ROOT = Path(__file__).resolve().parent / "documentation"
ROOT.mkdir(parents=True, exist_ok=True)
(ROOT / "how-to").mkdir(exist_ok=True)

SHELL = """\
<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{title} — cvm docs</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet" />
  <link rel="stylesheet" href="{asset}/docs.css" />
</head>
<body data-page="{page}" data-depth="{depth}">
  <header class="docs-topbar">
    <button class="menu-btn" id="menu-btn" aria-label="Open navigation">Menu</button>
    <a class="docs-brand" id="link-home" href="{home}">
      <span class="docs-brand-mark">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
          <rect x="5" y="8" width="14" height="10" rx="2" />
          <circle cx="9" cy="13" r="1.2" fill="currentColor" stroke="none" />
          <circle cx="15" cy="13" r="1.2" fill="currentColor" stroke="none" />
          <line x1="9" y1="16.5" x2="15" y2="16.5" stroke-width="1.5" />
          <line x1="12" y1="8" x2="12" y2="5" />
          <circle cx="12" cy="4.5" r="1" fill="currentColor" stroke="none" />
        </svg>
      </span>
      cvm
    </a>
    <div class="docs-topbar-sep"></div>
    <span class="docs-topbar-label">Documentation</span>
    <div class="docs-topbar-spacer"></div>
    <a class="docs-topbar-link" id="link-github" href="https://github.com/acwoss/cvm">GitHub</a>
    <button class="theme-toggle" id="theme-toggle" aria-label="Toggle theme"><span class="theme-toggle-thumb"></span></button>
  </header>
  <div class="docs-shell">
    <aside class="docs-sidebar" id="docs-sidebar"></aside>
    <main class="docs-main">
      <div class="docs-kicker">{kicker}</div>
      <h1>{h1}</h1>
      <p class="docs-lead">{lead}</p>
{body}
      <div class="docs-pager">
        {prev}
        {next}
      </div>
    </main>
  </div>
  <script src="{asset}/docs.js"></script>
</body>
</html>
"""


def pager(prev=None, nxt=None):
    p = (
        f'<a href="{prev[0]}"><span class="label">Previous</span><span class="title">{prev[1]}</span></a>'
        if prev
        else "<span></span>"
    )
    n = (
        f'<a class="next" href="{nxt[0]}"><span class="label">Next</span><span class="title">{nxt[1]}</span></a>'
        if nxt
        else "<span></span>"
    )
    return p, n


def write(rel, page, depth, title, kicker, h1, lead, body, prev=None, nxt=None):
    asset = "../assets" if depth == 0 else "../../assets"
    home = "../index.html" if depth == 0 else "../../index.html"
    p, n = pager(prev, nxt)
    html = SHELL.format(
        title=title,
        page=page,
        depth=depth,
        asset=asset,
        home=home,
        kicker=kicker,
        h1=h1,
        lead=lead,
        body=body,
        prev=p,
        next=n,
    )
    path = ROOT / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(html, encoding="utf-8")
    print("wrote", rel)


# Overview already exists but regenerate for consistency
write(
    "index.html",
    "overview",
    0,
    "Overview",
    "Overview",
    "cvm documentation",
    "Technical reference for isolating and sharing Claude Code environments with cvm.",
    """
<div class="card-grid">
  <a class="card-link" href="getting-started.html"><h3>Getting started</h3><p>Install cvm, wire shell hooks, create your first environment.</p></a>
  <a class="card-link" href="commands.html"><h3>Command reference</h3><p>Every public subcommand, flag, and alias in one place.</p></a>
  <a class="card-link" href="concepts.html"><h3>Core concepts</h3><p>CLAUDE_CONFIG_DIR, shims, activation protocol, and directory layout.</p></a>
  <a class="card-link" href="examples.html"><h3>Examples</h3><p>Copy-paste workflows for day-to-day Claude Code isolation.</p></a>
</div>
<h2>What cvm is</h2>
<p><strong>cvm</strong> (Claude Virtualenv Manager) isolates Claude Code configuration the way Python <code>venv</code> isolates packages. Each environment is a directory under <code>~/.cvm/envs/&lt;name&gt;</code>. When active, cvm points Claude Code at that directory via <code>CLAUDE_CONFIG_DIR</code> — without patching Claude Code.</p>
<p>Beyond isolation, cvm shares reproducible setups through a secrets-free <code>cvm.yaml</code>, can seed environments from <code>~/.claude</code>, auto-activates from a project <code>.cvm</code> file, and runs several Claude sessions in parallel.</p>
<h2>Audience</h2>
<ul>
  <li>Developers juggling multiple Claude Code configs (clients, repos, personal vs work).</li>
  <li>Teams that want matching MCP servers and skills without committing tokens.</li>
  <li>Anyone wiring statuslines or shell tooling around <code>CVM_ENV</code>.</li>
</ul>
<h2>Mental model</h2>
<ol class="steps">
  <li><strong>Create</strong> an environment — directory, optional credentials, optional inherited skills, <code>bin/</code> shims.</li>
  <li><strong>Activate</strong> with <code>cvm use</code> (shell hooks) or run once with <code>cvm open</code> / <code>cvm run</code>.</li>
  <li>Claude Code reads and writes under that environment’s config directory.</li>
  <li>Optionally <strong>export / import</strong> the non-secret surface as <code>cvm.yaml</code>.</li>
</ol>
<div class="callout"><strong>Shell integration is required for <code>use</code> / <code>deactivate</code>.</strong> A child process cannot mutate its parent shell. Install hooks with <code>cvm init</code>.</div>
<h2>How-to guides</h2>
<div class="card-grid">
  <a class="card-link" href="how-to/create-and-open.html"><h3>Create &amp; open</h3><p>One-shot <code>--open</code> and everyday activate flows.</p></a>
  <a class="card-link" href="how-to/inherit-skills.html"><h3>Inherit skills</h3><p>Seed from <code>~/.claude</code> with <code>--inherit</code>.</p></a>
  <a class="card-link" href="how-to/share-team-setup.html"><h3>Share with the team</h3><p>Export/import <code>cvm.yaml</code> without leaking secrets.</p></a>
  <a class="card-link" href="how-to/auto-activate.html"><h3>Auto-activate</h3><p>Project-local <code>.cvm</code> files like a lightweight direnv.</p></a>
  <a class="card-link" href="how-to/parallel-sessions.html"><h3>Parallel sessions</h3><p>Several <code>cvm open</code> processes side by side.</p></a>
  <a class="card-link" href="how-to/per-env-secrets.html"><h3>Per-env secrets</h3><p>Edit <code>.env</code> safely; values never export.</p></a>
</div>
""",
    nxt=("getting-started.html", "Getting started"),
)

write(
    "getting-started.html",
    "getting-started",
    0,
    "Getting started",
    "Start here",
    "Getting started",
    "Install the binary, enable shell hooks, and create your first isolated Claude Code environment.",
    """
<h2>1. Install</h2>
<h3>One-line installer</h3>
<pre><code>curl -fsSL https://raw.githubusercontent.com/acwoss/cvm/main/install.sh | bash</code></pre>
<p>Downloads the platform binary to <code>~/.cvm/bin</code> and can append PATH + hook lines to your shell rc.</p>
<h3>From a GitHub Release</h3>
<p>Download the archive for your OS from the <a href="https://github.com/acwoss/cvm/releases">Releases page</a>, extract it, and put <code>cvm</code> on your <code>PATH</code>.</p>
<h3>From source</h3>
<pre><code>git clone https://github.com/acwoss/cvm.git
cd cvm
cargo install --path . --locked</code></pre>
<h3>Update later</h3>
<pre><code>cvm update</code></pre>
<p>Pulls the latest release asset and replaces the running binary in place (needs <code>curl</code> and <code>tar</code>). Cargo installs should re-run <code>cargo install</code> instead.</p>
<h2>2. Enable shell integration</h2>
<p>Add the line for your shell, then restart or source the file:</p>
<div class="docs-table-wrap">
<table>
  <thead><tr><th>Shell</th><th>File</th><th>Line</th></tr></thead>
  <tbody>
    <tr><td>Bash</td><td><code>~/.bashrc</code></td><td><code>eval "$(cvm init bash)"</code></td></tr>
    <tr><td>Zsh</td><td><code>~/.zshrc</code></td><td><code>eval "$(cvm init zsh)"</code></td></tr>
    <tr><td>Fish</td><td><code>~/.config/fish/config.fish</code></td><td><code>cvm init fish | source</code></td></tr>
    <tr><td>PowerShell</td><td><code>$PROFILE</code></td><td><code>cvm init powershell | Out-String | Invoke-Expression</code></td></tr>
  </tbody>
</table>
</div>
<p>After upgrading cvm, re-run <code>cvm init</code> so you pick up prompt, PATH, and <code>.cvm</code> auto-activation hooks.</p>
<h2>3. Create and activate an environment</h2>
<pre><code>cvm create work
cvm use work
claude</code></pre>
<p>While active you should see a <code>(work)</code> prompt prefix. <code>claude</code> and <code>skills</code> resolve through <code>~/.cvm/envs/work/bin</code> shims.</p>
<pre><code>cvm current   # work
cvm list      # * work (active)
cvm deactivate</code></pre>
<div class="callout"><strong>Skip activation for one shot:</strong> <code>cvm open work</code> or <code>cvm create work --open</code> launches Claude without changing the parent shell.</div>
""",
    prev=("index.html", "Overview"),
    nxt=("concepts.html", "Core concepts"),
)

write(
    "concepts.html",
    "concepts",
    0,
    "Core concepts",
    "Concepts",
    "Core concepts",
    "How cvm maps onto Claude Code configuration without modifying Claude Code itself.",
    """
<h2>CLAUDE_CONFIG_DIR</h2>
<p>Claude Code locates its config via <code>CLAUDE_CONFIG_DIR</code> (default <code>~/.claude</code>). cvm sets that variable to <code>~/.cvm/envs/&lt;name&gt;</code> while an environment is active or for a single <code>run</code>/<code>open</code> child process.</p>
<h2>CVM_ENV</h2>
<p>Process-scoped name of the active environment. Used by <code>cvm current</code>, <code>cvm list</code>, and statusline scripts.</p>
<h2>Activation protocol</h2>
<p>The compiled binary cannot change the parent shell. Hidden resolvers print data the hook applies:</p>
<ul>
  <li><code>cvm __resolve-activate &lt;env&gt;</code> → <code>KEY=VALUE</code> lines (dotenv + <code>CLAUDE_CONFIG_DIR</code> + <code>CVM_ENV</code>)</li>
  <li><code>cvm __resolve-deactivate</code> → names to unset</li>
</ul>
<p>Shell hooks additionally:</p>
<ul>
  <li>Prefix the prompt with <code>(env)</code></li>
  <li>Backup <code>PATH</code> in <code>CVM_OLD_PATH</code> and prepend <code>$CLAUDE_CONFIG_DIR/bin</code></li>
  <li>Watch for project <code>.cvm</code> files (auto-activate)</li>
  <li>Deactivate the previous env before switching, so dotenv keys do not leak</li>
</ul>
<div class="callout warn"><strong>Reserved dotenv keys</strong> — <code>PATH</code>, <code>PS1</code>, <code>CLAUDE_CONFIG_DIR</code>, and any <code>CVM_*</code> key in an environment <code>.env</code> are ignored so activation state cannot be poisoned.</div>
<h2>Directory layout</h2>
<pre><code>~/.cvm/
├── bin/                  # cvm binary (install.sh)
└── envs/
    └── work/             # = CLAUDE_CONFIG_DIR when active
        ├── .env
        ├── skills/
        ├── bin/          # claude + skills shims
        ├── settings.json
        └── ...           # whatever Claude Code writes</code></pre>
<h2>Shims</h2>
<p>Each env ships <code>bin/claude</code> and <code>bin/skills</code> (plus <code>.cmd</code> on Windows). They set <code>CLAUDE_CONFIG_DIR</code>/<code>CVM_ENV</code> from their parent directory, strip their own <code>bin</code> from <code>PATH</code> to avoid recursion, then exec the real tool. The skills shim runs <code>npx --yes skills</code>.</p>
<h2>cvm.yaml surface</h2>
<p>Export/import only touch <code>settings.json</code> (permissions + mcpServers), skill <em>names</em>, and <code>.env</code> <em>keys</em>. Credentials, history, and <code>.env</code> values are never read into the manifest.</p>
""",
    prev=("getting-started.html", "Getting started"),
    nxt=("commands.html", "Command reference"),
)

write(
    "commands.html",
    "commands",
    0,
    "Command reference",
    "Reference",
    "Command reference",
    "Public CLI surface. Hidden resolvers used only by shell hooks are omitted.",
    """
<h2>Lifecycle</h2>
<div class="docs-table-wrap"><table>
<thead><tr><th>Command</th><th>Aliases</th><th>Description</th></tr></thead>
<tbody>
<tr><td><code>cvm init &lt;shell&gt;</code></td><td>—</td><td>Print hooks for <code>bash</code>, <code>zsh</code>, <code>fish</code>, or <code>powershell</code>.</td></tr>
<tr><td><code>cvm update</code></td><td>—</td><td>Install the latest GitHub Release binary in place.</td></tr>
<tr><td><code>cvm create &lt;env&gt; [--anonymous] [--inherit] [--open]</code></td><td>—</td><td>Create <code>~/.cvm/envs/&lt;env&gt;</code>. Copies global credentials unless <code>--anonymous</code>. <code>--inherit</code> links skills + copies settings. <code>--open</code> launches Claude immediately.</td></tr>
<tr><td><code>cvm list</code></td><td><code>ls</code></td><td>List environments; mark the active one.</td></tr>
<tr><td><code>cvm use &lt;env&gt;</code></td><td><code>activate</code></td><td>Activate in the current shell (needs hooks). Clears auto-pin (<code>CVM_AUTO</code>).</td></tr>
<tr><td><code>cvm deactivate</code></td><td>—</td><td>Restore global Claude config, prompt, and PATH.</td></tr>
<tr><td><code>cvm current</code></td><td>—</td><td>Print active environment name.</td></tr>
<tr><td><code>cvm remove &lt;env&gt; [-y]</code></td><td><code>rm</code></td><td>Delete an environment. <code>-y/--yes</code> skips confirm.</td></tr>
</tbody></table></div>
<h2>Run &amp; edit</h2>
<div class="docs-table-wrap"><table>
<thead><tr><th>Command</th><th>Description</th></tr></thead>
<tbody>
<tr><td><code>cvm edit [env]</code></td><td>Open <code>.env</code> in <code>$VISUAL</code>/<code>$EDITOR</code> (default: active). Creates the file if missing.</td></tr>
<tr><td><code>cvm run &lt;env&gt; -- &lt;cmd&gt;</code></td><td>Run a command with env context + <code>bin/</code> on PATH; parent shell unchanged.</td></tr>
<tr><td><code>cvm open &lt;env&gt;</code></td><td>Alias for <code>cvm run &lt;env&gt; -- claude</code>. Safe to parallelize.</td></tr>
</tbody></table></div>
<h2>Share</h2>
<div class="docs-table-wrap"><table>
<thead><tr><th>Command</th><th>Description</th></tr></thead>
<tbody>
<tr><td><code>cvm export [env] [-o file]</code></td><td>Write YAML manifest (default env: active; default file: <code>cvm.yaml</code>). Never includes secret values.</td></tr>
<tr><td><code>cvm import &lt;file&gt; [-n name]</code></td><td>Create/update an env from a manifest; blank <code>.env</code> placeholders for listed names.</td></tr>
</tbody></table></div>
<h2>create flags in depth</h2>
<ul>
  <li><strong>default</strong> — copy <code>~/.claude/.credentials.json</code> when present (file-backed platforms).</li>
  <li><strong><code>--anonymous</code></strong> — skip credential copy; empty login state.</li>
  <li><strong><code>--inherit</code></strong> — symlink (or copy) each <code>~/.claude/skills/*</code> entry; copy <code>settings.json</code> if present.</li>
  <li><strong><code>--open</code></strong> — after a successful create, open Claude and exit with its status code.</li>
</ul>
<div class="callout warn"><strong>macOS note:</strong> credentials may live in the Keychain. If there is no <code>.credentials.json</code> file, create still succeeds but you may need to log in once per environment.</div>
""",
    prev=("concepts.html", "Core concepts"),
    nxt=("examples.html", "Examples"),
)

write(
    "examples.html",
    "examples",
    0,
    "Examples",
    "Reference",
    "Examples",
    "Short, copy-paste oriented recipes. Prefer how-to guides when you need the why.",
    """
<h2>Everyday env</h2>
<pre><code>cvm create work
cvm use work
claude
skills   # npx --yes skills under this env</code></pre>
<h2>Create and open immediately</h2>
<pre><code>cvm create client-acme --open</code></pre>
<h2>Anonymous sandbox</h2>
<pre><code>cvm create sandbox --anonymous
cvm use sandbox
claude   # fresh login</code></pre>
<h2>Inherit global skills/settings</h2>
<pre><code>cvm create work --inherit --open</code></pre>
<h2>One-off without activating the shell</h2>
<pre><code>cvm run work -- claude mcp list
cvm open work</code></pre>
<h2>Secrets for MCP</h2>
<pre><code>cvm edit work
# POSTGRES_PASSWORD=...
# GITHUB_TOKEN=...
cvm run work -- claude</code></pre>
<h2>Team share</h2>
<pre><code>cvm export -o cvm.yaml
git add cvm.yaml &amp;&amp; git commit -m "Add shared Claude environment"

# teammate
cvm import cvm.yaml -n project-backend-api
cvm edit project-backend-api   # fill secrets
cvm use project-backend-api</code></pre>
<h2>Project auto-activate</h2>
<pre><code>echo project-backend-api &gt; .cvm
# with hooks installed, cd into this tree activates the env</code></pre>
<h2>Parallel clients</h2>
<pre><code>cvm open client-acme &amp;
cvm open client-globex &amp;
cvm open personal &amp;</code></pre>
""",
    prev=("commands.html", "Command reference"),
    nxt=("how-to/create-and-open.html", "Create & open"),
)

write(
    "how-to/create-and-open.html",
    "hto-create-open",
    1,
    "Create & open",
    "How-to",
    "Create an environment and open Claude",
    "Choose between activating your shell and launching a one-shot Claude process.",
    """
<ol class="steps">
  <li><strong>Create</strong> the environment.<pre><code>cvm create work</code></pre></li>
  <li><strong>Option A — activate the shell</strong> (needs <code>cvm init</code>).<pre><code>cvm use work
claude</code></pre><p>Prompt shows <code>(work)</code>; <code>PATH</code> prefers <code>~/.cvm/envs/work/bin</code>.</p></li>
  <li><strong>Option B — open without activating</strong>.<pre><code>cvm open work
# or
cvm create work --open</code></pre><p>Only the Claude child receives <code>CLAUDE_CONFIG_DIR</code> / <code>CVM_ENV</code>.</p></li>
</ol>
<div class="callout">Combine flags: <code>cvm create work --inherit --open</code>.</div>
""",
    prev=("../examples.html", "Examples"),
    nxt=("inherit-skills.html", "Inherit skills"),
)

write(
    "how-to/inherit-skills.html",
    "hto-inherit",
    1,
    "Inherit skills",
    "How-to",
    "Seed an environment from ~/.claude",
    "Use --inherit when you want global skills/settings as a starting point without sharing the global config dir.",
    """
<ol class="steps">
  <li>Ensure you have skills under <code>~/.claude/skills</code> and optionally <code>~/.claude/settings.json</code>.</li>
  <li>Create with inherit:<pre><code>cvm create work --inherit</code></pre></li>
  <li>cvm attempts to <strong>symlink</strong> each skill into <code>~/.cvm/envs/work/skills/</code>. If linking fails (common on some Windows setups), it <strong>copies</strong> recursively and continues.</li>
  <li><code>settings.json</code> is <strong>copied</strong> (not linked) so later edits stay local to the env.</li>
</ol>
<div class="callout warn"><strong>Credentials are separate.</strong> Default create still copies <code>.credentials.json</code> when present unless you also pass <code>--anonymous</code>.</div>
""",
    prev=("create-and-open.html", "Create & open"),
    nxt=("share-team-setup.html", "Share a team setup"),
)

write(
    "how-to/share-team-setup.html",
    "hto-share",
    1,
    "Share a team setup",
    "How-to",
    "Share a Claude setup with cvm.yaml",
    "Commit a reproducible, secrets-free manifest next to your project.",
    """
<ol class="steps">
  <li>Configure an environment the way you want (MCP servers, permissions, skills).</li>
  <li>Export:<pre><code>cvm use project-backend-api
cvm export -o cvm.yaml</code></pre></li>
  <li>Commit <code>cvm.yaml</code>. It contains settings, mcpServers, skill names, and <code>.env</code> <em>names</em> only.</li>
  <li>Teammates import:<pre><code>cvm import cvm.yaml -n project-backend-api
cvm edit project-backend-api   # fill blank secrets
cvm use project-backend-api</code></pre></li>
</ol>
<p>Import creates skill stubs (<code>skills/&lt;name&gt;/SKILL.md</code>) you can fill or reinstall from the original source. Existing <code>.env</code> values are preserved; new names get blank entries.</p>
<div class="callout"><strong>Security model:</strong> export never opens credentials, history, or <code>.env</code> values — only the managed surface.</div>
""",
    prev=("inherit-skills.html", "Inherit skills"),
    nxt=("auto-activate.html", "Auto-activate"),
)

write(
    "how-to/auto-activate.html",
    "hto-auto",
    1,
    "Auto-activate",
    "How-to",
    "Auto-activate with a .cvm file",
    "Bind a repository (or subdirectory) to an environment the way direnv binds env vars.",
    """
<ol class="steps">
  <li>Confirm shell hooks are installed (<code>cvm init</code>) and reloaded after upgrades.</li>
  <li>Create a <code>.cvm</code> file in the project root:<pre><code># Claude environment for this project
project-backend-api</code></pre><p>First non-empty, non-<code>#</code> line wins.</p></li>
  <li><code>cd</code> into the tree — hooks search upward, run <code>cvm use</code>, and set <code>CVM_AUTO=1</code>.</li>
  <li>Leave the tree — auto sessions deactivate. A manual <code>cvm use</code> clears <code>CVM_AUTO</code> (pinned) so leaving the directory does not deactivate.</li>
</ol>
<h3>direnv alternative</h3>
<pre><code>while IFS='=' read -r key value; do
  export "$key=$value"
done &lt; &lt;(cvm __resolve-activate project-backend-api)
PATH="$CLAUDE_CONFIG_DIR/bin:$PATH"
export PATH</code></pre>
""",
    prev=("share-team-setup.html", "Share a team setup"),
    nxt=("parallel-sessions.html", "Parallel sessions"),
)

write(
    "how-to/parallel-sessions.html",
    "hto-parallel",
    1,
    "Parallel sessions",
    "How-to",
    "Run parallel Claude sessions",
    "Each cvm open only annotates its child process — shells stay independent.",
    """
<ol class="steps">
  <li>Create one environment per context (<code>client-acme</code>, <code>client-globex</code>, …).</li>
  <li>Launch separately:<pre><code>cvm open client-acme
# other tab / pane
cvm open client-globex</code></pre></li>
  <li>Or background them:<pre><code>cvm open client-acme &amp;
cvm open client-globex &amp;</code></pre></li>
</ol>
<p>Parent shell activation (if any) is untouched. Use a statusline badge on <code>CVM_ENV</code> so each session shows which env it belongs to — see the statusline guide.</p>
""",
    prev=("auto-activate.html", "Auto-activate"),
    nxt=("per-env-secrets.html", "Per-env secrets"),
)

write(
    "how-to/per-env-secrets.html",
    "hto-secrets",
    1,
    "Per-env secrets",
    "How-to",
    "Keep secrets in the environment .env",
    "Store MCP credentials and other secrets beside the env — never in cvm.yaml.",
    """
<ol class="steps">
  <li>Open the editor:<pre><code>cvm edit work
# or, with an active env:
cvm edit</code></pre></li>
  <li>Add KEY=VALUE lines (standard dotenv conventions).</li>
  <li>Use / open / run load those variables into the process. Export only records the <em>names</em>.</li>
</ol>
<div class="callout warn">Do not put <code>PATH</code>, <code>CLAUDE_CONFIG_DIR</code>, or <code>CVM_*</code> keys in <code>.env</code> — they are ignored.</div>
""",
    prev=("parallel-sessions.html", "Parallel sessions"),
    nxt=("statusline.html", "Statusline badge"),
)

write(
    "how-to/statusline.html",
    "hto-statusline",
    1,
    "Statusline badge",
    "How-to",
    "Show the active env in Claude Code statusline",
    "Surface CVM_ENV so parallel tabs stay identifiable.",
    """
<p>Paste a prompt like the following into Claude Code and let it adapt your existing <code>statusLine</code> script:</p>
<pre><code>Please update my Claude Code statusline so it shows the active cvm
environment when there is one. Find my current statusLine configuration
and adjust its command/script in place without removing existing output.

Requirements:
- If CVM_ENV is set and non-empty, prepend a short badge (e.g. [env-name]).
- If CVM_ENV is unset/empty, render exactly as today.
- Keep the existing language/tooling of the statusline script.
- If none exists, create a minimal one with model name + CVM_ENV badge.

Show me the diff before writing it.</code></pre>
<p><code>CVM_ENV</code> is set for processes started via <code>use</code>, <code>run</code>, and <code>open</code>.</p>
""",
    prev=("per-env-secrets.html", "Per-env secrets"),
)

print("done")
