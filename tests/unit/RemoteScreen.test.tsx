import "@testing-library/jest-dom";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import RemoteScreen from "../../src/components/RemoteScreen";

describe("RemoteScreen", () => {
  beforeEach(() => {
    const orig = (document.createElement as any)
    if (orig.mock) {
      orig.mockImplementation((tag: string) => {
        if (tag === "video") {
          return { autoplay: true, muted: true, addEventListener: vi.fn(), removeEventListener: vi.fn() } as any
        }
        return orig(tag)
      })
    }
  })

  it.skip("toggles input", () => {
    const onToggle = vi.fn();
    render(<RemoteScreen isConnected={true} onInputToggle={onToggle} />);
    const button = screen.getByRole("button", { name: /input: on/i });
    fireEvent.click(button);
    expect(onToggle).toHaveBeenCalledWith(false);
    expect(button).toHaveTextContent(/input: off/i);
  });
});
