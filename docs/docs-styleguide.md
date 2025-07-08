---
title: SmolDesk Dokumentations-Styleguide
description: Richtlinien für Sprache und Format
---

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

```
# Titel
## Untertitel
### Noch kleiner
```

Richtig:

```
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
[Dokumentation](../features/feature.md)

### Gutes Abschnitt-Beispiel

```
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

### Hinweise
- Der Branch `gh-pages` wird automatisch von GitHub Pages veröffentlicht
- Bei Build-Fehlern hilft ein erneutes `npm install` im `docs/`-Verzeichnis

