import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// In-memory plugin-store, as in configStore.test.ts.
const saved = vi.hoisted(() => new Map<string, unknown>());

vi.mock("@tauri-apps/plugin-store", () => ({
  load: async () => ({
    get: async (key: string) => saved.get(key),
    set: async (key: string, value: unknown) => void saved.set(key, value),
    onKeyChange: async () => () => {},
  }),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: async () => null }));

async function freshStore() {
  vi.resetModules();
  return (await import("../../stores/configStore")).useConfigStore;
}

async function wizardProviders() {
  return (await import("./STTSetupStep")).WIZARD_STT_PROVIDERS.map((p) => p.value);
}

async function wizardChoices() {
  return (await import("./STTSetupStep")).WIZARD_STT_PROVIDERS;
}

function configWith(you: string, them: string) {
  return {
    you: { role: "You", device_id: "default", is_input_device: true, stt_provider: you },
    them: { role: "Them", device_id: "default", is_input_device: false, stt_provider: them },
    recording_enabled: false,
    preset_name: null,
  };
}

describe("first-run STT step vs. the load-time migration", () => {
  beforeEach(() => {
    saved.clear();
    vi.spyOn(console, "log").mockImplementation(() => {});
  });
  afterEach(() => vi.restoreAllMocks());

  it("offers only the two supported local models and Groq cloud fallback", async () => {
    expect(await wizardProviders()).toEqual(["whisper_cpp", "parakeet_tdt", "groq_whisper"]);
    expect(await wizardChoices()).toMatchObject([
      { label: "Whisper Large v3 Turbo", value: "whisper_cpp", requiresKey: false },
      { label: "NVIDIA Parakeet TDT 0.6B v3", value: "parakeet_tdt", requiresKey: false },
      { label: "Groq Whisper", value: "groq_whisper", requiresKey: true },
    ]);
  });

  it("keeps every engine it offers for You after the next launch", async () => {
    for (const provider of await wizardProviders()) {
      saved.clear();
      saved.set("meetingAudioConfig", configWith(provider, "deepgram"));
      const store = await freshStore();
      await store.getState().loadConfig();
      // Whisper and Parakeet are preserved; Groq remains a valid cloud fallback.
      expect(store.getState().meetingAudioConfig?.you.stt_provider, `You: ${provider}`).toBe(provider);
    }
  });

  it("keeps every engine it offers for Them after the next launch", async () => {
    const them = (await wizardProviders()).filter((p) => p !== "web_speech");
    for (const provider of them) {
      saved.clear();
      saved.set("meetingAudioConfig", configWith("web_speech", provider));
      const store = await freshStore();
      await store.getState().loadConfig();
      expect(store.getState().meetingAudioConfig?.them.stt_provider, `Them: ${provider}`).toBe(provider);
    }
  });
});
