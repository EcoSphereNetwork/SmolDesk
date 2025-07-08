import React from 'react';
import renderer from 'react-test-renderer';

jest.mock('react-native-webrtc', () => ({ RTCView: 'RTCView', MediaStream: class {} }));
jest.mock('../src/input/touchToMouse');
jest.mock('../src/services/files');
jest.mock('react-native-document-picker', () => ({
  __esModule: true,
  default: { pick: jest.fn() },
}));
jest.mock('react-native-fs', () => ({ readFile: jest.fn() }));
jest.mock('react-native-reanimated');

const ViewerScreen = require('../src/screens/ViewerScreen').default;

it('renders toolbar buttons', () => {
  const service = {
    disconnect: jest.fn(),
    sendRaw: jest.fn(),
    sendData: jest.fn(),
    on: jest.fn(),
  } as any;
  const signaling = { on: jest.fn() } as any;
  const stream: any = { toURL: () => 'test' };
  const tree = renderer.create(
    <ViewerScreen stream={stream} service={service} signaling={signaling} onDisconnect={() => {}} />
  ).toJSON();
  expect(tree).toBeTruthy();
});
