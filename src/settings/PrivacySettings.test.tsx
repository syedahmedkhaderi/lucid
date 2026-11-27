import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useConfigStore } from "../stores/configStore";
import { PrivacySettings } from "./PrivacySettings";

describe("PrivacySettings", () => {
  const setPrivacyMode = vi.fn<(_: "private_local" | "cloud_allowed") => Promise<void>>();

  beforeEach(() => {
    setPrivacyMode.mockReset().mockResolvedValue(undefined);
    useConfigStore.setState({
      privacyMode: "private_local",
      setPrivacyMode,
      activateLLMProvider: vi.fn().mockResolvedValue(undefined),
    });
  });

  afterEach(() => {
    cleanup();
  });

  it("defaults to Private Local and offers explicit cloud consent", () => {
    render(<PrivacySettings />);
    expect(screen.getByRole("radio", { name: /Private Local/ })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: /Cloud processing allowed/ })).toHaveAttribute("aria-checked", "false");
    expect(screen.getByText(/Whisper Large v3 Turbo, NVIDIA Parakeet TDT 0\.6B v3/)).toBeInTheDocument();
  });

  it("changes mode through the backend-backed store action", async () => {
    render(<PrivacySettings />);
    fireEvent.click(screen.getByRole("radio", { name: /Cloud processing allowed/ }));
    await waitFor(() => expect(setPrivacyMode).toHaveBeenCalledWith("cloud_allowed"));
  });

  it("shows backend refusals without changing the selected mode", async () => {
    setPrivacyMode.mockRejectedValueOnce(new Error("Stop the meeting before changing privacy mode."));
    render(<PrivacySettings />);
    fireEvent.click(screen.getByRole("radio", { name: /Cloud processing allowed/ }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/Stop the meeting/);
    expect(screen.getByRole("radio", { name: /Private Local/ })).toHaveAttribute("aria-checked", "true");
  });
});
