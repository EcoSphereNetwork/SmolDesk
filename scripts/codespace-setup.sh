#!/usr/bin/env bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo_step() {
    echo -e "${GREEN}==>${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Install Python dependencies
echo_step "Installing Python dependencies..."
poetry install

# Setup pre-commit hooks
echo_step "Setting up pre-commit hooks..."
pre-commit install

# Setup documentation
echo_step "Setting up documentation..."
cd docs
npm install
npm run build

# Return to project root
cd ..

# Setup git configuration if not already set
if ! git config --global user.email > /dev/null 2>&1; then
    echo_warning "Git email not set. Please run: git config --global user.email 'you@example.com'"
fi

if ! git config --global user.name > /dev/null 2>&1; then
    echo_warning "Git name not set. Please run: git config --global user.name 'Your Name'"
fi

echo_step "Codespace setup complete! 🚀"
echo "You can now:"
echo "1. Start the documentation server: cd docs && npm start"
echo "2. Run tests: poetry run pytest"
echo "3. Build the project: poetry build"