import "@testing-library/jest-dom";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

var listeners: Record<string, Function> = {};
var mocks = {
  connect: vi.fn(),
  disconnect: vi.fn(),
  createRoom: vi.fn(),
  joinRoom: vi.fn(),
  leaveRoom: vi.fn(),
};

vi.mock("../../src/utils/webrtc", () => ({
  WebRTCConnection: class {
    connect = mocks.connect;
    disconnect = mocks.disconnect;
    createRoom = mocks.createRoom;
    joinRoom = mocks.joinRoom;
    leaveRoom = mocks.leaveRoom;
    on(event: string, handler: Function) {
      listeners[event] = handler;
    }
  },
  WebRTCConnectionEvent: {
    PEER_JOINED: "peer-joined",
    SIGNALING_CONNECTED: "signaling-connected",
  },
  __listeners: listeners,
  __mocks: mocks,
}));

import ConnectionManager from "../../src/components/ConnectionManager";

describe("ConnectionManager", () => {
  it("connects when button clicked", () => {
    render(<ConnectionManager signalingServer="ws://test" />);
    const btn = screen.getByRole("button", { name: /connect to server/i });
    fireEvent.click(btn);
    expect(mocks.connect).toHaveBeenCalled();
  });

});
