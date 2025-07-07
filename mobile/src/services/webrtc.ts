import { EventEmitter } from 'events';
import {
  RTCPeerConnection,
  RTCSessionDescription,
  RTCIceCandidate,
  MediaStream,
} from 'react-native-webrtc';
import SignalingService, { SignalingMessage } from './signaling';

export interface WebRTCServiceOptions {
  signaling: SignalingService;
  iceServers?: RTCIceServer[];
}

export declare interface WebRTCService {
  on(event: 'stream', listener: (stream: MediaStream) => void): this;
  on(event: 'connectionState', listener: (state: RTCPeerConnectionState) => void): this;
}

export class WebRTCService extends EventEmitter {
  private signaling: SignalingService;
  private iceServers: RTCIceServer[];
  private pc: RTCPeerConnection | null = null;

  constructor(options: WebRTCServiceOptions) {
    super();
    this.signaling = options.signaling;
    this.iceServers = options.iceServers ?? [{ urls: 'stun:stun.l.google.com:19302' }];

    this.signaling.on('message', this.handleSignal.bind(this));
  }

  async join(roomId: string) {
    this.signaling.connect();
    this.signaling.joinRoom(roomId);
  }

  private createPeer() {
    if (this.pc) return;
    this.pc = new RTCPeerConnection({ iceServers: this.iceServers });

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

  disconnect() {
    this.signaling.leaveRoom();
    this.signaling.disconnect();
    if (this.pc) {
      this.pc.close();
      this.pc = null;
    }
  }
}

export default WebRTCService;
