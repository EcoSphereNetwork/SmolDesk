---
title: HTTP Endpoints
description: Signaling-Server und sonstige HTTP-Schnittstellen
---

Der mitgelieferte Signaling-Server stellt nur einen sehr kleinen HTTP-Teil bereit. Hauptkommunikationsweg ist eine WebSocket-Verbindung, die ebenfalls auf Port `3000` läuft.

## Endpoints

| Methode | Pfad | Beschreibung |
|---------|------|--------------|
| `GET` | `/` | Gibt den Text `SmolDesk Signaling Server` zurück und dient als einfache Funktionsprobe. |

Weitere REST- oder JSON-APIs sind aktuell nicht implementiert. Für die Peer-Verbindung wird stattdessen das unten beschriebene WebSocket-Protokoll genutzt.

Siehe auch [Protocols](protocols.md) für Details zu den Nachrichtentypen.
