---
title: OAuth2-Flow
description: ''
---
> ⚠️ Diese Datei wurde archiviert. Die aktuelle Version befindet sich unter `docs/development/security.md`

Dieser Abschnitt dokumentiert die Sicherheitsarchitektur der Mobile-App ab Phase 4.

## OAuth2-Flow
Die App verwendet einen OAuth2-PKCE-Flow. Die Konfiguration befindet sich in `src/config.ts`.
Nach erfolgreicher Autorisierung wird das Access Token sicher per `react-native-keychain` gespeichert.

## Tokenweitergabe an Signaling
Beim Aufbau der WebSocket-Verbindung sendet der Client eine Nachricht
`{ type: 'auth', token: <ACCESS_TOKEN> }`. Erst nach Antwort `authorized: true`
wird der Raum betreten.

## HMAC-Signierung
Optional kann eine SHA-256-HMAC über kritische Nachrichten gelegt werden. Die
Aktivierung und der Schlüssel sind ebenfalls in `config.ts` hinterlegt. Aktuell
werden `join-room`, `input_event` und `clipboard_event` damit signiert.

## AES-Datenkanalverschlüsselung
Textbasierte Daten über den WebRTC-Datenkanal werden mit AES (256 Bit) in CBC-
Modus verschlüsselt. Die IV ist 16 Byte lang und wird Base64-kodiert vor das
Ciphertext gehängt (`iv:cipher`). Der Schlüssel wird aus dem OAuth2-Login über
PBKDF2 abgeleitet.
