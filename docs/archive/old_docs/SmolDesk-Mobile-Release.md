---
title: SmolDesk Mobile Release Checkliste
description: ''
---
⚠️ Diese Datei wurde archiviert. Die aktuelle Version befindet sich unter `docs/release/release-process.md`

# SmolDesk Mobile Release Checkliste

## Vorbereitung
- [ ] Screenshots und Play-Store-Text prüfen
- [ ] Privacy Policy aktualisieren
- [ ] Version in `app/build.gradle` anheben

## Beta-Test
1. Release-Build erzeugen
   ```bash
   cd android
   ./gradlew assembleRelease
   ./gradlew bundleRelease
   ```
2. Entstandene `.apk` oder `.aab` signiert bereitstellen
3. In der Google Play Console unter "Internal testing" hochladen
4. Tester einladen und Feedback einsammeln

## Produktion
- [ ] Finalen Build hochladen
- [ ] Listing freigeben
