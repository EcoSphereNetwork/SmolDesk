---
title: Protocols
description: WebSocket-Nachrichten des Signaling-Servers
---

Die Verbindung zwischen den Peers wird über einen WebSocket-Server vermittelt. Nach dem Verbindungsaufbau tauschen Client und Server JSON-Nachrichten mit einem `type` Feld aus.

## Nachrichten vom Server

| Typ | Inhalt |
|-----|--------|
| `welcome` | Enthält `clientId` und ein zufällig generiertes `token`. |
| `room-created` | Bestätigung nach `create-room` mit der vergebenen `roomId`. |
| `room-joined` | Bestätigung nach `join-room` sowie Liste vorhandener Peers. |
| `peer-joined` | Ein neuer Peer ist dem Raum beigetreten. |
| `peer-left` | Ein Peer hat den Raum verlassen. |
| `peer-disconnected` | Verbindung zu einem Peer ging verloren. |
| `room-left` | Bestätigung nach Verlassen eines Raums. |
| `ice-candidate`, `offer`, `answer` | Weiterleitung der jeweiligen WebRTC-Daten. |
| `pong` | Antwort auf einen Ping des Clients. |

## Nachrichten vom Client

| Typ | Beschreibung |
|-----|-------------|
| `create-room` | Legt einen neuen Raum an. Optional kann eine `roomId` übermittelt werden. |
| `join-room` | Tritt einem bestehenden Raum bei. |
| `leave-room` | Verlasse den aktuellen Raum. |
| `ice-candidate` | Übermittelt ICE-Kandidaten an einen Peer. |
| `offer` | WebRTC Offer an einen Ziel-Peer. |
| `answer` | WebRTC Answer an einen Ziel-Peer. |
| `ping` | Lebenszeichen zur Verbindungsüberwachung. |

Das Protokoll ist textbasiert (JSON) und sieht keine Authentifizierung vor. Sicherheitsfunktionen befinden sich in der [Security](../features/security.md) Komponente.
