// src/utils/securityManager.ts

import { invoke } from '@tauri-apps/api/tauri';
import { nanoid } from 'nanoid';

/**
 * OAuth2 PKCE Parameter Interface
 */
export interface PKCEParams {
  code_verifier: string;
  code_challenge: string;
  state: string;
}

/**
 * OAuth2 Configuration
 */
export interface OAuthConfig {
  client_id: string;
  auth_url: string;
  token_url: string;
  redirect_uri: string;
  scope: string;
}

/**
 * User Information
 */
export interface User {
  id: string;
  username: string;
  role: 'Guest' | 'Member' | 'Moderator' | 'Admin' | 'Owner';
  access_rights: Array<'ViewOnly' | 'ControlInput' | 'FileTransfer' | 'AudioAccess' | 'FullAccess'>;
}

/**
 * Connection Mode
 */
export enum ConnectionMode {
  Public = 'Public',
  Protected = 'Protected',
  Authenticated = 'Authenticated',
  Private = 'Private',
}

/**
 * Security Manager for handling authentication, message signing, and security state
 */
export class SecurityManager {
  private static instance: SecurityManager;
  private isInitialized: boolean = false;
  private currentUser: User | null = null;
  private securityToken: string | null = null;
  private securityMode: ConnectionMode = ConnectionMode.Protected;
  private pkceParams: PKCEParams | null = null;
  private oauthConfig: OAuthConfig | null = null;

  /**
   * Get the singleton instance
   */
  public static getInstance(): SecurityManager {
    if (!SecurityManager.instance) {
      SecurityManager.instance = new SecurityManager();
    }
    return SecurityManager.instance;
  }

  private constructor() {
    // Private constructor to enforce singleton pattern
  }

  /**
   * Initialize the security manager
   */
  public async initialize(secretKey: string, connectionMode: ConnectionMode = ConnectionMode.Protected): Promise<boolean> {
    try {
      // Initialize backend security with Tauri
      await invoke('initialize_security', { 
        secretKey, 
        config: {
          mode: connectionMode,
          password_hash: null,
          allowed_users: null,
          default_access_rights: ['ViewOnly'],
          session_timeout_minutes: 60,
          use_encryption: true,
          max_failed_attempts: 5,
        }
      });
      
      this.isInitialized = true;
      this.securityMode = connectionMode;
      
      return true;
    } catch (error) {
      console.error('Failed to initialize security manager:', error);
      return false;
    }
  }

