---
title: CLI Commands
description: Nützliche Kommandozeilenaufrufe für Entwicklung und Betrieb
---

Im Projekt existieren mehrere npm-Skripte sowie Tauri- und Node-basierte Befehle. Die wichtigsten sind nachfolgend aufgeführt.

## npm scripts

| Befehl | Zweck |
|--------|-------|
| `npm run dev` | Startet das React-Frontend im Entwicklungsmodus. |
| `npm run tauri` | Erstellt die native Anwendung über die Tauri-Toolchain. |
| `npm run build` | Erstellt einen Produktionsbuild des Frontends. |
| `npm run signaling-server` | Startet den Signaling-Server (WebSocket). |
| `npm run test` | Führt die Vitest-Testsuite aus. |

Weitere Skripte sind in der `package.json` dokumentiert.

## Signaling Server

Im Unterordner `signaling-server` stehen zusätzliche Befehle zur Verfügung:

| Befehl | Beschreibung |
|--------|--------------|
| `npm start` | Startet den Server auf Port 3000. |
| `npm run dev` | Startet den Server mit automatischem Reload über nodemon. |
| `npm run docker:build` | Baut ein Docker-Image des Servers. |

Die genauen Optionen können der jeweiligen `package.json` entnommen werden.
