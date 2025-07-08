---
title: Release-Prozess
description: Manuelle Schritte zum Bauen und Veröffentlichen von SmolDesk Mobile
---

Dieser Leitfaden führt durch den gesamten Veröffentlichungsablauf für Android und iOS.

1. **Version anheben**
   - Android: `app/build.gradle`
   - iOS: Xcode-Projekt
2. **Changelog aktualisieren**: [changelog.md](./changelog.md)
3. **Builds erstellen**
   - Android siehe [playstore.md](./playstore.md)
   - iOS siehe [testflight.md](./testflight.md)
4. **Uploads durchführen**
   - Google Play Console (Internal Testing / Produktion)
   - App Store Connect / TestFlight
5. **Freigabe**
   - Nach bestandenen Tests Release in den Stores freischalten

Für die Einrichtung der Entwicklungsumgebungen verweise auf [../development/setup-android.md](../development/setup-android.md) und [../development/setup-ios.md](../development/setup-ios.md).
