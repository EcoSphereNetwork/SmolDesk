import WebRTCService from '../services/webrtc';

export interface KeyMessage {
  type: 'keyboard';
  key: string;
  code?: string;
  down: boolean;
}

export default class KeyboardInput {
  constructor(private rtc: WebRTCService) {}

  private send(msg: KeyMessage) {
    this.rtc.sendData(msg);
  }

  press(key: string, code?: string) {
    this.send({ type: 'keyboard', key, code, down: true });
    this.send({ type: 'keyboard', key, code, down: false });
  }
}
