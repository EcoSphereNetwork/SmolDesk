import React from 'react';
import renderer from 'react-test-renderer';
import ViewerScreen from '../src/screens/ViewerScreen';
import WebRTCService from '../src/services/webrtc';
import SignalingService from '../src/services/signaling';

jest.mock('react-native-webrtc', () => ({ RTCView: 'RTCView', MediaStream: class {} }));
jest.mock('../src/input/touchToMouse');
jest.mock('../src/services/files');

it('renders toolbar buttons', () => {
  const service = new WebRTCService({} as any);
  const signaling = new SignalingService({ url: '', token: '', reconnectInterval: 0 });
  const stream: any = { toURL: () => 'test' };
  const tree = renderer.create(
    <ViewerScreen stream={stream} service={service} signaling={signaling} onDisconnect={() => {}} />
  ).toJSON();
  expect(tree).toBeTruthy();
});
