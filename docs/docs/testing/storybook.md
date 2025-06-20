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

## Visual Regression with Screenshot Snapshots

Run automated screenshot tests for every Storybook story:

```bash
npm run test:storybook:snapshots
```

Screenshots are saved to `storybook-snapshots/` and uploaded as CI artifacts.

## \ud83d\udce4 Storybook Deployment

A GitHub Actions workflow builds the static Storybook and publishes it to the
`gh-pages` branch whenever changes land on `main`. Ensure the repository settings point GitHub Pages to this branch. After each merge you can open
`https://<user>.github.io/<repo>/` to preview all components and verify that the
latest build is available. During CI a zipped `storybook-static` folder is uploaded as an artifact for manual inspection.

The workflow writes a `.nojekyll` file so GitHub serves all assets correctly.

### Fehlerbehandlung GitHub Pages

Sollte der unter `https://ecospherenetwork.github.io/SmolDesk/` gehostete Storybook-Build einen 404-Fehler liefern, überprüfe zuerst, ob der `gh-pages`-Branch korrekt erzeugt wurde und ob die `publish_dir` im Workflow auf `storybook-static` zeigt. Prüfe außerdem den `homepage`-Eintrag in der `package.json` und ob eine `.nojekyll`-Datei im Ausgabeverzeichnis liegt.


### Fallback-Vorschau über CI-Artefakt

Falls GitHub Pages nicht erreichbar ist, stellt die CI den Inhalt von `storybook-static/` als Download-Artefakt bereit. Der Workflow kommentiert im Pull Request einen Hinweis mit dem Link zur Action, damit Reviewer die Vorschau manuell laden können.
