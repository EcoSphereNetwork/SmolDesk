# Contributing to the Project

First off, thank you for considering contributing to this project! It's people like you that make this project such a great tool.

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the issue list as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

* Use a clear and descriptive title
* Describe the exact steps which reproduce the problem
* Provide specific examples to demonstrate the steps
* Describe the behavior you observed after following the steps
* Explain which behavior you expected to see instead and why
* Include screenshots if possible

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

* Use a clear and descriptive title
* Provide a step-by-step description of the suggested enhancement
* Provide specific examples to demonstrate the steps
* Describe the current behavior and explain which behavior you expected to see instead
* Explain why this enhancement would be useful

### Pull Requests

1. Fork the repo and create your branch from `main`:
   ```bash
   git checkout -b feature/my-feature
   # or
   git checkout -b fix/my-fix
   ```

2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes:
   ```bash
   poetry run pytest
   ```

5. Make sure your code follows the style guidelines:
   ```bash
   poetry run black .
   poetry run ruff check .
   poetry run mypy .
   ```

6. Commit your changes using a descriptive commit message that follows our commit message conventions:
   ```bash
   git commit -m "feat: add amazing feature"
   # or
   git commit -m "fix: resolve issue with feature"
   ```

## Development Process

1. **Setup Development Environment**
   ```bash
   # Clone the repository
   git clone https://github.com/yourusername/project.git
   cd project

   # Install dependencies
   poetry install

   # Setup pre-commit hooks
   poetry run pre-commit install
   ```

2. **Create Feature Branch**
   ```bash
   git checkout -b feature/my-feature
   ```

3. **Make Changes**
   * Write your code
   * Add tests
   * Update documentation

4. **Test Your Changes**
   ```bash
   # Run tests
   poetry run pytest

   # Check code style
   poetry run black .
   poetry run ruff check .
   poetry run mypy .
   ```

5. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add amazing feature"
   ```

6. **Push and Create PR**
   ```bash
   git push origin feature/my-feature
   ```

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

* `feat:` - A new feature
* `fix:` - A bug fix
* `docs:` - Documentation only changes
* `style:` - Changes that do not affect the meaning of the code
* `refactor:` - A code change that neither fixes a bug nor adds a feature
* `perf:` - A code change that improves performance
* `test:` - Adding missing tests or correcting existing tests
* `chore:` - Changes to the build process or auxiliary tools

## Code Style

* Python code follows [Black](https://black.readthedocs.io/) code style
* Use type hints for all function arguments and return values
* Document all public functions and classes
* Keep functions focused and small
* Write meaningful variable and function names

## Testing

* Write unit tests for all new code
* Maintain or improve code coverage
* Include integration tests where appropriate
* Test edge cases and error conditions

## Documentation

* Update README.md with any necessary changes
* Add docstrings to all public functions and classes
* Update API documentation when changing interfaces
* Include examples in documentation

## Questions?

Feel free to ask for help in:
* GitHub Issues
* Pull Request comments
* Project Discord channel

Thank you for contributing! 🎉