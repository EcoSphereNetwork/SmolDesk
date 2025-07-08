import WebSocket, { Server } from 'ws';
import SignalingService from '../src/services/signaling';

// Node's WebSocket implementation for React Native polyfill
// ensures SignalingService can instantiate WebSocket during tests.
global.WebSocket = WebSocket as any;

describe('SignalingService', () => {
  test('sends auth token and receives messages', (done) => {
    const server = new Server({ port: 12345 }, () => {
      server.on('connection', (ws) => {
        ws.on('message', (data) => {
          const msg = JSON.parse(String(data));
          expect(msg).toEqual({ type: 'auth', token: 'test' });
          ws.send(JSON.stringify({ type: 'authorized' }));
          ws.send(JSON.stringify({ type: 'welcome' }));
        });
      });
      const service = new SignalingService({ url: 'ws://localhost:12345', token: 'test', reconnectInterval: 0 });
      service.on('authorized', () => {
        // authorized event before welcome
      });
      service.on('error', () => {});
      service.on('message', (msg) => {
        expect(msg.type).toBe('welcome');
        service.disconnect();
        server.close(() => done());
      });
      service.connect();
    });
  });
});
