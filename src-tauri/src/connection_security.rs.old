// src-tauri/src/connection_security.rs

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use base64::{Engine as _, engine::general_purpose};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

type HmacSha256 = Hmac<Sha256>;

// Typ-Aliase für bessere Lesbarkeit
pub type SessionId = String;
pub type Token = String;
pub type UserId = String;

// Sicherheitsfehler
#[derive(Debug)]
pub enum SecurityError {
    AuthenticationFailed(String),
    TokenInvalid(String),
    TokenExpired(String),
    PermissionDenied(String),
    EncryptionError(String),
    DecryptionError(String),
    ConfigurationError(String),
    ValidationError(String),
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::AuthenticationFailed(msg) => write!(f, "Authentifizierung fehlgeschlagen: {}", msg),
            SecurityError::TokenInvalid(msg) => write!(f, "Ungültiges Token: {}", msg),
            SecurityError::TokenExpired(msg) => write!(f, "Token abgelaufen: {}", msg),
            SecurityError::PermissionDenied(msg) => write!(f, "Zugriff verweigert: {}", msg),
            SecurityError::EncryptionError(msg) => write!(f, "Verschlüsselungsfehler: {}", msg),
            SecurityError::DecryptionError(msg) => write!(f, "Entschlüsselungsfehler: {}", msg),
            SecurityError::ConfigurationError(msg) => write!(f, "Konfigurationsfehler: {}", msg),
            SecurityError::ValidationError(msg) => write!(f, "Validierungsfehler: {}", msg),
        }
    }
}

impl Error for SecurityError {}

// Verbindungsmodi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionMode {
    Public,      // Öffentliche Verbindung, jeder kann beitreten
    Protected,   // Geschützt mit Passwort
    Authenticated, // Nur authentifizierte Benutzer
    Private,     // Nur für bestimmte Benutzer
}

// Zugriffsrechte
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessRight {
    ViewOnly,    // Nur Ansicht
    ControlInput, // Eingabesteuerung
    FileTransfer, // Dateiübertragung
    AudioAccess,  // Audiozugriff
    FullAccess,   // Vollzugriff
}

// Benutzerrollen
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Guest,      // Gast
    Member,     // Mitglied
    Moderator,  // Moderator
    Admin,      // Administrator
    Owner,      // Eigentümer
}

// Benutzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub role: UserRole,
    pub access_rights: Vec<AccessRight>,
}

// Session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub created_at: u64,
    pub expires_at: u64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

// JWT-Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId,              // Benutzer-ID
    pub session_id: SessionId,    // Sitzungs-ID
    pub role: UserRole,           // Rolle
    pub access_rights: Vec<AccessRight>, // Zugriffsrechte
    pub exp: u64,                 // Ablaufzeit
    pub iat: u64,                 // Ausstellungszeit
    pub iss: String,              // Aussteller
}

// Verbindungskonfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSecurityConfig {
    pub mode: ConnectionMode,
    pub password_hash: Option<String>,
    pub allowed_users: Option<Vec<UserId>>,
    pub default_access_rights: Vec<AccessRight>,
    pub session_timeout_minutes: u64,
    pub use_encryption: bool,
    pub max_failed_attempts: u32,
}

impl Default for ConnectionSecurityConfig {
    fn default() -> Self {
        ConnectionSecurityConfig {
            mode: ConnectionMode::Protected,
            password_hash: None,
            allowed_users: None,
            default_access_rights: vec![AccessRight::ViewOnly],
            session_timeout_minutes: 60,
            use_encryption: true,
            max_failed_attempts: 5,
        }
    }
}

// Verbindungssicherheitsmanager
pub struct ConnectionSecurityManager {
    config: Arc<Mutex<ConnectionSecurityConfig>>,
    secret_key: String,
    active_sessions: Arc<Mutex<Vec<Session>>>,
    failed_attempts: Arc<Mutex<std::collections::HashMap<String, (u32, u64)>>>, // IP -> (Anzahl, Zeitstempel)
}

