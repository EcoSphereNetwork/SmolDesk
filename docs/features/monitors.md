---
title: Multi-Monitor-Unterstützung
description: Wechsel zwischen mehreren Bildschirmen im laufenden Betrieb.
---

## Funktion & Zweck
SmolDesk erkennt verfügbare Monitore und erlaubt das Umschalten während einer Sitzung.

## UX-Verhalten / Interface
- Alle Monitore werden im Host-Tab angezeigt
- Umschalten erfolgt über einen Dialog im Viewer
- Jeder Bildschirm kann eigene Auflösung und Bildrate haben

Siehe [../usage/monitors.md](../usage/monitors.md) für Nutzerhinweise.

## Technische Architektur / Datenfluss
- Monitorinformationen werden vom Backend per Tauri-IPC geliefert
- `ConnectionManager` fordert bei einem Wechsel einen neuen Stream an
- RemoteScreen passt Größe und Skalierung automatisch an

## Sicherheit & Einschränkungen
- Bei sehr hohen Auflösungen steigt die Bandbreite deutlich an
- Nicht jeder Monitor unterstützt Hardwarebeschleunigung

## Verweise
- Architekturdetails unter [../docs/architecture.md](../docs/architecture.md)
