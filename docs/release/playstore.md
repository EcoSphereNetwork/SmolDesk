---
title: Android-Veröffentlichung
description: Anleitung zur Bereitstellung von SmolDesk Mobile im Google Play Store
---

Diese Anleitung beschreibt, wie SmolDesk Mobile für Android veröffentlicht wird. Für die Einrichtung der Entwicklungsumgebung siehe [../development/setup-android.md](../development/setup-android.md).

## Play Store Listing

### Kurzbeschreibung
Fernzugriff auf deinen Linux-Desktop per WebRTC

### Ausführliche Beschreibung
SmolDesk Mobile verbindet dein Telefon direkt mit deinem Linux-PC. Sichere Peer-to-Peer-Verbindung, Multi-Monitor und Dateiübertragung inklusive.

### What's New
Erste Beta-Version

Weitere Hinweise zur Datenschutzerklärung finden sich unter [../public/privacy-policy.html](../public/privacy-policy.html).

## Release-Build erstellen

1. Version in `app/build.gradle` erhöhen
2. Release-Build erzeugen

```bash
cd android
./gradlew assembleRelease
./gradlew bundleRelease
```

3. Die entstandene `.aab`-Datei signieren und in der Google Play Console hochladen
4. Tester über "Internal testing" einladen und Feedback einholen

Die detaillierte Release-Abfolge ist im [Release-Prozess](./release-process.md) beschrieben.
