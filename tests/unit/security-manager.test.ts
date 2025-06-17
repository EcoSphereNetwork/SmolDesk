// =============================================================================
// tests/unit/security-manager.test.ts - Security Manager Tests
// =============================================================================

import { describe, test, expect, beforeEach, vi, Mock } from 'vitest';
import { SecurityManager, ConnectionMode, User } from '../../src/utils/securityManager';
import { invoke } from '@tauri-apps/api/tauri';
vi.mock('nanoid', () => ({
  nanoid: vi.fn(() => 'test-room-id')
}));

// Mock crypto.getRandomValues
Object.defineProperty(global, 'crypto', {
  value: {
    getRandomValues: vi.fn((arr) => {
      for (let i = 0; i < arr.length; i++) {
        arr[i] = Math.floor(Math.random() * 256);
      }
      return arr;
    }),
    subtle: {
      digest: vi.fn().mockResolvedValue(new ArrayBuffer(32))
    }
  },
  writable: true
});

describe('SecurityManager', () => {
  let securityManager: SecurityManager;
  let mockInvoke: Mock;

  beforeEach(() => {
    mockInvoke = invoke as Mock;
    vi.clearAllMocks();
    
    // Reset singleton
    SecurityManager['instance'] = undefined as any;
    securityManager = SecurityManager.getInstance();
    
    mockInvoke.mockResolvedValue(true);
  });

  describe('Singleton Pattern', () => {
    test('should return same instance', () => {
      const instance1 = SecurityManager.getInstance();
      const instance2 = SecurityManager.getInstance();
      
      expect(instance1).toBe(instance2);
    });
  });

  describe('Initialization', () => {
    test('should initialize security manager successfully', async () => {
      const result = await securityManager.initialize('test-key', ConnectionMode.Protected);
      
      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('initialize_security', {
        secretKey: 'test-key',
        config: expect.objectContaining({
          mode: ConnectionMode.Protected,
          use_encryption: true
        })
      });
    });

    test('should handle initialization failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Init failed'));
      
      const result = await securityManager.initialize('test-key');
      
      expect(result).toBe(false);
    });

    test('should use default connection mode', async () => {
      await securityManager.initialize('test-key');
      
      expect(mockInvoke).toHaveBeenCalledWith('initialize_security', {
        secretKey: 'test-key',
        config: expect.objectContaining({
          mode: ConnectionMode.Protected
        })
      });
    });
  });

  describe('Password Management', () => {
    beforeEach(async () => {
      await securityManager.initialize('test-key');
    });

    test('should set connection password', async () => {
      const result = await securityManager.setConnectionPassword('password123');
      
      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('set_connection_password', {
        password: 'password123'
      });
    });

    test('should handle password setting failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Password failed'));
      
      const result = await securityManager.setConnectionPassword('password123');
      
      expect(result).toBe(false);
    });

    test('should require initialization for password setting', async () => {
      const uninitializedManager = new (SecurityManager as any)();
      
      const result = await uninitializedManager.setConnectionPassword('password123');
      
      expect(result).toBe(false);
    });
  });

  describe('Access Code Generation', () => {
    beforeEach(async () => {
      await securityManager.initialize('test-key');
    });

    test('should generate access code', async () => {
      mockInvoke.mockResolvedValueOnce('ABC123');
      
      const code = await securityManager.generateAccessCode();
      
      expect(code).toBe('ABC123');
      expect(mockInvoke).toHaveBeenCalledWith('generate_access_code');
    });

    test('should handle access code generation failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Generation failed'));
      
      const code = await securityManager.generateAccessCode();
      
      expect(code).toBeNull();
    });
  });

  describe('Authentication', () => {
    const testUser: User = {
      id: 'user-1',
      username: 'testuser',
      role: 'Member',
      access_rights: ['ViewOnly', 'ControlInput']
    };

    beforeEach(async () => {
      await securityManager.initialize('test-key');
    });

    test('should authenticate user successfully', async () => {
      const result = await securityManager.authenticate(
        ConnectionMode.Protected,
        'password123',
        testUser,
        '192.168.1.1'
      );
      
      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('authenticate', {
        mode: ConnectionMode.Protected,
        credentials: 'password123',
        userData: testUser,
        ipAddress: '192.168.1.1'
      });
    });

    test('should handle authentication failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Auth failed'));
      
      const result = await securityManager.authenticate(
        ConnectionMode.Protected,
        'wrongpassword',
        testUser
      );
      
      expect(result).toBe(false);
    });

    test('should require initialization for authentication', async () => {
      const uninitializedManager = new (SecurityManager as any)();
      
      const result = await uninitializedManager.authenticate(
        ConnectionMode.Protected,
        'password123',
        testUser
      );
      
      expect(result).toBe(false);
    });
  });

  describe('Token Management', () => {
    const testUser: User = {
      id: 'user-1',
      username: 'testuser',
      role: 'Member',
      access_rights: ['ViewOnly']
    };

    beforeEach(async () => {
      await securityManager.initialize('test-key');
      await securityManager.authenticate(ConnectionMode.Protected, 'password', testUser);
    });

    test('should generate token for authenticated user', async () => {
      mockInvoke.mockResolvedValueOnce('jwt-token-123');
      
      const token = await securityManager.generateToken('192.168.1.1', 'test-agent');
      
      expect(token).toBe('jwt-token-123');
      expect(mockInvoke).toHaveBeenCalledWith('generate_user_token', {
        user: testUser,
        ipAddress: '192.168.1.1',
        userAgent: 'test-agent'
      });
    });

    test('should validate token', async () => {
      mockInvoke.mockResolvedValueOnce({ sub: 'user-1', exp: Date.now() + 3600000 });
      
      const result = await securityManager.validateToken('jwt-token-123');
      
      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('validate_user_token', {
        token: 'jwt-token-123'
      });
    });

    test('should handle invalid token', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid token'));
      
      const result = await securityManager.validateToken('invalid-token');
      
      expect(result).toBe(false);
    });
  });

  describe('Data Signing and Verification', () => {
    beforeEach(async () => {
      await securityManager.initialize('test-key');
    });

    test('should sign data', async () => {
      mockInvoke.mockResolvedValueOnce('signature-123');
      
      const signature = await securityManager.signData('test-data');
      
      expect(signature).toBe('signature-123');
      expect(mockInvoke).toHaveBeenCalledWith('sign_data', {
        data: 'test-data'
      });
    });

    test('should verify signature', async () => {
      const result = await securityManager.verifySignature('test-data', 'signature-123');
      
      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('verify_data_signature', {
        data: 'test-data',
        signature: 'signature-123'
      });
    });

    test('should handle signing failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Signing failed'));
      
      const signature = await securityManager.signData('test-data');
      
      expect(signature).toBeNull();
    });
  });

  describe('OAuth2 PKCE', () => {
    const oauthConfig = {
      client_id: 'test-client',
      auth_url: 'https://auth.example.com/oauth/authorize',
      token_url: 'https://auth.example.com/oauth/token',
      redirect_uri: 'http://localhost:3000/callback',
      scope: 'read write'
    };

    beforeEach(async () => {
      await securityManager.initialize('test-key');
    });

    test('should initialize OAuth configuration', async () => {
      const result = await securityManager.initializeOAuth(oauthConfig);
      
      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('initialize_oauth', {
        oauthConfig
      });
    });

    test('should generate PKCE parameters', async () => {
      const mockParams = {
        code_verifier: 'verifier-123',
        code_challenge: 'challenge-123',
        state: 'state-123'
      };
      
      mockInvoke.mockResolvedValueOnce(mockParams);
      await securityManager.initializeOAuth(oauthConfig);
      
      const params = await securityManager.generatePKCEParams();
      
      expect(params).toEqual(mockParams);
    });

    test('should build authorization URL', async () => {
      const mockParams = {
        code_verifier: 'verifier-123',
        code_challenge: 'challenge-123',
        state: 'state-123'
      };
      
      mockInvoke
        .mockResolvedValueOnce(true) // initializeOAuth
        .mockResolvedValueOnce(mockParams) // generatePKCEParams
        .mockResolvedValueOnce('https://auth.example.com/oauth/authorize?...'); // getAuthorizationURL
      
      await securityManager.initializeOAuth(oauthConfig);
      await securityManager.generatePKCEParams();
      
      const url = await securityManager.getAuthorizationURL();
      
      expect(url).toBe('https://auth.example.com/oauth/authorize?...');
    });

    test('should fallback to client-side PKCE generation', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Backend not available'));
      await securityManager.initializeOAuth(oauthConfig);
      
      const params = await securityManager.generatePKCEParams();
      
      expect(params).toHaveProperty('code_verifier');
      expect(params).toHaveProperty('code_challenge');
      expect(params).toHaveProperty('state');
    });
  });

  describe('Secure Room Management', () => {
    beforeEach(async () => {
      await securityManager.initialize('test-key');
    });

    test('should create secure room', async () => {
      mockInvoke.mockResolvedValueOnce('signature-123');
      
      const roomId = await securityManager.createSecureRoom('password123');
      
      expect(roomId).toBe('test-room-id:signature-123');
      expect(mockInvoke).toHaveBeenCalledWith('set_connection_password', {
        password: 'password123'
      });
    });

    test('should join secure room', async () => {
      mockInvoke
        .mockResolvedValueOnce(true) // verifySignature
        .mockResolvedValueOnce(true); // authenticate
      
      const result = await securityManager.joinSecureRoom(
        'test-room-id:signature-123',
        'password123'
      );
      
      expect(result).toBe(true);
    });

    test('should reject invalid room ID format', async () => {
      const result = await securityManager.joinSecureRoom('invalid-format');
      
      expect(result).toBe(false);
    });

    test('should reject tampered room ID', async () => {
      mockInvoke.mockResolvedValueOnce(false); // verifySignature returns false
      
      const result = await securityManager.joinSecureRoom(
        'test-room-id:invalid-signature'
      );
      
      expect(result).toBe(false);
    });
  });

  describe('Data Encryption/Decryption', () => {
    beforeEach(async () => {
      await securityManager.initialize('test-key');
    });

    test('should encrypt data', async () => {
      mockInvoke.mockResolvedValueOnce('signature-123');
      
      const encrypted = await securityManager.encryptData('test-data');
      
      expect(encrypted).toBeTruthy();
      
      const payload = JSON.parse(encrypted!);
      expect(payload).toHaveProperty('data', 'test-data');
      expect(payload).toHaveProperty('signature', 'signature-123');
      expect(payload).toHaveProperty('timestamp');
    });

    test('should decrypt and verify data', async () => {
      mockInvoke
        .mockResolvedValueOnce('signature-123') // encrypt
        .mockResolvedValueOnce(true); // verify signature
      
      const encrypted = await securityManager.encryptData('test-data');
      const decrypted = await securityManager.decryptAndVerify(encrypted!);
      
      expect(decrypted).toBe('test-data');
    });

    test('should reject expired messages', async () => {
      const expiredPayload = JSON.stringify({
        data: 'test-data',
        signature: 'signature-123',
        timestamp: Date.now() - (6 * 60 * 1000) // 6 minutes ago
      });
      
      const result = await securityManager.decryptAndVerify(expiredPayload);
      
      expect(result).toBeNull();
    });

    test('should reject tampered messages', async () => {
      mockInvoke.mockResolvedValueOnce(false); // signature verification fails
      
      const payload = JSON.stringify({
        data: 'test-data',
        signature: 'invalid-signature',
        timestamp: Date.now()
      });
      
      const result = await securityManager.decryptAndVerify(payload);
      
      expect(result).toBeNull();
    });
  });

  describe('State Management', () => {
    test('should track security mode', () => {
      expect(securityManager.getSecurityMode()).toBe(ConnectionMode.Protected);
    });

    test('should track current user', async () => {
      const testUser: User = {
        id: 'user-1',
        username: 'testuser',
        role: 'Member',
        access_rights: ['ViewOnly']
      };
      
      await securityManager.initialize('test-key');
      await securityManager.authenticate(ConnectionMode.Protected, 'password', testUser);
      
      expect(securityManager.getCurrentUser()).toEqual(testUser);
    });

    test('should track initialization state', () => {
      expect(securityManager.isSecurityInitialized()).toBe(false);
      
      return securityManager.initialize('test-key').then(() => {
        expect(securityManager.isSecurityInitialized()).toBe(true);
      });
    });
  });

  describe('Error Handling', () => {
    test('should handle operations without initialization', async () => {
      const result = await securityManager.generateAccessCode();
      expect(result).toBeNull();
    });

    test('should handle token generation without user', async () => {
      await securityManager.initialize('test-key');
      
      const token = await securityManager.generateToken();
      expect(token).toBeNull();
    });

    test('should handle malformed encrypted data', async () => {
      await securityManager.initialize('test-key');
      
      const result = await securityManager.decryptAndVerify('invalid-json');
      expect(result).toBeNull();
    });
  });
});