impl ConnectionSecurityManager {
    pub fn new(secret_key: &str, config: ConnectionSecurityConfig) -> Self {
        // Stellen Sie sicher, dass der Secret-Key stark genug ist
        let mut actual_key = secret_key.to_string();
        if actual_key.len() < 32 {
            // Generiere einen starken Schlüssel, wenn der bereitgestellte zu schwach ist
            let random_suffix: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32 - actual_key.len())
                .map(char::from)
                .collect();
                
            actual_key = format!("{}{}", actual_key, random_suffix);
        }
        
        ConnectionSecurityManager {
            config: Arc::new(Mutex::new(config)),
            secret_key: actual_key,
            active_sessions: Arc::new(Mutex::new(Vec::new())),
            failed_attempts: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    // Zugangscode generieren
    pub fn generate_access_code() -> String {
        let code: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        
        code
    }
    
    // Passwort hashen
    pub fn hash_password(password: &str, salt: Option<&str>) -> String {
        let salt_value = salt.unwrap_or_else(|| {
            // Generiere zufälligen Salt, wenn keiner bereitgestellt wurde
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect::<String>()
                .as_str()
        });
        
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", salt_value, password));
        let result = hasher.finalize();
        
        format!("{}${}", salt_value, general_purpose::STANDARD.encode(result))
    }
    
    // Passwort verifizieren
    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        let parts: Vec<&str> = hash.split('$').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let salt = parts[0];
        let stored_hash = parts[1];
        
        let calculated_hash = Self::hash_password(password, Some(salt));
        let calculated_parts: Vec<&str> = calculated_hash.split('$').collect();
        
        if calculated_parts.len() != 2 {
            return false;
        }
        
        calculated_parts[1] == stored_hash
    }
    
    // JWT-Token generieren
    pub fn generate_token(&self, user: &User, ip_address: Option<&str>, user_agent: Option<&str>) -> Result<(Token, Session), SecurityError> {
        let config = self.config.lock().unwrap();
        
        // Prüfen, ob der Benutzer zugelassen ist
        if let ConnectionMode::Private = config.mode {
            if let Some(allowed_users) = &config.allowed_users {
                if !allowed_users.contains(&user.id) {
                    return Err(SecurityError::PermissionDenied(
                        "Benutzer nicht für diese Verbindung zugelassen".to_string()
                    ));
                }
            }
        }
        
        // Neue Sitzung erstellen
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SecurityError::ConfigurationError(format!("Systemzeit-Fehler: {}", e)))?
            .as_secs();
        
        let expires_at = now + (config.session_timeout_minutes * 60);
        
        let session_id = format!("session_{}", general_purpose::STANDARD.encode(thread_rng().gen::<[u8; 16]>()));
        
        let session = Session {
            id: session_id.clone(),
            user_id: user.id.clone(),
            created_at: now,
            expires_at,
            ip_address: ip_address.map(String::from),
            user_agent: user_agent.map(String::from),
        };
        
        // JWT-Claims erstellen
        let claims = Claims {
            sub: user.id.clone(),
            session_id: session.id.clone(),
            role: user.role.clone(),
            access_rights: user.access_rights.clone(),
            exp: expires_at,
            iat: now,
            iss: "SmolDesk".to_string(),
        };
        
