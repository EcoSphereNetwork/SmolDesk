import { EventEmitter } from 'events';
import {
  RTCPeerConnection,
  RTCSessionDescription,
  RTCIceCandidate,
  MediaStream,
} from 'react-native-webrtc';
import SignalingService, { SignalingMessage } from './signaling';
import AES from 'crypto-js/aes';
import HmacSHA256 from 'crypto-js/hmac-sha256';
import CryptoJS from 'crypto-js';
import encUtf8 from 'crypto-js/enc-utf8';
import encHex from 'crypto-js/enc-hex';
import { HMAC_KEY, HMAC_ENABLED } from '../config';

export interface WebRTCServiceOptions {
  signaling: SignalingService;
  iceServers?: RTCIceServer[];
  encryptionKey?: string;
}

export declare interface WebRTCService {
  on(event: 'stream', listener: (stream: MediaStream) => void): this;
  on(event: 'connectionState', listener: (state: RTCPeerConnectionState) => void): this;
  on(event: 'data', listener: (payload: any) => void): this;
}

export class WebRTCService extends EventEmitter {
  private signaling: SignalingService;
  private iceServers: RTCIceServer[];
  private pc: RTCPeerConnection | null = null;
  private dataChannel: RTCDataChannel | null = null;
  private encryptionKey: string | null = null;

  constructor(options: WebRTCServiceOptions) {
    super();
    this.signaling = options.signaling;
    this.iceServers = options.iceServers ?? [{ urls: 'stun:stun.l.google.com:19302' }];
    this.encryptionKey = options.encryptionKey ?? null;

    this.signaling.on('message', this.handleSignal.bind(this));
  }

  async join(roomId: string) {
    this.signaling.once('authorized', () => {
      this.signaling.joinRoom(roomId);
    });
    this.signaling.connect();
  }

  private createPeer() {
    if (this.pc) return;
    this.pc = new RTCPeerConnection({ iceServers: this.iceServers });

    // Create a data channel for input events.
    this.dataChannel = this.pc.createDataChannel('input');

    this.pc.ondatachannel = (e) => {
      if (e.channel.label === 'input') {
        this.dataChannel = e.channel;
        this.dataChannel.onmessage = (ev) => {
          let text = ev.data as string;
          let payload: any = null;
          try {
            payload = JSON.parse(text);
          } catch {
            if (this.encryptionKey) {
              try {
                const [ivB64, cipherB64] = text.split(':');
                const decrypted = AES.decrypt({
                  ciphertext: CryptoJS.enc.Base64.parse(cipherB64),
                } as any, CryptoJS.enc.Utf8.parse(this.encryptionKey), {
                  iv: CryptoJS.enc.Base64.parse(ivB64),
                });
                text = decrypted.toString(encUtf8);
                payload = JSON.parse(text);
              } catch {
                return;
              }
            }
          }
          if (payload) {
            this.emit('data', payload);
          }
        };
      }
    };

    this.pc.onicecandidate = (e) => {
      if (e.candidate) {
        this.signaling.send({
          type: 'ice-candidate',
          targetId: this.remoteId,
          candidate: e.candidate,
        });
      }
    };

    this.pc.ontrack = (ev) => {
      const stream = ev.streams[0];
      if (stream) {
        this.emit('stream', stream);
      }
    };

    this.pc.onconnectionstatechange = () => {
      if (this.pc) {
        this.emit('connectionState', this.pc.connectionState);
      }
    };
  }

  private remoteId: string | null = null;

  private async handleSignal(msg: SignalingMessage) {
    switch (msg.type) {
      case 'offer':
        this.remoteId = msg.peerId;
        await this.handleOffer(msg.offer);
        break;
      case 'answer':
        await this.pc?.setRemoteDescription(new RTCSessionDescription(msg.answer));
        break;
      case 'ice-candidate':
        if (this.pc) {
          await this.pc.addIceCandidate(new RTCIceCandidate(msg.candidate));
        }
        break;
      default:
        break;
    }
  }

  private async handleOffer(offer: RTCSessionDescriptionInit) {
    this.createPeer();
    if (!this.pc) return;

    await this.pc.setRemoteDescription(new RTCSessionDescription(offer));
    const answer = await this.pc.createAnswer();
    await this.pc.setLocalDescription(answer);

    if (this.remoteId) {
      this.signaling.send({ type: 'answer', targetId: this.remoteId, answer });
    }
  }

  sendData(payload: any) {
    if (this.dataChannel && this.dataChannel.readyState === 'open') {
      if (HMAC_ENABLED) {
        payload.hmac = HmacSHA256(JSON.stringify(payload), HMAC_KEY).toString(encHex);
      }
      let text = JSON.stringify(payload);
      if (this.encryptionKey) {
        const iv = CryptoJS.lib.WordArray.random(16);
        const encrypted = AES.encrypt(text, CryptoJS.enc.Utf8.parse(this.encryptionKey), { iv });
        text = iv.toString(CryptoJS.enc.Base64) + ':' + encrypted.ciphertext.toString(CryptoJS.enc.Base64);
      }
      this.dataChannel.send(text);
    }
  }

  sendRaw(text: string) {
    if (this.dataChannel && this.dataChannel.readyState === 'open') {
      this.dataChannel.send(text);
    }
  }

  disconnect() {
    this.signaling.leaveRoom();
    this.signaling.disconnect();
    if (this.pc) {
      this.pc.close();
      this.pc = null;
    }
    this.dataChannel = null;
  }
}

export default WebRTCService;
