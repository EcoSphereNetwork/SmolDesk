---
title: Remote-Verbindung
description: Steuere den Bildschirm über eine WebRTC-Verbindung mit Maus und Tastatur.
---

## Funktion & Zweck
SmolDesk ermöglicht die Fernsteuerung eines Linux-Hosts. Über eine Peer-to-Peer-Verbindung werden Bildschirm und Eingaben in Echtzeit übertragen.

## UX-Verhalten / Interface
- Verbindung per Raum-Code im **View**-Tab aufbauen
- **Input On/Off** zum Pausieren der Steuerung
- Vollbildmodus via `F11` oder Icon
- Mobile unterstützt Touch-Gesten für Klicks und Scrollen

Weitere Details unter [../usage/viewer.md](../usage/viewer.md).

## Technische Architektur / Datenfluss
- `ConnectionManager` stellt die WebRTC-Verbindung her und leitet Streams an `RemoteScreen` weiter
- Eingaben werden über denselben Kanal zurückgesendet
- Architekturüberblick siehe [../docs/architecture.md](../docs/architecture.md)

## Sicherheit & Einschränkungen
- Authentifizierung mit JWT-Token
- Datenkanal- und Transportverschlüsselung per DTLS/AES
- Verbindung kann bei schwacher Netzqualität abbrechen

## Verweise
- Komponenten: [ConnectionManager](../docs/components/ConnectionManager.md), [RemoteScreen](../docs/components/RemoteScreen.md)
- Entwicklertipps unter [../development/dev-tools.md](../development/dev-tools.md)
