---
title: Sicherheit
description: Übersicht der wichtigsten Sicherheitsmechanismen von SmolDesk.
---

## Funktion & Zweck
SmolDesk schützt Verbindungen und Daten durch mehrere Ebenen.

## UX-Verhalten / Interface
- Verbindungen zu geschützten Räumen erfordern ein Passwort
- Sicherheitsoptionen können im Einstellungsdialog konfiguriert werden

Weitere Hinweise zur Nutzung im [Viewer Guide](../usage/viewer.md).

## Technische Architektur / Datenfluss
- DTLS 1.2 sichert Transportebene, Datenkanäle werden zusätzlich per AES verschlüsselt
- JWT-Authentifizierung und optionaler HMAC-Schutz für Nachrichten
- Dateitransfers erhalten SHA256-Checksummen

## Sicherheit & Einschränkungen
- Minimal notwendige App-Berechtigungen
- UFW- und AppArmor-Beispiele siehe [../development/security.md](../development/security.md)
- Schwachstellen können vertraulich gemeldet werden

## Verweise
- Pentest-Skript unter [../../security/pentest/automated_security_scan.py](../../security/pentest/automated_security_scan.py)
