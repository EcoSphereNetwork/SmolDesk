import Clipboard from '@react-native-clipboard/clipboard';
import WebRTCService from './webrtc';

export default class ClipboardService {
  constructor(private rtc: WebRTCService) {}

  async copyLocal(text: string) {
    await Clipboard.setString(text);
    this.rtc.sendData({ type: 'clipboard', content: text });
  }

  async handleRemote(content: string) {
    await Clipboard.setString(content);
  }
}
