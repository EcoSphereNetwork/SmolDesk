#!/usr/bin/env python3
"""
Git Diff Anwendungs-Skript
Führt die spezifischen Änderungen aus der bereitgestellten Git Diff durch.
"""

import os
import sys
import re
from pathlib import Path

def apply_file_changes():
    """Führt alle Dateiänderungen durch"""
    changes = [
        {
            "file": "docs/archive/old_docs/Smodesk-Mobile-UX.md",
            "changes": [
                ("![Light vs Dark](../images/mobile-theme.png)", "![Light vs Dark](../../static/img/docusaurus.png)")
            ]
        },
        {
            "file": "docs/archive/old_docs/Smodesk-Mobile.md",
            "changes": [
                ("wss://<server-url>", "wss://&lt;server-url&gt;"),
                ("<RTCView>", "`<RTCView />`"),
                ("<200 ms", "&lt;200 ms")
            ]
        },
        {
            "file": "docs/archive/old_docs/USER_GUIDE.md",
            "changes": [
                ("#### Für niedrige Latenz (<100ms)", "#### Für niedrige Latenz (&lt;100ms)")
            ]
        },
        {
            "file": "docs/docs/SmolDesk/README.md",
            "changes": [
                ('<img src="./static/img/logo.png" alt="SmolDesk Logo" width="200">', '<img src="./static/img/logo.png" alt="SmolDesk Logo" width="200" />')
            ]
        },
        {
            "file": "docs/docs/SmolDesk/development/Entwickle-Prompt.md",
            "changes": [
                ("- **Latenz**: Ziel von <200ms für Bildschirmübertragung", "- **Latenz**: Ziel von &lt;200ms für Bildschirmübertragung"),
                ("- Fokus auf niedrige Latenz (<200ms) und hohe Bildqualität", "- Fokus auf niedrige Latenz (&lt;200ms) und hohe Bildqualität")
            ]
        },
        {
            "file": "docs/docs/SmolDesk/development/Entwicklungsplan.md",
            "changes": [
                ("  - CPU-Last <15% bei 1080p", "  - CPU-Last &lt;15% bei 1080p")
            ]
        },
        {
            "file": "docs/docs/SmolDesk/development/Implementation-Plan.md",
            "changes": [
                ("- Reduce CPU utilization to target (<15%)", "- Reduce CPU utilization to target (&lt;15%)")
            ]
        },
        {
            "file": "docs/docs/SmolDesk/development/Implementation-Status.md",
            "changes": [
                ("   - Achieve target latency of <200ms", "   - Achieve target latency of &lt;200ms")
            ]
        },
        {
            "file": "docs/docs/SmolDesk/development/Integration-Testing-Plan.md",
            "changes": [
                ("| WI-001 | Basic connection establishment | Host and client on same network | 1. Start SmolDesk host<br>2. Create room on host<br>3. Join room from client | Connection established successfully |", "| WI-001 | Basic connection establishment | Host and client on same network | 1. Start SmolDesk host<br />2. Create room on host<br />3. Join room from client | Connection established successfully |"),
                ("| WI-002 | Connection with NAT traversal | Host and client on different networks | 1. Start SmolDesk host behind NAT<br>2. Create room on host<br>3. Join room from client behind different NAT | Connection established through STUN/TURN |", "| WI-002 | Connection with NAT traversal | Host and client on different networks | 1. Start SmolDesk host behind NAT<br />2. Create room on host<br />3. Join room from client behind different NAT | Connection established through STUN/TURN |"),
                ("| WI-003 | Connection recovery after network interruption | Established connection | 1. Establish connection<br>2. Temporarily disable network on client<br>3. Re-enable network | Connection recovers automatically |", "| WI-003 | Connection recovery after network interruption | Established connection | 1. Establish connection<br />2. Temporarily disable network on client<br />3. Re-enable network | Connection recovers automatically |"),
                ("| SC-001 | X11 capture to WebRTC stream | X11 session | 1. Start capture on X11<br>2. Verify stream in client | Stream visible with < 200ms latency |", "| SC-001 | X11 capture to WebRTC stream | X11 session | 1. Start capture on X11<br />2. Verify stream in client | Stream visible with &lt; 200ms latency |"),
                ("| SC-002 | Wayland capture to WebRTC stream | Wayland session | 1. Start capture on Wayland<br>2. Verify stream in client | Stream visible with < 200ms latency |", "| SC-002 | Wayland capture to WebRTC stream | Wayland session | 1. Start capture on Wayland<br />2. Verify stream in client | Stream visible with &lt; 200ms latency |"),
                ("| SC-003 | Hardware acceleration | GPU with VAAPI/NVENC | 1. Enable hardware acceleration<br>2. Start streaming<br>3. Monitor CPU/GPU usage | CPU usage < 15%, smooth streaming |", "| SC-003 | Hardware acceleration | GPU with VAAPI/NVENC | 1. Enable hardware acceleration<br />2. Start streaming<br />3. Monitor CPU/GPU usage | CPU usage &lt; 15%, smooth streaming |"),
                ("| SC-004 | Multi-monitor selection | Setup with multiple monitors | 1. Start SmolDesk<br>2. Select different monitors<br>3. Verify stream | Correct monitor displayed each time |", "| SC-004 | Multi-monitor selection | Setup with multiple monitors | 1. Start SmolDesk<br />2. Select different monitors<br />3. Verify stream | Correct monitor displayed each time |"),
                ("| WC-001 | WebCodecs stream processing | Chrome/Edge browser | 1. Start capture<br>2. Monitor browser process | WebCodecs API used for decoding |", "| WC-001 | WebCodecs stream processing | Chrome/Edge browser | 1. Start capture<br />2. Monitor browser process | WebCodecs API used for decoding |"),
                ("| WC-002 | Fallback mechanism | Firefox (without WebCodecs) | 1. Start capture<br>2. Monitor browser process | Traditional canvas-based fallback used |", "| WC-002 | Fallback mechanism | Firefox (without WebCodecs) | 1. Start capture<br />2. Monitor browser process | Traditional canvas-based fallback used |"),
                ("| AU-001 | Password protection | Password protected room | 1. Create room with password<br>2. Attempt join without password<br>3. Attempt join with wrong password<br>4. Attempt join with correct password | Only correct password allows joining |", "| AU-001 | Password protection | Password protected room | 1. Create room with password<br />2. Attempt join without password<br />3. Attempt join with wrong password<br />4. Attempt join with correct password | Only correct password allows joining |"),
                ("| AU-002 | OAuth2 PKCE flow | Configured OAuth provider | 1. Initialize OAuth<br>2. Follow authentication flow<br>3. Verify token reception | Authentication completes successfully |", "| AU-002 | OAuth2 PKCE flow | Configured OAuth provider | 1. Initialize OAuth<br />2. Follow authentication flow<br />3. Verify token reception | Authentication completes successfully |"),
                ("| AU-003 | User-based access control | Multiple user accounts | 1. Set room to private mode<br>2. Attempt access with unauthorized user<br>3. Attempt access with authorized user | Only authorized user gains access |", "| AU-003 | User-based access control | Multiple user accounts | 1. Set room to private mode<br />2. Attempt access with unauthorized user<br />3. Attempt access with authorized user | Only authorized user gains access |"),
                ("| MS-001 | Message signing | Connected peers | 1. Send signed message<br>2. Verify signature on recipient<br>3. Attempt to tamper message | Untampered message verified, tampered message rejected |", "| MS-001 | Message signing | Connected peers | 1. Send signed message<br />2. Verify signature on recipient<br />3. Attempt to tamper message | Untampered message verified, tampered message rejected |"),
                ("| MS-002 | Secure room creation | SecurityManager initialized | 1. Create secure room<br>2. Extract signature<br>3. Attempt to join with tampered room ID | Original room ID works, tampered ID rejected |", "| MS-002 | Secure room creation | SecurityManager initialized | 1. Create secure room<br />2. Extract signature<br />3. Attempt to join with tampered room ID | Original room ID works, tampered ID rejected |"),
                ("| MS-003 | Data encryption | Connected peers | 1. Send encrypted data<br>2. Intercept and analyze network traffic<br>3. Decrypt on recipient | Data not readable in transit, correctly decrypted |", "| MS-003 | Data encryption | Connected peers | 1. Send encrypted data<br />2. Intercept and analyze network traffic<br />3. Decrypt on recipient | Data not readable in transit, correctly decrypted |"),
                ("| LT-001 | End-to-end latency (LAN) | LAN connection | 1. Start capture<br>2. Perform timed action on host<br>3. Measure time until visible on client | Latency < 100ms |", "| LT-001 | End-to-end latency (LAN) | LAN connection | 1. Start capture<br />2. Perform timed action on host<br />3. Measure time until visible on client | Latency &lt; 100ms |"),
                ("| LT-002 | End-to-end latency (WAN) | Simulated WAN | 1. Start capture<br>2. Perform timed action on host<br>3. Measure time until visible on client | Latency < 200ms + network RTT |", "| LT-002 | End-to-end latency (WAN) | Simulated WAN | 1. Start capture<br />2. Perform timed action on host<br />3. Measure time until visible on client | Latency &lt; 200ms + network RTT |"),
                ("| LT-003 | Input event latency | Connected session | 1. Start session<br>2. Send input from client<br>3. Measure time until action on host | Input latency < 50ms + network RTT |", "| LT-003 | Input event latency | Connected session | 1. Start session<br />2. Send input from client<br />3. Measure time until action on host | Input latency &lt; 50ms + network RTT |"),
                ("| RU-001 | CPU usage (software encoding) | Software encoding mode | 1. Start 1080p stream<br>2. Monitor CPU usage for 5 minutes<br>3. Perform various actions | Average CPU < 30%, peak < 50% |", "| RU-001 | CPU usage (software encoding) | Software encoding mode | 1. Start 1080p stream<br />2. Monitor CPU usage for 5 minutes<br />3. Perform various actions | Average CPU &lt; 30%, peak &lt; 50% |"),
                ("| RU-002 | CPU usage (hardware encoding) | Hardware encoding mode | 1. Start 1080p stream<br>2. Monitor CPU usage for 5 minutes<br>3. Perform various actions | Average CPU < 15%, peak < 25% |", "| RU-002 | CPU usage (hardware encoding) | Hardware encoding mode | 1. Start 1080p stream<br />2. Monitor CPU usage for 5 minutes<br />3. Perform various actions | Average CPU &lt; 15%, peak &lt; 25% |"),
                ("| RU-003 | Memory usage | Extended session | 1. Start session<br>2. Run for 2 hours<br>3. Monitor memory usage | No significant memory growth over time |", "| RU-003 | Memory usage | Extended session | 1. Start session<br />2. Run for 2 hours<br />3. Monitor memory usage | No significant memory growth over time |"),
                ("| RU-004 | Bandwidth usage | Network monitoring | 1. Stream at different quality levels<br>2. Measure bandwidth consumption | Bandwidth matches expected rates, adapts to conditions |", "| RU-004 | Bandwidth usage | Network monitoring | 1. Stream at different quality levels<br />2. Measure bandwidth consumption | Bandwidth matches expected rates, adapts to conditions |"),
                ("| AQ-001 | Network degradation response | Network throttling tool | 1. Start high-quality stream<br>2. Gradually restrict bandwidth<br>3. Monitor quality adjustments | Quality decreases smoothly as bandwidth decreases |", "| AQ-001 | Network degradation response | Network throttling tool | 1. Start high-quality stream<br />2. Gradually restrict bandwidth<br />3. Monitor quality adjustments | Quality decreases smoothly as bandwidth decreases |"),
                ("| AQ-002 | Network improvement response | Network throttling tool | 1. Start with restricted bandwidth<br>2. Gradually increase available bandwidth<br>3. Monitor quality adjustments | Quality increases as bandwidth becomes available |", "| AQ-002 | Network improvement response | Network throttling tool | 1. Start with restricted bandwidth<br />2. Gradually increase available bandwidth<br />3. Monitor quality adjustments | Quality increases as bandwidth becomes available |"),
                ("| AQ-003 | CPU load response | CPU load generator | 1. Start stream<br>2. Gradually increase CPU load<br>3. Monitor quality adjustments | Encoder adjusts to maintain performance under load |", "| AQ-003 | CPU load response | CPU load generator | 1. Start stream<br />2. Gradually increase CPU load<br />3. Monitor quality adjustments | Encoder adjusts to maintain performance under load |"),
                ("| EC-001 | Extremely poor network | Network degradation tool | 1. Configure 5% packet loss, 500ms latency<br>2. Establish connection<br>3. Maintain stream | Connection maintained, possibly at reduced quality |", "| EC-001 | Extremely poor network | Network degradation tool | 1. Configure 5% packet loss, 500ms latency<br />2. Establish connection<br />3. Maintain stream | Connection maintained, possibly at reduced quality |"),
                ("| EC-002 | Firewall restrictions | Restricted network | 1. Block UDP ports<br>2. Attempt connection<br>3. Monitor fallback to TCP | Connection established via TURN TCP fallback |", "| EC-002 | Firewall restrictions | Restricted network | 1. Block UDP ports<br />2. Attempt connection<br />3. Monitor fallback to TCP | Connection established via TURN TCP fallback |"),
                ("| EC-003 | Sudden disconnect | Established connection | 1. Establish connection<br>2. Abruptly disconnect network<br>3. Reconnect network after 30s | Session recovers or gracefully notifies disconnection |", "| EC-003 | Sudden disconnect | Established connection | 1. Establish connection<br />2. Abruptly disconnect network<br />3. Reconnect network after 30s | Session recovers or gracefully notifies disconnection |"),
                ("| DS-001 | Display server switch | System with both X11/Wayland | 1. Start capture in X11<br>2. Switch to Wayland<br>3. Restart capture | Capture works in both environments |", "| DS-001 | Display server switch | System with both X11/Wayland | 1. Start capture in X11<br />2. Switch to Wayland<br />3. Restart capture | Capture works in both environments |"),
                ("| DS-002 | Resolution change | Dynamic resolution change | 1. Start capture<br>2. Change display resolution<br>3. Monitor stream | Stream adapts to new resolution |", "| DS-002 | Resolution change | Dynamic resolution change | 1. Start capture<br />2. Change display resolution<br />3. Monitor stream | Stream adapts to new resolution |"),
                ("| DS-003 | Multi-GPU setup | System with multiple GPUs | 1. Configure displays on different GPUs<br>2. Capture from each display | Correct capture regardless of GPU |", "| DS-003 | Multi-GPU setup | System with multiple GPUs | 1. Configure displays on different GPUs<br />2. Capture from each display | Correct capture regardless of GPU |")
            ]
        },
        {
            "file": "docs/docs/SmolDesk/development/Optimierungsplan-WebRTC-Bildschirmübertragung.md",
            "changes": [
                ("- **Ziel-Latenz**: <200ms End-to-End", "- **Ziel-Latenz**: &lt;200ms End-to-End"),
                ("- **CPU-Auslastung**: <15% auf modernen Systemen", "- **CPU-Auslastung**: &lt;15% auf modernen Systemen"),
                ("- **Wiederherstellungszeit**: <2s nach Netzwerkunterbrechungen", "- **Wiederherstellungszeit**: &lt;2s nach Netzwerkunterbrechungen")
            ]
        },
        {
            "file": "docs/usage/clipboard.md",
            "changes": [
                ("![Bild]()", "![Beispiel](../static/img/docusaurus.png)")
            ]
        },
        {
            "file": "docs/usage/files.md",
            "changes": [
                ("![Bild]()", "![Beispiel](../static/img/docusaurus.png)")
            ]
        },
        {
            "file": "docs/usage/monitors.md",
            "changes": [
                ("![Bild]()", "![Beispiel](../static/img/docusaurus.png)")
            ]
        },
        {
            "file": "docs/usage/viewer.md",
            "changes": [
                ("![Bild]()", "![Beispiel](../static/img/docusaurus.png)")
            ]
        }
    ]
    
    # Datei löschen
    file_to_delete = "docs/src/pages/markdown-page.md"
    
    return changes, file_to_delete

