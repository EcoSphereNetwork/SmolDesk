---
title: SmolDesk Mobile Signaling
description: ""
---
# SmolDesk Mobile Signaling

Dieses Dokument beschreibt die Nachrichtenformate und den Ablauf der WebSocket-Kommunikation zwischen der Mobile-App und dem SmolDesk Signaling-Server.

## Verbindungsablauf
1. Die App stellt eine WebSocket-Verbindung zu `wss://<server>` her.
2. Nach dem Verbindungsaufbau sendet der Server eine `welcome`-Nachricht mit `clientId` und `token`.
3. Anschließend tritt der Client mit `join-room` einem Raum bei und wartet auf ein `offer` des Hosts.
4. Nach dem SDP-Austausch werden ICE-Kandidaten mittels `ice-candidate` ausgetauscht.

## Nachrichten vom Client
- `create-room` { `roomId?`, `settings?` }
- `join-room` { `roomId` }
- `leave-room`
- `offer` { `targetId`, `offer` }
- `answer` { `targetId`, `answer` }
- `ice-candidate` { `targetId`, `candidate` }
- `ping`

## Nachrichten vom Server
- `welcome` { `clientId`, `token` }
- `room-created` { `roomId` }
- `room-joined` { `roomId`, `peers`, `settings` }
- `room-left` { `roomId` }
- `peer-joined` { `peerId` }
- `peer-left` { `peerId` }
- `peer-disconnected` { `peerId` }
- `offer` { `peerId`, `offer` }
- `answer` { `peerId`, `answer` }
- `ice-candidate` { `peerId`, `candidate` }
- `error` { `message` }
- `pong`
