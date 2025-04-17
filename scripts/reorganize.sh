#!/bin/bash

# Function to print colored output
print_step() {
    echo -e "\033[0;34m==> $1\033[0m"
}

# Function to check and install Python package
ensure_package() {
    if ! python3 -c "import $1" &>/dev/null; then
        print_step "Installing $1..."
        python3 -m pip install --user "$1"
    fi
}

# Check Python installation
if ! command -v python3 &>/dev/null; then
    print_step "Python 3 is required but not installed. Please install Python 3 and try again."
    exit 1
fi

# Check pip installation
if ! command -v pip3 &>/dev/null; then
    print_step "Installing pip..."
    curl https://bootstrap.pypa.io/get-pip.py -o get-pip.py
    python3 get-pip.py --user
    rm get-pip.py
fi

# Install required packages
print_step "Checking required packages..."
ensure_package "toml"
ensure_package "rich"
ensure_package "git"

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Run the reorganization script
print_step "Running reorganization script..."
python3 "$SCRIPT_DIR/reorganize_standalone.py" "$@"