_**ESN_Repo-Template**_
# Quick Start Guide

This guide will help you get started with the project template.

## Prerequisites

- Python 3.9 or higher
- Git
- Docker (optional)

## Installation

1. Create a new repository from the template:
   ```bash
   gh repo create my-project --template EcoSphereNetwork/Repo-Template_-new-
   # or
   git clone https://github.com/EcoSphereNetwork/Repo-Template_-new-.git my-project
   ```

2. Initialize the project:
   ```bash
   cd my-project
   ./scripts/init.sh
   ```

3. Set up the development environment:
   ```bash
   poetry install
   pre-commit install
   ```

## Development

1. Activate the virtual environment:
   ```bash
   poetry shell
   ```

2. Run the application:
   ```bash
   poetry run app
   ```

3. Run tests:
   ```bash
   poetry run pytest
   ```

4. Check code quality:
   ```bash
   poetry run black .
   poetry run ruff check .
   poetry run mypy .
   ```

## Docker Support

1. Build the image:
   ```bash
   docker-compose build
   ```

2. Run the application:
   ```bash
   docker-compose up
   ```

## Next Steps

- Read the [Development Guide](../development/guide.md)
- Check out the [API Reference](../api/reference.md)