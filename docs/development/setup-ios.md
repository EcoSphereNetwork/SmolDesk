---
title: iOS Setup
description: Anleitung zur Einrichtung der iOS-Umgebung für SmolDesk Mobile
---

Diese Anleitung zeigt die wichtigsten Schritte, um SmolDesk Mobile auf iOS zu entwickeln. Hinweise zur Bedienung findest du auch im [Viewer Guide](../usage/viewer.md).

## Voraussetzungen

- macOS mit aktuellem Xcode
- Node.js 18+
- CocoaPods (für native Abhängigkeiten)

## Installation

1. Abhängigkeiten installieren und Pods einrichten:
   ```bash
   npm install
   cd ios
   pod install
   cd ..
   ```
2. Die App im Simulator oder auf einem Gerät starten:
   ```bash
   npx react-native run-ios
   # oder
   npm run ios
   ```
3. Release‑Build über Xcode erstellen:
   - In Xcode `Product > Archive` wählen und das Archiv hochladen

Weitere Details zur Portierung findest du in der [SmolDesk Mobile Planung](../archive/old_docs/Smodesk-Mobile.md).
