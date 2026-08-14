# Contributing to cvm

Thanks for taking the time to contribute! This document covers how to set up
your environment, the checks your change needs to pass, and how to open a
good pull request.

## Getting set up

`cvm` is a single Rust binary with no runtime dependencies beyond the crates
in `Cargo.toml`.

```sh
git clone https://github.com/acwoss/cvm.git
cd cvm
cargo build
cargo test
```

Rust stable (see `dtolnay/rust-toolchain@stable` in CI) is the only
requirement — no nightly features are used.

## Before opening a pull request

CI runs formatting, linting, and tests on Linux, macOS, and Windows. Run the
same checks locally before pushing:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

A PR that fails any of these will fail CI, so it's faster to catch it
locally first.

## Making changes

- Keep pull requests focused on a single change; unrelated cleanups make
  review harder and are easier to land as their own PR.
- Add or update tests for behavior you change. `tempfile` is already a
  dev-dependency for tests that touch the filesystem.
- If your change affects user-facing behavior (commands, flags, `cvm.yaml`
  fields, shell hooks), update `README.md` and, if relevant, the docs under
  `docs/documentation/`.
- Follow the existing code style; `cargo fmt` and `cargo clippy` are the
  source of truth, not personal preference.

## Commit messages

Use short, imperative commit messages that describe the *why* as much as the
*what* (e.g. `fix: avoid clobbering CLAUDE_CONFIG_DIR when inheriting`).
[Conventional Commits](https://www.conventionalcommits.org/) prefixes
(`feat:`, `fix:`, `docs:`, `chore:`, ...) are welcome but not required.

## Opening the pull request

1. Fork the repository and create a branch off `main`.
2. Push your branch and open a PR against `main`.
3. Fill in the PR template — it's short by design.
4. Make sure CI is green. A maintainer will review from there.

## Reporting bugs and requesting features

Please use the issue templates under **Issues → New issue** — they collect
the information (cvm version, OS, shell, reproduction steps) that's usually
needed to act on a report.

## Code of Conduct

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md). By
participating, you agree to uphold it.
