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
Phase 5.4 introduces automatic deployment of the static Storybook via GitHub
Pages so component previews are available at
`https://<user>.github.io/<repo>/` after each merge to `main`.

Phase 5.4.1 adds a CI fallback: sollte die GitHub Pages Instanz nach dem Merge nicht erreichbar sein und nur einen 404 liefern, wird der Storybook-Build als Artefakt hochgeladen. Ein Workflow kommentiert den Link direkt im Pull Request.
Phase 5.4.2 stellt die korrekte Veröffentlichung sicher. Der Deployment-Workflow
schreibt eine `.nojekyll`-Datei und pusht nach `gh-pages`, sodass die Vorschau
dauerhaft unter `https://ecospherenetwork.github.io/SmolDesk/` erreichbar ist.