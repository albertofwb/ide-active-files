#!/bin/bash
# Auto-completion setup script for ide-files

set -e

COMPLETION_DIR=""
SHELL_RC=""

echo "Setting up shell auto-completion for ide-files..."

# Detect shell and set appropriate completion directory
if [ -n "$ZSH_VERSION" ] || [ "$SHELL" = "/bin/zsh" ] || [ "$SHELL" = "/usr/bin/zsh" ]; then
    echo "Setting up ZSH completion..."
    COMPLETION_DIR="$HOME/.zsh/completions"
    SHELL_RC="$HOME/.zshrc"
    mkdir -p "$COMPLETION_DIR"
elif [ -n "$BASH_VERSION" ] || [ "$SHELL" = "/bin/bash" ] || [ "$SHELL" = "/usr/bin/bash" ]; then
    echo "Setting up BASH completion..."
    if [ -d "/usr/local/etc/bash_completion.d" ]; then
        COMPLETION_DIR="/usr/local/etc/bash_completion.d"
    elif [ -d "/etc/bash_completion.d" ]; then
        COMPLETION_DIR="/etc/bash_completion.d"
    else
        COMPLETION_DIR="$HOME/.bash_completion.d"
        mkdir -p "$COMPLETION_DIR"
    fi
    SHELL_RC="$HOME/.bashrc"
else
    echo "Unsupported shell: $SHELL"
    echo "Please set up completion manually or use bash/zsh."
    exit 1
fi

echo "Using completion directory: $COMPLETION_DIR"

# Generate completion script
echo "Generating completion script..."
cat > "$COMPLETION_DIR/_ide-files" << 'COMPLETION_EOF'
#compdef ide-files

# ZSH completion for ide-files
_ide-files() {
    local context state line
    typeset -A opt_args

    _arguments \
        '--ide[Specify IDE to detect]:ide:(goland pycharm idea vscode vs webstorm phpstorm rubymine clion vim nano)' \
        '--list-ides[List all supported IDEs]' \
        '--auto[Auto-detect any supported IDE]' \
        '--format[Output format]:format:(json plain paths)' \
        '--active[Only return the currently active file]' \
        '(-v --verbose)'{-v,--verbose}'[Enable verbose output]' \
        '--debug-processes[List all running processes]' \
        '--debug-process[List processes matching specific name]:name:' \
        '(-h --help)'{-h,--help}'[Show help]' \
        '(-V --version)'{-V,--version}'[Show version]'
}

# BASH completion for ide-files  
_ide_files_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    opts="--ide --list-ides --auto --format --active --verbose --debug-processes --debug-process --help --version"
    
    case "${prev}" in
        --ide)
            COMPREPLY=( $(compgen -W "goland pycharm idea vscode vs webstorm phpstorm rubymine clion vim nano" -- ${cur}) )
            return 0
            ;;
        --format)
            COMPREPLY=( $(compgen -W "json plain paths" -- ${cur}) )
            return 0
            ;;
        --debug-process)
            # Could complete with running process names, but keep simple for now
            return 0
            ;;
    esac

    if [[ ${cur} == -* ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
}

# Register completion function
if [ -n "$ZSH_VERSION" ]; then
    compdef _ide-files ide-files
elif [ -n "$BASH_VERSION" ]; then
    complete -F _ide_files_completion ide-files
fi
COMPLETION_EOF

# Set up shell integration
if [ -n "$ZSH_VERSION" ] || [ "$SHELL" = "/bin/zsh" ] || [ "$SHELL" = "/usr/bin/zsh" ]; then
    # ZSH setup
    if ! grep -q "fpath.*$COMPLETION_DIR" "$SHELL_RC" 2>/dev/null; then
        echo "" >> "$SHELL_RC"
        echo "# Add ide-files completion" >> "$SHELL_RC"
        echo "fpath=($COMPLETION_DIR \$fpath)" >> "$SHELL_RC"
        echo "autoload -U compinit && compinit" >> "$SHELL_RC"
        echo "Added completion setup to $SHELL_RC"
    else
        echo "Completion already configured in $SHELL_RC"
    fi
    echo "ZSH completion installed. Run 'exec zsh' or start a new terminal to activate."
else
    # BASH setup
    if [ "$COMPLETION_DIR" = "$HOME/.bash_completion.d" ]; then
        if ! grep -q "$COMPLETION_DIR" "$SHELL_RC" 2>/dev/null; then
            echo "" >> "$SHELL_RC"
            echo "# Load ide-files completion" >> "$SHELL_RC"
            echo "[ -f $COMPLETION_DIR/_ide-files ] && source $COMPLETION_DIR/_ide-files" >> "$SHELL_RC"
            echo "Added completion setup to $SHELL_RC"
        else
            echo "Completion already configured in $SHELL_RC"
        fi
    else
        echo "System-wide BASH completion installed to $COMPLETION_DIR"
    fi
    echo "BASH completion installed. Run 'exec bash' or start a new terminal to activate."
fi

echo ""
echo "✅ Completion setup complete!"
echo "Test it by typing: ide-files --<TAB>"
echo ""
echo "Available commands with completion:"
echo "  ide-files --ide <TAB>        # Show available IDEs"
echo "  ide-files --format <TAB>     # Show output formats"
echo "  ide-files --<TAB>            # Show all options"