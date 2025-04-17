_**ESN_Repo-Template**_
# Repository Reorganization Guide

This guide explains how to use the repository reorganization script to align existing repositories with the template structure.

## Overview

The reorganization script (`scripts/reorganize.py`) helps you:

- Analyze existing repository structure
- Compare it with the template structure
- Create a plan for reorganization
- Execute the reorganization
- Generate a report of changes

## Usage

### Basic Usage

```bash
# Navigate to your repository
cd /path/to/your/repo

# Run the reorganization script
/path/to/template/scripts/reorganize.py .
```

### Options

```bash
# Show help
./scripts/reorganize.py --help

# Dry run (show what would be done without making changes)
./scripts/reorganize.py --dry-run /path/to/repo

# Specify custom branch name
./scripts/reorganize.py --branch-name feature/restructure /path/to/repo

# Use different template
./scripts/reorganize.py --template-path /path/to/template /path/to/repo
```

## Process

1. **Analysis**
   - Scans current repository structure
   - Identifies missing directories and files
   - Finds files that need reorganization

2. **Planning**
   - Creates a plan for necessary changes
   - Shows proposed file moves and creations
   - Asks for confirmation

3. **Execution**
   - Creates new directories
   - Copies template files
   - Moves existing files to appropriate locations

4. **Reporting**
   - Generates a detailed report of changes
   - Saves report to `reorganization_report.md`

## Template Structure

The script enforces the following structure:

### Required Directories
```
.github/workflows/   # GitHub Actions workflows
docs/               # Documentation
src/                # Source code
tests/              # Tests
scripts/            # Utility scripts
```

### Required Files
```
README.md                 # Project documentation
LICENSE                   # Project license
.gitignore               # Git ignore rules
pyproject.toml           # Project configuration
.pre-commit-config.yaml  # Pre-commit hooks
Dockerfile               # Docker configuration
docker-compose.yml       # Docker Compose config
Makefile                 # Build automation
mkdocs.yml              # Documentation config
```

### Optional Directories
```
.devcontainer/     # Dev container config
docs/api/          # API documentation
docs/guides/       # User guides
docs/development/  # Development docs
tests/unit/        # Unit tests
tests/integration/ # Integration tests
tests/e2e/        # End-to-end tests
```

## Best Practices

1. **Before Running**
   - Commit or stash pending changes
   - Back up important files
   - Review current structure

2. **During Execution**
   - Review the analysis carefully
   - Check proposed changes
   - Use dry-run first

3. **After Completion**
   - Review the generated report
   - Test repository functionality
   - Update documentation if needed

## Troubleshooting

### Common Issues

1. **Permission Errors**
   ```bash
   chmod +x scripts/reorganize.py
   ```

2. **Git Issues**
   - Ensure you're in a git repository
   - Check for uncommitted changes
   - Verify branch permissions

3. **Missing Dependencies**
   ```bash
   poetry install
   ```

### Error Messages

- `Error: Not a git repository`: Run from repository root
- `Permission denied`: Check file permissions
- `Branch exists`: Choose different branch name

## Examples

### Basic Reorganization

```bash
cd my-project
/path/to/template/scripts/reorganize.py .
```

### Safe Reorganization

```bash
# 1. Create new branch
git checkout -b refactor/reorganize

# 2. Run with dry-run first
./scripts/reorganize.py --dry-run .

# 3. Run actual reorganization
./scripts/reorganize.py .

# 4. Review changes
git diff

# 5. Push changes
git push origin refactor/reorganize
```

### Multiple Repositories

```bash
#!/bin/bash
REPOS=(
    "/path/to/repo1"
    "/path/to/repo2"
)

for repo in "${REPOS[@]}"; do
    echo "Reorganizing $repo..."
    ./scripts/reorganize.py "$repo"
done
```