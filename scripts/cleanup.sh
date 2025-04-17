#!/usr/bin/env bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo_step() {
    echo -e "${GREEN}==>${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

echo_error() {
    echo -e "${RED}Error:${NC} $1"
}

# Create temporary directory for merging
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Step 1: Clean up docs structure
echo_step "Cleaning up documentation structure..."

# First, backup existing content
echo_step "Backing up existing content..."
if [ -d "docs/docs" ]; then
    cp -r docs/docs/* "$TEMP_DIR/" 2>/dev/null || true
fi

# Remove and recreate docs structure
rm -rf docs/docs
mkdir -p docs/docs/{api,development,guides}

# Restore content from backup
echo_step "Restoring content..."
cp -r "$TEMP_DIR"/* docs/docs/ 2>/dev/null || true

# Step 2: Process old documentation
echo_step "Processing old documentation..."

if [ -d "docs_old" ]; then
    # Copy API documentation
    if [ -d "docs_old/api" ]; then
        cp -r docs_old/api/* docs/docs/api/ 2>/dev/null || true
    fi
    
    # Copy development guides
    if [ -d "docs_old/development" ]; then
        cp -r docs_old/development/* docs/docs/development/ 2>/dev/null || true
    fi
    
    # Copy user guides
    if [ -d "docs_old/guides" ]; then
        cp -r docs_old/guides/* docs/docs/guides/ 2>/dev/null || true
    fi
    
    # Backup and remove old docs
    echo_step "Backing up old docs..."
    mv docs_old docs_old_backup_$(date +%Y%m%d_%H%M%S)
fi

# Step 3: Clean up root directory
echo_step "Cleaning up root directory..."

# Move roadmap to docs if it exists
if [ -f "scripts/roadmap.md" ]; then
    mv scripts/roadmap.md docs/docs/development/
fi
if [ -f "src/roadmap.md" ]; then
    rm src/roadmap.md  # Remove duplicate roadmap
fi

# Remove old MkDocs configuration if it exists
if [ -f "mkdocs.yml" ]; then
    rm mkdocs.yml
fi

# Step 4: Organize script directory
echo_step "Organizing scripts..."

# Ensure all scripts are executable
chmod +x scripts/*.sh 2>/dev/null || true

# Step 5: Final cleanup
echo_step "Performing final cleanup..."

# Remove temporary directory
rm -rf "$TEMP_DIR"

echo_step "Cleanup complete! 🎉"
echo "Repository structure has been reorganized."
echo ""
echo "Next steps:"
echo "1. Review the content in docs/docs/"
echo "2. Update docusaurus.config.ts if needed"
echo "3. Run 'cd docs && npm run start' to test the documentation"