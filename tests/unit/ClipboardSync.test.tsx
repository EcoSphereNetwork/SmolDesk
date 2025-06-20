import "@testing-library/jest-dom";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import ClipboardSync from "../../src/components/ClipboardSync";
import { invoke } from "@tauri-apps/api/tauri";
import { Mock } from "vitest";

vi.mock("../../src/utils/webrtc", () => ({
  WebRTCConnection: class {},
}));

describe("ClipboardSync component", () => {
  it("renders and toggles sync", async () => {
    const mockInvoke = invoke as Mock;
    mockInvoke.mockResolvedValueOnce("x11");
    mockInvoke.mockResolvedValueOnce(true);
    mockInvoke.mockResolvedValueOnce([]);

    render(<ClipboardSync />);
    await waitFor(() =>
      expect(screen.getByText("Clipboard Sync")).toBeInTheDocument(),
    );

    const toggle = await screen.findByRole("button", { name: /disable/i });
    fireEvent.click(toggle);
    expect(invoke).toHaveBeenCalledWith("stop_clipboard_monitoring");
  });
});
