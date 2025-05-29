#!/usr/bin/env python3
"""
SmolDesk Automated Security Scanner
Führt grundlegende Sicherheitstests durch
"""

import asyncio
import json
import ssl
import websockets
import requests
import subprocess
import sys
from pathlib import Path
import argparse

class SmolDeskSecurityScanner:
    def __init__(self, target_host="localhost", target_port=3000):
        self.target_host = target_host
        self.target_port = target_port
        self.results = {
            "vulnerabilities": [],
            "warnings": [],
            "info": [],
            "passed": []
        }
    
    async def scan_signaling_server(self):
        """Test Signaling-Server Sicherheit"""
        print("🔍 Scanning Signaling Server...")
        
        # Test WebSocket-Verbindung
        uri = f"ws://{self.target_host}:{self.target_port}"
        try:
            async with websockets.connect(uri) as websocket:
                # Test für Input-Validation
                malicious_payloads = [
                    '{"type": "create-room", "roomId": "../../../etc/passwd"}',
                    '{"type": "join-room", "roomId": "<script>alert(1)</script>"}',
                    '{"type": "' + 'A' * 10000 + '"}',  # Buffer overflow test
                    '{"type": null}',
                    'not-json-data',
                ]
                
                for payload in malicious_payloads:
                    await websocket.send(payload)
                    try:
                        response = await asyncio.wait_for(websocket.recv(), timeout=2)
                        if "error" not in response.lower():
                            self.results["vulnerabilities"].append({
                                "type": "Input Validation",
                                "payload": payload[:100],
                                "description": "Server accepts malicious input without proper validation"
                            })
                    except asyncio.TimeoutError:
                        pass
                
                self.results["passed"].append("WebSocket connection established successfully")
        
        except Exception as e:
            self.results["warnings"].append(f"Could not connect to signaling server: {e}")
    
    def scan_system_dependencies(self):
        """Überprüfe System-Abhängigkeiten"""
        print("🔍 Scanning System Dependencies...")
        
        dependencies = {
            "ffmpeg": "CVE database check needed",
            "xdotool": "Input injection vector",
            "ydotool": "Privilege escalation potential",
            "wl-clipboard": "Clipboard data leakage",
            "xclip": "X11 security context"
        }
        
        for dep, risk in dependencies.items():
            try:
                result = subprocess.run([dep, "--version"], 
                                      capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    version = result.stdout.split('\n')[0]
                    self.results["info"].append(f"{dep}: {version} - Risk: {risk}")
                else:
                    self.results["info"].append(f"{dep}: Not found")
            except (subprocess.TimeoutExpired, FileNotFoundError):
                self.results["info"].append(f"{dep}: Not available")
    
    def scan_file_permissions(self):
        """Überprüfe Dateiberechtigungen"""
        print("🔍 Scanning File Permissions...")
        
        # Überprüfe kritische Dateien
        critical_files = [
            "/opt/smoldesk/smoldesk",
            "/usr/bin/smoldesk", 
            "/etc/smoldesk/",
            "~/.local/share/smoldesk/",
        ]
        
        for file_path in critical_files:
            expanded_path = Path(file_path).expanduser()
            if expanded_path.exists():
                stat_info = expanded_path.stat()
                mode = oct(stat_info.st_mode)[-3:]
                
                # Überprüfe für unsichere Berechtigungen
                if mode in ['777', '776', '666']:
                    self.results["vulnerabilities"].append({
                        "type": "File Permissions",
                        "file": str(expanded_path),
                        "permissions": mode,
                        "description": "File has overly permissive permissions"
                    })
                elif mode in ['755', '644']:
                    self.results["passed"].append(f"{expanded_path}: Safe permissions ({mode})")
                else:
                    self.results["warnings"].append(f"{expanded_path}: Unusual permissions ({mode})")
    
    def scan_network_security(self):
        """Überprüfe Netzwerk-Sicherheitskonfiguration"""
        print("🔍 Scanning Network Security...")
        
        # Test HTTPS-Konfiguration falls verfügbar
        https_url = f"https://{self.target_host}:{self.target_port}"
        try:
            response = requests.get(https_url, timeout=5, verify=False)
            if response.status_code == 200:
                self.results["warnings"].append("HTTPS endpoint responds but certificate not verified")
            
            # TLS-Konfiguration testen
            context = ssl.create_default_context()
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE
            
            with context.wrap_socket(socket.socket(), server_hostname=self.target_host) as sock:
                sock.connect((self.target_host, self.target_port))
                cipher = sock.cipher()
                protocol = sock.version()
                
                if protocol < "TLSv1.2":
                    self.results["vulnerabilities"].append({
                        "type": "TLS Configuration",
                        "protocol": protocol,
                        "description": "Weak TLS version in use"
                    })
                else:
                    self.results["passed"].append(f"TLS version: {protocol}")
        
        except requests.exceptions.RequestException:
            self.results["info"].append("No HTTPS endpoint found")
        except Exception as e:
            self.results["info"].append(f"TLS scan failed: {e}")
    
    def scan_authentication_security(self):
        """Teste Authentifizierungsmechanismen"""
        print("🔍 Scanning Authentication Security...")
        
        # Test für schwache Passwörter
        weak_passwords = [
            "password", "123456", "admin", "smoldesk", 
            "password123", "qwerty", "", "test"
        ]
        
        # TODO: Implementiere tatsächliche Auth-Tests
        # Dies würde HTTP-Requests an Auth-Endpoints senden
        
        self.results["info"].append("Authentication testing requires running instance")
    
    def generate_report(self):
        """Generiere Sicherheitsbericht"""
        print("\n" + "="*60)
        print("🛡️  SMOLDESK SECURITY SCAN RESULTS")
        print("="*60)
        
        # Vulnerabilities
        if self.results["vulnerabilities"]:
            print("\n🚨 VULNERABILITIES FOUND:")
            for vuln in self.results["vulnerabilities"]:
                print(f"  ❌ {vuln['type']}: {vuln['description']}")
                if 'file' in vuln:
                    print(f"     File: {vuln['file']} (Permissions: {vuln.get('permissions', 'N/A')})")
                if 'payload' in vuln:
                    print(f"     Payload: {vuln['payload']}")
        else:
            print("\n✅ No critical vulnerabilities found")
        
        # Warnings
        if self.results["warnings"]:
            print("\n⚠️  WARNINGS:")
            for warning in self.results["warnings"]:
                print(f"  🔶 {warning}")
        
        # Informational
        if self.results["info"]:
            print("\nℹ️  INFORMATIONAL:")
            for info in self.results["info"]:
                print(f"  💡 {info}")
        
        # Passed checks
        if self.results["passed"]:
            print("\n✅ PASSED CHECKS:")
            for passed in self.results["passed"]:
                print(f"  ✅ {passed}")
        
        # Risk Score
        risk_score = (
            len(self.results["vulnerabilities"]) * 10 +
            len(self.results["warnings"]) * 3
        )
        
        print(f"\n📊 RISK SCORE: {risk_score}")
        if risk_score == 0:
            print("   🟢 LOW RISK")
        elif risk_score < 20:
            print("   🟡 MEDIUM RISK")
        else:
            print("   🔴 HIGH RISK")
        
        # Recommendations
        print("\n💡 RECOMMENDATIONS:")
        if self.results["vulnerabilities"]:
            print("  1. Address all critical vulnerabilities immediately")
        if self.results["warnings"]:
            print("  2. Review and mitigate warnings where possible")
        print("  3. Run this scan regularly as part of CI/CD")
        print("  4. Consider professional penetration testing")
        print("  5. Keep all dependencies updated")
        
        print("\n" + "="*60)
    
    async def run_scan(self):
        """Führe kompletten Sicherheitsscan durch"""
        print("🛡️  Starting SmolDesk Security Scan...")
        
        await self.scan_signaling_server()
        self.scan_system_dependencies()
        self.scan_file_permissions()
        self.scan_network_security()
        self.scan_authentication_security()
        
        self.generate_report()

def main():
    parser = argparse.ArgumentParser(description="SmolDesk Security Scanner")
    parser.add_argument("--host", default="localhost", help="Target host")
    parser.add_argument("--port", type=int, default=3000, help="Target port")
    parser.add_argument("--output", help="Output file for results")
    
    args = parser.parse_args()
    
    scanner = SmolDeskSecurityScanner(args.host, args.port)
    
    try:
        asyncio.run(scanner.run_scan())
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(scanner.results, f, indent=2)
            print(f"\nResults saved to {args.output}")
    
    except KeyboardInterrupt:
        print("\nScan interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\nScan failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
