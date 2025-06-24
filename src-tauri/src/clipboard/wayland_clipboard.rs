// src-tauri/src/clipboard/wayland_clipboard.rs - Wayland-spezifische Zwischenablage-Implementierung

use std::process::Command;
use crate::clipboard::types::ClipboardProvider;
use crate::clipboard::error::ClipboardError;

/// Wayland-spezifische Zwischenablage-Implementierung
pub struct WaylandClipboardProvider {
    /// Ob wl-clipboard verfügbar ist
    has_wl_clipboard: bool,
    
    /// Ob wl-copy und wl-paste verfügbar sind
    has_wl_copy: bool,
    has_wl_paste: bool,
}

impl WaylandClipboardProvider {
    /// Erstellt einen neuen WaylandClipboardProvider
    pub fn new() -> Result<Self, ClipboardError> {
        // Prüfen, ob wl-clipboard-Tools verfügbar sind
        let has_wl_copy = Self::check_tool_available("wl-copy");
        let has_wl_paste = Self::check_tool_available("wl-paste");
        let has_wl_clipboard = has_wl_copy && has_wl_paste;
        
        if !has_wl_clipboard {
            return Err(ClipboardError::ClipboardUnavailable(
                "wl-clipboard tools (wl-copy, wl-paste) are not available. Please install wl-clipboard package.".to_string()
            ));
        }
        
        Ok(WaylandClipboardProvider {
            has_wl_clipboard,
            has_wl_copy,
            has_wl_paste,
        })
    }
    
