---
title: Android Setup
description: Anleitung zur Einrichtung der Android-Umgebung für SmolDesk Mobile
---

Diese Anleitung beschreibt die grundlegenden Schritte, um SmolDesk Mobile unter Android zu entwickeln und zu testen. Eine Übersicht der App-Funktionen findest du in [../usage/viewer.md](../usage/viewer.md).

## Voraussetzungen

- Node.js 18 oder neuer
- Android Studio mit aktuellem Android SDK
- Java JDK (mindestens Version 11)
- React‑Native CLI (`npx react-native`)

## Installation

1. Repository klonen und Abhängigkeiten installieren:
   ```bash
   npm install
   ```
2. Android-Gerät oder Emulator vorbereiten und das Projekt starten:
   ```bash
   npx react-native run-android
   # oder
   npm run android
   ```
3. Für einen signierten Release‑Build:
   ```bash
   cd android
   ./gradlew assembleRelease
   ```

Weitere Hinweise zum Einsatz der App findest du in der [SmolDesk Mobile Dokumentation](../development/Smodesk-Mobile.md).