        // Token generieren
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_ref())
        ).map_err(|e| SecurityError::EncryptionError(format!("Token-Erstellung fehlgeschlagen: {}", e)))?;
        
        // Sitzung speichern
        let mut sessions = self.active_sessions.lock().unwrap();
        sessions.push(session.clone());
        
        Ok((token, session))
    }
    
    // JWT-Token validieren
    pub fn validate_token(&self, token: &str) -> Result<Claims, SecurityError> {
        // Token dekodieren
        let validation = Validation::new(Algorithm::HS256);
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_ref()),
            &validation
        ).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    SecurityError::TokenExpired("Token ist abgelaufen".to_string())
                },
                _ => SecurityError::TokenInvalid(format!("Ungültiges Token: {}", e))
            }
        })?;
        
        let claims = token_data.claims;
        
        // Überprüfen, ob die Sitzung noch aktiv ist
        let sessions = self.active_sessions.lock().unwrap();
        let session_exists = sessions.iter().any(|s| s.id == claims.session_id && s.user_id == claims.sub);
        
        if !session_exists {
            return Err(SecurityError::TokenInvalid("Sitzung nicht gefunden oder ungültig".to_string()));
        }
        
        Ok(claims)
    }
    
    // Verbindung authentifizieren
    pub fn authenticate_connection(&self, mode: ConnectionMode, credentials: Option<&str>, user: Option<&User>, ip_address: Option<&str>) -> Result<bool, SecurityError> {
        let config = self.config.lock().unwrap();
        
        // Überprüfen, ob zu viele fehlgeschlagene Versuche vorliegen
        if let Some(ip) = ip_address {
            let mut failed_attempts = self.failed_attempts.lock().unwrap();
            if let Some((attempts, timestamp)) = failed_attempts.get(ip) {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| SecurityError::ConfigurationError(format!("Systemzeit-Fehler: {}", e)))?
                    .as_secs();
                
                // Wenn die letzte Anfrage innerhalb der letzten 15 Minuten war und die maximale Anzahl überschritten wurde
                if now - timestamp < 15 * 60 && *attempts >= config.max_failed_attempts {
                    return Err(SecurityError::AuthenticationFailed(
                        "Zu viele fehlgeschlagene Versuche. Bitte versuchen Sie es später erneut.".to_string()
                    ));
                }
            }
        }
        
        // Authentifizierung je nach Modus
        match mode {
            ConnectionMode::Public => {
                // Öffentliche Verbindung, keine Authentifizierung erforderlich
                Ok(true)
            },
            ConnectionMode::Protected => {
                // Geschützte Verbindung mit Passwort
                if let Some(password) = credentials {
                    if let Some(hash) = &config.password_hash {
                        let verified = self.verify_password(password, hash);
                        
                        if !verified && ip_address.is_some() {
                            self.record_failed_attempt(ip_address.unwrap())?;
                        }
                        
                        Ok(verified)
                    } else {
                        Err(SecurityError::ConfigurationError("Kein Passwort-Hash konfiguriert".to_string()))
                    }
                } else {
                    if ip_address.is_some() {
                        self.record_failed_attempt(ip_address.unwrap())?;
                    }
                    
                    Err(SecurityError::AuthenticationFailed("Passwort erforderlich".to_string()))
                }
            },
            ConnectionMode::Authenticated => {
                // Nur authentifizierte Benutzer
                if let Some(user_data) = user {
                    // Hier könnte eine erweiterte Benutzerauthentifizierung stattfinden
                    // Zum Beispiel könnte überprüft werden, ob der Benutzer in einer Datenbank existiert
                    // Für einfachheit akzeptieren wir jeden Benutzer mit einer ID
                    Ok(!user_data.id.is_empty())
                } else {
                    if ip_address.is_some() {
                        self.record_failed_attempt(ip_address.unwrap())?;
                    }
                    
                    Err(SecurityError::AuthenticationFailed("Benutzerauthentifizierung erforderlich".to_string()))
                }
            },
            ConnectionMode::Private => {
                // Nur für bestimmte Benutzer
                if let Some(user_data) = user {
                    if let Some(allowed_users) = &config.allowed_users {
                        let allowed = allowed_users.contains(&user_data.id);
                        
                        if !allowed && ip_address.is_some() {
                            self.record_failed_attempt(ip_address.unwrap())?;
                        }
                        
                        Ok(allowed)
                    } else {
                        Err(SecurityError::ConfigurationError("Keine zugelassenen Benutzer konfiguriert".to_string()))
                    }
                } else {
                    if ip_address.is_some() {
                        self.record_failed_attempt(ip_address.unwrap())?;
                    }
                    
                    Err(SecurityError::AuthenticationFailed("Benutzerauthentifizierung erforderlich".to_string()))
                }
            }
        }
    }
    
    // Fehlgeschlagenen Versuch protokollieren
    fn record_failed_attempt(&self, ip_address: &str) -> Result<(), SecurityError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SecurityError::ConfigurationError(format!("Systemzeit-Fehler: {}", e)))?
            .as_secs();
        
        let mut failed_attempts = self.failed_attempts.lock().unwrap();
        
        let entry = failed_attempts.entry(ip_address.to_string()).or_insert((0, now));
        entry.0 += 1;
        entry.1 = now;
        
        Ok(())
    }
    
    // Sitzung beenden
    pub fn terminate_session(&self, session_id: &str) -> Result<(), SecurityError> {
        let mut sessions = self.active_sessions.lock().unwrap();
        let initial_len = sessions.len();
        
        sessions.retain(|s| s.expires_at > now);
        
        let removed_count = initial_len - sessions.len();
        
        Ok(removed_count)
    }
    
    // Zugriffsrechte überprüfen
    pub fn check_access_rights(&self, claims: &Claims, required_rights: &[AccessRight]) -> bool {
        for right in required_rights {
            if !claims.access_rights.contains(right) {
                return false;
            }
        }
        
        true
    }
    
    // Minimale Zugriffsrechte basierend auf der Rolle zuweisen
    pub fn assign_default_rights_by_role(role: &UserRole) -> Vec<AccessRight> {
        match role {
            UserRole::Guest => {
                vec![AccessRight::ViewOnly]
            },
            UserRole::Member => {
                vec![AccessRight::ViewOnly, AccessRight::ControlInput]
            },
            UserRole::Moderator => {
                vec![AccessRight::ViewOnly, AccessRight::ControlInput, AccessRight::AudioAccess]
            },
            UserRole::Admin | UserRole::Owner => {
                vec![AccessRight::ViewOnly, AccessRight::ControlInput, AccessRight::FileTransfer, AccessRight::AudioAccess, AccessRight::FullAccess]
            }
        }
    }
    
    // Konfiguration aktualisieren
    pub fn update_config(&self, config: ConnectionSecurityConfig) {
        let mut current_config = self.config.lock().unwrap();
        *current_config = config;
    }
    
    // Sicherheitsrelevante Nachrichten signieren (HMAC-SHA256)
    pub fn sign_message(&self, message: &str) -> Result<String, SecurityError> {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .map_err(|e| SecurityError::EncryptionError(format!("HMAC-Initialisierungsfehler: {}", e)))?;
        
        mac.update(message.as_bytes());
        
        let result = mac.finalize();
        let signature = general_purpose::STANDARD.encode(result.into_bytes());
        
        Ok(signature)
    }
    
    // Signatur verifizieren
    pub fn verify_signature(&self, message: &str, signature: &str) -> Result<bool, SecurityError> {
        let signature_bytes = general_purpose::STANDARD.decode(signature)
            .map_err(|e| SecurityError::ValidationError(format!("Ungültige Signatur-Kodierung: {}", e)))?;
        
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .map_err(|e| SecurityError::EncryptionError(format!("HMAC-Initialisierungsfehler: {}", e)))?;
        
        mac.update(message.as_bytes());
        
        mac.verify_slice(&signature_bytes)
            .map(|_| true)
            .map_err(|_| SecurityError::ValidationError("Signaturverifizierung fehlgeschlagen".to_string()))
    }
    
    // Passwort für geschützte Verbindung setzen
    pub fn set_connection_password(&self, password: &str) -> Result<(), SecurityError> {
        let password_hash = Self::hash_password(password, None);
        
        let mut config = self.config.lock().unwrap();
        config.password_hash = Some(password_hash);
        config.mode = ConnectionMode::Protected;
        
        Ok(())
    }
    
    // Erlaubte Benutzer für private Verbindung festlegen
    pub fn set_allowed_users(&self, user_ids: Vec<UserId>) -> Result<(), SecurityError> {
        if user_ids.is_empty() {
            return Err(SecurityError::ConfigurationError("Liste der erlaubten Benutzer darf nicht leer sein".to_string()));
        }
        
        let mut config = self.config.lock().unwrap();
        config.allowed_users = Some(user_ids);
        config.mode = ConnectionMode::Private;
        
        Ok(())
    }
    
    // Aktive Sitzungen abrufen
    pub fn get_active_sessions(&self) -> Vec<Session> {
        let sessions = self.active_sessions.lock().unwrap();
        sessions.clone()
    }
    
    // Sitzung anhand der ID finden
    pub fn find_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.active_sessions.lock().unwrap();
        sessions.iter().find(|s| s.id == session_id).cloned()
    }
    
    // Aktuellen Verbindungsmodus abrufen
    pub fn get_connection_mode(&self) -> ConnectionMode {
        let config = self.config.lock().unwrap();
        config.mode.clone()
    }
    
    // Überprüfen, ob Verschlüsselung aktiviert ist
    pub fn is_encryption_enabled(&self) -> bool {
        let config = self.config.lock().unwrap();
        config.use_encryption
    }
}

