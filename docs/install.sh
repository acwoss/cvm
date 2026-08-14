#!/bin/sh
# install.sh - install cvm (Claude Virtualenv Manager)
#
# Usage:
#   curl -fsSL https://getcvm.com/install.sh | bash
#
# Environment overrides:
#   CVM_INSTALL_DIR   Install prefix (default: $HOME/.cvm)
#   CVM_VERSION       Release tag to install (default: latest)

set -eu

REPO="acwoss/cvm"
BIN_NAME="cvm"
INSTALL_DIR="${CVM_INSTALL_DIR:-$HOME/.cvm}"
BIN_DIR="$INSTALL_DIR/bin"
VERSION="${CVM_VERSION:-latest}"

info() { printf '\033[1;34m==>\033[0m %s\n' "$1"; }
warn() { printf '\033[1;33mwarning:\033[0m %s\n' "$1" >&2; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$1" >&2; exit 1; }

detect_os() {
  case "$(uname -s)" in
    Linux) echo "unknown-linux-gnu" ;;
    Darwin) echo "apple-darwin" ;;
    *) die "unsupported operating system: $(uname -s). Install manually via 'cargo install cvm' or download a release from https://github.com/$REPO/releases" ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    x86_64 | amd64) echo "x86_64" ;;
    arm64 | aarch64) echo "aarch64" ;;
    *) die "unsupported CPU architecture: $(uname -m)" ;;
  esac
}

main() {
  os="$(detect_os)"
  arch="$(detect_arch)"
  target="${arch}-${os}"
  archive="${BIN_NAME}-${target}.tar.gz"

  if [ "$VERSION" = "latest" ]; then
    url="https://github.com/$REPO/releases/latest/download/$archive"
  else
    url="https://github.com/$REPO/releases/download/$VERSION/$archive"
  fi

  info "Target: $target"
  info "Downloading $url"

  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT

  if ! curl -fsSL "$url" -o "$tmp_dir/$archive"; then
    die "download failed. Check that a release exists for '$target' at https://github.com/$REPO/releases"
  fi

  tar -xzf "$tmp_dir/$archive" -C "$tmp_dir"

  mkdir -p "$BIN_DIR"
  extracted_bin="$(find "$tmp_dir" -type f -name "$BIN_NAME" | head -n 1)"
  [ -n "$extracted_bin" ] || die "could not find the '$BIN_NAME' binary inside the downloaded archive"

  install -m 755 "$extracted_bin" "$BIN_DIR/$BIN_NAME"
  info "Installed $BIN_NAME to $BIN_DIR/$BIN_NAME"

  configure_shell
  info "Done. Restart your shell, or run: source your shell profile"
}

# Appends PATH + `cvm init` hook to the user's shell rc file(s), if not
# already present. Never overwrites existing content.
configure_shell() {
  shell_name="$(basename "${SHELL:-sh}")"

  case "$shell_name" in
    zsh) rc_file="$HOME/.zshrc"; hook="eval \"\$(cvm init zsh)\"" ;;
    bash) rc_file="$HOME/.bashrc"; hook="eval \"\$(cvm init bash)\"" ;;
    fish) rc_file="$HOME/.config/fish/config.fish"; hook="cvm init fish | source" ;;
    *)
      warn "unrecognized shell '$shell_name'; add $BIN_DIR to PATH and configure 'cvm init' manually"
      return 0
      ;;
  esac

  path_line="export PATH=\"$BIN_DIR:\$PATH\""

  mkdir -p "$(dirname "$rc_file")"
  touch "$rc_file"

  if ! grep -qF "$BIN_DIR" "$rc_file" 2>/dev/null; then
    {
      echo ""
      echo "# Added by cvm installer"
      echo "$path_line"
    } >> "$rc_file"
    info "Added $BIN_DIR to PATH in $rc_file"
  fi

  if ! grep -qF "cvm init" "$rc_file" 2>/dev/null; then
    echo "$hook" >> "$rc_file"
    info "Added cvm shell hook to $rc_file"
  fi
}

main "$@"
