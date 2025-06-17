# Agents
## Introduction & Project Context
SmolDesk is a WebRTC based remote desktop tool for Linux. This guide allows LLM agents to build, test and extend the project autonomously.

## Module Overview
- `src/` React frontend
- `src-tauri/` Rust backend
- `signaling-server/` Node.js signaling server
- `tests/` unit, integration and e2e suites
- `docs/` documentation site

## System Requirements
- Linux host with X11 or Wayland
- Node.js 18+ and npm
- Rust stable toolchain
- FFmpeg, xdotool/ydotool, and clipboard utilities

## LLM Structure Notes
- Keep commits focused on one topic.
- Update or create tests with any code change.
- Write documentation to `docs/` when new features are added.




Diese Anleitung befähigt den Codex-Agenten, das Projekt **SmolDesk** lokal gemäß Best Practices vollautomatisch zu installieren, zu bauen und zu testen. Der Agent soll dabei auftretende Fehler fehlertolerant behandeln, jeden Fehler dokumentieren (Issues erstellen) und wenn möglich direkt beheben (Pull Requests erstellen).

## Vorbereitung: Umgebung einrichten

* **Systemvoraussetzungen installieren:** Stelle sicher, dass alle benötigten Entwicklungswerkzeuge verfügbar sind.

  * Installiere Rust (inkl. `rustc` und `cargo`) falls nicht vorhanden, z.B. mittels **rustup** (aktuelle stabile Toolchain verwenden, oder spezifische Rust-Version falls im Projekt vorgegeben).
  * Installiere Node.js (inkl. `npm` bzw. `yarn`) falls nicht vorhanden. Falls das Repository eine bestimmte Node-Version vorgibt (z.B. durch eine `.nvmrc`-Datei oder `"engines"`-Angabe in der `package.json`), verwende diese Version (z.B. mittels **nvm**).
  * Stelle eine POSIX-kompatible Shell-Umgebung bereit (z.B. Bash oder sh). Überprüfe die Shell-Skripte im Projekt auf erforderliche externe Tools (z.B. `make`, `grep`, `jq` etc.) und installiere fehlende Utilities, damit alle Skripte ausführbar sind.
  * Sorge dafür, dass Python installiert ist, falls Teile des Build-Prozesses oder der Tests Python-Skripte verwenden.
* **Repository vorbereiten:** Klone das Git-Repository (falls noch nicht geschehen) und wechsle ins Projektverzeichnis. Stelle sicher, dass eventuell vorhandene Untermodule initialisiert und aktualisiert sind. Die weitere Arbeit findet im Wurzelverzeichnis des Projekts statt.

## Build & Installation

1. **Automatisierte Build-Skripte nutzen:** Prüfe, ob das Projekt ein zentrales Build-/Installationsskript bereitstellt (z.B. ein `Makefile` mit Targets wie `all`, oder Shell-Skripte wie `install.sh`/`build.sh`).

   * Falls ja, versuche den vorgegebenen Build-Prozess auszuführen (z.B. `make`, `./install.sh`) und beobachte die Konsolenausgabe auf Fehler.
   * Schlägt der automatisierte Build fehl, analysiere das/die Skript(e) inhaltlich: Suche nach offensichtlichen Fehlerquellen wie falschen Pfaden, fehlenden Berechtigungen, veralteten Befehlen oder unerfüllten Voraussetzungen.
   * Korrigiere gefundene Probleme direkt im Skript, sofern der Fix klar ist (z.B. Pfad korrigieren, Kommando ersetzen, fehlende Optionen hinzufügen). Dokumentiere solche Änderungen in einem entsprechenden Commit oder Pull Request.
   * Führe den Build danach erneut aus, um zu prüfen, ob die Korrekturen wirken. Falls neue Fehler auftreten, notiere sie für die spätere Fehlerbehandlung.
