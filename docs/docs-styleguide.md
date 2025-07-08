---
title: SmolDesk Dokumentations-Styleguide
description: Richtlinien für Sprache und Format
---

<!-- markdownlint-disable MD025 -->

# SmolDesk Dokumentations-Styleguide

Dieser Styleguide definiert verbindliche Regeln für die gesamte SmolDesk-Dokumentation. Er soll ein konsistentes Erscheinungsbild gewährleisten und allen Autor:innen als Nachschlagewerk dienen.

## Empfohlene Schreibweise

- Anrede immer **du**.
- Projekte und Features mit festem Namen erwähnen, z. B. **"SmolDesk Mobile App"**.
- Klare, aktive Formulierungen verwenden.

## Sprachliche No-Gos

- Vermeide umgangssprachliche Formulierungen wie "klick einfach mal drauf".
- Keine vagen Aussagen wie "geht manchmal nicht".

## Formatkonventionen

- H1 wird nur für den Dokumenttitel verwendet.
- Codeblöcke immer mit Sprache angeben, z. B. `ts`, `bash` oder `rs`.
- Frontmatter enthält mindestens `title` und `description`.
- Links immer relativ setzen, etwa `../features/clipboard.md`.

## i18n-Hinweise

- Begriffe so sprachneutral wie möglich formulieren.
- Keine festen Verweise auf eine Sprache setzen; stattdessen strukturell lösen.

## Pflegehinweise

- Jede Dokumentations-Änderung muss diesen Styleguide berücksichtigen.
- Der Styleguide wird bei Bedarf erweitert und aktualisiert.

## Beispiele

### Sprache

Falsch:

> Sie können das Programm starten, wenn Sie auf den Button klicken.

Richtig:

> Starte das Programm, indem du auf den Button klickst.

### Format

Falsch:

```md
# Titel

## Untertitel

### Noch kleiner
```

Richtig:

```md
---
title: Beispiel
description: Kurze Beschreibung
---

# Beispiel

## Untertitel
```

### Verweise

Falsch:
[Dokumentation](https://example.com/docs/feature)

Richtig:
[Dokumentation](./features/remote.md)

### Gutes Abschnitt-Beispiel

```md
---
title: Setup
description: Einrichtung der Entwicklungsumgebung
---

# Setup

1. `npm install`
2. `npm run build`
```

## 🔧 Automatische Qualitätsprüfungen

Vor jedem Commit und im CI überprüfen Skripte die Dokumentation.
`markdownlint` meldet Formatfehler, während
`scripts/docs_validation.py` defekte Links und fehlendes Frontmatter sucht.
Bei Auffälligkeiten erzeugt das Python-Skript einen Bericht unter
`docs/validation/report.md`. Markdownlint agiert nur als Warnung und blockiert
den Build nicht.

## 🚀 Deployment der Dokumentation

### Voraussetzungen

- Node.js und npm installiert
- Schreibzugriff auf das GitHub Repository

### Workflow

1. `npm run deploy-docs` im Projektstamm ausführen
2. Das Skript baut die Doku und pusht den Inhalt auf den Branch `gh-pages`

Setze vor dem Ausführen entweder die Umgebungsvariable `USE_SSH=true` oder
`GIT_USER=<Dein GitHub Benutzername>`, damit Docusaurus Berechtigungen für den
Push hat.

### Hinweise

- Der Branch `gh-pages` wird automatisch von GitHub Pages veröffentlicht
- Bei Build-Fehlern hilft ein erneutes `npm install` im `docs/`-Verzeichnis

## 🚫 Ausgeschlossene Inhalte

Folgende Bereiche gehören nicht zur SmolDesk-Dokumentation und dürfen nicht über Navigation oder Links referenziert werden:
- Agenten-Systeme (`docs/agents/`)
- Docusaurus-Tutorial-Inhalte (`docs/docusaurus/`, `docs/blog/`)
- Beispielseiten (`src/pages/markdown-page.md`, etc.)

## 🧩 Startseiten-Komponente (HomepageFeatures)

- Datei: docs/src/components/HomepageFeatures/index.tsx
- Zeigt 3–6 projektrelevante SmolDesk-Funktionen auf der Startseite
- Klarer, verständlicher Text (Zielgruppe: interessierte Nutzer:innen)
- Icons oder Emojis zur visuellen Unterstützung

## 🎨 Designrichtlinien GitHub Pages

- Startseite nutzt interaktive Feature-Karten mit Icons/SVGs
- Logo im Header sichtbar, verlinkt auf /
- Sidebar strukturiert nach Modulen
- Header & Footer enthalten Navigation, GitHub-Link, Lizenz
- Icons aus icons/ verwenden

## 📅 Deployment-Protokoll

- **Datum:** 2025-07-08
- **Live-Version:** [https://ecospherenetwork.github.io/SmolDesk/](https://ecospherenetwork.github.io/SmolDesk/)
- **Hinweise:** Deployment-Skript erfordert ein konfiguriertes Git-Remote. Im CI erfolgt der Push auf `gh-pages` automatisch.

## 🧪 Interaktive Dokumentation

- Live-Demos liegen unter `docs/docs/demo` und nutzen `@theme/LiveCodeBlock`.
- API-Beispiele greifen auf `swagger-ui-react` und `openapi.yaml` zu.
- Halte Beispielcode kurz und frei von echten Zugangsdaten.
- Aktualisiere die OpenAPI-Datei bei API-Änderungen.
