_**ESN_Repo-Template**_
# Development Guide

This guide covers the development workflow and best practices for this project.

## Development Environment

### Using Poetry

1. Install dependencies:
   ```bash
   poetry install
   ```

2. Activate virtual environment:
   ```bash
   poetry shell
   ```

3. Add new dependencies:
   ```bash
   poetry add package-name
   poetry add --group dev package-name
   ```

### Using Docker

1. Development environment:
   ```bash
   docker-compose up --build
   ```

2. Run tests:
   ```bash
   docker-compose run --rm app pytest
   ```

## Code Quality

### Style Guide

- Follow PEP 8
- Use Black for formatting
- Use Ruff for linting
- Use MyPy for type checking

### Running Checks

```bash
# Format code
poetry run black .

# Lint code
poetry run ruff check .

# Type check
poetry run mypy .

# Run all checks
make lint
```

## Testing

### Writing Tests

- Place tests in the `tests/` directory
- Use pytest fixtures for common setup
- Follow AAA pattern (Arrange, Act, Assert)
- Use parametrize for multiple test cases

### Running Tests

```bash
# Run all tests
poetry run pytest

# Run specific tests
poetry run pytest tests/unit
poetry run pytest tests/integration
poetry run pytest tests/e2e

# Run with coverage
poetry run pytest --cov
```

## Documentation

### Building Docs

```bash
# Build documentation
poetry run mkdocs build

# Serve documentation locally
poetry run mkdocs serve
```

### Writing Docs

- Use Markdown for documentation
- Follow Google style for docstrings
- Include examples in docstrings
- Keep documentation up to date

## Git Workflow

1. Create feature branch:
   ```bash
   git checkout -b feature/my-feature
   ```

2. Make changes and commit:
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

3. Push changes:
   ```bash
   git push origin feature/my-feature
   ```

4. Create pull request

## CI/CD

### GitHub Actions

- Tests run on every push and PR
- Code quality checks are automated
- Security scanning is automated
- Documentation is auto-deployed

### Deployment

```bash
# Deploy to staging
./scripts/deploy.sh staging

# Deploy to production
./scripts/deploy.sh production
```

## Best Practices

1. **Code Quality**
   - Write clean, readable code
   - Follow the style guide
   - Add type hints
   - Document your code

2. **Testing**
   - Write tests for new features
   - Maintain high coverage
   - Test edge cases
   - Use meaningful assertions

3. **Documentation**
   - Keep docs up to date
   - Include examples
   - Document APIs
   - Write clear commit messages

4. **Security**
   - Follow security best practices
   - Keep dependencies updated
   - Run security scans
   - Review security alerts