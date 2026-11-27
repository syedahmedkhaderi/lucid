import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { MeetingAudioConfig } from "../lib/types";

vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(async () => null) }));
vi.mock("@tauri-apps/plugin-store", () => ({
  load: async () => ({ set: async () => {}, get: async () => undefined, onKeyChange: async () => () => {} }),
}));

import { ServiceStatusBar } from "./ServiceStatusBar";
import { useConfigStore } from "../stores/configStore";
import { useMeetingStore } from "../stores/meetingStore";

const config: MeetingAudioConfig = {
  you: { role: "You", device_id: "mic", is_input_device: true, stt_provider: "deepgram" },
  them: { role: "Them", device_id: "out", is_input_device: false, stt_provider: "deepgram" },
  recording_enabled: false,
  preset_name: null,
};

describe("English-only status bar", () => {
  beforeEach(() => {
    useConfigStore.setState({ meetingAudioConfig: config, sttLanguage: "en-US" });
  });
  afterEach(cleanup);

  it("does not show per-side language controls during an online meeting", () => {
    useMeetingStore.setState({ isRecording: true, audioMode: "online" });
    render(<ServiceStatusBar compact />);
    expect(screen.queryByRole("button", { name: /language/i })).toBeNull();
  });

  it("does not show a Room language control in an in-person meeting", () => {
    useMeetingStore.setState({ isRecording: true, audioMode: "in_person" });
    render(<ServiceStatusBar compact />);
    expect(screen.queryByRole("button", { name: /language/i })).toBeNull();
  });

  it("shows the registered MAI provider and exact model in the status chip", () => {
    useMeetingStore.setState({ audioMode: "online" });
    useConfigStore.setState({
      meetingAudioConfig: {
        ...config,
        you: { ...config.you, stt_provider: "azure_mai_streaming" },
      },
    });
    render(<ServiceStatusBar />);
    expect(screen.getByRole("button", { name: "Your STT: Azure MAI Streaming / MAI-Transcribe-2-Streaming" })).toBeInTheDocument();
  });

  it("is absent outside a meeting and in the launcher footer", () => {
    useMeetingStore.setState({ isRecording: false, audioMode: "online" });
    const { rerender } = render(<ServiceStatusBar compact />);
    expect(screen.queryByRole("button", { name: /language/ })).toBeNull();
    useMeetingStore.setState({ isRecording: true });
    rerender(<ServiceStatusBar />);
    expect(screen.queryByRole("button", { name: /language/ })).toBeNull();
  });
});
