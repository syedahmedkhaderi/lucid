import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { MeetingAudioConfig } from "../lib/types";
import type * as IPC from "../lib/ipc";

const state = vi.hoisted(() => ({
  capabilities: vi.fn(),
  credentials: new Map<string, string>(),
  testConnection: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: async () => null }));
vi.mock("@tauri-apps/plugin-store", () => ({
  load: async () => ({ set: async () => {}, get: async () => undefined, onKeyChange: async () => () => {} }),
}));
vi.mock("../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof IPC>()),
  getLocalSTTCapabilities: () => state.capabilities(),
  hasApiKey: async (provider: string) => state.credentials.has(provider),
  getCredentialSetting: async (setting: string) => state.credentials.get(setting) ?? null,
  storeApiKey: async (provider: string, key: string) => { state.credentials.set(provider, key); },
  deleteApiKey: async (provider: string) => { state.credentials.delete(provider); },
  testSTTConnection: async (provider: string, apiKey?: string, replaceSavedKey = false) => {
    const connected = await state.testConnection(provider);
    if (connected && replaceSavedKey && apiKey?.trim()) state.credentials.set(provider, apiKey.trim());
    return connected;
  },
  listLocalSTTEngines: async () => [],
}));

import { STTSettings } from "./STTSettings";
import { useConfigStore } from "../stores/configStore";

const config: MeetingAudioConfig = {
  you: { role: "You", device_id: "default", is_input_device: true, stt_provider: "groq_whisper" },
  them: { role: "Them", device_id: "default", is_input_device: false, stt_provider: "groq_whisper" },
  recording_enabled: false,
  preset_name: null,
};

beforeEach(() => {
  state.credentials.clear();
  state.capabilities.mockReset().mockResolvedValue({
    platform: "macOS",
    has_gpu: true,
    local_models_available: true,
    reason: "Local speech is available on this Mac.",
  });
  state.testConnection.mockReset().mockResolvedValue(true);
  useConfigStore.setState({
    sttProvider: "groq_whisper",
    meetingAudioConfig: config,
    verifiedCloudProviders: [],
  });
});

afterEach(cleanup);

it("shows only English and the two local models plus Groq", async () => {
  render(<STTSettings />);
  expect(await screen.findByRole("button", { name: /Whisper Large v3 Turbo/ })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: /NVIDIA Parakeet TDT 0.6B v3/ })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: /Groq Whisper/ })).toBeInTheDocument();
  expect(screen.getByText("Transcription language")).toBeInTheDocument();
  expect(screen.getByText("English")).toBeInTheDocument();
  expect(screen.queryByRole("combobox", { name: /language/i })).toBeNull();
  for (const retiredProvider of ["Web Speech", "Windows Speech", "Deepgram", "Azure Speech", "Cartesia"]) {
    expect(screen.queryByText(retiredProvider, { exact: true })).toBeNull();
  }
});

it("offers Groq only when the device has no Mac or supported GPU", async () => {
  state.capabilities.mockResolvedValue({
    platform: "Windows",
    has_gpu: false,
    local_models_available: false,
    reason: "No local GPU was detected.",
  });
  render(<STTSettings />);
  expect(await screen.findByText(/No local GPU was detected/)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: /Groq Whisper/ })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /Whisper Large v3 Turbo/ })).toBeNull();
  expect(screen.queryByRole("button", { name: /Parakeet/ })).toBeNull();
});

it("saves and tests the Groq key from the speech screen", async () => {
  const user = userEvent.setup();
  render(<STTSettings />);
  const key = await screen.findByLabelText("API Key");
  await user.type(key, "groq-test-key");
  await user.click(screen.getByRole("button", { name: "Save & Test" }));

  await waitFor(() => expect(state.credentials.get("groq_whisper")).toBe("groq-test-key"));
  expect(state.testConnection).toHaveBeenCalledWith("groq_whisper");
});

it("opens model download controls in the same screen after choosing a local model", async () => {
  const user = userEvent.setup();
  render(<STTSettings />);
  await user.click(await screen.findByRole("button", { name: /Whisper Large v3 Turbo/ }));
  expect(await screen.findByText("Download a model here. Lucid verifies it and activates the first download automatically.")).toBeInTheDocument();
});