// OAuth2 PKCE-Authentifizierung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKCEParams {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
}

pub struct OAuth2Manager {
    config: OAuthConfig,
}

impl OAuth2Manager {
    pub fn new(config: OAuthConfig) -> Self {
        OAuth2Manager {
            config,
        }
    }
    
    // PKCE-Parameter generieren
    pub fn generate_pkce_params(&self) -> Result<PKCEParams, SecurityError> {
        // Code-Verifier generieren (min. 43 Zeichen, max. 128 Zeichen)
        let code_verifier: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        
        // Code-Challenge aus Code-Verifier erstellen (SHA256)
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let challenge_bytes = hasher.finalize();
        let code_challenge = general_purpose::URL_SAFE_NO_PAD.encode(challenge_bytes);
        
        // State generieren (CSRF-Schutz)
        let state: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        
        Ok(PKCEParams {
            code_verifier,
            code_challenge,
            state,
        })
    }
    
    // Autorisierungs-URL erstellen
    pub fn build_authorization_url(&self, pkce_params: &PKCEParams) -> String {
        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
            self.config.auth_url,
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&self.config.scope),
            pkce_params.code_challenge,
            pkce_params.state
        )
    }
    
    // Token-Request erstellen
    pub fn build_token_request(&self, authorization_code: &str, code_verifier: &str) -> String {
        format!(
            "grant_type=authorization_code&client_id={}&code={}&redirect_uri={}&code_verifier={}",
            self.config.client_id,
            authorization_code,
            urlencoding::encode(&self.config.redirect_uri),
            code_verifier
        )
    }
}

