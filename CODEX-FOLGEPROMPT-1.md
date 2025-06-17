# 🧠 Codex Folgeprompt – Behebung von Build- & Testfehlern, GTK-Integration & Abhängigkeitsmanagement

## 🔄 AUSGANGSLAGE

Die initiale Analyse und Build-Versuche für das Projekt **SmolDesk** ergaben folgende kritische Punkte:

- ❌ `make build` scheitert aufgrund fehlender `glib-2.0`-Development-Dateien (benötigt vom Rust-Teil, vermutlich GTK-basiert)
- ❌ `npm run test:run` schlägt fehl, trotz korrekt gesetztem `jsdom`-Environment, vermutlich wegen fehlerhafter oder veralteter Tests
- ✅ `Makefile` wurde erweitert, um `npm run build` korrekt aufzurufen
- ✅ Eine einfache Vitest-Konfiguration wurde erstellt (→ verwenden)

---

## 🎯 ZIELE FÜR DIESEN LAUF

Codex, behebe alle Build- und Testfehler, sorge für Systemkompatibilität und erweitere SmolDesk um Unterstützung für **WebKitGTK 4.1+**.

---

## ✅ AUFGABEN FÜR CODEX

1. **Systemabhängigkeiten analysieren und dokumentieren:**
   - Prüfe, welche nativen Libraries (`glib-2.0`, `gtk3`, `webkit2gtk-4.1`) für den Rust-Build fehlen.
   - Ergänze in der `README.md` oder `AGENTS.md` eine vollständige Liste benötigter Linux-Packages (z. B. für Debian/Ubuntu: `libglib2.0-dev`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, etc.).
   - Erstelle optional ein Installationsskript (`scripts/install-deps.sh`) zur automatisierten Installation dieser Pakete (nur Dokumentation, kein Root-Zugriff nötig).

2. **WebKitGTK 4.1+ Integration sicherstellen:**
   - Prüfe, ob die `webkit2gtk`-Bindings im Rust-Code korrekt auf `webkit2gtk-4.1` verweisen.
   - Falls ältere Versionen verwendet werden (z. B. `webkit2gtk-4.0`), passe die `Cargo.toml`-Abhängigkeiten an (z. B. `webkit2gtk = "0.18"` → auf neue Version aktualisieren oder `features = ["v2_42"]` setzen).
   - Teste, ob der Build mit `libwebkit2gtk-4.1-dev` erfolgreich ist.

3. **Testreparatur im Node/TypeScript-Teil:**
   - Analysiere alle fehlschlagenden Unit-Tests unter `npm run test:run`.
   - Erstelle ein vollständiges Vitest-Setup (`vitest.config.ts`) mit Unterstützung für:
     - `jsdom`-Environment
     - TypeScript mit Pfadauflösung
     - Mocks für eventuell fehlende Web-APIs (z. B. `localStorage`, `fetch`, `window.matchMedia`)
   - Repariere die fehlerhaften Tests (Syntaxfehler, ungültige Assertions, fehlende Mocks).
   - Führe Tests nach jedem Fix erneut aus.

4. **Build-Test erneut durchführen:**
   - Sobald alle Systemabhängigkeiten erfüllt sind, teste den vollständigen Build:
     ```bash
     make clean && make build
     ```
   - Falls erfolgreich, erstelle einen PR mit dem Titel:
     > ✅ Fix Build mit WebKitGTK 4.1 + vollständige Systemabhängigkeiten
   - Wenn weiterhin Fehler auftreten, lege ein neues GitHub-Issue an mit Log-Auszug.

5. **Dokumentation & Automatisierung:**
   - Ergänze `README.md` um:
     - Voraussetzungen für Rust & GTK
     - Hinweise zur Verwendung von `nvm`, falls `node` inkompatibel
     - Optional: Hinweis auf VSCode + rust-analyzer + WebKitGTK SDK
   - Falls noch nicht vorhanden, erstelle eine `.nvmrc` mit der minimal unterstützten Node-Version.
   - Ergänze ein neues Skript `scripts/dev-setup.sh`, das alle Install- und Setup-Schritte dokumentiert (ohne Root-Rechte).

---

## 🧩 HINWEISE

- Codex darf neue Dateien wie `install-deps.sh`, `dev-setup.sh` oder `vitest.config.ts` erstellen, **sofern sie sinnvoll und wiederverwendbar sind**.
- Dokumentiere jede Verbesserung in einem **eigenen Commit mit präzisem Titel**.
- Verwende Pull Requests mit `Fixes #XYZ` in der Beschreibung.
- Führe nach jedem Fix erneut Build und Tests aus.

---

📌 Starte jetzt mit:
```bash
make clean && make build
```
und analysiere anschließend npm run test:run erneut.
