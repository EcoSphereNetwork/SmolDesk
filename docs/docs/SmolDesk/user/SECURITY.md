# SmolDesk Sicherheitsrichtlinien

## Sicherheitsarchitektur

### 1. Verbindungssicherheit
- **Ende-zu-Ende-Verschlüsselung**: Alle WebRTC-Verbindungen verwenden DTLS 1.2
- **Authentifizierung**: JWT-Token mit HMAC-SHA256-Signierung
- **Autorisierung**: Rollenbasierte Zugriffskontrolle mit konfigurierbaren Berechtigungen

### 2. Datenintegrität
- **Message-Signing**: Alle kritischen Nachrichten werden mit HMAC-SHA256 signiert
- **Hash-Verifizierung**: Dateitransfers werden mit SHA256-Hashes verifiziert
- **Replay-Schutz**: Zeitstempel-basierte Validierung verhindert Replay-Attacken

### 3. Netzwerksicherheit
- **STUN/TURN-Sicherheit**: Sichere ICE-Kandidaten-Sammlung
- **NAT-Traversal**: Minimiert Attack-Surface durch direkte P2P-Verbindungen
- **Firewall-freundlich**: Fallback auf TURN-Relay bei restriktiven Firewalls

## Bedrohungsmodell

### Identifizierte Bedrohungen
1. **Unbefugter Zugriff**: Schutz durch Authentifizierung und Autorisierung
2. **Man-in-the-Middle**: Schutz durch Ende-zu-Ende-Verschlüsselung
3. **Denial-of-Service**: Rate-Limiting und Verbindungsgrenzwerte
4. **Datenexfiltration**: Rollenbasierte Berechtigungen für Dateiübertragung
5. **Input-Injection**: Validierung und Sanitization aller Eingaben

### Nicht abgedeckte Bedrohungen
- **Host-System-Kompromittierung**: Schutz außerhalb des Anwendungsbereichs
- **Signaling-Server-Attacken**: Erfordert separate Infrastruktursicherheit
- **Browser-Schwachstellen**: Abhängig von Client-Browser-Sicherheit

## Sicherheitskonfiguration

### Produktionseinstellungen
```json
{
  "connectionMode": "Private",
  "sessionTimeoutMinutes": 30,
  "useEncryption": true,
  "maxFailedAttempts": 3,
  "enableSecureMode": true,
  "clipboardSyncFilter": {
    "minTextLength": 1,
    "maxTextLength": 10485760,
    "blockedMimeTypes": ["application/octet-stream", "application/x-executable"],
    "blockedFileExtensions": ["exe", "bat", "cmd", "com", "scr", "dll"]
  },
  "fileTransferConfig": {
    "maxFileSize": 104857600,
    "encryptionEnabled": true,
    "allowedMimeTypes": ["text/*", "image/*", "application/pdf"]
  }
}
```

### Härtungsmaßnahmen

#### Systemebene
```bash
# Firewall-Konfiguration (UFW)
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 3000/tcp    # Signaling-Server (falls lokal)
sudo ufw allow 3478/udp   # STUN/TURN
sudo ufw enable

# AppArmor-Profile (falls verfügbar)
sudo cp security/apparmor/smoldesk /etc/apparmor.d/
sudo apparmor_parser -r /etc/apparmor.d/smoldesk

# Systemd-Sicherheit für Signaling-Server
sudo systemctl edit smoldesk-signaling --force
```

Inhalt der Systemd-Override-Datei:
```ini
[Service]
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/smoldesk
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
User=smoldesk
Group=smoldesk
```

#### Anwendungsebene
```bash
# Sichere Umgebungsvariablen
export SMOLDESK_SECRET_KEY="$(openssl rand -hex 32)"
export SMOLDESK_LOG_LEVEL="warn"
export RUST_LOG="smoldesk=info"

# Memory-Limits
ulimit -v 2097152  # 2GB virtuelle Memory
ulimit -m 1048576  # 1GB physische Memory
```

## Vulnerability Reporting

### Verantwortliche Offenlegung
Wenn Sie eine Sicherheitslücke finden:

1. **Nicht öffentlich melden**: Verwenden Sie nicht GitHub Issues
2. **E-Mail senden**: security@smoldesk.example
3. **Verschlüsselung**: Verwenden Sie unseren PGP-Schlüssel (siehe unten)
4. **Details bereitstellen**: Reproduktionsschritte, Impact, vorgeschlagene Fixes

### PGP-Schlüssel
```
-----BEGIN PGP PUBLIC KEY BLOCK-----
[PGP-Schlüssel für security@smoldesk.example]
-----END PGP PUBLIC KEY BLOCK-----
```

### Belohnungsprogramm
- **Kritische Schwachstellen**: €500-1000
- **Hohe Schwachstellen**: €200-500
- **Mittlere Schwachstellen**: €50-200
- **Niedrige Schwachstellen**: €10-50

### Ausschlüsse
- DoS-Attacken auf öffentliche Services
- Social-Engineering-Attacken
- Schwachstellen in Drittanbieter-Abhängigkeiten
- Self-XSS ohne weitere Impact

## Compliance

### Standards
- **ISO 27001**: Informationssicherheits-Management
- **NIST Cybersecurity Framework**: Identify, Protect, Detect, Respond, Recover
- **GDPR**: Datenschutz-Grundverordnung (für EU-Nutzer)

### Zertifizierungen
- Security-Audit durch [Audit-Firma]
- Penetrationstests durch [PenTest-Firma]
- Code-Review durch [Security-Experten]

## Incident Response

### Prozess
1. **Erkennung**: Monitoring und Alerting
2. **Bewertung**: Severity und Impact-Analyse
3. **Eindämmung**: Sofortmaßnahmen
4. **Beseitigung**: Root-Cause-Analysis und Fix
5. **Wiederherstellung**: Service-Wiederherstellung
6. **Lessons Learned**: Verbesserung der Sicherheitsmaßnahmen

### Kontakte
- **Security Team**: security@smoldesk.example
- **Incident Response**: incident@smoldesk.example
- **Notfall (24/7)**: +49-XXX-XXXXXX
```
