import "@testing-library/jest-dom";
import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import FileTransfer from "../../src/components/FileTransfer";

describe("FileTransfer", () => {
  it("renders header", () => {
    render(<FileTransfer />);
    expect(screen.getByText(/file transfer/i)).toBeInTheDocument();
  });
});
