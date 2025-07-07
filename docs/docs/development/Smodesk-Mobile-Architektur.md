# SmolDesk Mobile Architektur

Die Mobile-App verwendet React Native und verbindet sich über WebRTC mit dem SmolDesk Signaling-Server. Die App bildet den Remote-Bildschirm als Videostream ab und sendet Eingaben über Datenkanäle.

## Hauptkomponenten
- **Signaling Service**: WebSocket Verbindung zum bestehenden Node.js Server
- **WebRTC Service**: Aufbau der PeerConnection, Empfang des Video-Streams
- **UI**: React-Native-Komponenten zur Anzeige des Streams und für Steuerelemente
