---
title: Test Coverage
description: ""
---
# Test Coverage

## Overview
Vitest can generate code coverage reports to show which files are exercised by tests.

### Local Usage
```bash
npm run coverage
```
The command outputs a summary in the console and writes reports to `coverage/`.

### In CI
The GitHub Actions workflow uploads coverage artifacts so they can be inspected in the UI. HTML reports are available as downloadable artifacts after a run.
