import { cleanup, render, screen, fireEvent, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const native = vi.hoisted(() => ({ localAvailable: false, commands: [] as string[] }));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (command: string) => {
    native.commands.push(command);
    if (command === "get_local_stt_capabilities") return JSON.stringify({
      platform: "Test", has_gpu: native.localAvailable,
      local_models_available: native.localAvailable,
      reason: native.localAvailable ? "GPU detected." : "No local GPU detected.",
    });
    if (command === "list_local_stt_engines") return "[]";
    return null;
  },
}));
vi.mock("@tauri-apps/plugin-store", () => ({
  load: async () => ({ get: async () => undefined, set: async () => {}, onKeyChange: async () => () => {} }),
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));

import { STTSetupStep } from "./STTSetupStep";

beforeEach(() => {
  native.localAvailable = false;
  native.commands = [];
});
afterEach(cleanup);

describe("first-run STT setup", () => {
  it("uses Groq as the only choice when no local GPU or Mac is detected", async () => {
    render(<STTSetupStep />);
    const groups = await screen.findAllByRole("radiogroup");
    expect(groups).toHaveLength(2);
    for (const group of groups) {
      expect(group.querySelectorAll('[role="radio"]')).toHaveLength(1);
      expect(group).toHaveTextContent("Groq Whisper");
    }
    expect(screen.getByText(/No local GPU detected/)).toBeInTheDocument();
    expect(native.commands).toContain("get_local_stt_capabilities");
    expect(native.commands).not.toContain("download_local_stt_model");
  });

  it("offers only Whisper Large v3 Turbo, Parakeet TDT and Groq when local support exists", async () => {
    native.localAvailable = true;
    render(<STTSetupStep />);
    expect(await screen.findAllByRole("radio", { name: /Whisper Large v3 Turbo/ })).toHaveLength(2);
    expect(screen.getAllByRole("radio", { name: /NVIDIA Parakeet TDT 0.6B v3/ })).toHaveLength(2);
    expect(screen.getAllByRole("radio", { name: /Groq Whisper/ })).toHaveLength(2);
    expect(screen.getByText("Install local models here")).toBeInTheDocument();
    expect(native.commands).toContain("list_local_stt_engines");
  });

  it("lets the user save a Groq key without leaving setup", async () => {
    render(<STTSetupStep />);
    await screen.findAllByRole("radiogroup");
    const input = within(screen.getByRole("region", { name: "Groq API key" })).getByLabelText("Groq API key");
    fireEvent.change(input, { target: { value: "gsk-test-key" } });
    fireEvent.click(screen.getByRole("button", { name: "Save key" }));
    expect(await screen.findByText("Saved securely")).toBeInTheDocument();
    expect(native.commands).toContain("store_api_key");
  });

  it("shows a privacy notice before Groq can send cloud audio", async () => {
    render(<STTSetupStep />);
    await screen.findAllByRole("radiogroup");
    expect(screen.getByText(/Cloud transcription is blocked while Private Local is on/)).toBeInTheDocument();
  });
});
