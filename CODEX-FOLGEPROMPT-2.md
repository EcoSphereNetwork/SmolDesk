# 🧠 Codex Folgeprompt – Test-Stabilisierung & Tauri-Mocking in SmolDesk

## 🔄 AUSGANGSLAGE

Die bisherigen Aufgaben wurden erfolgreich bearbeitet, jedoch bestehen weiterhin folgende Probleme:

- ✅ Build funktioniert mit GTK 4.1 und dokumentierten Abhängigkeiten
- ✅ `.nvmrc` wurde hinzugefügt, Node-Version ist geklärt
- ✅ Vitest-Konfiguration wurde verbessert (`jsdom`, globale Mocks, Setup-File)
- ❌ `npm run test:run` schlägt weiterhin fehl, **vor allem wegen fehlender Tauri-Umgebungen**
- ⚠️ Einige Netzwerkzugriffe (z. B. `crates.io`, `github.com`) wurden blockiert

---

## 🎯 ZIEL DIESES LAUFS

Codex, stabilisiere die Testumgebung vollständig durch **Mocking der Tauri-API** und überarbeite alle fehlerhaften Testfälle.

---

## ✅ AUFGABEN FÜR CODEX

1. **Fehlende Tauri-APIs mocken:**
   - Identifiziere alle fehlenden `@tauri-apps/api`-Funktionen, die im Testkontext Fehler verursachen.
   - Erstelle eine zentrale Mock-Datei (z. B. `test/__mocks__/tauri.ts`), die die notwendigen Funktionen bereitstellt.
     Beispiel:
     ```ts
     export const invoke = vi.fn(() => Promise.resolve());
     export const listen = vi.fn();
     export const emit = vi.fn();
     ```
   - Verwende in `vitest.config.ts` das Feld `alias` oder `mock` oder richte `setupFiles` so ein, dass die Tauri-API korrekt gemockt wird.

2. **Testsuite stabilisieren:**
   - Gehe alle Tests unter `*.test.ts`/`*.spec.ts` durch und prüfe:
     - Wird `@tauri-apps/api` direkt verwendet?
     - Wurden `window`- oder `navigator`-basierte APIs gemockt?
   - Ergänze fehlende Mocks:
     - `navigator.mediaDevices.getUserMedia`
     - `window.crypto.subtle`
     - `RTCPeerConnection`, `RTCSessionDescription`
   - Repariere alle Tests, die durch fehlende Imports, veraltete Syntax oder gebrochene Assertions fehlschlagen.

3. **Testarchitektur verbessern:**
   - Falls nicht vorhanden, erstelle eine zentrale Setup-Datei: `test/setup.ts`
     - Registriere dort globale Mocks
     - Definiere bei Bedarf globale Variablen für WebRTC/Tauri
   - Verlinke sie in der `vitest.config.ts` via `setupFiles: ['./test/setup.ts']`

4. **Tests erneut ausführen:**
   - Starte `npm run test:run` erneut.
   - Bei erfolgreichem Durchlauf:
     - Erstelle einen Pull Request mit dem Titel:
       > ✅ Stabilisierte Tests: Tauri-Mocking & Browser-APIs
   - Bei verbleibenden Fehlern:
     - Lege je ein **GitHub Issue** mit präzisem Testnamen, Dateipfad und Fehlerauszug an.

5. **Dokumentation aktualisieren:**
   - Ergänze `README.md`:
     - Hinweis zur Verwendung von Tauri-Mocks
     - Hinweise für Entwickler:innen zur Nutzung von `vitest --ui` zur Test-Diagnose
   - Dokumentiere das Verhalten im Offline- bzw. beschränkten Netzwerk-Modus

---

## 🧩 HINWEISE

- Vermeide echte Tauri-Initialisierung im Test – verwende nur `vi.mock()` oder lokale Mock-Dateien
- Dokumentiere jeden Fix in einem **eigenen Commit** mit Bezug zu Issue/PR
- Halte das Setup **modular**, nutze keine globalen Hardcodings

---

📌 Starte jetzt mit:
```bash
npm run test:run
````

und analysiere die Fehler, beginnend mit Tauri-Imports in `connection.test.ts`, `enhanced-webrtc.test.ts` und `state-handlers.test.ts`.

