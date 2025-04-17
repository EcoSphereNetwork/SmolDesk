#!/usr/bin/env bash

# Enhanced initialization script with improvements

set -euo pipefail
IFS=$'\n\t'

# Additional script configuration
readonly MINIMUM_PYTHON_VERSION="3.8"
readonly REQUIRED_COMMANDS=("git" "python3" "curl")
readonly BACKUP_DIR="${TARGET_DIR}/.backup-$(date +%Y%m%d_%H%M%S)"

# Enhanced color configuration
declare -A COLORS=(
    ["INFO"]='\033[0;34m'    # Blue
    ["SUCCESS"]='\033[0;32m'  # Green
    ["WARNING"]='\033[1;33m'  # Yellow
    ["ERROR"]='\033[0;31m'    # Red
    ["DEBUG"]='\033[0;90m'    # Gray
    ["NC"]='\033[0m'         # No Color
)

# Enhanced logging functions
log() {
    local level=$1
    shift
    echo -e "${COLORS[$level]}${level}:${COLORS["NC"]} $*"
}

debug() { [[ "${DEBUG:-0}" == "1" ]] && log "DEBUG" "$@"; }
info() { log "INFO" "$@"; }
success() { log "SUCCESS" "$@"; }
warning() { log "WARNING" "$@" >&2; }
error() { log "ERROR" "$@" >&2; }

# Enhanced error handling
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        error "Script failed with exit code $exit_code"
        if [[ -d "$BACKUP_DIR" ]]; then
            warning "Restoring from backup..."
            cp -r "$BACKUP_DIR"/* "$TARGET_DIR/"
            rm -rf "$BACKUP_DIR"
            success "Backup restored"
        fi
    fi
    exit $exit_code
}
trap cleanup EXIT

# Version checking function
check_python_version() {
    local version
    version=$(python3 -c 'import sys; print(".".join(map(str, sys.version_info[:2])))')
    if [[ $(echo -e "$MINIMUM_PYTHON_VERSION\n$version" | sort -V | head -n1) != "$MINIMUM_PYTHON_VERSION" ]]; then
        error "Python version $version is less than minimum required version $MINIMUM_PYTHON_VERSION"
        exit 1
    }
}

# Command checking function
check_required_commands() {
    local missing_commands=()
    for cmd in "${REQUIRED_COMMANDS[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_commands+=("$cmd")
        fi
    done
    
    if [[ ${#missing_commands[@]} -gt 0 ]]; then
        error "Missing required commands: ${missing_commands[*]}"
        exit 1
    fi
}

# Enhanced file backup function
backup_existing_files() {
    if [[ -d "$TARGET_DIR" ]] && [[ "$(ls -A "$TARGET_DIR")" ]]; then
        info "Creating backup of existing files..."
        mkdir -p "$BACKUP_DIR"
        cp -r "$TARGET_DIR"/* "$BACKUP_DIR/"
        success "Backup created at $BACKUP_DIR"
    fi
}

# Enhanced project name validation
validate_project_name() {
    local name=$1
    if [[ ! $name =~ ^[a-z][a-z0-9-]{1,213}[a-z0-9]$ ]]; then
        error "Invalid project name: $name"
        echo "Project name must:"
        echo "- Start with a lowercase letter"
        echo "- Contain only lowercase letters, numbers, and hyphens"
        echo "- Be between 2 and 214 characters"
        echo "- End with a letter or number"
        exit 1
    fi
}

# Enhanced dependency installation
install_dependencies() {
    info "Installing project dependencies..."
    
    # Create virtual environment if not using Poetry
    if [[ "${USE_VENV:-0}" == "1" ]]; then
        python3 -m venv .venv
        source .venv/bin/activate
        pip install --upgrade pip
        pip install -r requirements.txt
    else
        if ! command -v poetry &> /dev/null; then
            info "Installing Poetry..."
            curl -sSL https://install.python-poetry.org | python3 -
        fi
        poetry install --no-root
    fi
}

# Enhanced git initialization
initialize_git() {
    info "Initializing Git repository..."
    if [[ -d .git ]]; then
        warning "Git repository already exists"
    else
        git init
        git add .
        git commit -m "feat: initial project setup from template"
    fi
}

# Main script execution
main() {
    # Parse command line arguments
    local dry_run=0
    local force=0
    local use_venv=0
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run) dry_run=1 ;;
            --force) force=1 ;;
            --use-venv) use_venv=1 ;;
            --debug) DEBUG=1 ;;
            *) error "Unknown option: $1"; exit 1 ;;
        esac
        shift
    done
    
    # Perform initial checks
    check_required_commands
    check_python_version
    
    # Backup existing files unless force flag is used
    if [[ $force -eq 0 ]]; then
        backup_existing_files
    fi
    
    # Get project information
    read -p "Enter project name (lowercase, hyphens only): " PROJECT_NAME
    validate_project_name "$PROJECT_NAME"
    
    read -p "Enter project description: " PROJECT_DESCRIPTION
    read -p "Enter your name: " AUTHOR_NAME
    read -p "Enter your email: " AUTHOR_EMAIL
    
    # Perform setup
    if [[ $dry_run -eq 0 ]]; then
        initialize_git
        install_dependencies
        setup_documentation
        initialize_tests
        
        success "Project initialization complete! 🎉"
    else
        info "Dry run completed - no changes made"
    fi
}

# Execute main function
main "$@"
