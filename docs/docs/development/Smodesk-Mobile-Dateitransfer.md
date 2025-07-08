---
title: SmolDesk Mobile Dateitransfer
description: 
---
# SmolDesk Mobile Dateitransfer

Dieses Dokument beschreibt den Aufbau der Dateiübertragung zwischen Mobilgerät und Remote-PC.

## Nachrichtenformat
- **file_header**: enthält `id`, `name`, `mime`, `size` (unkodiert)
- **file_chunk**: Stück des Datei-Inhalts als Base64-String (optionale Verschlüsselung)
- **file_end**: Abschluss der Übertragung für eine `id`

Der Header wird immer unverschlüsselt gesendet, damit der Empfänger Dateiname und Typ erkennen kann. Die Nutzdaten können bei aktivierter Datenkanalverschlüsselung geschützt werden.

## Übertragungslogik
1. Im Viewer wird „Datei senden“ gewählt und eine Datei mit `react-native-document-picker` ausgesucht.
2. Die Datei wird in 64 KB Blöcken mit `react-native-fs` gelesen und über den WebRTC-Datenkanal geschickt.
3. Der Empfänger rekonstruiert die Datei im Speicher und speichert sie im Download-Verzeichnis ab.
4. Bei einer Unterbrechung kann der Transfer erneut gestartet werden; bereits empfangene Blöcke werden verworfen.

Maximalgröße pro Datei ist derzeit auf wenige Hundert MB begrenzt, abhängig von Gerät und Verbindung. Die Chunk-Größe kann bei Bedarf angepasst werden.

## Sicherheit
- Header bleibt im Klartext, der Payload (Chunks) wird bei gesetztem Schlüssel verschlüsselt.
- Der Nutzer wird bei großen Dateien gewarnt und kann den Zielpfad einsehen.

## UI-Fluss
1. Start der Übertragung durch Button im Viewer.
2. Fortschritt wird während des Sendens angezeigt.
3. Empfangene Dateien erscheinen im Download-Ordner und können über die Share-Funktion geöffnet werden.
