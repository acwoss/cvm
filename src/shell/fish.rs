pub const SCRIPT: &str = r#"# cvm shell integration for fish
# Add this to your ~/.config/fish/config.fish:
#   cvm init fish | source

function cvm
    switch $argv[1]
        case use activate
            if test -z "$argv[2]"
                echo "Usage: cvm use <env_name>" >&2
                return 1
            end
            set -l __cvm_out (command cvm __resolve-activate $argv[2]); or return $status
            if not set -q CVM_OLD_PATH
                set -gx CVM_OLD_PATH $PATH
            end
            if not functions -q __cvm_old_fish_prompt
                functions -c fish_prompt __cvm_old_fish_prompt
            end
            for __cvm_line in $__cvm_out
                set -l __cvm_parts (string split -m 1 '=' -- $__cvm_line)
                switch $__cvm_parts[1]
                    case PATH PS1 CVM_OLD_PATH CVM_OLD_PS1 CVM_OLD_PROMPT CVM_HOME CVM_AUTO CVM_AUTO_ROOT CVM_AUTO_LAST_PWD
                        continue
                end
                if test -n "$__cvm_parts[1]"
                    set -gx $__cvm_parts[1] $__cvm_parts[2]
                end
            end
            set -gx PATH "$CLAUDE_CONFIG_DIR/bin" $CVM_OLD_PATH
            function fish_prompt
                printf '(%s) ' "$CVM_ENV"
                __cvm_old_fish_prompt
            end
            set -e CVM_AUTO
            set -e CVM_AUTO_ROOT
        case deactivate
            set -l __cvm_out (command cvm __resolve-deactivate); or return $status
            if set -q CVM_OLD_PATH
                set -gx PATH $CVM_OLD_PATH
                set -e CVM_OLD_PATH
            end
            for __cvm_line in $__cvm_out
                if test -n "$__cvm_line"
                    set -e $__cvm_line
                end
            end
            if functions -q __cvm_old_fish_prompt
                functions -e fish_prompt
                functions -c __cvm_old_fish_prompt fish_prompt
                functions -e __cvm_old_fish_prompt
            end
            set -e CVM_AUTO
            set -e CVM_AUTO_ROOT
        case '*'
            command cvm $argv
    end
end

function __cvm_auto_check --on-variable PWD
    if set -q CVM_AUTO_LAST_PWD; and test "$CVM_AUTO_LAST_PWD" = "$PWD"
        return 0
    end
    set -g CVM_AUTO_LAST_PWD "$PWD"

    set -l __cvm_dir "$PWD"
    set -l __cvm_found
    set -l __cvm_name
    while test -n "$__cvm_dir"
        if test -f "$__cvm_dir/.cvm"
            set __cvm_found "$__cvm_dir"
            while read -l __cvm_line
                set -l __cvm_trimmed (string trim -- "$__cvm_line")
                if test -n "$__cvm_trimmed"; and not string match -q '#*' -- "$__cvm_trimmed"
                    set __cvm_name "$__cvm_trimmed"
                    break
                end
            end < "$__cvm_dir/.cvm"
            break
        end

        set -l __cvm_parent (path dirname "$__cvm_dir")
        if test "$__cvm_parent" = "$__cvm_dir"
            break
        end
        set __cvm_dir "$__cvm_parent"
    end

    if test -n "$__cvm_name"
        if not set -q CVM_ENV; or test "$CVM_ENV" != "$__cvm_name"
            cvm use "$__cvm_name"; or return $status
            set -gx CVM_AUTO 1
            set -gx CVM_AUTO_ROOT "$__cvm_found"
        end
    else if set -q CVM_AUTO; and test "$CVM_AUTO" = 1
        cvm deactivate; or return $status
        set -e CVM_AUTO
        set -e CVM_AUTO_ROOT
    end
end

__cvm_auto_check
"#;
