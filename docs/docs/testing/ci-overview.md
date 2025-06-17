# Continuous Integration Overview

## Zielsetzung

Automate builds and run tests for every pull request. Lint sources and package artifacts on successful runs.

## Tools

- **vitest** – unit tests for React components
- **jest-axe** – accessibility checks
- **playwright** (optional) – end-to-end testing
- **cargo test** – Rust backend tests

## Strategien

- Use GitHub Actions with a matrix for frontend and backend
- Run linting and formatting checks before tests
- Gate merges on full test success

