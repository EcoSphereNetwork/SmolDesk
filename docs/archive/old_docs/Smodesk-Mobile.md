---
title: Smodesk-Mobile
description: ''
---
> ⚠️ Diese Datei wurde archiviert. Die aktuelle Version befindet sich unter `docs/development/setup-android.md`

Ziel: Entwicklung einer plattformübergreifenden React Native App (Android zuerst, danach iOS) für SmolDesk, die alle Funktionen der Desktop-Version unterstützt. SmolDesk ist eine WebRTC-basierte Remote-Desktop-Lösung für Linux mit Fokus auf niedriger Latenz und Sicherheit. Die Mobile-App richtet sich an Endnutzer und soll mobile-first optimiert sein (intuitive Touch-Bedienung, responsives UI etc.). Im Folgenden ein detaillierter Schritt-für-Schritt-Plan:

Schritt 1: Projektinitialisierung und Architektur festlegen

Zunächst wird die Basis für die Mobile-App geschaffen. Dies umfasst die Auswahl des Tech-Stacks, das Aufsetzen des Projekts und das Sicherstellen der Kompatibilität mit der bestehenden SmolDesk-Infrastruktur.

Aufgaben in Schritt 1:

React Native einrichten: Initialisiere ein neues React-Native-Projekt (z.B. mit npx react-native init SmolDeskMobile). Richte die Entwicklungsumgebung für Android ein (Android Studio, SDK) und später für iOS (Xcode). Lege eine modulare Projektstruktur an (z.B. Separate Verzeichnisse für Components, Screens, Services etc.).

Abhängigkeiten hinzufügen: Installiere zentrale Bibliotheken: insbesondere eine WebRTC-Bibliothek für React Native (z.B. react-native-webrtc) zur Peer-to-Peer Kommunikation. Diese ermöglicht der App, Video-/Audio-Streams über WebRTC zu empfangen und zu senden – fundamental für SmolDesk’s Streaming-Architektur. Weitere Abhängigkeiten: z.B. @react-navigation (für App-Navigation), UI-Library (etwa React Native Paper) und Module für spezielle Funktionen (Clipboard, Dateizugriff).

