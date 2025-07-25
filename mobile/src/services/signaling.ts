import { EventEmitter } from 'events';

export interface SignalingOptions {
  url: string;
  reconnectInterval?: number;
  token?: string;
  hmacKey?: string;
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
  on(event: 'authorized', listener: () => void): this;
  on(event: 'unauthorized', listener: () => void): this;
  on(event: 'monitors', listener: (list: MonitorInfo[]) => void): this;
}

export interface MonitorInfo {
  id: number;
  width: number;
  height: number;
  name?: string;
}

export class SignalingService extends EventEmitter {
  private url: string;
  private reconnectInterval: number;
  private socket: WebSocket | null = null;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private token: string | null = null;
  private hmacKey: string | null = null;

  constructor(options: SignalingOptions) {
    super();
    this.url = options.url;
    this.reconnectInterval = options.reconnectInterval ?? 5000;
    this.token = options.token ?? null;
    this.hmacKey = options.hmacKey ?? null;
  }

  connect(token?: string) {
    if (token) this.token = token;
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      return;
    }
    this.socket = new WebSocket(this.url);

    this.socket.onopen = () => {
      this.emit('open');
      if (this.token) {
        this.send({ type: 'auth', token: this.token });
      }
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
        if (msg.type === 'authorized') {
          this.emit('authorized');
        } else if (msg.type === 'unauthorized') {
          this.emit('unauthorized');
        } else if (msg.type === 'monitors') {
          this.emit('monitors', msg.monitors as MonitorInfo[]);
        } else {
          this.emit('message', msg);
        }
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
      if (this.hmacKey && msg.type === 'join-room') {
        const h = require('crypto-js/hmac-sha256');
        const enc = require('crypto-js/enc-hex');
        msg.hmac = h(msg.roomId, this.hmacKey).toString(enc);
      }
      this.socket.send(JSON.stringify(msg));
    }
  }

  joinRoom(roomId: string) {
    this.send({ type: 'join-room', roomId });
  }

  leaveRoom() {
    this.send({ type: 'leave-room' });
  }

  selectMonitor(id: number) {
    this.send({ type: 'select-monitor', monitorId: id });
  }
}

export default SignalingService;