def apply_changes_to_file(file_path, changes):
    """Wendet die Änderungen auf eine einzelne Datei an"""
    if not os.path.exists(file_path):
        print(f"⚠️  Datei nicht gefunden: {file_path}")
        return False
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        changes_applied = 0
        
        for old_text, new_text in changes:
            if old_text in content:
                content = content.replace(old_text, new_text)
                changes_applied += 1
                print(f"  ✅ Ersetzt: {old_text[:50]}...")
            else:
                print(f"  ⚠️  Nicht gefunden: {old_text[:50]}...")
        
        if changes_applied > 0:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"  📝 {changes_applied} Änderung(en) in {file_path} angewendet")
        
        return changes_applied > 0
        
    except Exception as e:
        print(f"❌ Fehler bei {file_path}: {str(e)}")
        return False

def delete_file(file_path):
    """Löscht eine Datei"""
    if os.path.exists(file_path):
        try:
            os.remove(file_path)
            print(f"🗑️  Datei gelöscht: {file_path}")
            return True
        except Exception as e:
            print(f"❌ Fehler beim Löschen von {file_path}: {str(e)}")
            return False
    else:
        print(f"⚠️  Datei zum Löschen nicht gefunden: {file_path}")
        return False

def main():
    """Hauptfunktion"""
    print("🚀 Starte Git Diff Anwendung...")
    print("="*50)
    
    changes, file_to_delete = apply_file_changes()
    
    total_files = len(changes)
    successful_files = 0
    
    # Änderungen anwenden
    for file_info in changes:
        file_path = file_info["file"]
        file_changes = file_info["changes"]
        
        print(f"\n📁 Bearbeite: {file_path}")
        
        if apply_changes_to_file(file_path, file_changes):
            successful_files += 1
    
    # Datei löschen
    print(f"\n🗑️  Lösche Datei: {file_to_delete}")
    delete_file(file_to_delete)
    
    # Zusammenfassung
    print("\n" + "="*50)
    print(f"📊 Zusammenfassung:")
    print(f"   • {successful_files}/{total_files} Dateien erfolgreich bearbeitet")
    print(f"   • 1 Datei gelöscht")
    print("✅ Git Diff Anwendung abgeschlossen!")

if __name__ == "__main__":
    # Prüfen ob wir im richtigen Verzeichnis sind
    if not os.path.exists("docs"):
        print("❌ Bitte führe das Skript im Hauptverzeichnis des Projekts aus (dort wo sich der 'docs' Ordner befindet)")
        sys.exit(1)
    
    main()
