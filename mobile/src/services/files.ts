import DocumentPicker from 'react-native-document-picker';
import RNFS from 'react-native-fs';
import Toast from 'react-native-toast-message';
import WebRTCService from './webrtc';

export interface FileHeader {
  type: 'file_header';
  id: string;
  name: string;
  mime: string;
  size: number;
}

export default class FileTransferService {
  private transfers: Record<string, { header: FileHeader; chunks: string[] }> = {};

  constructor(private rtc: WebRTCService) {
    rtc.on('data', this.handleData.bind(this));
  }

  async pickAndSend() {
    const file = await DocumentPicker.pickSingle();
    await this.sendFile(file.uri, file.name, file.type || 'application/octet-stream', file.size || 0);
  }

  async sendFile(uri: string, name: string, mime: string, size: number) {
    const id = Date.now().toString();
    const header: FileHeader = { type: 'file_header', id, name, mime, size };
    Toast.show({ type: 'info', text1: 'Dateiübertragung gestartet' });
    this.rtc.sendRaw(JSON.stringify(header));
    const data = await RNFS.readFile(uri, 'base64');
    const chunkSize = 64 * 1024;
    for (let offset = 0; offset < data.length; offset += chunkSize) {
      const chunk = data.slice(offset, offset + chunkSize);
      this.rtc.sendData({ type: 'file_chunk', id, data: chunk });
    }
    this.rtc.sendData({ type: 'file_end', id });
    Toast.show({ type: 'success', text1: 'Datei gesendet' });
  }

  private async handleData(payload: any) {
    switch (payload.type) {
      case 'file_header':
        this.transfers[payload.id] = { header: payload, chunks: [] };
        break;
      case 'file_chunk':
        const t = this.transfers[payload.id];
        if (t) t.chunks.push(payload.data);
        break;
      case 'file_end': {
        const t = this.transfers[payload.id];
        if (!t) return;
        const base64 = t.chunks.join('');
        const path = `${RNFS.DownloadDirectoryPath}/${t.header.name}`;
        await RNFS.writeFile(path, base64, 'base64');
        Toast.show({ type: 'success', text1: 'Datei empfangen' });
        delete this.transfers[payload.id];
        break;
      }
    }
  }
}