  /**
   * Set a password for protected connections
   */
  public async setConnectionPassword(password: string): Promise<boolean> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return false;
    }

    try {
      await invoke('set_connection_password', { password });
      this.securityMode = ConnectionMode.Protected;
      return true;
    } catch (error) {
      console.error('Failed to set connection password:', error);
      return false;
    }
  }

  /**
   * Generate a random access code
   */
  public async generateAccessCode(): Promise<string | null> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return null;
    }

    try {
      const code = await invoke<string>('generate_access_code');
      return code;
    } catch (error) {
      console.error('Failed to generate access code:', error);
      return null;
    }
  }

  /**
   * Authenticate a user
   */
  public async authenticate(
    mode: ConnectionMode,
    credentials?: string,
    user?: User,
    ipAddress?: string
  ): Promise<boolean> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return false;
    }

    try {
      const result = await invoke<boolean>('authenticate', {
        mode,
        credentials,
        userData: user,
        ipAddress,
      });

      if (result && user) {
        this.currentUser = user;
      }

      return result;
    } catch (error) {
      console.error('Failed to authenticate:', error);
      return false;
    }
  }

  /**
   * Generate a JWT token for the current user
   */
  public async generateToken(ipAddress?: string, userAgent?: string): Promise<string | null> {
    if (!this.isInitialized || !this.currentUser) {
      console.error('Security manager not initialized or no current user');
      return null;
    }

    try {
      const token = await invoke<string>('generate_user_token', {
        user: this.currentUser,
        ipAddress,
        userAgent,
      });

      this.securityToken = token;
      return token;
    } catch (error) {
      console.error('Failed to generate token:', error);
      return null;
    }
  }

  /**
   * Validate a security token
   */
  public async validateToken(token: string): Promise<boolean> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return false;
    }

    try {
      const claims = await invoke('validate_user_token', { token });
      return claims !== null;
    } catch (error) {
      console.error('Failed to validate token:', error);
      return false;
    }
  }

  /**
   * Sign data with the security key
   */
  public async signData(data: string): Promise<string | null> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return null;
    }

    try {
      const signature = await invoke<string>('sign_data', { data });
      return signature;
    } catch (error) {
      console.error('Failed to sign data:', error);
      return null;
    }
  }

  /**
   * Verify data signature
   */
  public async verifySignature(data: string, signature: string): Promise<boolean> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return false;
    }

    try {
      const isValid = await invoke<boolean>('verify_data_signature', {
        data,
        signature,
      });
      return isValid;
    } catch (error) {
      console.error('Failed to verify signature:', error);
      return false;
    }
  }

  /**
   * Initialize OAuth2 PKCE authentication
   */
  public async initializeOAuth(config: OAuthConfig): Promise<boolean> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return false;
    }

    try {
      await invoke('initialize_oauth', { oauthConfig: config });
      this.oauthConfig = config;
      return true;
    } catch (error) {
      console.error('Failed to initialize OAuth:', error);
      return false;
    }
  }

  /**
   * Generate PKCE parameters for OAuth2
   */
  public async generatePKCEParams(): Promise<PKCEParams | null> {
    if (!this.isInitialized || !this.oauthConfig) {
      console.error('Security manager not initialized or OAuth not configured');
      return null;
    }

    try {
      const params = await invoke<PKCEParams>('generate_pkce_params');
      this.pkceParams = params;
      return params;
    } catch (error) {
      console.error('Failed to generate PKCE params:', error);
      
      // Fallback client-side implementation if backend is not available
      return this.generateClientSidePKCE();
    }
  }

  /**
   * Get the authorization URL for OAuth2 authentication
   */
  public async getAuthorizationURL(): Promise<string | null> {
    if (!this.isInitialized || !this.oauthConfig || !this.pkceParams) {
      console.error('Security manager not initialized, OAuth not configured, or PKCE params not generated');
      return null;
    }

    try {
      const url = await invoke<string>('get_authorization_url', {
        pkceParams: this.pkceParams,
      });
      return url;
    } catch (error) {
      console.error('Failed to get authorization URL:', error);
      
      // Fallback client-side implementation
      if (this.oauthConfig && this.pkceParams) {
        return this.buildAuthorizationUrl(this.oauthConfig, this.pkceParams);
      }
      
      return null;
    }
  }

  /**
   * Build a token request for OAuth2
   */
  public async buildTokenRequest(authorizationCode: string): Promise<string | null> {
    if (!this.isInitialized || !this.oauthConfig || !this.pkceParams) {
      console.error('Security manager not initialized, OAuth not configured, or PKCE params not generated');
      return null;
    }

    try {
      const request = await invoke<string>('build_token_request', {
        authorizationCode,
        codeVerifier: this.pkceParams.code_verifier,
      });
      return request;
    } catch (error) {
      console.error('Failed to build token request:', error);
      
      // Fallback client-side implementation
      if (this.oauthConfig && this.pkceParams) {
        return this.buildTokenRequestClientSide(
          this.oauthConfig,
          authorizationCode,
          this.pkceParams.code_verifier
        );
      }
      
      return null;
    }
  }

  /**
   * Create a secure connection room with security features
   */
  public async createSecureRoom(password?: string): Promise<string | null> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return null;
    }
    
    // Generate a random room ID
    const roomId = nanoid(10);
    
    // If in protected mode, set the password
    if (this.securityMode === ConnectionMode.Protected && password) {
      const success = await this.setConnectionPassword(password);
      if (!success) {
        return null;
      }
    }
    
    // Sign the room ID to ensure it hasn't been tampered with
    const signature = await this.signData(roomId);
    if (!signature) {
      return null;
    }
    
    // Return room ID with signature for verification on join
    return `${roomId}:${signature}`;
  }

  /**
   * Join a secure room with verification
   */
  public async joinSecureRoom(
    secureRoomId: string, 
    password?: string, 
    user?: User
  ): Promise<boolean> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return false;
    }
    
    // Split the room ID and signature
    const parts = secureRoomId.split(':');
    if (parts.length !== 2) {
      console.error('Invalid secure room ID format');
      return false;
    }
    
    const [roomId, signature] = parts;
    
    // Verify the signature
    const isValid = await this.verifySignature(roomId, signature);
    if (!isValid) {
      console.error('Room ID signature verification failed');
      return false;
    }
    
    // Authenticate based on the security mode
    return await this.authenticate(this.securityMode, password, user);
  }

  /**
   * Encrypt data for secure transmission
   */
  public async encryptData(data: string): Promise<string | null> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return null;
    }
    
    // This would call a Tauri command for encryption, but since that's not implemented yet,
    // we'll use a simple signature-based approach for now
    const signature = await this.signData(data);
    if (!signature) {
      return null;
    }
    
    // Return data with signature
    const payload = {
      data,
      signature,
      timestamp: Date.now()
    };
    
    return JSON.stringify(payload);
  }

  /**
   * Decrypt and verify data
   */
  public async decryptAndVerify(encryptedData: string): Promise<string | null> {
    if (!this.isInitialized) {
      console.error('Security manager not initialized');
      return null;
    }
    
    try {
      // Parse the payload
      const payload = JSON.parse(encryptedData);
      const { data, signature, timestamp } = payload;
      
      // Check for expired messages (older than 5 minutes)
      const now = Date.now();
      if (now - timestamp > 5 * 60 * 1000) {
        console.error('Message expired');
        return null;
      }
      
      // Verify the signature
      const isValid = await this.verifySignature(data, signature);
      if (!isValid) {
        console.error('Signature verification failed');
        return null;
      }
      
      return data;
    } catch (error) {
      console.error('Failed to decrypt and verify data:', error);
      return null;
    }
  }

  // Helper methods for client-side fallbacks
  
  /**
   * Generate PKCE parameters client-side
   */
  private generateClientSidePKCE(): PKCEParams {
    // Generate a random code verifier (43-128 characters)
    const codeVerifier = this.generateRandomString(64);
    
    // Create code challenge using SHA-256
    const codeChallenge = this.generateCodeChallenge(codeVerifier);
    
    // Generate a random state for CSRF protection
    const state = this.generateRandomString(32);
    
    return {
      code_verifier: codeVerifier,
      code_challenge: codeChallenge,
      state,
    };
  }
  
  /**
   * Generate a random string for PKCE
   */
  private generateRandomString(length: number): string {
    const array = new Uint8Array(length);
    crypto.getRandomValues(array);
    return Array.from(array)
      .map(byte => String.fromCharCode(byte % 26 + 97))
      .join('');
  }
  
  /**
   * Generate code challenge from code verifier using SHA-256
   */
  private async generateCodeChallenge(codeVerifier: string): Promise<string> {
    const encoder = new TextEncoder();
    const data = encoder.encode(codeVerifier);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    
    // Base64url encode the hash
    return btoa(String.fromCharCode(...new Uint8Array(hashBuffer)))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=+$/, '');
  }
  
  /**
   * Build authorization URL client-side
   */
  private buildAuthorizationUrl(config: OAuthConfig, pkceParams: PKCEParams): string {
    const url = new URL(config.auth_url);
    
    url.searchParams.append('response_type', 'code');
    url.searchParams.append('client_id', config.client_id);
    url.searchParams.append('redirect_uri', encodeURIComponent(config.redirect_uri));
    url.searchParams.append('scope', encodeURIComponent(config.scope));
    url.searchParams.append('code_challenge', pkceParams.code_challenge);
    url.searchParams.append('code_challenge_method', 'S256');
    url.searchParams.append('state', pkceParams.state);
    
    return url.toString();
  }
  
  /**
   * Build token request client-side
   */
  private buildTokenRequestClientSide(
    config: OAuthConfig, 
    authorizationCode: string, 
    codeVerifier: string
  ): string {
    const params = new URLSearchParams();
    
    params.append('grant_type', 'authorization_code');
    params.append('client_id', config.client_id);
    params.append('code', authorizationCode);
    params.append('redirect_uri', config.redirect_uri);
    params.append('code_verifier', codeVerifier);
    
    return params.toString();
  }
  
  /**
   * Get the current security mode
   */
  public getSecurityMode(): ConnectionMode {
    return this.securityMode;
  }
  
  /**
   * Get the current user
   */
  public getCurrentUser(): User | null {
    return this.currentUser;
  }
  
  /**
   * Check if the security manager is initialized
   */
  public isSecurityInitialized(): boolean {
    return this.isInitialized;
  }
}
