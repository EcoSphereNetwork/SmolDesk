import { EventEmitter } from 'events';

export interface SignalingOptions {
  url: string;
  reconnectInterval?: number;
}

export type SignalingMessage = {
  type: string;
  [key: string]: any;
};

export declare interface SignalingService {
  on(event: 'open', listener: () => void): this;
  on(event: 'close', listener: () => void): this;
  on(event: 'message', listener: (msg: SignalingMessage) => void): this;
  on(event: 'error', listener: (err: any) => void): this;
}

export class SignalingService extends EventEmitter {
  private url: string;
  private reconnectInterval: number;
  private socket: WebSocket | null = null;
  private reconnectTimer: NodeJS.Timeout | null = null;

  constructor(options: SignalingOptions) {
    super();
    this.url = options.url;
    this.reconnectInterval = options.reconnectInterval ?? 5000;
  }

  connect() {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      return;
    }
    this.socket = new WebSocket(this.url);

    this.socket.onopen = () => {
      this.emit('open');
    };

    this.socket.onclose = () => {
      this.emit('close');
      if (this.reconnectTimer) return;
      this.reconnectTimer = setTimeout(() => {
        this.reconnectTimer = null;
        this.connect();
      }, this.reconnectInterval);
    };

    this.socket.onerror = (err) => {
      this.emit('error', err);
    };

    this.socket.onmessage = (ev) => {
      try {
        const msg: SignalingMessage = JSON.parse(ev.data);
        this.emit('message', msg);
      } catch (e) {
        this.emit('error', e);
      }
    };
  }

  disconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
  }

  send(msg: SignalingMessage) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(msg));
    }
  }

  joinRoom(roomId: string) {
    this.send({ type: 'join-room', roomId });
  }

  leaveRoom() {
    this.send({ type: 'leave-room' });
  }
}

export default SignalingService;
