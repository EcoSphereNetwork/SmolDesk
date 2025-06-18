# Storybook Guide

This project uses Storybook with the Vite builder to document React components.

## Running Storybook

```bash
npm run storybook
```

## Building Static Storybook

```bash
npm run build-storybook
```

## Snapshot and Interaction Tests

Storybook stories can be tested with the `@storybook/test-runner`. Unit tests
compose stories and assert on the rendered output:

```ts
import { composeStory } from '@storybook/testing-react';
import * as stories from '../../src/components/Button.stories';

const Primary = composeStory(stories.Primary, stories.default);
```

## Accessibility Checks

Use `jest-axe` to ensure each component has no accessibility violations:

```ts
import { axe, toHaveNoViolations } from 'jest-axe';
```

## Komponenten vollständig abdecken

Every component under `src/components/` should ship with a Storybook story
and matching snapshot plus accessibility tests. The table in
[storybook-status.md](../components/storybook-status.md) tracks coverage.

Validate the setup:

```bash
bash scripts/validate-storybook.sh
```
