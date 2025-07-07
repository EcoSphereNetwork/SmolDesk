import fs from 'fs';
import os from 'os';
import path from 'path';
import { EventEmitter } from 'events';
import FileTransferService from '../src/services/files';

jest.mock('react-native-document-picker', () => ({ pickSingle: jest.fn() }));

jest.mock('react-native-fs', () => {
  const fs = require('fs');
  const os = require('os');
  return {
    readFile: (p: string, enc: string) => fs.readFileSync(p, enc),
    writeFile: (p: string, data: string, enc: string) => fs.writeFileSync(p, data, enc),
    DownloadDirectoryPath: os.tmpdir(),
  };
});

test('sends header and chunks', async () => {
  class MockRTC extends EventEmitter {
    sent: any[] = [];
    sendData(msg: any) { this.sent.push(msg); }
    sendRaw(text: string) { this.sent.push(JSON.parse(text)); }
  }
  const rtc = new MockRTC();
  const svc = new FileTransferService(rtc as any);
  const tmp = path.join(os.tmpdir(), 'f.txt');
  fs.writeFileSync(tmp, 'hello');
  await svc.sendFile(tmp, 'f.txt', 'text/plain', 5);
  expect(rtc.sent[0].type).toBe('file_header');
  expect(rtc.sent[1].type).toBe('file_chunk');
  fs.unlinkSync(tmp);
});

test('reassembles received file', () => {
  class MockRTC extends EventEmitter {
    sent: any[] = [];
    sendData() {}
    sendRaw() {}
  }
  const rtc = new MockRTC();
  const svc = new FileTransferService(rtc as any);
  const id = '1';
  const header = { type: 'file_header', id, name: 'g.txt', mime: 'text/plain', size: 4 };
  const chunk = { type: 'file_chunk', id, data: Buffer.from('test').toString('base64') };
  const end = { type: 'file_end', id };
  rtc.emit('data', header);
  rtc.emit('data', chunk);
  rtc.emit('data', end);
  const p = path.join(os.tmpdir(), 'g.txt');
  expect(fs.readFileSync(p, 'utf8')).toBe('test');
  fs.unlinkSync(p);
});
