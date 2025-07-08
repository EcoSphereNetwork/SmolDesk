---
title: Zwischenablage-Sync
description: Überträgt Texte und Bilder zwischen Host und Client.
---

## Funktion & Zweck
Die Clipboard-Funktion gleicht Inhalte automatisch ab und speichert eine Historie lokaler Einträge.

## UX-Verhalten / Interface
- Desktop: Text und Bilder (PNG, JPEG, GIF) werden übernommen
- HTML wird als reiner Text gesendet
- Auf Mobilgeräten ist derzeit nur Text unterstützt

Mehr zur Bedienung in [../usage/clipboard.md](../usage/clipboard.md).

## Technische Architektur / Datenfluss
- Komponente [`ClipboardSync`](../docs/components/ClipboardSync.md) lauscht über Tauri auf lokale Änderungen
- Über den WebRTC-Datenkanal werden Einträge an den Peer übertragen
- Standardlimit: 10 MB pro Eintrag, anpassbar über die Konfiguration

## Sicherheit & Einschränkungen
- Synchronisation kann in den Einstellungen deaktiviert werden
- Große oder unbekannte Dateitypen werden gefiltert

## Verweise
- Entwicklerhinweise unter [../development/security.md](../development/security.md)