2. **Manueller Build (falls nötig):** Wenn kein funktionierendes zentrales Build-Skript existiert **oder** weiterhin Fehler auftreten, führe die Build-Schritte manuell durch:

   * **Rust-Komponenten:** Installiere alle Rust-Abhängigkeiten und baue die Rust-Teile des Projekts. Führe z.B. `cargo build` im entsprechenden Crate/Workspace aus (bei Bedarf zuerst `cargo update`, um veraltete Dependencies zu aktualisieren). Achte auf spezielle Anforderungen wie Feature-Flags oder ein Workspace-Setup im `Cargo.toml`. Bei Erfolg kann optional ein Release-Build (`cargo build --release`) erfolgen, um sicherzustellen, dass auch im Optimized-Modus alles kompiliert.
   * **Node/TypeScript-Komponenten:** Installiere die JavaScript/TypeScript-Abhängigkeiten (z.B. mit `npm install` oder `yarn install`). Anschließend kompiliere bzw. bundle den TypeScript-Code gemäß den Projektvorgaben (typischerweise via `npm run build` oder direkt `tsc`). Stelle sicher, dass dabei alle notwendigen Umgebungsvariablen gesetzt sind und die Ausgabepfade korrekt konfiguriert sind (prüfe z.B. ob ein Build-Output in ein bestimmtes Verzeichnis erwartet wird).
   * **Shell-Komponenten:** Falls Teile des Projekts in Shell implementiert sind (z.B. Skripte in einem `scripts/`- oder `bin/`-Ordner), mache sie ausführbar (`chmod +x <datei>`) und führe etwaige erforderliche Installations- oder Initialisierungsschritte aus. Beispielsweise könnten Shell-Skripte weitere Abhängigkeiten herunterladen oder konfigurieren – stelle sicher, dass diese Schritte erfolgreich durchlaufen.
   * Wiederhole die obigen Teil-Builds, bis alle Komponenten ohne Fehler gebaut sind. Tritt ein Fehler auf, dokumentiere ihn gemäß den Richtlinien zur Fehlerbehandlung (siehe unten) und versuche, ihn zu beheben.
   * Nachdem alle Teile erfolgreich gebaut wurden, verifiziere falls möglich die Funktion des Gesamtsystems (z.B. durch einen Probelauf der Anwendung oder Aufruf zentraler Funktionen), um sicherzustellen, dass die Installation vollständig und konsistent ist.

## Testausführung

1. **Vorhandene Tests ausführen:** Führe alle vorhandenen Testsuiten durch, um die Funktionalität des Projekts zu überprüfen.

   * **Rust-Tests:** Führe `cargo test` im Rust-Projektteil aus (ggf. mit `--all` in einem Workspace, um alle Teilprojekte abzudecken). Achte auf Fehlermeldungen oder panics in der Ausgabe.
   * **TypeScript/Node-Tests:** Führe `npm test` bzw. das im Projekt vorgesehene Testkommando im Node/TS-Teil aus (z.B. `npm run test` oder ein Test-Skript aus `package.json`). Stelle sicher, dass zuvor alle notwendigen Voraussetzungen (wie ein erfolgreicher Build oder eine lokale Testdatenbank, falls benötigt) erfüllt sind.
   * **Shell-Tests:** Falls es Shell-basierte Tests oder Prüfscripte gibt (z.B. `.sh`-Dateien im `tests/`-Verzeichnis oder als Teil des Repository), führe diese aus (z.B. `./run_tests.sh`). Achte darauf, die Ausführungsrechte zu setzen und die Skripte in der vorgesehenen Umgebung laufen zu lassen.
   * **Weitere Tests:** Prüfe, ob es weitere Testarten gibt (z.B. Python-Tests, Integrationstests, End-to-End-Tests) und führe diese ebenfalls aus. Nutze die im Projekt dokumentierten Befehle hierfür (z.B. `cargo test --features integration` oder ähnliche).
   * Protokolliere die Ergebnisse aller Tests und notiere, welche Tests bestanden wurden und welche (wenn überhaupt) fehlgeschlagen sind.
