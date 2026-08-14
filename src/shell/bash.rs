pub const SCRIPT: &str = r#"# cvm shell integration for bash
# Add this to your ~/.bashrc:
#   eval "$(cvm init bash)"

cvm() {
  case "$1" in
    use|activate)
      if [ -z "$2" ]; then
        echo "Usage: cvm use <env_name>" >&2
        return 1
      fi
      local __cvm_out
      __cvm_out="$(command cvm __resolve-activate "$2")" || return $?
      if [ -z "${CVM_OLD_PATH+x}" ]; then
        export CVM_OLD_PATH="$PATH"
      fi
      if [ -z "${CVM_OLD_PS1+x}" ]; then
        CVM_OLD_PS1="${PS1-}"
        export CVM_OLD_PS1
      fi
      local __cvm_key __cvm_val
      while IFS='=' read -r __cvm_key __cvm_val; do
        case "$__cvm_key" in
          PATH|PS1|CVM_OLD_PATH|CVM_OLD_PS1|CVM_OLD_PROMPT|CVM_HOME|CVM_AUTO)
            continue
            ;;
        esac
        [ -n "$__cvm_key" ] && export "$__cvm_key=$__cvm_val"
      done <<< "$__cvm_out"
      export PATH="${CLAUDE_CONFIG_DIR}/bin:${CVM_OLD_PATH}"
      PS1="(${CVM_ENV}) ${CVM_OLD_PS1}"
      export PS1
      ;;
    deactivate)
      local __cvm_out
      __cvm_out="$(command cvm __resolve-deactivate)" || return $?
      if [ -n "${CVM_OLD_PATH+x}" ]; then
        export PATH="$CVM_OLD_PATH"
        unset CVM_OLD_PATH
      fi
      if [ -n "${CVM_OLD_PS1+x}" ]; then
        PS1="$CVM_OLD_PS1"
        export PS1
        unset CVM_OLD_PS1
      fi
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
