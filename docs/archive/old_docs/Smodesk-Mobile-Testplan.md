⚠️ Diese Datei wurde archiviert. Die aktuelle Version befindet sich unter `docs/development/testplan.md`


# Testplan für SmolDesk Mobile

1. **Unit Tests** mit Jest für Hilfsfunktionen und Services
2. **Manuelle Tests** der Verbindung zu einem Linux-Host
3. **Gesten-Tests**: Pinch-Zoom, Drag und Rechtsklick per Two-Finger-Tap
4. **Clipboard-Synchronisation** zwischen Host und Phone
5. **Login/Logout-Tests**: OAuth2-Flow inklusive Refresh Token
6. **Token-Validierung** beim Signaling
7. **Datenkanalverschlüsselung** testen (verschlüsselt vs. unverschlüsselt)
8. **UI-Tests** optional mit Detox
9. **Datei senden/empfangen** (verschlüsselt & unverschlüsselt)
10. **Monitorwechsel** mit mehreren angeschlossenen Bildschirmen
11. **Dark/Light Mode** Umschalten während der Session
12. **Responsive Layout** auf kleinen Phones und Tablets
13. **Hintergrund/Rückkehr**: Session bleibt stabil, Reconnect bei Bedarf
