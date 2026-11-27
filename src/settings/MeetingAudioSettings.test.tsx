import { invoke } from "@tauri-apps/api/core";
import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { MeetingAudioConfig } from "../lib/types";
import type * as IPC from "../lib/ipc";

const localEngineInventory = vi.hoisted(() => ({ current: [] as unknown[] }));

vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(async () => null) }));
vi.mock("@tauri-apps/plugin-store", () => ({
  load: async () => ({ set: async () => {}, get: async () => undefined, onKeyChange: async () => () => {} }),
}));
vi.mock("../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof IPC>()),
  listAudioDevices: async () => ({
    inputs: [{ id: 'Studio: USB "Audio"', name: 'Studio: USB "Audio"', is_input: true, is_default: false }],
    outputs: [{ id: 'Studio: USB "Audio"', name: 'Studio: USB "Audio"', is_input: false, is_default: false }],
  }),
  getAudioSessions: async () => [],
  listLocalSTTEngines: async () => localEngineInventory.current as never,
  startDeviceMonitor: async () => {},
  stopDeviceMonitor: async () => {},
  stopAudioTest: async () => false,
}));

import { MeetingAudioSettings } from "./MeetingAudioSettings";
import { AudioSetupStep } from "../components/wizard/AudioSetupStep";
import { useConfigStore } from "../stores/configStore";

const initialConfig: MeetingAudioConfig = {
  you: { role: "You", device_id: 'Studio: USB "Audio"', is_input_device: true, stt_provider: "groq_whisper" },
  them: { role: "Them", device_id: 'Studio: USB "Audio"', is_input_device: true, stt_provider: "groq_whisper" },
  recording_enabled: false,
  preset_name: null,
};

const initialDeepgramConfig = useConfigStore.getState().deepgramConfig;
beforeEach(() => {
  localEngineInventory.current = [];
  useConfigStore.setState({ meetingAudioConfig: initialConfig });
});
afterEach(() => {
  cleanup();
  // A test's Deepgram model (Flux, say) must not reach the next test.
  useConfigStore.setState({ deepgramConfig: initialDeepgramConfig });
});

it("offers Groq on unsupported hardware before its key is added", async () => {
  const user = userEvent.setup();
  render(<MeetingAudioSettings />);

  for (const side of ["You", "Them"] as const) {
    await user.click(screen.getByRole("button", { name: `${side} speech-to-text provider` }));
    const groqOptions = await screen.findAllByRole("button", { name: /Groq Whisper/ });
    const groqProvider = groqOptions.find((option) => option.textContent?.includes("Add an API key"));
    expect(groqProvider).toBeInTheDocument();
    expect(screen.getByText("Add an API key in Speech & Models")).toBeInTheDocument();
    await user.click(groqProvider!);
    expect(useConfigStore.getState().meetingAudioConfig?.[side.toLowerCase() as "you" | "them"].stt_provider)
      .toBe("groq_whisper");
    for (const retiredProvider of ["Deepgram", "Azure Speech", "Azure MAI Streaming", "Cartesia"]) {
      expect(screen.queryByRole("button", { name: retiredProvider })).toBeNull();
    }
  }
});

it("offers the downloaded Whisper Turbo model as a live provider for either meeting side", async () => {
  localEngineInventory.current = [{
    engine: "whisper_cpp",
    name: "Whisper.cpp",
    description: "Local Whisper transcription",
    models: [{
      id: "large-v3-turbo",
      engine: "whisper_cpp",
      name: "Whisper Large v3 Turbo",
      size_bytes: 1_620_000_000,
      accuracy_rating: 5,
      speed_rating: 5,
      is_streaming: true,
      is_downloaded: true,
    }],
  }];
  const user = userEvent.setup();
  render(<MeetingAudioSettings />);

  for (const side of ["You", "Them"] as const) {
    await user.click(screen.getByRole("button", { name: `${side} speech-to-text provider` }));
    await user.click(await screen.findByRole("button", { name: "Whisper Large v3 Turbo" }));
    expect(useConfigStore.getState().meetingAudioConfig?.[side.toLowerCase() as "you" | "them"])
      .toMatchObject({ stt_provider: "whisper_cpp", local_model_id: "large-v3-turbo" });
  }
});

it("shows English as the fixed transcription language and no language pickers", () => {
  render(<MeetingAudioSettings />);
  expect(screen.getByText("Transcription: English")).toBeInTheDocument();
  expect(screen.queryByRole("combobox", { name: /language/i })).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
});

describe.each([
  { surface: "settings", Component: MeetingAudioSettings },
  { surface: "wizard", Component: AudioSetupStep },
])("$surface audio source selection", ({ Component }) => {
  it.each(["you", "them"] as const)("keeps the %s input and output distinct when their names match", async (party) => {
    const user = userEvent.setup();
    const view = render(<Component />);
    const group = await screen.findAllByRole("group", { name: "Speakers / Output" });
    const output = within(group[party === "you" ? 0 : 1]).getByRole<HTMLOptionElement>("option");
    const select = output.closest("select")!;
    await waitFor(() => expect(select).toBeEnabled());

    await user.selectOptions(select, output);
    expect(useConfigStore.getState().meetingAudioConfig?.[party]).toMatchObject({
      device_id: 'Studio: USB "Audio"',
      is_input_device: false,
    });
    expect(output.selected).toBe(true);

    view.unmount();
    render(<Component />);
    const restoredGroups = await screen.findAllByRole("group", { name: "Speakers / Output" });
    const restoredOutput = within(restoredGroups[party === "you" ? 0 : 1]).getByRole<HTMLOptionElement>("option");
    expect(restoredOutput.selected).toBe(true);
    const restoredSelect = restoredOutput.closest("select")!;
    const micGroup = within(restoredSelect).getByRole("group", { name: "Microphones" });
    const input = within(micGroup).getByRole<HTMLOptionElement>("option");
    await waitFor(() => expect(restoredSelect).toBeEnabled());
    await user.selectOptions(restoredSelect, input);
    expect(useConfigStore.getState().meetingAudioConfig?.[party]).toMatchObject({
      device_id: 'Studio: USB "Audio"',
      is_input_device: true,
    });
    expect(input.selected).toBe(true);
  });

  it.each([
    { party: "you" as const, isInput: true },
    { party: "them" as const, isInput: false },
  ])("uses the correct direction for the $party default", async ({ party, isInput }) => {
    const user = userEvent.setup();
    render(<Component />);
    const groups = await screen.findAllByRole("group", { name: "Microphones" });
    const select = groups[party === "you" ? 0 : 1].closest("select")!;
    await waitFor(() => expect(select).toBeEnabled());
    await user.selectOptions(select, select.options[0]);
    expect(useConfigStore.getState().meetingAudioConfig?.[party]).toMatchObject({
      device_id: "default",
      is_input_device: isInput,
    });
    expect(select.options[0].selected).toBe(true);
  });
});

it("offers one fixed English setting and no per-side language pickers", () => {
  render(<MeetingAudioSettings />);
  expect(screen.getByText("Transcription: English")).toBeInTheDocument();
  expect(screen.queryByRole("combobox", { name: /language/i })).toBeNull();
  expect(screen.queryByText(/Auto-detect/)).toBeNull();
});
