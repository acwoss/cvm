pub const SCRIPT: &str = r#"# cvm shell integration for zsh
# Add this to your ~/.zshrc:
#   eval "$(cvm init zsh)"

cvm() {
  case "$1" in
    use|activate)
      if [ -z "$2" ]; then
        echo "Usage: cvm use <env_name>" >&2
        return 1
      fi
      local __cvm_out
      __cvm_out="$(command cvm __resolve-activate "$2")" || return $?
      local __cvm_key __cvm_val
      while IFS='=' read -r __cvm_key __cvm_val; do
        [ -n "$__cvm_key" ] && export "$__cvm_key=$__cvm_val"
      done <<< "$__cvm_out"
      ;;
    deactivate)
      local __cvm_out
      __cvm_out="$(command cvm __resolve-deactivate)" || return $?
      local __cvm_key
      while IFS= read -r __cvm_key; do
        [ -n "$__cvm_key" ] && unset "$__cvm_key"
      done <<< "$__cvm_out"
      ;;
    *)
      command cvm "$@"
      ;;
  esac
}
"#;