2. **Grundlegende Tests erstellen (falls keine vorhanden):** Falls das Projekt keine oder nur sehr wenige automatische Tests mitliefert, generiere automatisch einige grundlegende Tests, um die Kernfunktionalitäten abzudecken:

   * **Rust:** Erstelle neue Testfunktionen (mit `#[test]`) in den entsprechenden Modulen oder in einem `tests/`-Verzeichnis. Konzentriere dich auf zentrale Funktionen und Datenstrukturen des Rust-Codes. Beispielsweise kann ein Test prüfen, ob eine Hauptfunktion ohne Fehler durchläuft oder erwartete Rückgabewerte liefert. Verwende `assert!`/`assert_eq!` Makros, um Ergebnisse zu verifizieren.
   * **TypeScript/JavaScript:** Richte ein Testframework ein (falls noch nicht vorhanden, z.B. Jest oder Mocha) und schreibe einfache Tests für wichtige Module oder Klassen. Importiere die Hauptmodule und prüfe grundlegende Behavior, z.B. ob eine Funktion mit bestimmten Eingaben den erwarteten Wert zurückgibt oder Ausnahmen wirft.
   * Führe die neu erstellten Tests aus, um zu sehen, ob sie erfolgreich sind. Falls diese Tests Fehler im Code aufdecken (d.h. wenn neue Tests fehlschlagen), behandle diese Fehler entsprechend (siehe Fehlerbehandlung unten).
   * Integriere die neuen Tests ins Repository (commit auf einem neuen Branch und eröffne einen Pull Request dafür), sodass das Projekt künftig über ein Mindestmaß an Testabdeckung verfügt.
3. **Umgang mit Testfehlern:** Wenn irgendein Test (bestehender oder neu erstellter) fehlschlägt oder während der Testausführung Fehler auftreten:

   * Notiere den spezifischen fehlgeschlagenen Test und die aufgetretene Fehlermeldung.
   * Fahre dann mit der **Fehlerbehandlung** (nächster Abschnitt) fort, um den Fehler zu dokumentieren und – falls möglich – zu beheben.
   * Nachdem ein Fehler behoben wurde, führe die betreffenden Tests erneut aus, um zu verifizieren, dass das Problem gelöst ist und keine neuen Fehler entstanden sind.

## Fehlerbehandlung: Issues & Pull Requests

* **Issue-Erstellung pro Fehler:** Für **jeden** festgestellten Fehler oder Build/Test-Fehlschlag erstellt der Agent ein separates GitHub-Issue im Repository:

  * Formuliere einen aussagekräftigen Titel (z.B. *"Build-Fehler: Bibliothek X nicht gefunden"* oder *"Test Y schlägt bei Eingabe Z fehl"*).
  * Beschreibe im Issue kurz, was gemacht wurde und was dabei schiefging (z.B. *"Beim Ausführen von `cargo build` tritt folgender Fehler auf: ..."*). Füge relevante Ausschnitte der Fehlermeldung oder des Logs bei, damit das Problem nachvollziehbar ist.
  * Wenn bereits ersichtlich, nenne mögliche Ursachen oder Lösungsansätze. (Z.B. *"Vermutlich fehlt die Abhängigkeit X in der Cargo.toml"* oder *"Die Funktion Y behandelt den Fall Z nicht korrekt"*.)
* **Behebung durchführen (Bugfix):** Ist die Ursache eines Fehlers bekannt oder der Fix trivial, soll der Agent den Fehler direkt beheben:

  * Nimm die notwendigen Code- oder Skriptänderungen vor. Beispiele: Fehlende Abhängigkeit in eine Konfigurationsdatei eintragen, einen Tippfehler im Code korrigieren, einen falschen Dateipfad anpassen.
  * Teste die Änderung sofort lokal, indem du den vorher fehlgeschlagenen Schritt erneut ausführst (z.B. erneut `cargo build` oder den entsprechenden Test). Verifiziere, dass der Fehler dadurch verschwunden ist **und** keine neuen Probleme entstehen.
  * Checke die Änderung in einem neuen Git-Branch ein. Verwende eine prägnante Commit Message, die das Problem und die Lösung beschreibt. Referenziere dabei das zugehörige Issue (z.B. *"Fix #123: Füge fehlende Abhängigkeit X hinzu, um Build zu reparieren"*).
  * Eröffne einen Pull Request für den Fix. Im PR-Beschreibungstext verweise auf das Issue (z.B. *"Fixes #123"*), damit das Issue automatisch geschlossen wird, sobald der PR gemergt ist.
