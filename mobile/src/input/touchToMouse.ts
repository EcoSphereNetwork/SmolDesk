/**
 * Utility for mapping touch gestures to mouse events.
 * Events are sent via the WebRTC data channel in JSON format.
 *
 * Gesture Mapping:
 * - single tap -> left mouse click
 * - long press -> right mouse click
 * - one finger drag -> move cursor (relative)
 * - two finger tap -> right mouse click
 * - two finger drag -> mouse wheel scroll
 */
import WebRTCService from '../services/webrtc';

export type MouseAction = 'move' | 'down' | 'up' | 'scroll';

export interface MouseMessage {
  type: 'mouse';
  action: MouseAction;
  button?: number;
  x?: number;
  y?: number;
  dx?: number;
  dy?: number;
}

export default class TouchToMouse {
  constructor(private rtc: WebRTCService) {}

  private send(msg: MouseMessage) {
    this.rtc.sendData(msg);
  }

  tap(x: number, y: number) {
    this.send({ type: 'mouse', action: 'down', button: 0, x, y });
    this.send({ type: 'mouse', action: 'up', button: 0, x, y });
  }

  longPress(x: number, y: number) {
    this.send({ type: 'mouse', action: 'down', button: 2, x, y });
    this.send({ type: 'mouse', action: 'up', button: 2, x, y });
  }

  move(dx: number, dy: number) {
    this.send({ type: 'mouse', action: 'move', dx, dy });
  }

  scroll(dy: number) {
    this.send({ type: 'mouse', action: 'scroll', dy });
  }
}