// Frontend-Integrationsschnittstelle

/*
// In main.rs müssen die folgenden Tauri-Befehle hinzugefügt werden:

#[tauri::command]
fn initialize_security(secret_key: String, config: Option<ConnectionSecurityConfig>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Standard-Konfiguration verwenden, wenn keine bereitgestellt wird
    let security_config = config.unwrap_or_default();
    
    let security_manager = ConnectionSecurityManager::new(&secret_key, security_config);
    
    let mut app_security = state.connection_security.lock().unwrap();
    *app_security = Some(security_manager);
    
    Ok(())
}

#[tauri::command]
fn set_connection_password(password: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let security = state.connection_security.lock().unwrap();
    
    if let Some(manager) = &*security {
        manager.set_connection_password(&password)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    } else {
        Err("Sicherheitsmanager nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn generate_access_code(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let code = ConnectionSecurityManager::generate_access_code();
    Ok(code)
}

#[tauri::command]
fn authenticate(mode: String, credentials: Option<String>, user_data: Option<User>, ip_address: Option<String>, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let security = state.connection_security.lock().unwrap();
    
    if let Some(manager) = &*security {
        let connection_mode = match mode.as_str() {
            "public" => ConnectionMode::Public,
            "protected" => ConnectionMode::Protected,
            "authenticated" => ConnectionMode::Authenticated,
            "private" => ConnectionMode::Private,
            _ => return Err("Ungültiger Verbindungsmodus".to_string()),
        };
        
        manager.authenticate_connection(
            connection_mode,
            credentials.as_deref(),
            user_data.as_ref(),
            ip_address.as_deref()
        ).map_err(|e| e.to_string())
    } else {
        Err("Sicherheitsmanager nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn generate_user_token(user: User, ip_address: Option<String>, user_agent: Option<String>, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let security = state.connection_security.lock().unwrap();
    
    if let Some(manager) = &*security {
        let (token, _) = manager.generate_token(&user, ip_address.as_deref(), user_agent.as_deref())
            .map_err(|e| e.to_string())?;
        
        Ok(token)
    } else {
        Err("Sicherheitsmanager nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn validate_user_token(token: String, state: tauri::State<'_, AppState>) -> Result<Claims, String> {
    let security = state.connection_security.lock().unwrap();
    
    if let Some(manager) = &*security {
        manager.validate_token(&token)
            .map_err(|e| e.to_string())
    } else {
        Err("Sicherheitsmanager nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn sign_data(data: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let security = state.connection_security.lock().unwrap();
    
    if let Some(manager) = &*security {
        manager.sign_message(&data)
            .map_err(|e| e.to_string())
    } else {
        Err("Sicherheitsmanager nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn verify_data_signature(data: String, signature: String, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let security = state.connection_security.lock().unwrap();
    
    if let Some(manager) = &*security {
        manager.verify_signature(&data, &signature)
            .map_err(|e| e.to_string())
    } else {
        Err("Sicherheitsmanager nicht initialisiert".to_string())
    }
}

// OAuth2 PKCE-Befehle
#[tauri::command]
fn initialize_oauth(oauth_config: OAuthConfig, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let oauth_manager = OAuth2Manager::new(oauth_config);
    
    let mut app_oauth = state.oauth_manager.lock().unwrap();
    *app_oauth = Some(oauth_manager);
    
    Ok(())
}

#[tauri::command]
fn generate_pkce_params(state: tauri::State<'_, AppState>) -> Result<PKCEParams, String> {
    let oauth = state.oauth_manager.lock().unwrap();
    
    if let Some(manager) = &*oauth {
        manager.generate_pkce_params()
            .map_err(|e| e.to_string())
    } else {
        Err("OAuth-Manager nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn get_authorization_url(pkce_params: PKCEParams, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let oauth = state.oauth_manager.lock().unwrap();
    
    if let Some(manager) = &*oauth {
        Ok(manager.build_authorization_url(&pkce_params))
    } else {
        Err("OAuth-Manager nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn build_token_request(authorization_code: String, code_verifier: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let oauth = state.oauth_manager.lock().unwrap();
    
    if let Some(manager) = &*oauth {
        Ok(manager.build_token_request(&authorization_code, &code_verifier))
    } else {
        Err("OAuth-Manager nicht initialisiert".to_string())
    }
}
*/.len();
        
        sessions.retain(|s| s.id != session_id);
        
        if sessions.len() == initial_len {
            return Err(SecurityError::ValidationError("Sitzung nicht gefunden".to_string()));
        }
        
        Ok(())
    }
    
    // Alle Sitzungen eines Benutzers beenden
    pub fn terminate_user_sessions(&self, user_id: &str) -> Result<usize, SecurityError> {
        let mut sessions = self.active_sessions.lock().unwrap();
        let initial_len = sessions.len();
        
        sessions.retain(|s| s.user_id != user_id);
        
        let terminated_count = initial_len - sessions.len();
        
        Ok(terminated_count)
    }
    
    // Abgelaufene Sitzungen bereinigen
    pub fn cleanup_expired_sessions(&self) -> Result<usize, SecurityError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SecurityError::ConfigurationError(format!("Systemzeit-Fehler: {}", e)))?
            .as_secs();
        
        let mut sessions = self.active_sessions.lock().unwrap();
        let initial_len = sessions
