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
          PATH|PS1|CVM_OLD_PATH|CVM_OLD_PS1|CVM_OLD_PROMPT|CVM_HOME|CVM_AUTO|CVM_AUTO_ROOT|CVM_AUTO_LAST_PWD)
            continue
            ;;
        esac
        [ -n "$__cvm_key" ] && export "$__cvm_key=$__cvm_val"
      done <<< "$__cvm_out"
      export PATH="${CLAUDE_CONFIG_DIR}/bin:${CVM_OLD_PATH}"
      PS1="(${CVM_ENV}) ${CVM_OLD_PS1}"
      export PS1
      unset CVM_AUTO CVM_AUTO_ROOT
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
      unset CVM_AUTO CVM_AUTO_ROOT
      ;;
    *)
      command cvm "$@"
      ;;
  esac
}

__cvm_auto_check() {
  [ "${CVM_AUTO_LAST_PWD-}" = "$PWD" ] && return 0
  CVM_AUTO_LAST_PWD="$PWD"

  local __cvm_dir="$PWD" __cvm_found="" __cvm_name=""
  while [ -n "$__cvm_dir" ]; do
    if [ -f "$__cvm_dir/.cvm" ]; then
      __cvm_found="$__cvm_dir"
      __cvm_name=$(grep -v '^[[:space:]]*#' "$__cvm_dir/.cvm" |
        sed '/^[[:space:]]*$/d' |
        head -n 1 |
        tr -d '\r' |
        sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
      break
    fi
    [ "$__cvm_dir" = "/" ] && break
    __cvm_dir=$(dirname "$__cvm_dir")
  done

  if [ -n "$__cvm_name" ]; then
    if [ "${CVM_ENV-}" != "$__cvm_name" ]; then
      cvm use "$__cvm_name" || return $?
      export CVM_AUTO=1
      export CVM_AUTO_ROOT="$__cvm_found"
    fi
  elif [ "${CVM_AUTO-}" = "1" ]; then
    cvm deactivate || return $?
    unset CVM_AUTO CVM_AUTO_ROOT
  fi
}

if declare -p PROMPT_COMMAND 2>/dev/null | grep -q '^declare -a'; then
  case " ${PROMPT_COMMAND[*]-} " in
    *" __cvm_auto_check "*) ;;
    *) PROMPT_COMMAND+=(__cvm_auto_check) ;;
  esac
else
  case "${PROMPT_COMMAND-}" in
    *"__cvm_auto_check"*) ;;
    "") PROMPT_COMMAND="__cvm_auto_check" ;;
    *) PROMPT_COMMAND="${PROMPT_COMMAND}"$'\n__cvm_auto_check' ;;
  esac
fi
"#;
