# 🧠 Codex Folgeprompt – Analyse der Projekt-Dokumentation & Erweiterte Teststabilisierung

## 🔄 AUSGANGSLAGE

Die Testumgebung von **SmolDesk** wurde deutlich verbessert, jedoch bestehen weiterhin instabile oder fehlschlagende Tests rund um:

- WebRTC-Funktionalitäten (`RTCPeerConnection`, `MediaStream`)
- Bildschirmübertragung (`this.mediaStream.getTracks is not a function`)
- Zusammenspiel von Tauri + Web + Systemfunktionen

Gleichzeitig wurde die globale Teststruktur konsolidiert, Mocks eingeführt und dokumentiert.

---

## 🎯 ZIEL DIESES LAUFS

Codex, analysiere die gesamte Dokumentation von SmolDesk, um tieferes Verständnis über:

- Architektur
- API-Design
- technische Ziele
- relevante Komponenten

zu erhalten – und nutze dieses Wissen, um die noch fehlschlagenden Tests korrekt zu stabilisieren, zu verbessern oder gezielt auszulassen.

---

## ✅ AUFGABEN FÜR CODEX

### 1. 📚 Projektverständnis durch Dokumentation
- Analysiere folgende Dokumentationspfade vollständig:
  - `./docs/docs/SmolDesk` – besonders `development/*` `docs/docs/SmolDesk/README.md`
  - `./README.md` – Hinweise zu Zielen, Komponentenstruktur, Feature-Flags

- Extrahiere:
  - Zielarchitektur (Frontend/Backend-Kommunikation, Tauri-Anbindung)
  - Nutzung von WebRTC, Signaling, Events
  - geplante oder beschriebene Teststrategien
- Dokumentiere eine Kurz-Zusammenfassung als `docs/docs/summary.project-insights.md`

### 2. 🧪 Erweiterte Test-Stabilisierung
- Nutze das neue Wissen über Architektur & Funktionalität, um die Tests rund um **ScreenCapture & MediaStreams** zu verbessern:
  - Ersetze oder erweitere die bisherigen Mock-Implementierungen:
    ```ts
    globalThis.navigator.mediaDevices = {
      getDisplayMedia: vi.fn(() => Promise.resolve({ getTracks: () => [] })),
      getUserMedia: vi.fn(() => Promise.resolve({ getTracks: () => [] })),
    };
    ```
  - Erstelle ggf. `MockMediaStream`, `MockTrack` Klassen im Setup.
- Prüfe ob bestimmte Tests durch Feature-Flags deaktiviert sein sollten (temporäres `.skip()`), falls sie nur im echten Tauri-Context ausführbar wären.
- Überprüfe besonders:
  - `screen-capture.test.ts`
  - `connection.test.ts`
  - `enhanced-webrtc.test.ts`
  - `state-handlers.test.ts`

### 3. ✅ Verbesserte Teststrategie dokumentieren
- Lege eine Datei `docs/testing/strategy.md` an mit:
  - Übersicht über alle Testarten im Projekt (Unit, Integration, Mock-based Simulation)
  - Hinweise zu realen vs. gemockten Laufumgebungen (Tauri vs. Node)
  - Warnhinweise bei flakey Tests und Feature-Simulationen

### 4. 🧪 Tests ausführen & bewerten
- Führe `npm run test:run` erneut aus
- Prüfe:
  - Bestehen alle WebRTC/Screen-Tests?
  - Sind Fehler reproduzierbar oder instabil?
- Bei Fehlern:
  - Erstelle pro Datei/Fehler ein GitHub Issue mit Beschreibung + Stacktrace
  - Falls Fix möglich: Commit mit Titel wie:
    > ✅ Fix: WebRTC Test getTracks Mocked Correctly

---

## 🧩 HINWEISE

- Die Projekt-Dokumentation liefert **Kontext, den du zur Fehlerkorrektur brauchst** – lies sie gründlich.
- Verwende synthetische Objekte nur dort, wo echte APIs fehlen – vermeide Overmocking.
- Setze auf nachvollziehbare, dokumentierte Architekturentscheidungen.

---

📌 Starte jetzt mit:
```bash
# 1. Dokumentation lesen & analysieren
cat ./docs/*.md ./docs/wiki/**/*.md ./README.md

# 2. Teststruktur verbessern
npm run test:run
````

und dokumentiere deine Erkenntnisse in `docs/summary.project-insights.md`.
