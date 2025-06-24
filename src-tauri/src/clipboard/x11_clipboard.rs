// src-tauri/src/clipboard/x11_clipboard.rs - X11-spezifische Zwischenablage-Implementierung

use std::process::Command;
use crate::clipboard::types::ClipboardProvider;
use crate::clipboard::error::ClipboardError;
use base64::{Engine as _, engine::general_purpose};

/// X11-spezifische Zwischenablage-Implementierung
pub struct X11ClipboardProvider {
    /// Ob xclip verfügbar ist
    has_xclip: bool,
    
    /// Ob xsel verfügbar ist  
    has_xsel: bool,
    
    /// Bevorzugtes Tool (xclip oder xsel)
    preferred_tool: X11ClipboardTool,
}

#[derive(Debug, Clone, Copy)]
enum X11ClipboardTool {
    XClip,
    XSel,
    None,
}

impl X11ClipboardProvider {
    /// Erstellt einen neuen X11ClipboardProvider
    pub fn new() -> Result<Self, ClipboardError> {
        // Prüfen, welche Tools verfügbar sind
        let has_xclip = Self::check_tool_available("xclip");
        let has_xsel = Self::check_tool_available("xsel");
        
        if !has_xclip && !has_xsel {
            return Err(ClipboardError::ClipboardUnavailable(
                "Neither xclip nor xsel is available. Please install one of them.".to_string()
            ));
        }
        
        // xclip bevorzugen, falls verfügbar
        let preferred_tool = if has_xclip {
            X11ClipboardTool::XClip
        } else if has_xsel {
            X11ClipboardTool::XSel
        } else {
            X11ClipboardTool::None
        };
        
        Ok(X11ClipboardProvider {
            has_xclip,
            has_xsel,
            preferred_tool,
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
    
    /// Führt einen xclip-Befehl aus
    fn run_xclip_command(&self, args: &[&str], input: Option<&str>) -> Result<String, ClipboardError> {
        if !self.has_xclip {
            return Err(ClipboardError::UnsupportedOperation("xclip not available".to_string()));
        }
        
        let mut cmd = Command::new("xclip");
        cmd.args(args);
        
        if let Some(input_data) = input {
            use std::process::Stdio;
            use std::io::Write;
            
            cmd.stdin(Stdio::piped())
               .stdout(Stdio::piped())
               .stderr(Stdio::piped());
            
            let mut child = cmd.spawn()
                .map_err(|e| ClipboardError::IoError(format!("Failed to spawn xclip: {}", e)))?;
            
            if let Some(stdin) = child.stdin.take() {
                let mut stdin = stdin;
                stdin.write_all(input_data.as_bytes())
                    .map_err(|e| ClipboardError::IoError(format!("Failed to write to xclip stdin: {}", e)))?;
            }
            
            let output = child.wait_with_output()
                .map_err(|e| ClipboardError::IoError(format!("Failed to wait for xclip: {}", e)))?;
            
            if !output.status.success() {
                return Err(ClipboardError::IoError(
                    format!("xclip failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
            
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let output = cmd.output()
                .map_err(|e| ClipboardError::IoError(format!("Failed to execute xclip: {}", e)))?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("selection is empty") {
                    return Err(ClipboardError::EmptyClipboard);
                }
                return Err(ClipboardError::IoError(format!("xclip failed: {}", stderr)));
            }
            
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }
    
    /// Führt einen xsel-Befehl aus
    fn run_xsel_command(&self, args: &[&str], input: Option<&str>) -> Result<String, ClipboardError> {
        if !self.has_xsel {
            return Err(ClipboardError::UnsupportedOperation("xsel not available".to_string()));
        }
        
        let mut cmd = Command::new("xsel");
        cmd.args(args);
        
        if let Some(input_data) = input {
            use std::process::Stdio;
            use std::io::Write;
            
            cmd.stdin(Stdio::piped())
               .stdout(Stdio::piped())
               .stderr(Stdio::piped());
            
            let mut child = cmd.spawn()
                .map_err(|e| ClipboardError::IoError(format!("Failed to spawn xsel: {}", e)))?;
            
            if let Some(stdin) = child.stdin.take() {
                let mut stdin = stdin;
                stdin.write_all(input_data.as_bytes())
                    .map_err(|e| ClipboardError::IoError(format!("Failed to write to xsel stdin: {}", e)))?;
            }
            
            let output = child.wait_with_output()
                .map_err(|e| ClipboardError::IoError(format!("Failed to wait for xsel: {}", e)))?;
            
            if !output.status.success() {
                return Err(ClipboardError::IoError(
                    format!("xsel failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
            
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let output = cmd.output()
                .map_err(|e| ClipboardError::IoError(format!("Failed to execute xsel: {}", e)))?;
            
            if !output.status.success() {
                return Err(ClipboardError::EmptyClipboard);
            }
            
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }
    
    /// Holt verfügbare MIME-Typen aus der X11-Zwischenablage
    fn get_available_mime_types(&self) -> Result<Vec<String>, ClipboardError> {
        match self.preferred_tool {
            X11ClipboardTool::XClip => {
                let output = self.run_xclip_command(&["-selection", "clipboard", "-t", "TARGETS", "-o"], None)?;
                
                Ok(output.lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect())
            },
            X11ClipboardTool::XSel => {
                // xsel kann nicht direkt MIME-Typen auflisten, also Standard-Liste zurückgeben
                Ok(vec!["text/plain".to_string()])
            },
            X11ClipboardTool::None => {
                Err(ClipboardError::ClipboardUnavailable("No clipboard tool available".to_string()))
            }
        }
    }
}

impl ClipboardProvider for X11ClipboardProvider {
    fn get_text(&mut self) -> Result<String, ClipboardError> {
        match self.preferred_tool {
            X11ClipboardTool::XClip => {
                let output = self.run_xclip_command(&["-selection", "clipboard", "-o"], None)?;
                if output.is_empty() {
                    Err(ClipboardError::EmptyClipboard)
                } else {
                    Ok(output)
                }
            },
            X11ClipboardTool::XSel => {
                let output = self.run_xsel_command(&["-b", "-o"], None)?;
                if output.is_empty() {
                    Err(ClipboardError::EmptyClipboard)
                } else {
                    Ok(output)
                }
            },
            X11ClipboardTool::None => {
                Err(ClipboardError::ClipboardUnavailable("No clipboard tool available".to_string()))
            }
        }
    }
    
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        if text.is_empty() {
            return Ok(());
        }
        
        match self.preferred_tool {
            X11ClipboardTool::XClip => {
                self.run_xclip_command(&["-selection", "clipboard", "-i"], Some(text))?;
                Ok(())
            },
            X11ClipboardTool::XSel => {
                self.run_xsel_command(&["-b", "-i"], Some(text))?;
                Ok(())
            },
            X11ClipboardTool::None => {
                Err(ClipboardError::ClipboardUnavailable("No clipboard tool available".to_string()))
            }
        }
    }
    
    fn get_image(&mut self) -> Result<Vec<u8>, ClipboardError> {
        match self.preferred_tool {
            X11ClipboardTool::XClip => {
                // Versuche PNG-Format zu holen
                let png_result = self.run_xclip_command(&["-selection", "clipboard", "-t", "image/png", "-o"], None);
                
                if let Ok(output) = png_result {
                    if !output.is_empty() {
                        // Ausgabe ist bereits Binärdaten, direkt zurückgeben
                        return Ok(output.into_bytes());
                    }
                }
                
                // Fallback auf JPEG
                let jpeg_result = self.run_xclip_command(&["-selection", "clipboard", "-t", "image/jpeg", "-o"], None);
                
                if let Ok(output) = jpeg_result {
                    if !output.is_empty() {
                        return Ok(output.into_bytes());
                    }
                }
                
                Err(ClipboardError::EmptyClipboard)
            },
            X11ClipboardTool::XSel => {
                // xsel unterstützt keine Bilder direkt
                Err(ClipboardError::UnsupportedOperation("Image clipboard not supported with xsel".to_string()))
            },
            X11ClipboardTool::None => {
                Err(ClipboardError::ClipboardUnavailable("No clipboard tool available".to_string()))
            }
        }
    }
    
    fn set_image(&mut self, image_data: &[u8], format: &str) -> Result<(), ClipboardError> {
        match self.preferred_tool {
            X11ClipboardTool::XClip => {
                // Bestimme MIME-Typ basierend auf Format
                let mime_type = match format.to_lowercase().as_str() {
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "bmp" => "image/bmp",
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
                
                // Verwende xclip mit Datei-Input
                let output = Command::new("xclip")
                    .args(&["-selection", "clipboard", "-t", mime_type, "-i", &temp_file])
                    .output()
                    .map_err(|e| ClipboardError::IoError(format!("Failed to execute xclip: {}", e)))?;
                
                // Temporäre Datei löschen
                let _ = std::fs::remove_file(&temp_file);
                
                if !output.status.success() {
                    return Err(ClipboardError::IoError(
                        format!("xclip failed: {}", String::from_utf8_lossy(&output.stderr))
                    ));
                }
                
                Ok(())
            },
            X11ClipboardTool::XSel => {
                Err(ClipboardError::UnsupportedOperation("Image clipboard not supported with xsel".to_string()))
            },
            X11ClipboardTool::None => {
                Err(ClipboardError::ClipboardUnavailable("No clipboard tool available".to_string()))
            }
        }
    }
    
    fn get_html(&mut self) -> Result<String, ClipboardError> {
        match self.preferred_tool {
            X11ClipboardTool::XClip => {
                let output = self.run_xclip_command(&["-selection", "clipboard", "-t", "text/html", "-o"], None)?;
                if output.is_empty() {
                    // Fallback auf Text
                    self.get_text()
                } else {
                    Ok(output)
                }
            },
            X11ClipboardTool::XSel => {
                // xsel unterstützt keine HTML-spezifischen Operationen, Fallback auf Text
                self.get_text()
            },
            X11ClipboardTool::None => {
                Err(ClipboardError::ClipboardUnavailable("No clipboard tool available".to_string()))
            }
        }
    }
    
    fn set_html(&mut self, html: &str) -> Result<(), ClipboardError> {
        match self.preferred_tool {
            X11ClipboardTool::XClip => {
                // Setze sowohl HTML als auch Text
                self.run_xclip_command(&["-selection", "clipboard", "-t", "text/html", "-i"], Some(html))?;
                
                // Zusätzlich als plain text setzen für Kompatibilität
                let text_content = html_to_text(html);
                self.run_xclip_command(&["-selection", "clipboard", "-t", "text/plain", "-i"], Some(&text_content))?;
                
                Ok(())
            },
            X11ClipboardTool::XSel => {
                // Konvertiere HTML zu Text und setze als Text
                let text_content = html_to_text(html);
                self.set_text(&text_content)
            },
            X11ClipboardTool::None => {
                Err(ClipboardError::ClipboardUnavailable("No clipboard tool available".to_string()))
            }
        }
    }
    
    fn get_files(&mut self) -> Result<Vec<String>, ClipboardError> {
        match self.preferred_tool {
            X11ClipboardTool::XClip => {
                // Versuche URI-Liste zu holen
                let output = self.run_xclip_command(&["-selection", "clipboard", "-t", "text/uri-list", "-o"], None)?;
                
                if output.is_empty() {
                    return Err(ClipboardError::EmptyClipboard);
                }
                
                // Parse URI-Liste
                let files: Vec<String> = output.lines()
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(|line| {
                        if line.starts_with("file://") {
                            line[7..].to_string() // Remove "file://" prefix
                        } else {
                            line.to_string()
                        }
                    })
                    .collect();
                
                Ok(files)
            },
            X11ClipboardTool::XSel => {
                Err(ClipboardError::UnsupportedOperation("File clipboard not supported with xsel".to_string()))
            },
            X11ClipboardTool::None => {
                Err(ClipboardError::ClipboardUnavailable("No clipboard tool available".to_string()))
            }
        }
    }
    
    fn is_available(&self) -> bool {
        matches!(self.preferred_tool, X11ClipboardTool::XClip | X11ClipboardTool::XSel)
    }
    
    fn create_clone(&self) -> Box<dyn ClipboardProvider> {
        Box::new(X11ClipboardProvider {
            has_xclip: self.has_xclip,
            has_xsel: self.has_xsel,
            preferred_tool: self.preferred_tool,
        })
    }
    
    fn get_available_formats(&self) -> Vec<String> {
        self.get_available_mime_types().unwrap_or_else(|_| vec!["text/plain".to_string()])
    }
}

/// Einfache HTML-zu-Text-Konvertierung
fn html_to_text(html: &str) -> String {
    // Einfache Regex-basierte HTML-Tag-Entfernung
    // Für komplexere HTML-Parsing wäre eine richtige HTML-Parser-Bibliothek besser
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    let text = re.replace_all(html, "");
    
    // HTML-Entitäten dekodieren (grundlegend)
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}