* **Schrittweise Abarbeitung:** Setze den Prozess fort, bis alle bekannten Probleme behoben sind:

  * Starte nach jedem Fix wieder den entsprechenden Build- oder Testlauf, um festzustellen, ob weitere Fehler auftreten.
  * Behandle neue Fehler wiederum mit einem eigenen Issue und (falls möglich) einem eigenen Fix/PR, wie oben beschrieben.
  * Achte darauf, immer nur **eine Problemursache pro Issue/PR** zu adressieren. So bleibt die Fehlerverfolgung übersichtlich und jeder Fix ist klar einem Problem zugeordnet.
* **Fehlertoleranz sicherstellen:** Brich den Gesamtprozess **nicht** beim ersten Fehler ab. Der Agent soll im selben Durchlauf so viele Fehler wie möglich identifizieren und einzeln abarbeiten. Dadurch können auch versteckte oder aufeinanderfolgende Probleme aufgedeckt werden. Der Prozess gilt erst als erfolgreich, wenn das Projekt komplett baut und alle Tests ohne Fehler durchlaufen.

## Wiederholbarkeit & Abschluss

* **Reproduzierbare Abläufe:** Stelle sicher, dass die oben beschriebenen Schritte jederzeit in einer frischen Umgebung wiederholbar sind.

  * Automatisiere die Einrichtung und den Build/Test-Prozess so weit wie möglich (z.B. via Skripte, Docker-Container oder CI-Pipeline), damit ein neuer Durchlauf auf einem sauberen System dieselben Ergebnisse liefert.
  * Dokumentiere alle nötigen Schritte und Abhängigkeiten eindeutig (ggf. in der README oder ergänzenden Entwickler-Dokumentation), sodass auch Entwickler ohne Codex-Unterstützung das Projekt nach dieser Anleitung erfolgreich aufsetzen können.
* **Aufräumen und Zusammenfassung:** Nachdem das Projekt erfolgreich gebaut und getestet wurde (alle Tests grün, keine offenen Fehler):

  * Entferne etwaige temporäre Dateien oder Build-Artefakte, sofern sie nicht ins Repository gehören (z.B. mittels `cargo clean`, Löschen von generierten Dateien oder dem `node_modules`-Ordner, falls diese nicht mehr benötigt werden).
  * Stelle sicher, dass alle erstellten Issues und PRs sauber miteinander verlinkt sind und ausreichend Informationen enthalten. Schließe Issues durch Verweis in den PRs oder kommentiere, wenn weitere Schritte nötig sind.
  * Aktualisiere bei Bedarf Projektdokumentation (z.B. die README.md) mit Erkenntnissen aus diesem Durchlauf – etwa korrigierte Installationsanleitungen, neu hinzugefügte Tests oder angepasste Build-Schritte, um zukünftige Installationen zu erleichtern.
  * **Erfolgskriterium:** Der Vorgang ist abgeschlossen, wenn das Projekt ohne Fehler lokal installiert ist, der Build erfolgreich durchläuft und sämtliche Tests bestanden sind. Alle identifizierten Probleme sollten entweder behoben (durch gemergte PRs) oder zumindest als GitHub-Issues festgehalten sein. Der Codex-Agent hat dann seine Aufgabe erfüllt und der Projektstatus ist nun konsistent und überprüft.

## Development Phases
For automated iterations follow these stages:
1. Initial analysis
2. Module validation
3. Component completion
4. Test strategy implementation
5. CI/CD automation
6. Refactoring and cleanup
7. Feature expansion

