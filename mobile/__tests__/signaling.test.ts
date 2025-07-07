import WebSocket, { Server } from 'ws';
import SignalingService from '../src/services/signaling';

// Node's WebSocket implementation for React Native polyfill
// ensures SignalingService can instantiate WebSocket during tests.
global.WebSocket = WebSocket as any;

describe('SignalingService', () => {
  test('connects and handles messages', (done) => {
    const server = new Server({ port: 12345 }, () => {
      const service = new SignalingService({ url: 'ws://localhost:12345' });
      service.on('open', () => {
        server.clients.forEach((ws) => ws.send(JSON.stringify({ type: 'welcome' })));
      });
      service.on('message', (msg) => {
        expect(msg.type).toBe('welcome');
        server.close();
        done();
      });
      service.connect();
    });
  });
});
