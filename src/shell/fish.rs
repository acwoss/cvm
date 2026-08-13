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
            for __cvm_line in $__cvm_out
                set -l __cvm_parts (string split -m 1 '=' -- $__cvm_line)
                if test -n "$__cvm_parts[1]"
                    set -gx $__cvm_parts[1] $__cvm_parts[2]
                end
            end
        case deactivate
            set -l __cvm_out (command cvm __resolve-deactivate); or return $status
            for __cvm_line in $__cvm_out
                if test -n "$__cvm_line"
                    set -e $__cvm_line
                end
            end
        case '*'
            command cvm $argv
    end
end
"#;
