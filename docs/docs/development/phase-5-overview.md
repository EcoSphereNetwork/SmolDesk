# Phase 5 Overview

This phase focuses on validating individual UI components in Storybook.

## Goals
- Provide a Storybook story for every component
- Capture visual snapshots using Playwright component tests
- Run accessibility checks with `jest-axe`

Snapshots will serve as a reference for future UI changes. Storybook will also document props and usage examples.

## Progress
Phase 5.1 started with Storybook setup and initial tests for the Button component.
Phase 5.2 covers all components with stories, snapshot tests and accessibility checks.
Phase 5.3 adds visual screenshot testing and uploads snapshots as CI artifacts.
Phase 5.3 is now complete after fixing a JSON parse error in the snapshot setup.
Snapshots are saved under `storybook-snapshots/` and uploaded in CI.
See [storybook-status.md](../components/storybook-status.md) for details.
