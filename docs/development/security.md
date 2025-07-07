---
title: Sicherheit
description: Überblick zu Sicherheitsfunktionen und Härtungsmaßnahmen
---

Dieser Abschnitt fasst die wichtigsten Sicherheitsaspekte von SmolDesk zusammen. Weitere Nutzungshinweise findest du im [Viewer Guide](../usage/viewer.md).

## Verbindungssicherheit

- Ende-zu-Ende-Verschlüsselung über DTLS 1.2
- Authentifizierung mittels JWT‑Token
- Optionaler HMAC‑Schutz für Nachrichten
- AES‑Verschlüsselung des WebRTC‑Datenkanals

## Datenintegrität und Berechtigungen

- Dateitransfers werden per SHA256 verifiziert
- Zugriffsbeschränkungen über Rollen und Berechtigungen
- Minimal notwendige App‑Permissions (Netzwerk, optional Speicherzugriff)

## Härtungsmaßnahmen

```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 3000/tcp   # Signaling Server
sudo ufw allow 3478/udp  # STUN/TURN
```

AppArmor-Profile und systemd‑Sandboxing findest du im Verzeichnis `security/`.

## Meldung von Schwachstellen

Bitte melde Sicherheitslücken vertraulich per E‑Mail an `security@smoldesk.example`. Der PGP‑Schlüssel befindet sich im Archiv.
