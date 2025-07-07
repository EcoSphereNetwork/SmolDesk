export const DEFAULT_SIGNALING_SERVER = 'wss://signaling.smoldesk.example';

export const OAUTH_CONFIG = {
  issuer: 'https://auth.example.com',
  clientId: 'smoldesk-mobile',
  redirectUrl: 'smoldesk://callback',
  scopes: ['openid', 'profile'],
};

export const HMAC_ENABLED = false;
export const HMAC_KEY = '';

export const ENCRYPTION_KEY_SALT = 'smoldesk-salt';
