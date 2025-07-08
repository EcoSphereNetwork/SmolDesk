---
title: SmolDesk Mobile Eingabesteuerung
description: ''
---
> ⚠️ Diese Datei wurde archiviert. Der aktuelle Inhalt befindet sich unter `docs/usage/viewer.md`

# SmolDesk Mobile Eingabesteuerung

Dieses Dokument beschreibt die Eingabelogik der Mobile-App.

## Touch Mapping

| Geste | Aktion |
|-------|-------|
| Tap | Mausklick links |
| Long Press | Rechtsklick |
| 1-Finger-Drag | Mauszeiger bewegen |
| 2-Finger-Tap | Rechtsklick (Alternative) |
| 2-Finger-Drag | Scrollrad |
| Pinch | Zoom im Viewer |

## Tastaturcodes

Die App sendet einfache JSON-Nachrichten über den WebRTC-Datenkanal:

```json
{ "type": "keyboard", "key": "A", "down": true }
```

Sondertasten wie Strg, Alt, Esc werden über eigene Buttons ausgelöst.

## Clipboard-Formate

Aktuell wird nur Text übertragen. Der Inhalt wird über Nachrichten des Typs
`clipboard` synchronisiert und lokal in die Zwischenablage geschrieben.