Architektur abstimmen: Die SmolDesk-Desktop-App besteht aus einem Rust/Tauri-Backend (für Bildschirmaufnahme und Eingabe-Injection) und einem React-Frontend sowie einem Node.js Signaling-Server für WebRTC. Plane die Mobile-App als Client, der sich mit dem bestehenden Signaling-Server verbindet und als Viewer gegenüber dem Linux-Host fungiert. Definiere Schnittstellen: z.B. URL/Adresse des Signaling-Servers konfigurierbar machen (standardmäßig z.B. wss://&lt;server-url&gt;). Die App benötigt kein eigenes Backend, sondern integriert in die vorhandene P2P-Architektur.

Code-Sharing prüfen: Analysiere, ob bestehende Frontend-Logik wiederverwendbar ist. SmolDesk hat z.B. einen React-Hook useSmolDesk und modulare Frontend-Komponenten für Verbindung und Streaming. Gegebenenfalls können Protokoll-Details (z.B. WebRTC-Nachrichtenformate, Signaling-Nachrichten) in die RN-App übernommen werden, um Konsistenz zu gewährleisten. Dokumentiere diese Formate aus dem bestehenden Code.


Schritt 2: Grundlegende Verbindung (Signaling) & WebRTC-Integration

In diesem Schritt steht die Kernfunktion – die Verbindung zum Remote-Desktop – im Vordergrund. Die App muss dem Benutzer ermöglichen, eine Session aufzubauen, was das Zusammenspiel von Signaling-Server und WebRTC-Peer-Verbindung erfordert.

Aufgaben in Schritt 2:

UI für Verbindungsaufbau: Implementiere einen Startbildschirm mit Eingabemöglichkeit für Verbindungsdaten. Beispielsweise können Benutzer eine Sitzungs-ID oder einen Code eingeben, um sich mit dem Host zu verbinden (analog zur Desktop-App, die Room Creation/Joining unterstützt). Alternativ könnte ein QR-Code-Scan in Erwägung gezogen werden (Host generiert QR mit Verbindungsdetails).

Signaling via WebSocket: Integriere die Verbindung zum SmolDesk Signaling-Server (Node.js). Verwende z.B. react-native-websocket oder die WebSocket-API von React Native, um eine Verbindung aufzubauen. Implementiere das Signaling-Protokoll identisch zur Desktop-Variante: Austausch von SDP-Angebot/Antwort und ICE-Kandidaten über den Server, um die WebRTC-Verbindung zwischen Mobilgerät und Host auszuhandeln. Stelle sicher, dass die Mobile-App sich als „Viewer“ korrekt beim Server registriert und einem Raum beitritt.

WebRTC PeerConnection aufbauen: Nutze react-native-webrtc, um einen PeerConnection zu erstellen, sobald Signaling abgeschlossen ist. Führe peerConnection.setRemoteDescription mit dem vom Host erhaltenen SDP aus und sende dein Angebot/Antwort entsprechend zurück. Konfiguriere die ICE-Server (STUN/TURN) genauso wie in SmolDesk, sodass auch Verbindungen hinter NAT reibungslos funktionieren.

Video-Stream empfangen: Füge einen Listener hinzu, der auf peerConnection.ontrack reagiert. Der Host sendet den Bildschirm als Video-Stream; die Mobile-App sollte diesen Stream empfangen und anzeigen. Verwende dazu in RN den bereitgestellten `<RTCView />` (oder ähnliches) Component, um den eingehenden Videostream darzustellen. Sorge dafür, dass der Videostream im Vollbild oder an den Bildschirm des Geräts angepasst angezeigt wird (mit Zoom-/Scroll-Möglichkeit für große Desktops). Hinweis: SmolDesk unterstützt verschiedene Video-Codecs (H.264, VP8/VP9, AV1) – stelle sicher, dass die mobilen WebRTC-Bibliotheken diese decodieren können (moderne Geräte unterstützen H.264 hardwarebeschleunigt).

Audio-Stream (optional): Falls SmolDesk Audio-Streaming implementiert oder plant, konfiguriere auch einen Audio-Track. Erlaube der App, den Audio-Track des Remote-PCs wiederzugeben (unter Beachtung der Lautstärkeeinstellungen des Phones). Prüfe, ob für Audio ein Nutzerbefehl nötig ist (manche Plattformen verlangen Interaktion, bevor Audio autoplay funktioniert).


Nach diesem Schritt sollte die App in der Lage sein, sich mit einem Host zu verbinden und dessen Bildschirm in Echtzeit anzuzeigen. Die Latenz sollte dabei möglichst gering sein (SmolDesk strebt <200 ms an); teste dies in einem lokalen Netzwerk und optimiere ggf. P2P-Einstellungen.

Schritt 3: Remote-Bildschirm anzeigen & mobile UI optimieren

Nun steht die Benutzeroberfläche und Nutzererfahrung im Fokus, insbesondere für das Betrachten des Remote-Desktops auf einem kleinen Bildschirm. Hier geht es darum, die Anzeige anzupassen und mobile Interaktionen zu ermöglichen.

Aufgaben in Schritt 3:

Bildschirm-Viewer UI: Entwickle einen dedizierten Screen in der App (z.B. RemoteDesktopScreen), der den laufenden Videostream anzeigt. Ergänze Bedienelemente wie z.B. eine Leiste oder halbtransparente Buttons für essenzielle Aktionen (Verbindung trennen, Tastatur einblenden, evtl. Monitor wechseln). Stelle sicher, dass diese Controls die Sicht auf den Remote-Bildschirm nicht dauerhaft verdecken (einblendbar per Tap).

Skalierung und Rotation: Implementiere Zoom- und Scroll-Funktionen für die Bildschirmansicht. Da Desktop-Auflösungen größer als Handybildschirme sind, soll der Nutzer via Pinch-to-Zoom in den Remote-Screen hineinzoomen können. Erlaube Draggen mit zwei Fingern, um bei vergrößerter Ansicht zu verschieben. Unterstütze außerdem Landscape-Modus – viele Benutzer werden ihr Gerät quer halten, um mehr vom Remote-Bild zu sehen. Teste die Darstellung bei verschiedenen Displaygrößen und Ausrichtungen.

Verbindungsstatus-Anzeige: Integriere visuelles Feedback für die Verbindung. Beispielsweise ein Status-Indicator oder Toast-Meldungen („Verbinde...“, „Verbunden“, „Verbindung verloren – Reconnect...“). SmolDesk enthält bereits Monitoring für Verbindungsqualität – eine vereinfachte Anzeige der Latenz oder Qualität (z.B. Balken) könnte dem User helfen.

Mobil-optimierte Navigation: Falls die App mehrere Bildschirme hat (z.B. Start/Connect, Settings, Viewer), setze eine Navigation ein, die Touch-optimiert ist. Z.B. eine Bottom-Tab-Navigation für Hauptbereiche (falls sinnvoll), oder modale Popups für Einstellungen während einer Session. Der Fokus der App liegt aber vermutlich hauptsächlich auf dem Viewer, sodass die Navigation einfach gehalten werden kann.


Schritt 4: Eingabesteuerung (Maus & Tastatur) via Touch umsetzen

Ein Kernfeature von SmolDesk ist die Fernsteuerung der Maus und Tastatur des entfernten Linux-PCs. Die Mobile-App muss Touch-Ereignisse in Mausbewegungen/-klicks und Tastatureingaben umsetzen, damit der Nutzer den Remote-Desktop voll steuern kann.

Aufgaben in Schritt 4:

Touch -> Maus Mapping: Entwickle ein System, um Touch-Gesten in Maussteuerung umzusetzen. Übliche Ansätze: Direktes Tap auf den Remote-Bildschirm erzeugt einen Mausklick an entsprechender Position, Langdruck könnte einen Rechtsklick senden. Ein einzelner Finger-Drag ohne Zoom (bzw. im speziellen Steuerungsmodus) bewegt die Remote-Maus (relative Bewegung). Im Vergleich zum Desktop-Client (der physische Mausbewegungen hat) muss auf Mobile evtl. zwischen Zeiger-Modus und Scroll-Modus umgeschaltet werden. Implementiere ggf. einen Toggle-Button: z.B. „Mauszeiger bewegen“ vs. „Bild verschieben“, um Konflikte zwischen Scrollen des Viewers und Bewegen des Remote-Cursors zu vermeiden.

Multi-Touch Gesten: Nutze zwei-Finger-Gesten für sekundäre Aktionen: z.B. Zwei-Finger-Tap = Rechtsklick (Alternative zu Langdruck), Zwei-Finger-Drag = Scrollrad (für vertikales Scrollen auf dem Remote-PC). Diese Gesten verbessern die Nutzbarkeit, da typische Remote-Desktop-Apps ähnliche Bedienkonzepte nutzen.

Tastatur-Eingabe: Implementiere eine Möglichkeit, Texteingaben an den Remote-PC zu senden. Biete im UI einen Tastatur Button, der die native Bildschirmtastatur öffnet. Erfasse die Eingaben und sende die Tastendrücke als Keyboard-Events über die WebRTC-Datenkanäle an den Host. SmolDesk unterstützt bereits Tastatur-Forwarding inkl. Sondertasten – stelle sicher, dass auch Modifikator-Tasten wie Strg, Alt, Esc, etc. abgesetzt werden können. Dafür könnten in der App spezielle Buttons vorgesehen werden (z.B. ein kleines Overlay mit „Strg“, „Alt“, „Entf“ etc., die der Nutzer drücken kann, um diese an den Host zu senden).

Input-Security beachten: Da der Host Eingaben vom Client entgegennimmt, hat SmolDesk Sicherheitsmaßnahmen (z.B. Validierung der Eingaben). Die Mobile-App sollte diese respektieren – z.B. Limitierung, falls nötig, oder Bestätigung kritischer Aktionen. Ggf. ist das aber hauptsächlich Serverseitig gelöst. Auf Client-Seite sorge dafür, dass Eingaben nicht ungewollt gesendet werden (z.B. kein dauerhaft gedrückter virtueller Button).

Test der Eingabesteuerung: Prüfe die Steuerung gründlich: Bewege den Mauszeiger präzise über Icons, führe Doppelklicks aus, tippe Text in verschiedene Anwendungen. Stelle sicher, dass keine Off-by-Offset-Probleme auftreten (korrekte Positionsberechnung auch bei verschiedenen Zoomstufen und Monitor-Auflösungen). Teste Sondertasten in typischen Szenarien (z.B. STRG+C/STRG+V via mobile Buttons zur Kopie & Paste, siehe Clipboard-Sync im nächsten Schritt).


Schritt 5: Sicherheits- und Authentifizierungsfunktionen integrieren

SmolDesk legt großen Wert auf Security. Die Desktop-App implementiert OAuth2 mit PKCE für Authentifizierung sowie Nachrichten-Signierung (HMAC) und Zugriffssteuerung. Die Mobile-App muss diese Sicherheitsmechanismen ebenfalls unterstützen, um Verbindungen abzusichern.

Aufgaben in Schritt 5:

OAuth2 Login-Flow: Falls SmolDesk einen Login erfordert (z.B. Anmeldung an einem Konto oder an den Host via OAuth2), implementiere diesen Flow in der App. Nutze ggf. ein OAuth2-Client-Paket oder AppAuth. Die App sollte einen Login-Bildschirm bereitstellen, wo der Nutzer auf eine Anmeldeseite geleitet wird (Browser oder WebView) und anschließend mit einem Auth-Code zurückkommt (PKCE-Code-Exchange). Speichere das erhaltene Token sicher (Secure Store) und verwende es für nachfolgende Verbindungsanfragen.

Token-Übergabe an Signaling: Modifiziere den Signaling-Vorgang so, dass das OAuth2-Token bzw. eine signierte Zugriffsberechtigung übermittelt wird. SmolDesk generiert signierte Raum-IDs und erwartet womöglich eine HMAC-Signatur oder Token zur Verifizierung. Stelle sicher, dass die Mobile-App im Signaling-Protokoll das Token mitgibt, damit der Host die Berechtigung prüft. Dieses Vorgehen schützt vor unbefugtem Zugriff – nur authentisierte Clients dürfen die Remote-Session starten.

Datenkanal-Verschlüsselung: WebRTC verschlüsselt Medienströme standardmäßig (SRTP), aber eventuell nutzt SmolDesk zusätzliche Verschlüsselung für Datenkanäle oder Messages. Falls Ende-zu-Ende Verschlüsselung für Steuerdaten vorgesehen ist (z.B. eigenständige AES-Verschlüsselung der Eingabe-Events), implementiere diese analog zur Desktop-App. Übernimm dieselben Algorithmen/Schlüsselableitungen, sodass die Mobile-App nahtlos mit dem Host kommunizieren kann.

App-Sicherheit & Berechtigungen: Achte auch auf mobile-spezifische Sicherheitsaspekte. Fordere nur nötige Berechtigungen an (Netzwerkzugriff ist klar, evtl. Speicherzugriff für Dateitransfer, Kamera nur falls QR-Scan genutzt). Hinterlege Privacy Labels (bei iOS) und erkläre dem Nutzer, wofür Berechtigungen benötigt werden. Implementiere Schutz gegen Man-in-the-Middle im Signaling (z.B. Certificate Pinning für den Signaling-Server, falls dieser übers Internet läuft).


Schritt 6: Erweiterte Funktionen (Clipboard, Dateiübertragung, Multi-Monitor)

Nachdem die Grundfunktionalität steht, werden nun alle übrigen Features ergänzt, damit die Mobile-App funktionsgleich zur Desktop-Version ist. Insbesondere sind dies Zwischenablage-Synchronisation, Dateiübertragungen sowie Multi-Monitor-Unterstützung.

Aufgaben in Schritt 6:

Clipboard-Synchronisation: Implementiere eine bidirektionale Zwischenablage zwischen Mobile und Remote-PC. Nutze die RN-Clipboard API, um Zugriff auf die Geräte-Zwischenablage zu erhalten. Wenn der Nutzer auf dem Phone Text/Bild kopiert, sende dies über den WebRTC-Datenkanal an den Host; umgekehrt empfange vom Host Clipboard-Inhalte und aktualisiere die lokale Zwischenablage. Achte auf Format-Unterstützung: Textübertragung hat Priorität, evtl. lassen sich später Bilder oder HTML-Formate übertragen. Baue in der App ggf. eine Einstellungsoption ein, um Clipboard-Sync aus Datenschutzgründen ein-/auszuschalten.

Dateiübertragung: Füge die Möglichkeit hinzu, Dateien zwischen Mobilgerät und Remote-PC auszutauschen. Auf Mobile-Seite bedeutet das: z.B. einen „Datei senden“-Button im UI, der einen Datei-Browser öffnet (über RN's DocumentPicker oder MediaLibrary). Die ausgewählte Datei wird dann via WebRTC-Datenkanal gestreamt. Implementiere auf Protokollebene eine Dateitransfer-Message mit Meta-Daten (Dateiname, Größe) und segmentiere die Datei in Blöcke, falls nötig. Ebenso unterstütze den Empfang von Dateien vom Host: Frage den Nutzer, wo die empfangene Datei gespeichert werden soll (bei Android z.B. Download-Ordner, bei iOS in einen App-spezifischen Ordner, evtl. mit der Möglichkeit via Share-Sheet zu exportieren). Zeige Fortschrittsbalken während Transfers an und implementiere eine Resume-Logik für unterbrochene Übertragungen (sofern vom Protokoll vorgesehen).

Multi-Monitor Support: SmolDesk unterstützt mehrere Monitore (Erkennung und Auswahl). Die Mobile-App sollte erlauben, zwischen mehreren Remote-Bildschirmen umzuschalten. Falls der Host alle Monitore gleichzeitig streamen kann, könnte der Nutzer einen auswählen; wahrscheinlicher ist, dass nur ein Stream aktiv ist und per Befehl der Monitor gewechselt wird. Implementiere im UI einen Monitor-Wechsel-Dialog – z.B. ein Icon „Monitor“ öffnet eine Liste der erkannten Monitore (mit Bezeichnungen oder Indizes). Bei Auswahl sendet die App einen Steuerbefehl an den Host, der dann den entsprechenden Monitor streamt. Aktualisiere die Anzeige entsprechend. Teste das mit einer Multi-Monitor-Setup am Host.

Weitere Features & Feinschliff: Ergänze sonstige in der Roadmap geplante Funktionen, sofern relevant für den Client. Beispiele: Session Recording (die App könnte eine Aufnahmemöglichkeit anbieten, um den Remote-Screen als Video lokal aufzuzeichnen – optional), Anzeige von Verbindungsstatistiken (Bitrate, FPS, Latenz – zur Diagnose, möglicherweise unter einem Info-Overlay), Themes (Dark/Light-Mode Umschaltung, angepasst an Systemtheme), Internationalisierung der App-Oberfläche (falls geplant, z.B. mehrsprachige UI-Texte analog zur Desktop-Dokumentation).


Schritt 7: Optimierung der Benutzererfahrung (Mobile-First Feinschliff)

In diesem Schritt wird die App hinsichtlich Performance, Usability und Plattform-Konventionen optimiert, um eine hochwertige mobile Nutzererfahrung sicherzustellen.

Aufgaben in Schritt 7:

Performance-Tuning: Überprüfe die Performance der Videowiedergabe und Eingabeverarbeitung auf verschiedenen Geräten. Optimiere die Render- und Decode-Leistung: stelle sicher, dass die Decodierung des Video-Streams möglichst in nativer Hardware erfolgt (was i.d.R. durch WebRTC gegeben ist). Achte auf die CPU-/Speicherauslastung – SmolDesk zielte auf <15% CPU bei 1080p auf modernen Systemen; auf Mobilgeräten sollten ähnliche effiziente Werte angestrebt werden. Wenn nötig, reduziere Standard-Framerate oder -Auflösung für Mobil-Clients oder implementiere adaptive Qualitätsanpassung abhängig von Netzwerk/Geräteperformance (diese Funktionalität ist teils im Backend schon vorhanden).

Mobil-spezifische UI/UX: Führe einen UX-Audit durch: Ist alles gut mit dem Finger bedienbar? Sind Schaltflächen groß genug und an sinnvollen Positionen (z.B. wichtige Controls eher am Bildschirmrand gut erreichbar mit dem Daumen)? Stelle sicher, dass im Hochformat die UI nicht zu gedrängt ist – evtl. bevorzugt man ohnehin Querformat während der Session, aber die App sollte beide Lagen unterstützen. Implementiere Haptisches Feedback für bestimmte Aktionen (z.B. kurze Vibration bei langem Druck = Rechtsklick, um dem Nutzer physisches Feedback zu geben). Nutze Plattform-Konventionen, z.B. auf Android einen Zurück-Button Handler (Verbindung trennen/bestätigen, statt App einfach zu beenden).

Stabilität & Fehlerbehandlung: Verbessere die Robustheit: Fange Netzwerkfehler ab (z.B. Verbindungsverlust zum Signaling-Server oder Peer) und implementiere einen automatischen Reconnect-Mechanismus. Sollte die Verbindung abbrechen, versucht die App, neu zu verbinden, und informiert den Nutzer. Stelle sicher, dass Inkonsistenzen (z.B. kein Stream empfangen) gut gehandhabt werden – etwa durch Anzeige eines Hinweis und Option, die Session neu zu starten. Crash-Logging integrieren (z.B. Sentry oder Firebase Crashlytics), um Fehler im Feld nachverfolgen zu können.


Schritt 8: Testing (umfangreiche Tests auf Android)

Bevor die App veröffentlicht wird, muss sie intensiv getestet werden. Da zunächst Android im Fokus steht, sollten hier auf diversen Geräten Tests durchgeführt werden.

Aufgaben in Schritt 8:

Funktionale Tests: Prüfe jede Funktion manuell: Verbindungsaufbau zu einem Test-Host, Live-Video anzeigen, Maus bewegen, klicken, tippen, Clipboard sync (z.B. Text vom Handy aufs Remote-Terminal einfügen und umgekehrt), Datei senden und empfangen, Monitor wechseln, etc. Teste auch die Sicherheitsflows – z.B. ob nur mit gültigem Token verbunden werden kann, ob falsche Token abgewiesen werden.

Geräte- und Versionsvielfalt: Teste auf verschiedenen Android-Geräten (Smartphones, evtl. Tablets) mit unterschiedlichen Bildschirmgrößen und Android-Versionen. Achte besonders auf ältere Versionen (min. Android 8 oder 9, je nach Festlegung) sowie aktuelle Versionen. Prüfe Leistung auf schwächeren Geräten, um sicherzustellen, dass auch dort zumindest eine reduzierte Qualität noch nutzbar ist.

Netzwerkbedingungen simulieren: Führe Tests bei unterschiedlichen Netzwerkbedingungen durch – z.B. schnelles WLAN vs. mobiles 4G/LTE. Verwende ggf. einen Network Link Conditioner, um höhere Latenzen oder Paketverlust zu simulieren. Beobachte, ob die adaptive Qualitätssteuerung greift und die Verbindung stabil bleibt.

Automatisierte Tests: Wenn möglich, schreibe einige automatisierte End-to-End-Tests für Kernabläufe. Tools wie Detox (für React Native) könnten helfen, zumindest UI-Interaktionen zu testen (z.B. Navigation, Anzeige von Stream-View). Integrationstests für die Signaling- und WebRTC-Logik sind komplexer – hier ggf. auf instrumentierte Tests mit einem Dummy-WebRTC-Endpunkt setzen. Zudem Unit-Tests für Hilfsfunktionen (z.B. Formatierung von Datenpaketen) erstellen.

Feedback einholen: Führe eine geschlossene Beta mit einigen Nutzern durch (Google Play Beta-Channel). Sammle Feedback zur Bedienung (ist das Touch-Steuerungsschema intuitiv? irgendwelche Schwierigkeiten?). Nutze dieses Feedback, um letzte UX-Anpassungen vorzunehmen.


Schritt 9: Launch der Android-App (Veröffentlichung)

Nach erfolgreich bestandenen Tests ist die Android-App bereitzustellen. Dieser Schritt umfasst Vorbereitung der App für den Launch im Google Play Store.

Aufgaben in Schritt 9:

Build & Signatur: Erstelle einen Release-Build der React Native App für Android (.apk oder .aab). Stelle sicher, dass die App mit einem Release Keystore signiert ist. Überprüfe, dass die ProGuard/Minification keine kritischen Teile strippt (insbesondere native WebRTC libraries).

Store-Vorbereitung: Bereite den Play Store Eintrag vor – erstelle aussagekräftige Screenshots (evtl. mit einem verbundenen Session-Bild), eine Beschreibung der App und liste die Hauptfunktionen (Remote-Desktop für Linux, sichere P2P-Verbindung, etc.). Achte darauf, die Alleinstellungsmerkmale hervorzuheben, z.B. WebRTC-P2P für niedrige Latenz, X11/Wayland Unterstützung auf Host-Seite, Open-Source etc.

Berechtigungserklärungen: Falls besondere Berechtigungen verwendet werden, füge im Store-Eintrag Privacy- und Permission-Hinweise hinzu (z.B. Zugriff auf Dateien für Dateitransfer). Stelle sicher, dass die App den Play Store Richtlinien entspricht. Remote-Access-Apps sind in der Regel zulässig, solange sie nicht missbräuchlich sind – betone legitime Nutzung (eigener Desktop-Zugriff).

Release Management: Veröffentliche die App zunächst als Beta/Stage Rollout, um evtl. letzte Probleme abzufangen. Beobachte Crash-Reports und Nutzerbewertungen, um schnell reagieren zu können. Plane Updates ein: z.B. einen Patch nach dem ersten Nutzerfeedback, falls kleinere Korrekturen nötig sind.


Schritt 10: Portierung auf iOS und App Store Launch

Nachdem die Android-Version läuft, wird die App auf iOS portiert und veröffentlicht. Dank React Native ist der Grossteil des Codes wiederverwendbar, jedoch sind einige iOS-spezifische Anpassungen nötig.

Aufgaben in Schritt 10:

iOS-Projekt einrichten: Öffne das React-Native-Projekt in Xcode und stelle sicher, dass alle notwendigen iOS-Abhängigkeiten installiert sind (Pod-Install für CocoaPods im iOS-Verzeichnis ausführen, z.B. für react-native-webrtc). Lege ein App Icon und LaunchScreen für iOS an. Konfiguriere in Xcode die App Capabilities und Entitlements (Netzwerk, evtl. Background Modes falls nötig für VoIP/WebRTC – wobei Remote Desktop vermutlich im Vordergrund läuft).

Plattformspezifische Anpassungen: Überprüfe die UI auf iPhone-Screens (verschiedene Größen, Notch). Passe Layouts an, wo Safe-Areas berücksichtigt werden müssen. Implementiere ggf. iOS-typische UI-Details (z.B. Nutzung der iOS Action Sheet/Stil bei bestimmten Dialogen, um den UX-Konventionen zu entsprechen). Achte darauf, dass der Zurück-Mechanismus unter iOS (Swipe-Geste oder NavigationBar) konsistent gehandhabt wird.

Test auf iOS-Geräten: Führe analog zu Android umfangreiche Tests auf echten iOS-Geräten durch (verschiedene iPhone-Modelle, iPad falls unterstützt). Insbesondere prüfe die WebRTC-Funktion – die Bibliothek sollte auf iOS funktionieren, teste Verbindungsaufbau, Streaming, Performance (iPhones haben i.d.R. starke Hardware, sollten 1080p Streams gut handhaben). Überprüfe Audio (iOS erfordert evtl. eine Erlaubnis in Info.plist, selbst wenn nur Audio empfangen wird – füge bei Bedarf einen NSMicrophoneUsageDescription hinzu, falls nötig für WebRTC).

App Store Vorbereitung: Erstelle ein iOS Release (IPA) mit korrekter Signierung (Distribution-Provisioning-Profile, Apple Developer Account). Erstelle den App Store Connect Eintrag: Beschreibungen (ggf. kann viel von Android übernommen werden), Screenshots (von einem iPhone in Aktion). Beachte, dass Apple strenge Richtlinien hat – betone die Sicherheit (OAuth2, Verschlüsselung) und legitime Nutzung. Reiche die App zur Review ein. Bei etwaigen Nachfragen (z.B. Demonstrations-Login für Review-Team bereitstellen, falls nötig) schnell reagieren.

Cross-Plattform Abgleich: Nach Veröffentlichung auf iOS, stelle sicher, dass die Codebasis konsistent bleibt. Nutzen beide Plattformen denselben Funktionsumfang, und eventuelle Plattformunterschiede werden via Code (conditional rendering/styling) gehandhabt. Plane die zukünftige Entwicklung so, dass neue Features gleichzeitig für beide Plattformen umgesetzt und getestet werden.



---

Abschließend: Mit diesem Vorgehensplan wird die SmolDesk Mobile App Schritt für Schritt entwickelt. Durch die enge Anlehnung an die bestehende SmolDesk-Architektur (WebRTC-P2P, Signaling, Sicherheitsmodelle) und das Beachten aller Hauptfeatures – von Bildschirmübertragung und Eingabefernsteuerung bis zu Dateiablage und Multi-Monitor – entsteht eine vollwertige mobile Anwendung. Eine gründliche Test- und Optimierungsphase stellt sicher, dass die App performant, sicher und benutzerfreundlich ist. So wird SmolDesk künftig nicht nur auf Desktop, sondern auch auf mobilen Geräten ein hochwertiges Remote-Desktop-Erlebnis für Endnutzer bieten können.

Quellen: Die oben genannten Punkte basieren auf der Analyse der SmolDesk-Dokumentation und Codebasis, u.a. des SmolDesk Implementation Plans, des Implementation Status sowie weiteren Entwicklungsdokumenten, welche die Systemarchitektur und geplanten Features detailliert beschreiben. Diese Informationen gewährleisteten, dass der Entwicklungsplan alle relevanten Aspekte der SmolDesk-Plattform berücksichtigt.

