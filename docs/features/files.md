---
title: Dateiübertragung
description: Sendet Dateien sicher über den WebRTC-Datenkanal.
---

## Funktion & Zweck
Über den eingebauten Dateitransfer können Dateien zwischen Client und Host ausgetauscht werden.

## UX-Verhalten / Interface
- Dateien per Drag & Drop ablegen oder **Datei senden** auswählen
- Mehrfachübertragungen sowie Pausieren/Fortsetzen werden unterstützt
- Standardlimit 100 MB pro Datei
- Mobile: Versand über Dokument-Picker, Empfang im Download-Ordner

Details zur Nutzung unter [../usage/files.md](../usage/files.md).

## Technische Architektur / Datenfluss
- `FileTransfer`-Komponente arbeitet mit dem WebRTC-Datenkanal
- Blöcke mit 64 KB Größe werden sequentiell übertragen
- Fortschritt wird lokal gespeichert, um Wiederaufnahme zu ermöglichen

## Sicherheit & Einschränkungen
- Übertragene Dateien werden per SHA256 verifiziert
- Große Dateien können Verbindungslatenzen erhöhen

## Verweise
- Komponenten-Dokumentation: [FileTransfer](../docs/components/FileTransfer.md)
- Sicherheitshinweise unter [../development/security.md](../development/security.md)