    /// Prüft, ob ein Tool verfügbar ist
    fn check_tool_available(tool: &str) -> bool {
        Command::new("which")
            .arg(tool)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Führt wl-paste aus
    fn run_wl_paste(&self, args: &[&str]) -> Result<String, ClipboardError> {
        if !self.has_wl_paste {
            return Err(ClipboardError::UnsupportedOperation("wl-paste not available".to_string()));
        }
        
        let mut cmd = Command::new("wl-paste");
        cmd.args(args);
        
        let output = cmd.output()
            .map_err(|e| ClipboardError::IoError(format!("Failed to execute wl-paste: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("No selection") || stderr.contains("nothing to paste") {
                return Err(ClipboardError::EmptyClipboard);
            }
            return Err(ClipboardError::IoError(format!("wl-paste failed: {}", stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Führt wl-copy aus
    fn run_wl_copy(&self, args: &[&str], input: Option<&str>) -> Result<(), ClipboardError> {
        if !self.has_wl_copy {
            return Err(ClipboardError::UnsupportedOperation("wl-copy not available".to_string()));
        }
        
        let mut cmd = Command::new("wl-copy");
        cmd.args(args);
        
        if let Some(input_data) = input {
            use std::process::Stdio;
            use std::io::Write;
            
            cmd.stdin(Stdio::piped())
               .stdout(Stdio::piped())
               .stderr(Stdio::piped());
            
            let mut child = cmd.spawn()
                .map_err(|e| ClipboardError::IoError(format!("Failed to spawn wl-copy: {}", e)))?;
            
            if let Some(stdin) = child.stdin.take() {
                let mut stdin = stdin;
                stdin.write_all(input_data.as_bytes())
                    .map_err(|e| ClipboardError::IoError(format!("Failed to write to wl-copy stdin: {}", e)))?;
            }
            
            let output = child.wait_with_output()
                .map_err(|e| ClipboardError::IoError(format!("Failed to wait for wl-copy: {}", e)))?;
            
            if !output.status.success() {
                return Err(ClipboardError::IoError(
                    format!("wl-copy failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        } else {
            let output = cmd.output()
                .map_err(|e| ClipboardError::IoError(format!("Failed to execute wl-copy: {}", e)))?;
            
            if !output.status.success() {
                return Err(ClipboardError::IoError(
                    format!("wl-copy failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        }
        
        Ok(())
    }
    
    /// Holt verfügbare MIME-Typen aus der Wayland-Zwischenablage
    fn get_available_mime_types(&self) -> Result<Vec<String>, ClipboardError> {
        let output = self.run_wl_paste(&["-l"])?;
        
        Ok(output.lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect())
    }
}

impl ClipboardProvider for WaylandClipboardProvider {
    fn get_text(&mut self) -> Result<String, ClipboardError> {
        let output = self.run_wl_paste(&["-n"])?; // -n verhindert newline am Ende
        if output.is_empty() {
            Err(ClipboardError::EmptyClipboard)
        } else {
            Ok(output)
        }
    }
    
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        if text.is_empty() {
            // Leere Zwischenablage durch Setzen von leerem String
            self.run_wl_copy(&[], Some(""))?;
        } else {
            self.run_wl_copy(&[], Some(text))?;
        }
        Ok(())
    }
    
    fn get_image(&mut self) -> Result<Vec<u8>, ClipboardError> {
        // Versuche PNG-Format zu holen
        let png_result = Command::new("wl-paste")
            .args(&["-t", "image/png"])
            .output();
        
        if let Ok(output) = png_result {
            if output.status.success() && !output.stdout.is_empty() {
                return Ok(output.stdout);
            }
        }
        
        // Fallback auf JPEG
        let jpeg_result = Command::new("wl-paste")
            .args(&["-t", "image/jpeg"])
            .output();
        
        if let Ok(output) = jpeg_result {
            if output.status.success() && !output.stdout.is_empty() {
                return Ok(output.stdout);
            }
        }
        
        // Fallback auf GIF
        let gif_result = Command::new("wl-paste")
            .args(&["-t", "image/gif"])
            .output();
        
        if let Ok(output) = gif_result {
            if output.status.success() && !output.stdout.is_empty() {
                return Ok(output.stdout);
            }
        }
        
        Err(ClipboardError::EmptyClipboard)
    }
    
    fn set_image(&mut self, image_data: &[u8], format: &str) -> Result<(), ClipboardError> {
        // Bestimme MIME-Typ basierend auf Format
        let mime_type = match format.to_lowercase().as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "webp" => "image/webp",
            _ => "image/png", // Standard-Fallback
        };
        
        // Schreibe Bilddaten in temporäre Datei
        use std::io::Write;
        let temp_file = format!("/tmp/smoldesk_clipboard_image_{}.{}", 
            std::process::id(), format);
        
        let mut file = std::fs::File::create(&temp_file)
            .map_err(|e| ClipboardError::IoError(format!("Failed to create temp file: {}", e)))?;
        
        file.write_all(image_data)
            .map_err(|e| ClipboardError::IoError(format!("Failed to write temp file: {}", e)))?;
        
        // Verwende wl-copy mit Datei-Input
        let output = Command::new("wl-copy")
            .args(&["-t", mime_type])
            .arg("<")
            .arg(&temp_file)
            .output();
        
        // Alternative: Verwende cat mit pipe zu wl-copy
        let output = Command::new("sh")
            .arg("-c")
            .arg(&format!("cat '{}' | wl-copy -t '{}'", temp_file, mime_type))
            .output()
            .map_err(|e| ClipboardError::IoError(format!("Failed to execute wl-copy with image: {}", e)))?;
        
        // Temporäre Datei löschen
        let _ = std::fs::remove_file(&temp_file);
        
        if !output.status.success() {
            return Err(ClipboardError::IoError(
                format!("wl-copy failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        Ok(())
    }
    
    fn get_html(&mut self) -> Result<String, ClipboardError> {
        let html_result = self.run_wl_paste(&["-t", "text/html"]);
        
        match html_result {
            Ok(html) if !html.is_empty() => Ok(html),
            _ => {
                // Fallback auf Text
                self.get_text()
            }
        }
    }
    
    fn set_html(&mut self, html: &str) -> Result<(), ClipboardError> {
        // Setze HTML-Content
        self.run_wl_copy(&["-t", "text/html"], Some(html))?;
        
        // Zusätzlich als plain text setzen für Kompatibilität
        let text_content = html_to_text(html);
        self.run_wl_copy(&["-t", "text/plain"], Some(&text_content))?;
        
        Ok(())
    }
    
    fn get_files(&mut self) -> Result<Vec<String>, ClipboardError> {
        // Versuche URI-Liste zu holen
        let output = self.run_wl_paste(&["-t", "text/uri-list"])?;
        
        if output.is_empty() {
            return Err(ClipboardError::EmptyClipboard);
        }
        
        // Parse URI-Liste
        let files: Vec<String> = output.lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                if line.starts_with("file://") {
                    // URL-Dekodierung für Dateinamen mit Sonderzeichen
                    urlencoding::decode(&line[7..])
                        .map(|decoded| decoded.to_string())
                        .unwrap_or_else(|_| line[7..].to_string())
                } else {
                    line.to_string()
                }
            })
            .collect();
        
        Ok(files)
    }
    
    fn is_available(&self) -> bool {
        self.has_wl_clipboard
    }
    
    fn create_clone(&self) -> Box<dyn ClipboardProvider> {
        Box::new(WaylandClipboardProvider {
            has_wl_clipboard: self.has_wl_clipboard,
            has_wl_copy: self.has_wl_copy,
            has_wl_paste: self.has_wl_paste,
        })
    }
    
    fn get_available_formats(&self) -> Vec<String> {
        self.get_available_mime_types().unwrap_or_else(|_| vec!["text/plain".to_string()])
    }
}

/// Einfache HTML-zu-Text-Konvertierung
fn html_to_text(html: &str) -> String {
    // Einfache Regex-basierte HTML-Tag-Entfernung
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    let text = re.replace_all(html, "");
    
    // HTML-Entitäten dekodieren (grundlegend)
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("&#x60;", "`")
        .replace("&#x3D;", "=")
}
