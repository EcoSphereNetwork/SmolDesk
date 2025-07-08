---
title: Testplan SmolDesk Mobile
description: Teststrategie und Testfälle für die mobile App
---

# Einleitung
Dieser Testplan definiert Vorgehen und Umgebungen zur Prüfung von SmolDesk Mobile. Er ergänzt die [allgemeine Teststrategie](../testing/strategy.md) und dient als Grundlage für das [Testprotokoll](./testprotokoll.md).

## Testumgebungen
- Android-Emulator und reale Geräte gemäß [Setup-Anleitung](./setup-android.md)
- iOS-Simulator gemäß [Setup-Anleitung](./setup-ios.md)
- Lokale Node-Umgebung für Unit- und Integrationstests

## Testarten
- **Unit Tests** via Vitest/Jest
- **Integrationstests** mit gemockten WebRTC-Komponenten
- **Manuelle Szenarien** auf Endgeräten
- **End-to-End-Tests** optional mit Playwright oder Detox

## Testfälle
1. Verbindung zu einem Linux-Host herstellen
2. Gestenbedienung: Pinch-Zoom, Drag, Rechtsklick (Zwei-Finger-Tap)
3. [Clipboard-Synchronisation](../usage/clipboard.md) zwischen Host und Mobilgerät
4. Login- und Logout-Prozess über OAuth2 inklusive Token-Refresh
5. Tokenweitergabe und -prüfung beim Signaling
6. Verschlüsselte und unverschlüsselte Dateiübertragungen – siehe [Dateien senden und empfangen](../usage/files.md)
7. Monitorwechsel bei mehreren Bildschirmen – siehe [Monitorsteuerung](../usage/monitors.md)
8. Umschalten zwischen Dark- und Light-Mode
9. Stabilität bei Hintergrundbetrieb und Rückkehr zur App
10. Responsives Layout auf kleinen Phones und Tablets im [Viewer](../usage/viewer.md)

## Ergebnisse
Alle Resultate und offene Punkte werden im [Testprotokoll](./testprotokoll.md) festgehalten.
